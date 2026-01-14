use std::collections::HashMap;

use rand::Rng;

use super::game_configuration::{GameConfigurationError, GameConfiguration, MapSize};
use super::player::{PlayerId, Player};
use super::planet::{PlanetId, Planet};
use super::game_state::{GameState, GameStateError};
use super::planet_graph::{NameGenerator, GraphGenerator};
use super::planet_graph::name_generator::NameGeneratorError;
use super::planet_graph::graph_generator::GraphGeneratorError;
use super::commands::{Command, ParseError};
use super::utils::get_player_input;

#[derive(Debug, thiserror::Error)]
pub enum GameError {
    #[error(transparent)]
    GameConfigurationError(#[from] GameConfigurationError),

    #[error(transparent)]
    GameStateError(#[from] GameStateError),

    #[error(transparent)]
    NameGeneratorError(#[from] NameGeneratorError),

    #[error(transparent)]
    GraphGeneratorError(#[from] GraphGeneratorError),

    #[error(transparent)]
    ParseError(#[from] ParseError),
}

pub struct Game {
    pub(crate) game_state: GameState,
}

impl Game {
    pub fn new(game_configuration: GameConfiguration) -> Result<Self, GameError> {
        // Create players
        let mut players: HashMap<PlayerId, Player> = HashMap::new();
        for (index, name) in game_configuration.player_names.iter().enumerate() {
            let player_id = format!("CMDR-{:03}", index + 1);
            
            players.insert(
                player_id.clone(),
                Player { 
                    id: player_id, 
                    name: name.clone(),
                    planets: Vec::new()
                }
            );
        }
        
        // Generate planet system
        let planets = Self::generate_planet_system(game_configuration.map_size)?;
        
        // Assign starting planets to players
        Self::assign_starting_planets(&planets, &mut players);
        
        Ok(
            Game {
                game_state: GameState::new(
                    players,
                    planets
                )?
            }
        )
    }

    pub fn run(&mut self) {
        println!("Initializing command interface...");
        println!("Type 'help' for available commands\n");

        loop {
            let input = get_player_input(|input| Ok(String::from(input)));
            if input == "terminate" || input == "exit" {
                println!("\nTerminating session...");
                println!("Colony management interface offline.");
                break;
            }

            if !input.is_empty() {
                // Parse and execute the command
                match Command::parse(&input) {
                    Ok(command) => match self.execute_command(command) {
                        Ok(message) => println!("{}", message),
                        Err(error) => eprintln!("ERROR: {}", error),
                    },
                    Err(error) => eprintln!("PARSE ERROR: {}", error),
                }
            }
        }
    }

    
    /// Execute a parsed command and return the result message
    ///
    /// This is the main command router - it delegates to specific
    /// execution methods based on command type
    pub fn execute_command(&mut self, command: Command) -> Result<String, GameError> {
        match command {
            Command::Help(help_cmd) => self.execute_help_command(help_cmd),
            Command::EndTurn => self.execute_end_turn(),
            Command::Planet(planet_cmd) => todo!(),
            // Future commands:
            // Cmd::Fleet(fleet_cmd) => self.execute_fleet_command(fleet_cmd),
            // Cmd::Status => self.execute_status(),
        }
    }

    fn generate_planet_system(map_size: MapSize) -> Result<HashMap<PlanetId, Planet>, GameError> {
        let num_planets = map_size.num_planets();
        
        let name_generator = NameGenerator::new()?;

        let mut graph_generator = GraphGenerator::new(num_planets, name_generator)?;
        let planets = graph_generator.generate()?;

        Ok(planets)
    }

    fn assign_starting_planets(planets: &HashMap<PlanetId, Planet>, players: &mut HashMap<PlayerId, Player>) {
        let mut rng = rand::rng();
        let mut available_ids: Vec<_> = planets.keys().cloned().collect();

        for player in players.values_mut() {
            let index = rng.random_range(0..available_ids.len());
            let planet_id = available_ids.swap_remove(index);  // removes and returns
            player.planets.push(planet_id);
        }
    }

    /// Execute help command
    fn execute_help_command(
        &self,
        _help_cmd: super::commands::help::HelpCommand,
    ) -> Result<String, GameError> {
        Ok(self.get_help_text())
    }

    /// Execute end-turn command
    fn execute_end_turn(&mut self) -> Result<String, GameError> {
        self.game_state.turn += 1;
        self.process_turn();
        Ok(format!("Turn {} processed", self.game_state.turn))
    }

    /// Get help text showing available commands
    fn get_help_text(&self) -> String {
        let mut help = String::from("=== COLONY PROTOCOL - AVAILABLE COMMANDS ===\n\n");
        help.push_str("General:\n");
        help.push_str("  help                     - Show this help message\n");
        help.push_str("  end-turn                 - End current turn and process game state\n");
        help.push_str("  exit, terminate          - Exit the game\n\n");
        help.push_str("Planet Commands:\n");
        help.push_str("  planet build <id> <type> - Build a structure on a planet\n");
        help.push_str("  planet view <id>         - View planet details\n");
        help
    }

    fn process_turn(&self) {
        // TODO: Implement turn processing
        // - Process planet production
        // - Process fleet movements
        // - Resolve battles
        // - Process structure upgrades
    }
}
