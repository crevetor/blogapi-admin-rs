use async_trait::async_trait;
use color_eyre::eyre::Result;
use ratatui::{prelude::*, widgets::*};

use super::Component;
use crate::{
    action::Action,
    area::Area,
    config::{key_event_to_string, Config},
    mode::Mode,
    tui::Frame,
};

#[derive(Default)]
pub struct Footer {
    mode: Mode,
    config: Config,
}

impl Footer {
    pub fn new() -> Self {
        Footer::default()
    }

    fn get_keybindings(&self) -> String {
        let mut ret = String::new();
        if let Some(bindings) = self.config.keybindings.get(&self.mode) {
            for (i, (events, action)) in bindings.iter().enumerate() {
                ret.push_str(
                    &events
                        .iter()
                        .map(|x| key_event_to_string(x))
                        .collect::<Vec<String>>()
                        .join(","),
                );
                ret.push_str(" : ");
                ret.push_str(&action.to_string());
                if i != bindings.len() - 1 {
                    ret.push_str(", ");
                }
            }
        }

        ret
    }
}

#[async_trait]
impl Component for Footer {
    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    async fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Action::TabChange(newmode) = action {
            self.mode = newmode;
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        f.render_widget(
            Paragraph::new(self.get_keybindings())
                .block(Block::new().borders(Borders::TOP))
                .white(),
            area,
        );
        Ok(())
    }

    fn focused(&self) -> bool {
        true
    }

    fn component_type(&self) -> Area {
        Area::Footer
    }
}
