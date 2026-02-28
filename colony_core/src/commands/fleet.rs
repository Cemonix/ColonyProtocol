use crate::commands::command::{CommandEffect, CommandError};
use crate::commands::parser::Parseable;
use crate::game_state::GameState;
use crate::planet::PlanetId;
use crate::ship::{FleetId, ShipInstanceId};

#[derive(Debug)]
pub enum FleetAction {
    Create { name: String, ship_ids: Vec<ShipInstanceId> },
    Add { fleet_id: FleetId, ship_ids: Vec<ShipInstanceId> },
    Remove { fleet_id: FleetId, ship_ids: Vec<ShipInstanceId> },
    Disband { fleet_id: FleetId },
    Move { fleet_id: FleetId, target_planet: PlanetId },
    Bombard { fleet_id: FleetId },
    CancelBombard { fleet_id: FleetId },
    Colonize { fleet_id: FleetId },
}

#[derive(Debug)]
pub struct FleetArgs {
    pub action: FleetAction,
}

impl Parseable for FleetArgs {
    fn parse(args: Vec<&str>) -> Result<Self, CommandError> {
        if args.is_empty() {
            return Err(CommandError::MissingArguments {
                command: String::from("fleet"),
                expected: String::from("fleet <create|add|remove|disband|move> ..."),
            });
        }

        let action = match args[0] {
            "create" => {
                if args.len() < 3 {
                    return Err(CommandError::MissingArguments {
                        command: String::from("fleet create"),
                        expected: String::from("fleet create <name> <ship_id> [ship_id...]"),
                    });
                }
                let name = args[1].to_string();
                let ship_ids: Vec<ShipInstanceId> = args[2..].iter().map(|s| s.to_string()).collect();
                FleetAction::Create { name, ship_ids }
            }
            "add" => {
                if args.len() < 3 {
                    return Err(CommandError::MissingArguments {
                        command: String::from("fleet add"),
                        expected: String::from("fleet add <fleet_id> <ship_id> [ship_id...]"),
                    });
                }
                let fleet_id = args[1].to_string();
                let ship_ids: Vec<ShipInstanceId> = args[2..].iter().map(|s| s.to_string()).collect();
                FleetAction::Add { fleet_id, ship_ids }
            }
            "remove" => {
                if args.len() < 3 {
                    return Err(CommandError::MissingArguments {
                        command: String::from("fleet remove"),
                        expected: String::from("fleet remove <fleet_id> <ship_id> [ship_id...]"),
                    });
                }
                let fleet_id = args[1].to_string();
                let ship_ids: Vec<ShipInstanceId> = args[2..].iter().map(|s| s.to_string()).collect();
                FleetAction::Remove { fleet_id, ship_ids }
            }
            "disband" => {
                if args.len() < 2 {
                    return Err(CommandError::MissingArguments {
                        command: String::from("fleet disband"),
                        expected: String::from("fleet disband <fleet_id>"),
                    });
                }
                let fleet_id = args[1].to_string();
                FleetAction::Disband { fleet_id }
            }
            "move" => {
                if args.len() < 3 {
                    return Err(CommandError::MissingArguments {
                        command: String::from("fleet move"),
                        expected: String::from("fleet move <fleet_id> <target_planet>"),
                    });
                }
                let fleet_id = args[1].to_string();
                let target_planet = args[2].to_string();
                FleetAction::Move { fleet_id, target_planet }
            }
            "bombard" => {
                if args.len() < 2 {
                    return Err(CommandError::MissingArguments {
                        command: String::from("fleet bombard"),
                        expected: String::from("fleet bombard <fleet_id>"),
                    });
                }
                let fleet_id = args[1].to_string();
                FleetAction::Bombard { fleet_id }
            }
            "cancel-bombard" => {
                if args.len() < 2 {
                    return Err(CommandError::MissingArguments {
                        command: String::from("fleet cancel-bombard"),
                        expected: String::from("fleet cancel-bombard <fleet_id>"),
                    });
                }
                let fleet_id = args[1].to_string();
                FleetAction::CancelBombard { fleet_id }
            }
            "colonize" => {
                if args.len() < 2 {
                    return Err(CommandError::MissingArguments {
                        command: String::from("fleet colonize"),
                        expected: String::from("fleet colonize <fleet_id>"),
                    });
                }
                let fleet_id = args[1].to_string();
                FleetAction::Colonize { fleet_id }
            }
            _ => {
                return Err(CommandError::InvalidArgument {
                    command: String::from("fleet"),
                    argument: args[0].to_string(),
                    reason: String::from("valid actions are: create, add, remove, disband, move, bombard, cancel-bombard, colonize"),
                });
            }
        };

        Ok(FleetArgs { action })
    }
}

