use std::collections::{HashMap, VecDeque};

use rand::Rng;
use rand::seq::SliceRandom;

use crate::commands::command::{CommandEffect, CommandError};
use crate::commands::parser;
use crate::game_configuration::{GameConfigurationError, GameConfiguration};
use crate::planet_name_generator::{PlanetNameGenerator, PlanetNameGeneratorError};
use crate::player::{PlayerId, Player};
use crate::planet::PlanetError;
use crate::game_state::{GameState, GameStateError};
use crate::map::{MapSize, Map, MapError};
use crate::configs::structure_config::{StructureConfig, StructureConfigError};
use crate::utils;

#[derive(Debug, thiserror::Error)]
pub enum GameError {
    #[error(transparent)]
    GameConfigurationError(#[from] GameConfigurationError),

    #[error(transparent)]
    PlanetNameGeneratorError(#[from] PlanetNameGeneratorError),

    #[error(transparent)]
    MapError(#[from] MapError),


    #[error(transparent)]
    GameStateError(#[from] GameStateError),

    #[error(transparent)]
    CommandError(#[from] CommandError),

    #[error(transparent)]
    PlanetError(#[from] PlanetError),

    #[error(transparent)]
    StructureConfigError(#[from] StructureConfigError),
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
                    planets: Vec::new(),
                    pending_actions: Vec::new(),
                }
            );
        }

        let mut rng = rand::rng();
        let mut player_ids: Vec<_> = players.keys().collect();
        player_ids.shuffle(&mut rng);
        let players_order: VecDeque<_> = player_ids.into_iter()
            .map(|p| p.clone()).collect();

        // Load structure config early so we can use it for colonization
        let structure_config = StructureConfig::load()?;

        // Generate planet system
        let mut map = Self::generate_map(game_configuration.map_size)?;

        // Assign starting planets to players and colonize them
        Self::assign_starting_planets(&mut map, &mut players, &structure_config)?;

        Ok(
            Game {
                game_state: GameState::new(
                    players,
                    players_order,
                    map,
                    structure_config,
                )?
            }
        )
    }

    pub fn run(&mut self) -> Result<(), GameError> {
        println!("Initializing command interface...");
        println!("Type 'help' for available commands\n");

        loop {
            // Process pending actions at the start of each turn
            let completion_messages = self.process_pending_actions();
            if !completion_messages.is_empty() {
                println!("\n=== Turn Processing ===");
                for message in completion_messages {
                    println!("{}", message);
                }
                println!();
            }

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
                        // Get current player
                        let current_player_id = self.game_state.current_player().clone();

                        // Check if player already has a pending action on this planet
                        let player = self.game_state.players.get(&current_player_id)
                            .expect("Current player must exist in game state");
                        if player.has_pending_action_on_planet(&planet_id) {
                            eprintln!("ERROR: Planet already has a pending action");
                            continue;
                        }

                        // Validate and get build info
                        let planet = self.game_state.map.planets.get(&planet_id)
                            .expect("Planet must exist (validated by command)");
                        let build_info = match planet.validate_build_structure(&structure_id, &self.game_state.structure_config) {
                            Ok(info) => info,
                            Err(e) => {
                                eprintln!("ERROR: {e}");
                                continue;
                            }
                        };

                        // Deduct resources from planet
                        let planet = self.game_state.map.planets.get_mut(&planet_id)
                            .expect("Planet must exist (validated by command)");
                        planet.available_resources -= &build_info.cost;

                        // Create pending action
                        use crate::pending_action::{PendingAction, ActionType};
                        let pending_action = PendingAction::new(
                            ActionType::BuildStructure(structure_id),
                            planet_id,
                            build_info.turns,
                            build_info.cost.clone(),
                        );

                        // Add to player's pending actions
                        let player = self.game_state.players.get_mut(&current_player_id)
                            .expect("Current player must exist in game state");
                        player.pending_actions.push(pending_action);

                        println!(
                            "Construction queued. Resources spent: {}. Turns to complete: {}",
                            build_info.cost, build_info.turns
                        );
                    },
                    CommandEffect::CancelAction { planet_id } => {
                        let current_player_id = self.game_state.current_player().clone();

                        // Remove pending action and get the reserved resources
                        let player = self.game_state.players.get_mut(&current_player_id)
                            .expect("Current player must exist in game state");
                        let action = player.remove_pending_action_on_planet(&planet_id)
                            .expect("Pending action must exist (validated by command)");

                        let refund = action.reserved_resources.clone();

                        // Get planet and calculate available space
                        let planet = self.game_state.map.planets.get_mut(&planet_id)
                            .expect("Planet must exist (validated by command)");
                        let space_available = planet.storage_capacity.clone() - planet.available_resources.clone();

                        // Refund resources with overflow handling
                        if !space_available.has_enough(&refund) {
                            // Partial refund - add what fits, waste the rest
                            let wasted = refund.clone() - space_available.clone();
                            planet.available_resources += &space_available;

                            println!(
                                "Action cancelled on planet {}. Resources refunded: {}. Wasted (storage full): {}",
                                planet.name, space_available, wasted
                            );
                        } else {
                            // Full refund
                            planet.available_resources += &refund;

                            println!(
                                "Action cancelled on planet {}. Resources refunded: {}",
                                planet.name, refund
                            );
                        }
                    },
                    CommandEffect::None { message } => {
                        println!("{message}")
                    }
                },
                Err(e) => eprintln!("ERROR: {e}"),
            }
        }

        Ok(())
    }

    
    fn generate_map(map_size: MapSize) -> Result<Map, GameError> {
        let mut name_generator = PlanetNameGenerator::new()?;
        let map = Map::generate(map_size, &mut name_generator)?;
        Ok(map)
    }

