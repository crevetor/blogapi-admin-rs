use std::{
    fmt::{self, Display},
    string::ToString,
};

use serde::{
    de::{self, Deserializer, Visitor},
    Deserialize, Serialize,
};
use strum::Display;

use crate::mode::Mode;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Display, Deserialize)]
pub enum Action {
    Tick,
    Render,
    Resize(u16, u16),
    TabChange(Mode),
    CrudEdit(Mode, i32),
    CrudNew(Mode),
    Edit,
    New,
    NextTab,
    Delete,
    Save,
    Down,
    Up,
    Select,
    Tab,
    Suspend,
    Resume,
    Quit,
    Refresh,
    Error(String),
    Back,
    Help,
}

impl Action {
    pub fn is_focus_changed(&self) -> bool {
        match self {
            Action::TabChange(_) => true,
            Action::CrudEdit(_, _) => true,
            Action::CrudNew(_) => true,
            _ => false,
        }
    }
}
