use async_trait::async_trait;
use color_eyre::eyre::Result;
use ratatui::{prelude::*, widgets::*};

use super::Component;
use crate::{action::Action, area::Area, mode::Mode, style::TabStyle, tui::Frame};

pub struct TabBar<'a> {
    pub tabbar: Tabs<'a>,
}

impl TabBar<'_> {
    pub fn new(curmode: Mode) -> Self {
        TabBar {
            tabbar: Tabs::new(
                (0..3)
                    .map(|x| Mode::try_from(x).unwrap().to_string())
                    .collect(),
            )
            .style(TabStyle::normal())
            .highlight_style(TabStyle::highlighted())
            .select(curmode as usize),
        }
    }
}

#[async_trait]
impl Component for TabBar<'_> {
    async fn update(&mut self, action: Action) -> Result<Option<Action>> {
        if let Action::TabChange(newmode) = action {
            self.tabbar = self.tabbar.clone().select(newmode as usize);
            return Ok(Some(Action::Render));
        }
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
        f.render_widget(self.tabbar.clone(), area);
        Ok(())
    }

    fn focused(&self) -> bool {
        true
    }

    fn component_type(&self) -> Area {
        Area::Header
    }
}
