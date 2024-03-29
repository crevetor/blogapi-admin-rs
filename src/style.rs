use ratatui::style::{Color, Style, Stylize as _};

pub(crate) struct TabStyle;

impl TabStyle {
    pub(crate) fn normal() -> Style {
        Style::new()
    }
    pub(crate) fn highlighted() -> Style {
        Self::normal().reversed()
    }
}

pub(crate) struct TableStyle;
impl TableStyle {
    pub(crate) fn normal() -> Style {
        Style::new()
    }

    pub(crate) fn highlighted() -> Style {
        Self::normal().reversed()
    }

    pub(crate) fn header() -> Style {
        Style::new().fg(Color::LightGreen)
    }
}

pub(crate) struct FormStyle;
impl FormStyle {
    pub(crate) fn normal() -> Style {
        Style::new()
    }

    pub(crate) fn highlighted() -> Style {
        Self::normal().fg(Color::Blue)
    }
}
