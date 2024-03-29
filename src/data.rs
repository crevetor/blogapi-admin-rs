use async_trait::async_trait;
use color_eyre::Result;
use ratatui::{
    layout::{Constraint, Rect},
    Frame,
};
use sea_orm::DatabaseConnection;

pub mod posts;
pub mod users;

#[async_trait]
pub trait CrudData: Default + Send {
    fn headers(&self) -> Vec<String>;
    fn rows(&self) -> Vec<Vec<String>>;
    fn widths(&self) -> Vec<Constraint>;
    fn num_rows(&self) -> usize;
    fn set_db(&mut self, cnx: Option<DatabaseConnection>);
    async fn delete(&self, idx: usize) -> Result<()>;
    async fn refresh(&mut self) -> Result<()>;
    fn to_db_id(&self, idx: usize) -> i32;
}

#[derive(Default)]
enum CrudEditMode {
    #[default]
    New,
    Edit,
}

#[async_trait]
pub trait CrudRow: Default + Send {
    async fn edit(&mut self, idx: i32) -> Result<()>;
    fn new(&mut self);
    async fn save(&mut self) -> Result<()>;
    fn focus_next_field(&mut self);
    fn input(&mut self, c: char);
    fn delete_last_char(&mut self);
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()>;
    fn set_db(&mut self, db: Option<DatabaseConnection>);
}
