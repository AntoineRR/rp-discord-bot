use std::fmt::Display;

use crate::commands::Command;

/// Returned when the parsing of a command failed
pub struct ParsingError;

impl Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Couldn't parse message")
    }
}

/// Parse a message to find out if it was a command and return it
pub fn parse(message: &str) -> Result<Command, ParsingError> {
    if !message.starts_with('!') {
        return Err(ParsingError);
    }
    // Remove the "!" and separate command from arguments
    let command_and_args: Vec<&str> = message[1..].split(' ').collect();
    if command_and_args.is_empty() {
        return Err(ParsingError);
    }
    // Match the command to the enum
    match command_and_args[0] {
        "ping" => Ok(Command::Ping),
        "roll" => Ok(Command::Roll),
        _ => Err(ParsingError),
    }
}
