use colony_core::game::{Game};
use colony_core::game_configuration::{GameConfiguration};

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
    match game.run() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("CRITICAL ERROR: {}", error);
            eprintln!("Connection terminated. Please restart the protocol.");
            return;
        }
    }
}