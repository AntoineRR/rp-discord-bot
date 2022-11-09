use anyhow::{bail, Result};
use thiserror::Error;

use crate::commands::Command;

/// Returned when the parsing of a command failed
#[derive(Debug, Error)]
pub enum ParsingError {
    #[error("Not a command")]
    NoCommand,
    #[error("Unknown command")]
    UnknownCommand,
}

/// Parse a message to find out if it was a command and return it
pub fn parse(message: &str) -> Result<Command> {
    if !message.starts_with('!') {
        bail!(ParsingError::NoCommand);
    }
    // Remove the "!" and separate command from arguments
    let command_and_args: Vec<&str> = message[1..].split(' ').collect();
    if command_and_args.is_empty() {
        bail!(ParsingError::NoCommand);
    }
    // Match the command to the enum
    match command_and_args[0] {
        "help" => Ok(Command::Help),
        "ping" => Ok(Command::Ping),
        "roll" => Ok(Command::Roll),
        _ => bail!(ParsingError::UnknownCommand),
    }
}