pub fn execute(args: FleetArgs, game_state: &GameState) -> Result<CommandEffect, CommandError> {
    match args.action {
        FleetAction::Create { name, ship_ids } => validate_create(&name, &ship_ids, game_state),
        FleetAction::Add { fleet_id, ship_ids } => validate_add(&fleet_id, &ship_ids, game_state),
        FleetAction::Remove { fleet_id, ship_ids } => validate_remove(&fleet_id, &ship_ids, game_state),
        FleetAction::Disband { fleet_id } => validate_disband(&fleet_id, game_state),
        FleetAction::Move { fleet_id, target_planet } => validate_move(&fleet_id, &target_planet, game_state),
        FleetAction::Bombard { fleet_id } => validate_bombard(&fleet_id, game_state),
        FleetAction::CancelBombard { fleet_id } => validate_cancel_bombard(&fleet_id, game_state),
        FleetAction::Colonize { fleet_id } => validate_colonize(&fleet_id, game_state),
    }
}

fn validate_create(
    name: &str,
    ship_ids: &[ShipInstanceId],
    game_state: &GameState,
) -> Result<CommandEffect, CommandError> {
    let current_player_id = game_state.current_player();
    let player = game_state
        .players
        .get(current_player_id)
        .expect("Current player must exist");

    // Check all ships exist and belong to player
    let mut location: Option<&PlanetId> = None;
    for ship_id in ship_ids {
        let ship = player.ships.get(ship_id).ok_or_else(|| CommandError::InvalidArgument {
            command: String::from("fleet create"),
            argument: ship_id.clone(),
            reason: String::from("ship not found"),
        })?;

        // Check ship is not already in a fleet
        if ship.fleet_id.is_some() {
            return Err(CommandError::InvalidArgument {
                command: String::from("fleet create"),
                argument: ship_id.clone(),
                reason: format!("ship is already in fleet '{}'", ship.fleet_id.as_ref().unwrap()),
            });
        }

        // Check all ships are at the same location
        match location {
            None => location = Some(&ship.location),
            Some(loc) if loc != &ship.location => {
                return Err(CommandError::InvalidArgument {
                    command: String::from("fleet create"),
                    argument: ship_id.clone(),
                    reason: format!("ship is at different location ({})", ship.location),
                });
            }
            _ => {}
        }
    }

    let location = location.expect("At least one ship required").clone();

    Ok(CommandEffect::CreateFleet {
        name: name.to_string(),
        ship_ids: ship_ids.to_vec(),
        location,
    })
}

fn validate_add(
    fleet_id: &FleetId,
    ship_ids: &[ShipInstanceId],
    game_state: &GameState,
) -> Result<CommandEffect, CommandError> {
    let current_player_id = game_state.current_player();
    let player = game_state
        .players
        .get(current_player_id)
        .expect("Current player must exist");

    // Check fleet exists
    let fleet = player.fleets.get(fleet_id).ok_or_else(|| CommandError::InvalidArgument {
        command: String::from("fleet add"),
        argument: fleet_id.clone(),
        reason: String::from("fleet not found"),
    })?;

    let fleet_location = &fleet.location;

    // Check all ships exist, belong to player, not in fleet, and at fleet's location
    for ship_id in ship_ids {
        let ship = player.ships.get(ship_id).ok_or_else(|| CommandError::InvalidArgument {
            command: String::from("fleet add"),
            argument: ship_id.clone(),
            reason: String::from("ship not found"),
        })?;

        if ship.fleet_id.is_some() {
            return Err(CommandError::InvalidArgument {
                command: String::from("fleet add"),
                argument: ship_id.clone(),
                reason: format!("ship is already in fleet '{}'", ship.fleet_id.as_ref().unwrap()),
            });
        }

        if &ship.location != fleet_location {
            return Err(CommandError::InvalidArgument {
                command: String::from("fleet add"),
                argument: ship_id.clone(),
                reason: format!(
                    "ship is at {} but fleet is at {}",
                    ship.location, fleet_location
                ),
            });
        }
    }

    Ok(CommandEffect::AddToFleet {
        fleet_id: fleet_id.clone(),
        ship_ids: ship_ids.to_vec(),
    })
}

