use crate::commands::command::{CommandEffect, CommandError};
use crate::game_state::GameState;

pub fn execute(_game_state: &GameState) -> Result<CommandEffect, CommandError> {
    let help_text = r#"=== Colony Protocol - Command Reference ===

GAME COMMANDS
  status turn              Show current turn number
  status planets           List all planets in the system
  status planet <id>       Show details for a specific planet
  status player            Show your player status
  map                      Display the star system map

BUILDING
  build <planet_id> <structure_id>    Queue structure construction
  build_ship <planet_id> <ship_id>    Queue ship construction
  cancel <planet_id>                  Cancel pending action on planet

SHIPS & FLEETS
  ships                               List all your ships
  fleets                              List all your fleets
  fleet create <name> <ship_id>...    Create fleet from ships (same location)
  fleet add <fleet_id> <ship_id>...   Add ships to fleet
  fleet remove <fleet_id> <ship_id>...Remove ships from fleet
  fleet disband <fleet_id>            Disband fleet (ships become standalone)

TURN
  end_turn, end            End your turn and pass to next player

SYSTEM
  help                     Show this help message
  exit, terminate          End the game session

TIPS
  - Planet IDs are shown in parentheses, e.g. "Kepler VII (c418)"
  - Ship IDs follow pattern: interceptor_1, ravager_2, etc.
  - Fleet IDs follow pattern: fleet_1, fleet_2, etc.
  - Only one pending action per planet allowed"#;

    Ok(CommandEffect::None {
        message: help_text.to_string(),
    })
}
