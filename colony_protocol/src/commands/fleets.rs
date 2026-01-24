use crate::commands::command::{CommandEffect, CommandError};
use crate::game_state::GameState;

pub fn execute(game_state: &GameState) -> Result<CommandEffect, CommandError> {
    let current_player_id = game_state.current_player();
    let player = game_state
        .players
        .get(current_player_id)
        .expect("Current player must exist");

    if player.fleets.is_empty() {
        return Ok(CommandEffect::None {
            message: String::from("No fleets formed. Use 'fleet create <name> <ship_id>...' to create one."),
        });
    }

    let mut msg = String::from("=== Your Fleets ===\n");

    for fleet in player.fleets.values() {
        let planet_name = game_state
            .map
            .planets
            .get(&fleet.location)
            .map(|p| p.name.as_str())
            .unwrap_or("Unknown");

        msg.push_str(&format!(
            "\n{} ({}) at {} ({}):\n",
            fleet.name, fleet.id, planet_name, fleet.location
        ));

        if fleet.ships.is_empty() {
            msg.push_str("  (empty)\n");
        } else {
            for ship_id in &fleet.ships {
                if let Some(ship) = player.ships.get(ship_id) {
                    msg.push_str(&format!("  - {} ({})\n", ship.id, ship.ship_type));
                }
            }
        }

        msg.push_str(&format!("  Ships: {}\n", fleet.ship_count()));
    }

    msg.push_str(&format!("\nTotal fleets: {}", player.fleets.len()));

    Ok(CommandEffect::None { message: msg })
}
