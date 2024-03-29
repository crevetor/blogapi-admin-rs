use std::collections::HashMap;

use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::Rect,
    Frame,
};
use sea_orm::{Database, DatabaseConnection};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::{
    action::Action,
    area::Area,
    components::{
        crudedit::CrudEdit, crudlist::CrudList, footer::Footer, tabbar::TabBar, Component,
    },
    config::Config,
    data::{
        posts::{PostEdit, Posts},
        users::{UserEdit, Users},
    },
    mode::Mode,
    tui,
};

pub struct App {
    pub config: Config,
    pub db: Option<DatabaseConnection>,
    pub tick_rate: f64,
    pub frame_rate: f64,
    pub components: Vec<Box<dyn Component + Send>>,
    pub should_quit: bool,
    pub should_suspend: bool,
    pub mode: Mode,
    pub last_tick_key_events: Vec<KeyEvent>,
}

impl App {
    pub fn new(tick_rate: f64, frame_rate: f64) -> Result<Self> {
        let config = Config::new()?;
        let mode = Mode::default();
        Ok(Self {
            tick_rate,
            frame_rate,
            components: vec![
                Box::new(TabBar::new(mode)),
                Box::new(Footer::new()),
                Box::new(CrudList::new(Posts::default(), Mode::Posts)),
                Box::new(CrudEdit::new(PostEdit::default(), Mode::Posts)),
                Box::new(CrudList::new(Users::default(), Mode::Users)),
                Box::new(CrudEdit::new(UserEdit::default(), Mode::Users)),
            ],
            should_quit: false,
            should_suspend: false,
            config,
            db: None,
            mode,
            last_tick_key_events: Vec::new(),
        })
    }

    pub fn draw(&mut self, f: &mut Frame<'_>) -> Result<()> {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Max(2),
                Constraint::Percentage(80),
                Constraint::Max(2),
            ])
            .split(f.size());

        for c in self.components.iter_mut() {
            let layout_idx = match c.component_type() {
                Area::Header => 0,
                Area::Footer => 2,
                Area::Main => 1,
            };
            if c.focused() {
                c.draw(f, layout[layout_idx])?;
            }
        }
        Ok(())
    }

    pub async fn run(&mut self) -> Result<()> {
        let (action_tx, mut action_rx) = mpsc::unbounded_channel();
        action_tx.send(Action::TabChange(Mode::default()))?;

        let mut tui = tui::Tui::new()?
            .tick_rate(self.tick_rate)
            .frame_rate(self.frame_rate);
        // tui.mouse(true);
        tui.enter()?;

        self.db = Some(Database::connect(self.config.db.clone()).await?);
        for component in self.components.iter_mut() {
            component.register_db_handler(self.db.clone())?;
        }

        for component in self.components.iter_mut() {
            component.register_action_handler(action_tx.clone())?;
        }

        for component in self.components.iter_mut() {
            component.register_config_handler(self.config.clone())?;
        }

        for component in self.components.iter_mut() {
            component.init(tui.size()?)?;
        }

        loop {
            if let Some(e) = tui.next().await {
                match e {
                    tui::Event::Quit => action_tx.send(Action::Quit)?,
                    tui::Event::Tick => action_tx.send(Action::Tick)?,
                    tui::Event::Render => action_tx.send(Action::Render)?,
                    tui::Event::Resize(x, y) => action_tx.send(Action::Resize(x, y))?,
                    tui::Event::Key(key) => {
                        if let Some(keymap) = self.config.keybindings.get(&self.mode) {
                            if let Some(action) = keymap.get(&vec![key]) {
                                log::info!("Got action: {action:?}");
                                action_tx.send(action.clone())?;
                            } else {
                                // If the key was not handled as a single key action,
                                // then consider it for multi-key combinations.
                                self.last_tick_key_events.push(key);

                                // Check for multi-key combinations
                                if let Some(action) = keymap.get(&self.last_tick_key_events) {
                                    log::info!("Got action: {action:?}");
                                    action_tx.send(action.clone())?;
                                }
                            }
                        };
                    }
                    _ => {}
                }
                for component in self.components.iter_mut() {
                    if let Some(action) = component.handle_events(Some(e.clone()))? {
                        action_tx.send(action)?;
                    }
                }
            }

            while let Ok(action) = action_rx.try_recv() {
                if action != Action::Tick && action != Action::Render {
                    log::debug!("{action:?}");
                }
                match action {
                    Action::Tick => {
                        self.last_tick_key_events.drain(..);
                    }
                    Action::Quit => self.should_quit = true,
                    Action::Suspend => self.should_suspend = true,
                    Action::Resume => self.should_suspend = false,
                    Action::Resize(w, h) => {
                        tui.resize(Rect::new(0, 0, w, h))?;
                        tui.draw(|f| {
                            if let Err(e) = self.draw(f) {
                                action_tx
                                    .send(Action::Error(format!(
                                        "Error while trying to draw: {:?}",
                                        e
                                    )))
                                    .unwrap();
                            }
                        })?;
                    }
                    Action::Render => {
                        tui.draw(|f| {
                            if let Err(e) = self.draw(f) {
                                action_tx
                                    .send(Action::Error(format!(
                                        "Error while trying to draw: {:?}",
                                        e
                                    )))
                                    .unwrap();
                            }
                        })?;
                    }
                    Action::TabChange(mode) => self.mode = mode,
                    Action::NextTab => action_tx.send(Action::TabChange(self.mode.next()))?,
                    _ => {}
                }
                for component in self.components.iter_mut() {
                    if component.focused() || action.is_focus_changed() {
                        if let Some(action) = component.update(action.clone()).await? {
                            action_tx.send(action)?
                        };
                    }
                }
            }
            if self.should_suspend {
                tui.suspend()?;
                action_tx.send(Action::Resume)?;
                tui = tui::Tui::new()?
                    .tick_rate(self.tick_rate)
                    .frame_rate(self.frame_rate);
                // tui.mouse(true);
                tui.enter()?;
            } else if self.should_quit {
                if let Some(cnx) = &self.db {
                    cnx.clone().close().await?;
                    self.db = None;
                }
                tui.stop()?;
                break;
            }
        }
        tui.exit()?;
        Ok(())
    }
}
