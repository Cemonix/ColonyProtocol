use std::collections::{HashMap, VecDeque};

use rand::Rng;
use rand::seq::SliceRandom;

use crate::commands::command::{CommandEffect, CommandError};
use crate::commands::parser;
use crate::game_configuration::{GameConfigurationError, GameConfiguration, MapSize};
use crate::player::{PlayerId, Player};
use crate::planet::{Planet, PlanetError, PlanetId};
use crate::game_state::{GameState, GameStateError};
use crate::planet_graph::{NameGenerator, GraphGenerator};
use crate::planet_graph::name_generator::NameGeneratorError;
use crate::planet_graph::graph_generator::GraphGeneratorError;
use crate::utils;

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
    CommandError(#[from] CommandError),

    #[error(transparent)]
    PlanetError(#[from] PlanetError),
}

pub struct Game {
    pub(crate) game_state: GameState,
}

impl Game {
    pub fn new(game_configuration: GameConfiguration) -> Result<Self, GameError> {
        // Create players
        let mut players: HashMap<PlayerId, Player> = HashMap::new();
        for name in game_configuration.player_names.iter() {
            let player_id = utils::name_to_id(name);
            
            players.insert(
                player_id.clone(),
                Player { 
                    id: player_id, 
                    name: name.clone(),
                    planets: Vec::new()
                }
            );
        }

        let mut rng = rand::rng();
        let mut player_ids: Vec<_> = players.keys().collect();
        player_ids.shuffle(&mut rng);
        let players_order: VecDeque<_> = player_ids.into_iter().map(
            |p| p.clone()
        ).collect();

        // Generate planet system
        let mut planets = Self::generate_planet_system(game_configuration.map_size)?;
        
        // Assign starting planets to players
        Self::assign_starting_planets(&mut planets, &mut players);

        Ok(
            Game {
                game_state: GameState::new(
                    players,
                    players_order,
                    planets,
                )?
            }
        )
    }

    pub fn run(&mut self) -> Result<(), GameError> {
        println!("Initializing command interface...");
        println!("Type 'help' for available commands\n");

        loop {
            let input = utils::get_player_input(|input| Ok(String::from(input)));
            if input == "terminate" || input == "exit" {
                println!("\nTerminating session...");
                println!("Colony management interface offline.");
                break;
            }

            // Parse and execute, but don't crash on errors
            let result = parser::parse(&input)
                .and_then(|command| command.execute(&self.game_state));

            match result {
                Ok(effect) => match effect {
                    CommandEffect::BuildStructure { planet_id, structure_id } => {
                        let planet = self.game_state.planets.get_mut(&planet_id).unwrap();
                        let cost = planet.build_structure(structure_id, &self.game_state.structure_config)?;
                        println!("Structure has been built using {cost} resources.");
                    },
                    _ => ()
                },
                Err(e) => eprintln!("ERROR: {e}"),
            }
        }

        Ok(())
    }

    
    fn generate_planet_system(map_size: MapSize) -> Result<HashMap<PlanetId, Planet>, GameError> {
        let num_planets = map_size.num_planets();
        
        let name_generator = NameGenerator::new()?;

        let mut graph_generator = GraphGenerator::new(num_planets, name_generator)?;
        let planets = graph_generator.generate()?;

        Ok(planets)
    }

    fn assign_starting_planets(
        planets: &mut HashMap<PlanetId, Planet>, players: &mut HashMap<PlayerId, Player>
    ) {
        let mut rng = rand::rng();
        let mut available_ids: Vec<_> = planets.keys().cloned().collect();

        for player in players.values_mut() {
            let index = rng.random_range(0..available_ids.len());
            let planet_id = available_ids.swap_remove(index);  // removes and returns
            
            // Set player as owner of the planet
            if let Some(planet) = planets.get_mut(&planet_id) {
                planet.set_owner(player.id.clone());
            }
            
            player.planets.push(planet_id);
        }
    }

    fn process_turn(&self) {
        // TODO: Implement turn processing
        // - Process planet production
        // - Process fleet movements
        // - Resolve battles
        // - Process structure upgrades
    }
}
