use crate::commands::parser::Parseable;
use crate::game_state::GameState;
use crate::commands::command::{CommandEffect, CommandError};
use crate::utils;

pub struct UpgradeArgs {
    pub planet_name: String,
    pub structure_name: String,
}

impl Parseable for UpgradeArgs {
    fn parse(args: Vec<&str>) -> Result<Self, CommandError> {
        if args.len() < 2 {
            return Err(CommandError::MissingArguments {
                command: String::from("upgrade"),
                expected: String::from("upgrade <planet_name> <structure_name>"),
            });
        }
        Ok(UpgradeArgs {
            planet_name: args[0].to_string(),
            structure_name: args[1].to_string(),
        })
    }
}

pub fn execute(args: UpgradeArgs, game_state: &GameState) -> Result<CommandEffect, CommandError> {
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

    // Check structure exists on planet
    let structure_id = utils::name_to_id(&args.structure_name);
    if !planet.get_structures().contains_key(&structure_id) {
        return Err(CommandError::InvalidArgument {
            command: String::from("upgrade"),
            argument: args.structure_name.clone(),
            reason: format!("Structure '{}' does not exist on planet '{}'", args.structure_name, args.planet_name),
        });
    }

    Ok(CommandEffect::UpgradeStructure { planet_id, structure_id })
}
