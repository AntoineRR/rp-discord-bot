use std::fmt::Display;

use strum_macros::EnumIter;

#[derive(Debug, Clone, Copy, EnumIter)]
pub enum Stats {
    Agility,
    Strength,
    Stamina,
}

impl Display for Stats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
