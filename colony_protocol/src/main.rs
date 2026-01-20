mod resources;
mod configs;
mod structure;
mod planet;
mod planet_name_generator;
mod player;
mod pending_action;
mod map;
mod game_state;
mod game_configuration;
mod game;
mod commands;
mod utils;

use crate::{game::Game, game_configuration::GameConfiguration};

fn main() {
    #[cfg(debug_assertions)]
    let config_result = GameConfiguration::debug_default();

    #[cfg(not(debug_assertions))]
    let config_result = GameConfiguration::new();

    let game_configuration = match config_result {
        Ok(config) => config,
        Err(error) => {
            eprintln!("CRITICAL ERROR: Colonial Command initialization failed - {}", error);
            eprintln!("Connection terminated. Please restart the protocol.");
            return;
        }
    };

    let mut game = match Game::new(game_configuration) {
        Ok(game) => game,
        Err(e) => {
            eprintln!("Failed to initialize game: {}", e);
            std::process::exit(1);
        }
    };
    game.run();
}