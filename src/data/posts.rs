use std::{collections::HashMap, fmt::Display, iter::MapWhile, ops::Range};

use async_trait::async_trait;
use blogapi::models::_entities::posts::{
    ActiveModel as ActivePost, Entity as PostEntity, Model as Post,
};
use blogapi::models::_entities::users::Model as User;
use color_eyre::{eyre::eyre, Result};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use sea_orm::{
    ActiveModelBehavior, ActiveModelTrait, DatabaseConnection, EntityTrait, ModelTrait, Set,
};

use crate::components::crudedit::CrudEdit;
use crate::style::FormStyle;

use super::{CrudData, CrudEditMode, CrudRow};

#[derive(Default, Eq, PartialEq, Hash, Debug, Copy, Clone)]
enum PostField {
    #[default]
    Title = 0,
    Summary,
    Content,
}

impl PostField {
    fn next(&self) -> Self {
        match *self {
            PostField::Title => PostField::Summary,
            PostField::Summary => PostField::Content,
            PostField::Content => PostField::Title,
        }
    }
}

#[derive(Debug)]
struct PostFieldError;
impl TryFrom<usize> for PostField {
    type Error = PostFieldError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(PostField::Title),
            1 => Ok(PostField::Summary),
            2 => Ok(PostField::Content),
            _ => Err(PostFieldError),
        }
    }
}

impl Display for PostField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Default)]
pub struct Posts {
    db: Option<DatabaseConnection>,
    posts: Vec<Post>,
}

#[async_trait]
impl CrudData for Posts {
    fn headers(&self) -> Vec<String> {
        (0..3)
            .map(|x| PostField::try_from(x).unwrap().to_string())
            .collect()
    }

    fn rows(&self) -> Vec<Vec<String>> {
        self.posts
            .iter()
            .map(|x| {
                vec![
                    x.title.clone(),
                    x.summary.clone().unwrap_or(String::new()),
                    x.content.clone().unwrap_or(String::new()),
                ]
            })
            .collect()
    }

    fn widths(&self) -> Vec<Constraint> {
        vec![
            Constraint::Percentage(20),
            Constraint::Percentage(40),
            Constraint::Percentage(40),
        ]
    }

    fn num_rows(&self) -> usize {
        self.posts.len()
    }

    fn set_db(&mut self, cnx: Option<DatabaseConnection>) {
        self.db = cnx;
    }

    async fn delete(&self, idx: usize) -> Result<()> {
        if let Some(cnx) = &self.db {
            self.posts[idx].clone().delete(cnx).await?;
            Ok(())
        } else {
            Err(eyre!("Database is not connected"))
        }
    }

    async fn refresh(&mut self) -> Result<()> {
        if let Some(cnx) = &self.db {
            self.posts = PostEntity::find().all(cnx).await?;
            Ok(())
        } else {
            Err(eyre!("Database is not connected"))
        }
    }

    fn to_db_id(&self, idx: usize) -> i32 {
        self.posts[idx].id
    }
}

#[derive(Default)]
pub struct PostEdit {
    mode: CrudEditMode,
    db: Option<DatabaseConnection>,
    row: Option<Post>,
    focused_field: Option<PostField>,
    fields: HashMap<PostField, String>,
}

#[async_trait]
impl CrudRow for PostEdit {
    async fn edit(&mut self, idx: i32) -> Result<()> {
        if let Some(cnx) = &self.db {
            self.row = PostEntity::find_by_id(idx).one(cnx).await?;
            if let Some(post) = &self.row {
                self.fields.insert(PostField::Title, post.title.clone());
                self.fields.insert(
                    PostField::Summary,
                    post.summary.clone().unwrap_or("".to_string()),
                );
                self.fields.insert(
                    PostField::Content,
                    post.content.clone().unwrap_or("".to_string()),
                );
                self.mode = CrudEditMode::Edit;
                self.focused_field = Some(PostField::Title);
                return Ok(());
            }

            Ok(())
        } else {
            Err(eyre!("Database is not connected"))
        }
    }

    fn new(&mut self) {
        self.fields.insert(PostField::Title, String::new());
        self.fields.insert(PostField::Summary, String::new());
        self.fields.insert(PostField::Content, String::new());
        self.focused_field = Some(PostField::Title);
        self.mode = CrudEditMode::New;
    }

    async fn save(&mut self) -> Result<()> {
        if let Some(cnx) = &self.db {
            let user = User::find_by_email(cnx, "a.reversat@gmail.com").await?;
            let mut post: ActivePost = match self.mode {
                CrudEditMode::New => ActiveModelTrait::default(),
                CrudEditMode::Edit => {
                    if let Some(post) = &self.row {
                        post.clone().into()
                    } else {
                        return Err(eyre!("Edit mode with no row"));
                    }
                }
            };
            post.title = Set(self
                .fields
                .get(&PostField::Title)
                .unwrap_or(&"".to_string())
                .to_owned());
            post.summary = Set(Some(
                self.fields
                    .get(&PostField::Summary)
                    .unwrap_or(&"".to_string())
                    .to_owned(),
            ));
            post.content = Set(Some(
                self.fields
                    .get(&PostField::Content)
                    .unwrap_or(&"".to_string())
                    .to_owned(),
            ));
            post.user_id = Set(user.id);

            match self.mode {
                CrudEditMode::Edit => post.update(cnx).await?,
                CrudEditMode::New => post.insert(cnx).await?,
            };
        }
        Ok(())
    }

    fn focus_next_field(&mut self) {
        if let Some(field) = self.focused_field {
            self.focused_field = Some(field.next());
        }
    }

    fn input(&mut self, c: char) {
        if let Some(fieldname) = self.focused_field {
            if let Some(field) = self.fields.get_mut(&fieldname) {
                field.push(c);
            }
        }
    }

    fn delete_last_char(&mut self) {
        if let Some(fieldname) = self.focused_field {
            if let Some(field) = self.fields.get_mut(&fieldname) {
                field.pop();
            }
        }
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(10),
                Constraint::Percentage(30),
                Constraint::Percentage(60),
            ])
            .split(area);

        for i in 0..3 {
            let field = PostField::try_from(i).unwrap();
            let style = if Some(field) == self.focused_field {
                FormStyle::highlighted()
            } else {
                FormStyle::normal()
            };
            let block = Block::default()
                .borders(Borders::ALL)
                .title(field.to_string())
                .style(style);
            let value = Paragraph::new(self.fields.get(&field).unwrap_or(&"".to_string()).clone())
                .block(block);
            f.render_widget(value, layout[i]);
        }
        Ok(())
    }

    fn set_db(&mut self, db: Option<DatabaseConnection>) {
        self.db = db;
    }
}
