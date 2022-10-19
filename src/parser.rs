use std::fmt::Display;

pub struct ParsingError;
impl Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Couldn't parse message")
    }
}

pub enum Command {
    Ping,
}

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
        _ => Err(ParsingError),
    }
}
