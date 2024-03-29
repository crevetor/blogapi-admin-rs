use std::{collections::HashMap, fmt::Display};

use async_trait::async_trait;
use blogapi::models::_entities::posts::{ActiveModel, Entity, Model};
use blogapi::models::_entities::users::{
    ActiveModel as UserActive, Entity as UserEntity, Model as UserModel,
};
use color_eyre::eyre::{self, eyre, Result};
use color_eyre::owo_colors::OwoColorize;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::style::{Color, Stylize};
use ratatui::widgets::{Row, Table, TableState};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::Rect,
    style::Style,
    widgets::{Block, Borders, List, ListState, Paragraph},
};
use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, EntityTrait, ModelTrait, Set};
use serde::Serializer;

use super::Component;
use crate::style::{FormStyle, TableStyle};
use crate::{
    action::Action,
    area::Area,
    data::CrudData,
    mode::{CrudMode, Mode},
    tui::{self, Frame},
};

#[derive(Default)]
pub struct CrudList<'a, T: CrudData + Send> {
    mode: Mode,
    focused: bool,
    db: Option<DatabaseConnection>,
    data: T,
    table: Table<'a>,
    test: Box<String>,
    table_state: TableState,
}

impl<T: CrudData + Default> CrudList<'_, T> {
    pub fn new(data: T, mode: Mode) -> Self {
        CrudList {
            data,
            mode,
            ..Default::default()
        }
    }

    async fn populate_table(&mut self) -> Result<()> {
        self.data.refresh().await?;
        let header = Row::new(self.data.headers()).style(TableStyle::header());
        let rows: Vec<Row> = self
            .data
            .rows()
            .iter()
            .cloned()
            .map(|x| Row::new(x))
            .collect();
        let widths = self.data.widths();
        self.table = Table::new(rows, widths)
            .style(TableStyle::normal())
            .highlight_style(TableStyle::highlighted())
            .header(header);

        if self.data.num_rows() > 0 {
            self.table_state.select(Some(0));
        }

        return Ok(());
    }

    async fn delete_selected_post(&mut self) -> Result<()> {
        if let Some(idx) = self.table_state.selected() {
            self.data.delete(idx).await?;
        }
        return Ok(());
    }

    fn select_next(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i < self.data.num_rows() - 1 {
                    i + 1
                } else {
                    i
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    fn select_prev(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i > 0 {
                    i - 1
                } else {
                    0
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }
}

#[async_trait]
impl<'a, T: CrudData + Default + Send> Component for CrudList<'a, T> {
    fn register_db_handler(&mut self, db: Option<DatabaseConnection>) -> Result<()> {
        self.db = db;
        self.data.set_db(self.db.clone());
        Ok(())
    }

    async fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::TabChange(newmode) => {
                self.focused = newmode == self.mode;
                if self.focused {
                    self.populate_table().await?;
                    return Ok(Some(Action::Render));
                }
            }
            Action::Delete => {
                self.delete_selected_post().await?;
                self.populate_table().await?;
                return Ok(Some(Action::Render));
            }
            Action::New => {
                self.focused = false;
                return Ok(Some(Action::CrudNew(self.mode)));
            }
            Action::Edit => {
                if let Some(idx) = self.table_state.selected() {
                    self.focused = false;
                    return Ok(Some(Action::CrudEdit(self.mode, self.data.to_db_id(idx))));
                }
            }
            Action::Up => {
                self.select_prev();
                return Ok(Some(Action::Render));
            }
            Action::Down => {
                self.select_next();
                return Ok(Some(Action::Render));
            }
            _ => (),
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        f.render_stateful_widget(self.table.clone(), area, &mut self.table_state);
        Ok(())
    }

    fn component_type(&self) -> Area {
        Area::Main
    }

    fn focused(&self) -> bool {
        self.focused
    }
}
