use crate::commands::parser::Parseable;
use crate::game_state::GameState;
use crate::commands::command::{CommandEffect, CommandError};
use crate::utils;

pub struct BuildArgs {
    pub planet_name: String,
    pub structure_name: String,
}

impl Parseable for BuildArgs {
    fn parse(args: Vec<&str>) -> Result<Self, CommandError> {
        if args.len() < 2 {
            return Err(CommandError::MissingArguments {
                command: String::from("build"),
                expected: String::from("build <planet_name> <structure_name>"),
            });
        }
        Ok(BuildArgs {
            planet_name: args[0].to_string(),
            structure_name: args[1].to_string(),
        })
    }
}

pub fn execute(args: BuildArgs, game_state: &GameState) -> Result<CommandEffect, CommandError> {
    // Check planet exists
    let planet_id = utils::name_to_id(&args.planet_name);
    
    let planet = game_state.planets.get(&planet_id)
    .ok_or(CommandError::UnknownPlanet(args.planet_name.clone()))?;

    // Check player owns planet
    match planet.get_owner() {
        Some(owner) if owner == game_state.current_player() => {},
        Some(_) => return Err(CommandError::WrongPlanetOwner(args.planet_name.clone())),
        None => return Err(CommandError::PlanetNotOwned(args.planet_name.clone())),
    }

    // Check structure type is valid
    let structure_id = utils::name_to_id(&args.structure_name);
    game_state.structure_config.get(&structure_id).ok_or(
        CommandError::UnknownStructure(args.structure_name.clone())
    )?;

    Ok(CommandEffect::BuildStructure {planet_id, structure_id})
}
