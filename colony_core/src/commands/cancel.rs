use crate::commands::parser::Parseable;
use crate::game_state::GameState;
use crate::commands::command::{CommandEffect, CommandError};
use crate::utils;

pub struct CancelArgs {
    pub planet_name: String,
}

impl Parseable for CancelArgs {
    fn parse(args: Vec<&str>) -> Result<Self, CommandError> {
        if args.is_empty() {
            return Err(CommandError::MissingArguments {
                command: String::from("cancel"),
                expected: String::from("cancel <planet_name>"),
            });
        }
        Ok(CancelArgs {
            planet_name: args[0].to_string(),
        })
    }
}

pub fn execute(args: CancelArgs, game_state: &GameState) -> Result<CommandEffect, CommandError> {
    // Check planet exists
    let planet_id = utils::name_to_id(&args.planet_name);

    let planet = game_state.map.planets.get(&planet_id)
        .ok_or(CommandError::UnknownPlanet(args.planet_name.clone()))?;

    // Check player owns planet
    match planet.get_owner() {
        Some(owner) if owner == game_state.current_player() => {},
        Some(_) => return Err(CommandError::WrongPlanetOwner(args.planet_name.clone())),
        None => return Err(CommandError::PlanetNotOwned(args.planet_name.clone())),
    }

    // Check if there's a pending action on this planet
    let player = game_state.players.get(game_state.current_player())
        .expect("Current player must exist");

    if !player.has_pending_action_on_planet(&planet_id) {
        return Err(CommandError::InvalidArgument {
            command: String::from("cancel"),
            argument: args.planet_name.clone(),
            reason: String::from("No pending action on this planet"),
        });
    }

    Ok(CommandEffect::CancelAction { planet_id })
}