fn validate_remove(
    fleet_id: &FleetId,
    ship_ids: &[ShipInstanceId],
    game_state: &GameState,
) -> Result<CommandEffect, CommandError> {
    let current_player_id = game_state.current_player();
    let player = game_state
        .players
        .get(current_player_id)
        .expect("Current player must exist");

    // Check fleet exists
    let fleet = player.fleets.get(fleet_id).ok_or_else(|| CommandError::InvalidArgument {
        command: String::from("fleet remove"),
        argument: fleet_id.clone(),
        reason: String::from("fleet not found"),
    })?;

    // Check all ships are in this fleet
    for ship_id in ship_ids {
        if !fleet.ships.contains(ship_id) {
            return Err(CommandError::InvalidArgument {
                command: String::from("fleet remove"),
                argument: ship_id.clone(),
                reason: format!("ship is not in fleet '{}'", fleet_id),
            });
        }
    }

    Ok(CommandEffect::RemoveFromFleet {
        fleet_id: fleet_id.clone(),
        ship_ids: ship_ids.to_vec(),
    })
}

fn validate_disband(fleet_id: &FleetId, game_state: &GameState) -> Result<CommandEffect, CommandError> {
    let current_player_id = game_state.current_player();
    let player = game_state
        .players
        .get(current_player_id)
        .expect("Current player must exist");

    // Check fleet exists
    if !player.fleets.contains_key(fleet_id) {
        return Err(CommandError::InvalidArgument {
            command: String::from("fleet disband"),
            argument: fleet_id.clone(),
            reason: String::from("fleet not found"),
        });
    }

    Ok(CommandEffect::DisbandFleet {
        fleet_id: fleet_id.clone(),
    })
}

fn validate_move(
    fleet_id: &FleetId,
    target_planet: &PlanetId,
    game_state: &GameState,
) -> Result<CommandEffect, CommandError> {
    let current_player_id = game_state.current_player();
    let player = game_state
        .players
        .get(current_player_id)
        .expect("Current player must exist");

    // Check fleet exists
    let fleet = player.fleets.get(fleet_id).ok_or_else(|| CommandError::InvalidArgument {
        command: String::from("fleet move"),
        argument: fleet_id.clone(),
        reason: String::from("fleet not found"),
    })?;

    // Check fleet is not empty
    if fleet.is_empty() {
        return Err(CommandError::InvalidArgument {
            command: String::from("fleet move"),
            argument: fleet_id.clone(),
            reason: String::from("fleet has no ships"),
        });
    }

    // Check fleet doesn't already have a pending move
    if player.has_pending_fleet_move(fleet_id) {
        return Err(CommandError::InvalidArgument {
            command: String::from("fleet move"),
            argument: fleet_id.clone(),
            reason: String::from("fleet already has a pending move"),
        });
    }

    // Check fleet is not bombarding
    if player.has_pending_fleet_bombardment(fleet_id) {
        return Err(CommandError::InvalidArgument {
            command: String::from("fleet move"),
            argument: fleet_id.clone(),
            reason: String::from("fleet is bombarding - cancel bombardment first"),
        });
    }

    // Check target planet exists
    if !game_state.map.planets.contains_key(target_planet) {
        return Err(CommandError::UnknownPlanet(target_planet.clone()));
    }

    // Check fleet is not already at target
    if &fleet.location == target_planet {
        return Err(CommandError::InvalidArgument {
            command: String::from("fleet move"),
            argument: target_planet.clone(),
            reason: String::from("fleet is already at this planet"),
        });
    }

    // Check connection exists from current location to target
    let current_location = game_state
        .map
        .planets
        .get(&fleet.location)
        .expect("Fleet location must exist");

    let connection = current_location
        .get_connections()
        .iter()
        .find(|conn| &conn.to == target_planet)
        .ok_or_else(|| CommandError::InvalidArgument {
            command: String::from("fleet move"),
            argument: target_planet.clone(),
            reason: format!("no connection from {} to {}", fleet.location, target_planet),
        })?;

    let distance = connection.distance;

    Ok(CommandEffect::MoveFleet {
        fleet_id: fleet_id.clone(),
        target_planet: target_planet.clone(),
        distance,
    })
}

