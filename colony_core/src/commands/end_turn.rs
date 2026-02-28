use crate::commands::command::{CommandEffect, CommandError};
use crate::game_state::GameState;

pub fn execute(game_state: &GameState) -> Result<CommandEffect, CommandError> {
    let current_player_id = game_state.current_player();
    let player = game_state
        .players
        .get(current_player_id)
        .expect("Current player must exist");

    Ok(CommandEffect::EndTurn {
        player_name: player.name.clone(),
    })
}
