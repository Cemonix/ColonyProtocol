use std::collections::HashMap;

use crate::game_state::GameState;
use crate::commands::command::{CommandEffect, CommandError};

pub fn execute(game_state: &GameState) -> Result<CommandEffect, CommandError> {
    // Build a HashMap of PlayerId -> player name for the map renderer
    let player_names: HashMap<_, _> = game_state.players
        .iter()
        .map(|(id, player)| (id.clone(), player.name.clone()))
        .collect();

    let map_render = game_state.map.render_full(&player_names);
    Ok(CommandEffect::None { message: map_render })
}