fn validate_bombard(
    fleet_id: &FleetId,
    game_state: &GameState,
) -> Result<CommandEffect, CommandError> {
    let current_player_id = game_state.current_player();
    let player = game_state
        .players
        .get(current_player_id)
        .expect("Current player must exist");

    // Check fleet exists
    let fleet = player.fleets.get(fleet_id).ok_or_else(|| CommandError::InvalidArgument {
        command: String::from("fleet bombard"),
        argument: fleet_id.clone(),
        reason: String::from("fleet not found"),
    })?;

    // Check fleet is not empty
    if fleet.is_empty() {
        return Err(CommandError::InvalidArgument {
            command: String::from("fleet bombard"),
            argument: fleet_id.clone(),
            reason: String::from("fleet has no ships"),
        });
    }

    // Check fleet doesn't already have a pending bombardment
    if player.has_pending_fleet_bombardment(fleet_id) {
        return Err(CommandError::InvalidArgument {
            command: String::from("fleet bombard"),
            argument: fleet_id.clone(),
            reason: String::from("fleet is already bombarding"),
        });
    }

    // Calculate total bombardment power
    let bombardment_power = game_state.calculate_fleet_bombardment(current_player_id, fleet_id);

    if bombardment_power == 0 {
        return Err(CommandError::InvalidArgument {
            command: String::from("fleet bombard"),
            argument: fleet_id.clone(),
            reason: String::from("fleet has no bombardment capability"),
        });
    }

    // Get the planet being bombarded
    let target_planet = fleet.location.clone();
    let planet = game_state
        .map
        .planets
        .get(&target_planet)
        .expect("Fleet location must exist");

    // Check if planet is owned by someone else (can't bombard your own planet)
    match planet.get_owner() {
        Some(owner_id) if owner_id == current_player_id => {
            return Err(CommandError::InvalidArgument {
                command: String::from("fleet bombard"),
                argument: fleet_id.clone(),
                reason: String::from("cannot bombard your own planet"),
            });
        }
        None => {
            return Err(CommandError::InvalidArgument {
                command: String::from("fleet bombard"),
                argument: fleet_id.clone(),
                reason: String::from("cannot bombard neutral planets - use colonize instead"),
            });
        }
        _ => {}
    }

    Ok(CommandEffect::BombardPlanet {
        fleet_id: fleet_id.clone(),
        target_planet,
        bombardment_power,
    })
}

fn validate_cancel_bombard(
    fleet_id: &FleetId,
    game_state: &GameState,
) -> Result<CommandEffect, CommandError> {
    let current_player_id = game_state.current_player();
    let player = game_state
        .players
        .get(current_player_id)
        .expect("Current player must exist");

    // Check fleet exists
    if !player.fleets.contains_key(fleet_id) {
        return Err(CommandError::InvalidArgument {
            command: String::from("fleet cancel-bombard"),
            argument: fleet_id.clone(),
            reason: String::from("fleet not found"),
        });
    }

    // Check fleet is actually bombarding
    if !player.has_pending_fleet_bombardment(fleet_id) {
        return Err(CommandError::InvalidArgument {
            command: String::from("fleet cancel-bombard"),
            argument: fleet_id.clone(),
            reason: String::from("fleet is not bombarding"),
        });
    }

    Ok(CommandEffect::CancelBombard {
        fleet_id: fleet_id.clone(),
    })
}

