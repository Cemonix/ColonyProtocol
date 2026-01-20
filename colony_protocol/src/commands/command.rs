use crate::commands::status::{self, StatusArgs};
use crate::game_state::GameState;
use crate::commands::build::{self, BuildArgs};
use crate::commands::cancel::{self, CancelArgs};
use crate::commands::map;
use crate::planet::PlanetId;
use crate::structure::StructureId;

#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("No command was entered")]
    NoCommandEntered,

    #[error("Missing arguments for command: {command}. Expected: {expected}")]
    MissingArguments {
        command: String,
        expected: String
    },

    #[error("Unknown command: {0}")]
    UnknownCommand(String),

    #[error("Planet {0} does not exist")]
    UnknownPlanet(String),
    
    #[error("Structure {0} does not exist")]
    UnknownStructure(String),

    #[error("Planet {0} is not owned by anyone")]
    PlanetNotOwned(String),

    #[error("Planet is not owned by {0}")]
    WrongPlanetOwner(String),

    #[error("Invalid argument '{argument}' for command '{command}': {reason}")]
    InvalidArgument {
        command: String,
        argument: String,
        reason: String,
    },
}

pub enum Command {
    Build(BuildArgs),
    Cancel(CancelArgs),
    Status(StatusArgs),
    Map,
}

impl Command {
    pub fn execute(self, game_state: &GameState) -> Result<CommandEffect, CommandError> {
        match self {
            Command::Build(args) => build::execute(args, game_state),
            Command::Cancel(args) => cancel::execute(args, game_state),
            Command::Status(args) => status::execute(args, game_state),
            Command::Map => map::execute(game_state),
        }
    }
}

pub enum CommandEffect {
    None { message: String },
    BuildStructure { planet_id: PlanetId, structure_id: StructureId },
    CancelAction { planet_id: PlanetId },
}