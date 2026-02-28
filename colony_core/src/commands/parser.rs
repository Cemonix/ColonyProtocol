use crate::commands::command::{Command, CommandError};
use crate::commands::build::BuildArgs;
use crate::commands::build_ship::BuildShipArgs;
use crate::commands::cancel::CancelArgs;
use crate::commands::fleet::FleetArgs;
use crate::commands::status::StatusArgs;

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
        "build_ship" => Ok(Command::BuildShip(BuildShipArgs::parse(command_args)?)),
        "cancel" => Ok(Command::Cancel(CancelArgs::parse(command_args)?)),
        "status" => Ok(Command::Status(StatusArgs::parse(command_args)?)),
        "map" => Ok(Command::Map),
        "ships" => Ok(Command::Ships),
        "fleets" => Ok(Command::Fleets),
        "fleet" => Ok(Command::Fleet(FleetArgs::parse(command_args)?)),
        "help" => Ok(Command::Help),
        "end_turn" | "end" => Ok(Command::EndTurn),
        _ => Err(CommandError::UnknownCommand(command_name.to_string())),
    }
}