use crate::game_state::GameState;
use crate::commands::command::{CommandEffect, CommandError};

pub fn execute(game_state: &GameState) -> Result<CommandEffect, CommandError> {
    let map_render = game_state.map.render_full();
    Ok(CommandEffect::None { message: map_render })
}
