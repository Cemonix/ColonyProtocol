use std::collections::HashMap;

use crate::commands::command::{CommandEffect, CommandError};
use crate::game_state::GameState;
use crate::planet::PlanetId;

pub fn execute(game_state: &GameState) -> Result<CommandEffect, CommandError> {
    let current_player_id = game_state.current_player();
    let player = game_state
        .players
        .get(current_player_id)
        .expect("Current player must exist");

    if player.ships.is_empty() {
        return Ok(CommandEffect::None {
            message: String::from("No ships in your fleet."),
        });
    }

    // Group ships by location
    let mut ships_by_location: HashMap<&PlanetId, Vec<&crate::ship::Ship>> = HashMap::new();
    for ship in player.ships.values() {
        ships_by_location
            .entry(&ship.location)
            .or_default()
            .push(ship);
    }

    let mut msg = String::from("=== Your Ships ===\n");

    for (planet_id, ships) in ships_by_location {
        let planet_name = game_state
            .map
            .planets
            .get(planet_id)
            .map(|p| p.name.as_str())
            .unwrap_or("Unknown");

        msg.push_str(&format!("\n{} ({}):\n", planet_name, planet_id));

        for ship in ships {
            let fleet_info = match &ship.fleet_id {
                Some(fleet_id) => format!(" [Fleet: {}]", fleet_id),
                None => String::new(),
            };
            msg.push_str(&format!("  - {} ({}){}\n", ship.id, ship.ship_type, fleet_info));
        }
    }

    msg.push_str(&format!("\nTotal ships: {}", player.ships.len()));

    Ok(CommandEffect::None { message: msg })
}
