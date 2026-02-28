use crate::commands::build::{self, BuildArgs};
use crate::commands::build_ship::{self, BuildShipArgs};
use crate::commands::cancel::{self, CancelArgs};
use crate::commands::end_turn;
use crate::commands::fleet::{self, FleetArgs};
use crate::commands::fleets;
use crate::commands::help;
use crate::commands::map;
use crate::commands::ships;
use crate::commands::status::{self, StatusArgs};
use crate::configs::ship_config::ShipId;
use crate::game_state::GameState;
use crate::planet::PlanetId;
use crate::resources::Resources;
use crate::ship::{FleetId, ShipInstanceId};
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

    #[error("Ship type {0} does not exist")]
    UnknownShip(String),

    #[error("Shipyard level too low: requires level {required}, current level {current}")]
    ShipyardLevelTooLow {
        required: u16,
        current: u16,
    },

    #[error("Planet {planet_name} does not have enough resources. Cost: {cost}")]
    NotEnoughResources {
        planet_name: String,
        cost: Resources,
    },
}

pub enum Command {
    Build(BuildArgs),
    BuildShip(BuildShipArgs),
    Cancel(CancelArgs),
    Status(StatusArgs),
    Map,
    Ships,
    Fleets,
    Fleet(FleetArgs),
    Help,
    EndTurn,
}

impl Command {
    pub fn execute(self, game_state: &GameState) -> Result<CommandEffect, CommandError> {
        match self {
            Command::Build(args) => build::execute(args, game_state),
            Command::BuildShip(args) => build_ship::execute(args, game_state),
            Command::Cancel(args) => cancel::execute(args, game_state),
            Command::Status(args) => status::execute(args, game_state),
            Command::Map => map::execute(game_state),
            Command::Ships => ships::execute(game_state),
            Command::Fleets => fleets::execute(game_state),
            Command::Fleet(args) => fleet::execute(args, game_state),
            Command::Help => help::execute(game_state),
            Command::EndTurn => end_turn::execute(game_state),
        }
    }
}

pub enum CommandEffect {
    None { message: String },
    BuildStructure { planet_id: PlanetId, structure_id: StructureId },
    BuildShip { planet_id: PlanetId, ship_id: ShipId },
    CancelAction { planet_id: PlanetId },
    CreateFleet { name: String, ship_ids: Vec<ShipInstanceId>, location: PlanetId },
    AddToFleet { fleet_id: FleetId, ship_ids: Vec<ShipInstanceId> },
    RemoveFromFleet { fleet_id: FleetId, ship_ids: Vec<ShipInstanceId> },
    DisbandFleet { fleet_id: FleetId },
    MoveFleet { fleet_id: FleetId, target_planet: PlanetId, distance: u8 },
    BombardPlanet { fleet_id: FleetId, target_planet: PlanetId, bombardment_power: u32 },
    CancelBombard { fleet_id: FleetId },
    ColonizePlanet { fleet_id: FleetId, planet_id: PlanetId },
    EndTurn { player_name: String },
}