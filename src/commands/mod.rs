pub mod ping;
pub mod roll;
pub mod utils;

/// The type of commands that can be used with this bot
pub enum Command {
    Ping,
    Roll,
}
