use crate::commands::command::{Command, CommandError};
use crate::commands::build::BuildArgs;

pub trait Parseable {
    fn parse(args: Vec<&str>) -> Result<Self, CommandError> where Self: Sized;
}

pub fn parse(input: &str) -> Result<Command, CommandError> {
    if input.is_empty() {
        return Err(CommandError::NoCommandEntered);
    }

    let mut parts = input.split_whitespace();

    let command_name = parts.next().unwrap(); // Safe: checked non-empty
    let command_args: Vec<&str> = parts.collect();

    match command_name {
        "build" => Ok(Command::Build(BuildArgs::parse(command_args)?)),
        _ => Err(CommandError::UnknownCommand(command_name.to_string())),
    }
}