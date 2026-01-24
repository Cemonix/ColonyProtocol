use crate::commands::parser::Parseable;
use crate::game_state::GameState;
use crate::commands::command::{CommandEffect, CommandError};
use crate::utils;

pub struct BuildShipArgs {
    pub planet_name: String,
    pub ship_name: String,
}

impl Parseable for BuildShipArgs {
    fn parse(args: Vec<&str>) -> Result<Self, CommandError> {
        if args.len() < 2 {
            return Err(CommandError::MissingArguments {
                command: String::from("build_ship"),
                expected: String::from("build_ship <planet_name> <ship_type>"),
            });
        }
        Ok(BuildShipArgs {
            planet_name: args[0].to_string(),
            ship_name: args[1].to_string(),
        })
    }
}

pub fn execute(args: BuildShipArgs, game_state: &GameState) -> Result<CommandEffect, CommandError> {
    let planet_id = utils::name_to_id(&args.planet_name);

    // Check planet exists
    let planet = game_state.map.planets.get(&planet_id)
        .ok_or(CommandError::UnknownPlanet(args.planet_name.clone()))?;

    // Check player owns planet
    match planet.get_owner() {
        Some(owner) if owner == game_state.current_player() => {},
        Some(_) => return Err(CommandError::WrongPlanetOwner(args.planet_name.clone())),
        None => return Err(CommandError::PlanetNotOwned(args.planet_name.clone())),
    }

    // Check ship type is valid
    let ship_id = utils::name_to_id(&args.ship_name);
    let ship_def = game_state.ship_config.get(&ship_id)
        .ok_or(CommandError::UnknownShip(args.ship_name.clone()))?;

    // Check orbital_shipyard level requirement
    let shipyard_level = planet.get_structure_level(&String::from("orbital_shipyard"));
    if shipyard_level < ship_def.required_shipyard_level {
        return Err(CommandError::ShipyardLevelTooLow {
            required: ship_def.required_shipyard_level,
            current: shipyard_level,
        });
    }

    // Check planet has enough resources
    if !planet.available_resources.has_enough(&ship_def.cost) {
        return Err(CommandError::NotEnoughResources {
            planet_name: args.planet_name.clone(),
            cost: ship_def.cost.clone(),
        });
    }

    Ok(CommandEffect::BuildShip { planet_id, ship_id })
}