    fn assign_starting_planets(
        map: &mut Map,
        players: &mut HashMap<PlayerId, Player>,
        structure_config: &StructureConfig,
    ) -> Result<(), GameError> {
        let mut rng = rand::rng();
        let mut available_ids: Vec<_> = map.planets.keys().cloned().collect();

        for player in players.values_mut() {
            let index = rng.random_range(0..available_ids.len());
            let planet_id = available_ids.swap_remove(index);

            if let Some(planet) = map.planets.get_mut(&planet_id) {
                planet.set_owner(player.id.clone());
                planet.colonize(structure_config)?;
            }

            player.planets.push(planet_id);
        }

        Ok(())
    }

    /// Process pending actions for the current player at the start of their turn.
    /// Returns messages describing completed actions.
    fn process_pending_actions(&mut self) -> Vec<String> {
        let mut completion_messages = Vec::new();
        let current_player_id = self.game_state.current_player().clone();

        // Get mutable reference to current player
        let player = self.game_state.players.get_mut(&current_player_id)
            .expect("Current player must exist");

        // Decrement all cooldowns
        for action in player.pending_actions.iter_mut() {
            action.tick();
        }

        // Collect completed actions (cooldown reached 0)
        let mut completed_actions = Vec::new();
        player.pending_actions.retain(|action| {
            if action.is_complete() {
                completed_actions.push(action.clone());
                false // Remove from pending
            } else {
                true // Keep in pending
            }
        });

        // Execute completed actions
        for action in completed_actions {
            use crate::pending_action::ActionType;

            match action.action_type {
                ActionType::BuildStructure(structure_id) => {
                    let planet = self.game_state.map.planets.get_mut(&action.planet_id)
                        .expect("Planet must exist for pending action");

                    match planet.complete_build_structure(structure_id.clone(), &self.game_state.structure_config) {
                        Ok(()) => {
                            completion_messages.push(format!(
                                "Construction completed: {} on planet {}",
                                structure_id, planet.name
                            ));
                        }
                        Err(e) => {
                            completion_messages.push(format!(
                                "Construction failed for {} on planet {}: {}",
                                structure_id, planet.name, e
                            ));
                        }
                    }
                }

                ActionType::UpgradeStructure(structure_id) => {
                    let planet = self.game_state.map.planets.get_mut(&action.planet_id)
                        .expect("Planet must exist for pending action");

                    match planet.complete_upgrade_structure(&structure_id) {
                        Ok(()) => {
                            completion_messages.push(format!(
                                "Upgrade completed: {} on planet {}",
                                structure_id, planet.name
                            ));
                        }
                        Err(e) => {
                            completion_messages.push(format!(
                                "Upgrade failed for {} on planet {}: {}",
                                structure_id, planet.name, e
                            ));
                        }
                    }
                }

                ActionType::BuildShips { ship_type, count } => {
                    // TODO: Implement when Fleet system is ready
                    completion_messages.push(format!(
                        "Ships built: {} x{} at planet {} (Fleet system pending)",
                        ship_type, count, action.planet_id
                    ));
                }
            }
        }

        completion_messages
    }

    fn process_turn(&self) {
        // TODO: Implement turn processing
        // - Process planet production
        // - Process fleet movements
        // - Resolve battles
        // - Process structure upgrades
    }
}
