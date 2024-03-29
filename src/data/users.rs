use std::{collections::HashMap, fmt::Display};

use async_trait::async_trait;
use blogapi::models::users::{
    ActiveModel as ActiveUser, Entity as UserEntity, Model as User, RegisterParams,
};
use color_eyre::{eyre::eyre, Result};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, ModelTrait, Set};

use crate::{components::crudedit::CrudEdit, style::FormStyle};

use super::{CrudData, CrudEditMode, CrudRow};

#[derive(Default)]
pub struct Users {
    db: Option<DatabaseConnection>,
    users: Vec<User>,
}

#[async_trait]
impl CrudData for Users {
    fn headers(&self) -> Vec<String> {
        vec!["Name".to_string(), "Email".to_string()]
    }

    fn rows(&self) -> Vec<Vec<String>> {
        self.users
            .iter()
            .map(|x| vec![x.name.clone(), x.email.clone()])
            .collect()
    }

    fn widths(&self) -> Vec<Constraint> {
        vec![Constraint::Percentage(50), Constraint::Percentage(50)]
    }

    fn num_rows(&self) -> usize {
        self.users.len()
    }

    fn set_db(&mut self, cnx: Option<DatabaseConnection>) {
        self.db = cnx;
    }

    async fn delete(&self, idx: usize) -> Result<()> {
        if let Some(cnx) = &self.db {
            self.users[idx].clone().delete(cnx).await?;
        }
        Ok(())
    }

    async fn refresh(&mut self) -> Result<()> {
        if let Some(cnx) = &self.db {
            self.users = UserEntity::find().all(cnx).await?;
        }
        Ok(())
    }

    fn to_db_id(&self, idx: usize) -> i32 {
        self.users[idx].id
    }
}

#[derive(Default, Debug, Hash, PartialEq, Eq)]
enum UserField {
    #[default]
    Name,
    Email,
    Password1,
    Password2,
}

impl UserField {
    fn next(&self) -> Self {
        match *self {
            UserField::Name => UserField::Email,
            UserField::Email => UserField::Password1,
            UserField::Password1 => UserField::Password2,
            UserField::Password2 => UserField::Name,
        }
    }
}

#[derive(Debug)]
struct UserFieldError;
impl TryFrom<usize> for UserField {
    type Error = UserFieldError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(UserField::Name),
            1 => Ok(UserField::Email),
            2 => Ok(UserField::Password1),
            3 => Ok(UserField::Password2),
            _ => Err(UserFieldError),
        }
    }
}

impl Display for UserField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Default)]
pub struct UserEdit {
    mode: CrudEditMode,
    db: Option<DatabaseConnection>,
    row: Option<User>,
    focused_field: UserField,
    fields: HashMap<UserField, String>,
}

#[async_trait]
impl CrudRow for UserEdit {
    async fn edit(&mut self, idx: i32) -> Result<()> {
        if let Some(cnx) = &self.db {
            self.row = UserEntity::find_by_id(idx).one(cnx).await?;
            if let Some(user) = &self.row {
                self.fields.insert(UserField::Name, user.name.clone());
                self.fields.insert(UserField::Email, user.email.clone());
                self.fields.insert(UserField::Password1, String::new());
                self.fields.insert(UserField::Password2, String::new());
                self.mode = CrudEditMode::Edit;
                return Ok(());
            }

            Ok(())
        } else {
            Err(eyre!("Database is not connected"))
        }
    }

    fn new(&mut self) {
        self.fields.insert(UserField::Name, String::new());
        self.fields.insert(UserField::Email, String::new());
        self.fields.insert(UserField::Password1, String::new());
        self.fields.insert(UserField::Password2, String::new());
        self.mode = CrudEditMode::New;
    }

    async fn save(&mut self) -> Result<()> {
        let password =
            if self.fields.get(&UserField::Password1) == self.fields.get(&UserField::Password2) {
                self.fields.get(&UserField::Password1).clone()
            } else {
                None
            };
        if let Some(cnx) = &self.db {
            if let Some(pw) = password {
                match self.mode {
                    CrudEditMode::New => {
                        let user = User::create_with_password(
                            cnx,
                            &RegisterParams {
                                name: self
                                    .fields
                                    .get(&UserField::Name)
                                    .unwrap_or(&"".to_string())
                                    .clone(),
                                email: self
                                    .fields
                                    .get(&UserField::Email)
                                    .unwrap_or(&"".to_string())
                                    .clone(),
                                password: pw.to_string(),
                            },
                        )
                        .await?;
                        let mutuser: ActiveUser = user.into();
                        mutuser.verified(cnx).await?;
                    }
                    CrudEditMode::Edit => {
                        if let Some(user) = &self.row {
                            let mut mutuser: ActiveUser = user.clone().into();

                            if !user.verify_password(pw) {
                                mutuser.clone().reset_password(cnx, pw).await?;
                            }

                            mutuser.name = Set(self
                                .fields
                                .get(&UserField::Name)
                                .unwrap_or(&"".to_string())
                                .clone());
                            mutuser.email = Set(self
                                .fields
                                .get(&UserField::Email)
                                .unwrap_or(&"".to_string())
                                .clone());
                            mutuser.update(cnx).await?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn focus_next_field(&mut self) {
        self.focused_field = self.focused_field.next();
    }

    fn input(&mut self, c: char) {
        if let Some(field) = self.fields.get_mut(&self.focused_field) {
            field.push(c);
        }
    }

    fn delete_last_char(&mut self) {
        if let Some(field) = self.fields.get_mut(&self.focused_field) {
            field.pop();
        }
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),
                Constraint::Min(3),
                Constraint::Min(3),
                Constraint::Min(3),
            ])
            .split(area);

        for i in 0..4 {
            let field = UserField::try_from(i).unwrap();
            let style = if field == self.focused_field {
                FormStyle::highlighted()
            } else {
                FormStyle::normal()
            };
            let block = Block::default()
                .borders(Borders::ALL)
                .title(field.to_string())
                .style(style);
            let value = match field {
                UserField::Password1 | UserField::Password2 => Paragraph::new(str::repeat(
                    "*",
                    self.fields.get(&field).unwrap_or(&"".to_string()).len(),
                ))
                .block(block),
                _ => Paragraph::new(self.fields.get(&field).unwrap_or(&"".to_string()).clone())
                    .block(block),
            };
            f.render_widget(value, layout[i]);
        }
        Ok(())
    }

    fn set_db(&mut self, db: Option<DatabaseConnection>) {
        self.db = db;
    }
}
