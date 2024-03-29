use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mode {
    #[default]
    Posts,
    Tags,
    Users,
}

#[derive(Debug)]
pub struct InvalidValue;
impl TryFrom<usize> for Mode {
    type Error = InvalidValue;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Mode::Posts),
            1 => Ok(Mode::Tags),
            2 => Ok(Mode::Users),
            _ => Err(InvalidValue),
        }
    }
}

impl Mode {
    pub fn next(&self) -> Self {
        match *self {
            Mode::Posts => Mode::Tags,
            Mode::Tags => Mode::Users,
            Mode::Users => Mode::Posts,
        }
    }
}

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Default, Eq, PartialEq)]
pub enum CrudMode {
    #[default]
    List,
    Edit,
    New,
}
