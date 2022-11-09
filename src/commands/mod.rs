use std::fmt::Display;

pub mod help;
pub mod parser;
pub mod ping;
pub mod roll;
pub mod utils;

/// The type of commands that can be used with this bot
#[derive(Debug)]
pub enum Command {
    Help,
    Ping,
    Roll,
}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
