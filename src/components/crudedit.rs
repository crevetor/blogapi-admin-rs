use crate::tui::Event;
use async_trait::async_trait;
use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use ratatui::{layout::Rect, Frame};
use sea_orm::DatabaseConnection;

use crate::{action::Action, area::Area, data::CrudRow, mode::Mode};

use super::Component;

#[derive(Default)]
pub struct CrudEdit<T: CrudRow + Send> {
    mode: Mode,
    focused: bool,
    db: Option<DatabaseConnection>,
    data: T,
}

impl<T: CrudRow + Send> CrudEdit<T> {
    pub fn new(data: T, mode: Mode) -> Self {
        Self {
            data,
            mode,
            ..Default::default()
        }
    }
}

#[async_trait]
impl<T: CrudRow + Send> Component for CrudEdit<T> {
    fn register_db_handler(&mut self, db: Option<DatabaseConnection>) -> Result<()> {
        self.db = db;
        self.data.set_db(self.db.clone());
        Ok(())
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
        if !key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char(c) => {
                    self.data.input(c);
                    return Ok(Some(Action::Render));
                }
                KeyCode::Backspace => {
                    self.data.delete_last_char();
                    return Ok(Some(Action::Render));
                }
                KeyCode::Enter => {
                    self.data.input('\n');
                    return Ok(Some(Action::Render));
                }
                _ => (),
            }
        }
        Ok(None)
    }

    async fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::CrudEdit(mode, idx) => {
                if mode == self.mode {
                    self.data.edit(idx).await?;
                    self.focused = true;
                }
            }
            Action::CrudNew(mode) => {
                if mode == self.mode {
                    self.data.new();
                    self.focused = true;
                }
            }
            Action::Save => {
                self.data.save().await?;
                self.focused = false;
                return Ok(Some(Action::TabChange(self.mode)));
            }
            Action::Back => {
                self.focused = false;
                return Ok(Some(Action::TabChange(self.mode)));
            }
            Action::Tab => self.data.focus_next_field(),
            _ => (),
        }
        Ok(None)
    }

    fn focused(&self) -> bool {
        self.focused
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        self.data.draw(f, area)
    }

    fn component_type(&self) -> Area {
        Area::Main
    }
}
