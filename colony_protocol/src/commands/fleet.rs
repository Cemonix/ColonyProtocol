use crate::commands::command::{CommandEffect, CommandError};
use crate::commands::parser::Parseable;
use crate::game_state::GameState;
use crate::planet::PlanetId;
use crate::ship::{FleetId, ShipInstanceId};

pub enum FleetAction {
    Create { name: String, ship_ids: Vec<ShipInstanceId> },
    Add { fleet_id: FleetId, ship_ids: Vec<ShipInstanceId> },
    Remove { fleet_id: FleetId, ship_ids: Vec<ShipInstanceId> },
    Disband { fleet_id: FleetId },
}

pub struct FleetArgs {
    pub action: FleetAction,
}

impl Parseable for FleetArgs {
    fn parse(args: Vec<&str>) -> Result<Self, CommandError> {
        if args.is_empty() {
            return Err(CommandError::MissingArguments {
                command: String::from("fleet"),
                expected: String::from("fleet <create|add|remove|disband> ..."),
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
            _ => {
                return Err(CommandError::InvalidArgument {
                    command: String::from("fleet"),
                    argument: args[0].to_string(),
                    reason: String::from("valid actions are: create, add, remove, disband"),
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
