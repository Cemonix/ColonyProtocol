mod resources;
mod configs;
mod structure;
mod planet;
mod player;
mod game_state;
mod game_configuration;
mod game;
mod planet_graph;
mod commands;
mod utils;

use crate::{game::Game, game_configuration::GameConfiguration};

fn main() {
    let game_configuration = match GameConfiguration::new() {
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