fn validate_colonize(
    fleet_id: &FleetId,
    game_state: &GameState,
) -> Result<CommandEffect, CommandError> {
    let current_player_id = game_state.current_player();
    let player = game_state
        .players
        .get(current_player_id)
        .expect("Current player must exist");

    // Check fleet exists
    let fleet = player.fleets.get(fleet_id).ok_or_else(|| CommandError::InvalidArgument {
        command: String::from("fleet colonize"),
        argument: fleet_id.clone(),
        reason: String::from("fleet not found"),
    })?;

    // Check fleet is not empty
    if fleet.is_empty() {
        return Err(CommandError::InvalidArgument {
            command: String::from("fleet colonize"),
            argument: fleet_id.clone(),
            reason: String::from("fleet has no ships"),
        });
    }

    // Check fleet has an ark ship
    let has_ark = fleet.ships.iter()
        .any(|ship_id| {
            player.ships.get(ship_id)
                .map(|ship| ship.ship_type == "ark")
                .unwrap_or(false)
        });

    if !has_ark {
        return Err(CommandError::InvalidArgument {
            command: String::from("fleet colonize"),
            argument: fleet_id.clone(),
            reason: String::from("fleet requires an ark ship to colonize"),
        });
    }

    // Get the planet being colonized
    let target_planet = fleet.location.clone();
    let planet = game_state
        .map
        .planets
        .get(&target_planet)
        .expect("Fleet location must exist");

    // Check planet is not already owned by current player
    match planet.get_owner() {
        Some(owner_id) if owner_id == current_player_id => {
            return Err(CommandError::InvalidArgument {
                command: String::from("fleet colonize"),
                argument: fleet_id.clone(),
                reason: String::from("you already own this planet"),
            });
        }
        _ => {}
    }

    // Check planet shields are at 0
    if planet.get_shield_hp() > 0 {
        return Err(CommandError::InvalidArgument {
            command: String::from("fleet colonize"),
            argument: fleet_id.clone(),
            reason: format!(
                "planet shields must be destroyed first (current: {} HP)",
                planet.get_shield_hp()
            ),
        });
    }

    Ok(CommandEffect::ColonizePlanet {
        fleet_id: fleet_id.clone(),
        planet_id: target_planet,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::parser::Parseable;

    #[test]
    fn test_parse_bombard_command() {
        let args = vec!["bombard", "fleet_alpha"];
        let result = FleetArgs::parse(args);

        assert!(result.is_ok());
        let parsed = result.unwrap();
        match parsed.action {
            FleetAction::Bombard { fleet_id } => {
                assert_eq!(fleet_id, "fleet_alpha");
            }
            _ => panic!("Expected Bombard action"),
        }
    }

    #[test]
    fn test_parse_bombard_missing_fleet_id() {
        let args = vec!["bombard"];
        let result = FleetArgs::parse(args);

        assert!(result.is_err());
        match result.unwrap_err() {
            CommandError::MissingArguments { command, expected } => {
                assert_eq!(command, "fleet bombard");
                assert!(expected.contains("fleet bombard <fleet_id>"));
            }
            _ => panic!("Expected MissingArguments error"),
        }
    }

    #[test]
    fn test_parse_bombard_in_help_message() {
        let args = vec!["invalid_action"];
        let result = FleetArgs::parse(args);

        assert!(result.is_err());
        match result.unwrap_err() {
            CommandError::InvalidArgument { command, argument, reason } => {
                assert_eq!(command, "fleet");
                assert_eq!(argument, "invalid_action");
                assert!(reason.contains("bombard"));
            }
            _ => panic!("Expected InvalidArgument error"),
        }
    }

    #[test]
    fn test_parse_cancel_bombard_command() {
        let args = vec!["cancel-bombard", "fleet_alpha"];
        let result = FleetArgs::parse(args);

        assert!(result.is_ok());
        let parsed = result.unwrap();
        match parsed.action {
            FleetAction::CancelBombard { fleet_id } => {
                assert_eq!(fleet_id, "fleet_alpha");
            }
            _ => panic!("Expected CancelBombard action"),
        }
    }

    #[test]
    fn test_parse_cancel_bombard_missing_fleet_id() {
        let args = vec!["cancel-bombard"];
        let result = FleetArgs::parse(args);

        assert!(result.is_err());
        match result.unwrap_err() {
            CommandError::MissingArguments { command, expected } => {
                assert_eq!(command, "fleet cancel-bombard");
                assert!(expected.contains("fleet cancel-bombard <fleet_id>"));
            }
            _ => panic!("Expected MissingArguments error"),
        }
    }

    #[test]
    fn test_parse_cancel_bombard_in_help_message() {
        let args = vec!["invalid_action"];
        let result = FleetArgs::parse(args);

        assert!(result.is_err());
        match result.unwrap_err() {
            CommandError::InvalidArgument { command, argument, reason } => {
                assert_eq!(command, "fleet");
                assert_eq!(argument, "invalid_action");
                assert!(reason.contains("cancel-bombard"));
            }
            _ => panic!("Expected InvalidArgument error"),
        }
    }

    #[test]
    fn test_parse_colonize_command() {
        let args = vec!["colonize", "fleet_alpha"];
        let result = FleetArgs::parse(args);

        assert!(result.is_ok());
        let parsed = result.unwrap();
        match parsed.action {
            FleetAction::Colonize { fleet_id } => {
                assert_eq!(fleet_id, "fleet_alpha");
            }
            _ => panic!("Expected Colonize action"),
        }
    }

    #[test]
    fn test_parse_colonize_missing_fleet_id() {
        let args = vec!["colonize"];
        let result = FleetArgs::parse(args);

        assert!(result.is_err());
        match result.unwrap_err() {
            CommandError::MissingArguments { command, expected } => {
                assert_eq!(command, "fleet colonize");
                assert!(expected.contains("fleet colonize <fleet_id>"));
            }
            _ => panic!("Expected MissingArguments error"),
        }
    }

    #[test]
    fn test_parse_colonize_in_help_message() {
        let args = vec!["invalid_action"];
        let result = FleetArgs::parse(args);

        assert!(result.is_err());
        match result.unwrap_err() {
            CommandError::InvalidArgument { command, argument, reason } => {
                assert_eq!(command, "fleet");
                assert_eq!(argument, "invalid_action");
                assert!(reason.contains("colonize"));
            }
            _ => panic!("Expected InvalidArgument error"),
        }
    }
}
