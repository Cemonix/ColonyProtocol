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
use crate::configs::ship_config::{ShipConfig, ShipConfigError};
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

    #[error(transparent)]
    ShipConfigError(#[from] ShipConfigError),
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
            players.insert(player_id.clone(), Player::new(player_id, name.clone()));
        }

        let mut rng = rand::rng();
        let mut player_ids: Vec<_> = players.keys().collect();
        player_ids.shuffle(&mut rng);
        let players_order: VecDeque<_> = player_ids.into_iter()
            .map(|p| p.clone()).collect();

        // Load configs early so we can use them for colonization
        let structure_config = StructureConfig::load()?;
        let ship_config = ShipConfig::load()?;

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
                    ship_config,
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

            let result = parser::parse(&input)
                .and_then(|command| command.execute(&self.game_state));

            match result {
                Ok(effect) => {
                    if let Err(e) = self.apply_effect(effect) {
                        eprintln!("ERROR: {e}");
                    }
                }
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

    fn apply_effect(&mut self, command_effect: CommandEffect) -> Result<(), String> {
        match command_effect {
            CommandEffect::BuildStructure { planet_id, structure_id } => {
                // Get current player
                let current_player_id = self.game_state.current_player().clone();

                // Check if player already has a pending action on this planet
                let player = self.game_state.players.get(&current_player_id)
                    .expect("Current player must exist in game state");
                if player.has_pending_action_on_planet(&planet_id) {
                    return Err(String::from("Planet already has a pending action"))
                }

                // Validate and get build info
                let planet = self.game_state.map.planets.get(&planet_id)
                    .expect("Planet must exist (validated by command)");
                let build_info = planet.validate_build_structure(
                    &structure_id, &self.game_state.structure_config
                ).map_err(|e| e.to_string())?;

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
            CommandEffect::BuildShip { planet_id, ship_id } => {
                let current_player_id = self.game_state.current_player().clone();

                // Check if player already has a pending action on this planet
                let player = self.game_state.players.get(&current_player_id)
                    .expect("Current player must exist in game state");
                if player.has_pending_action_on_planet(&planet_id) {
                    return Err(String::from("Planet already has a pending action"))
                }

                // Get ship definition
                let ship_def = self.game_state.ship_config.get(&ship_id)
                    .expect("Ship must exist (validated by command)");

                // Deduct resources from planet
                let planet = self.game_state.map.planets.get_mut(&planet_id)
                    .expect("Planet must exist (validated by command)");
                planet.available_resources -= &ship_def.cost;

                let build_time = ship_def.build_time;
                let cost = ship_def.cost.clone();

                // Create pending action
                use crate::pending_action::{PendingAction, ActionType};
                let pending_action = PendingAction::new(
                    ActionType::BuildShip(ship_id.clone()),
                    planet_id,
                    build_time,
                    cost.clone(),
                );

                // Add to player's pending actions
                let player = self.game_state.players.get_mut(&current_player_id)
                    .expect("Current player must exist in game state");
                player.pending_actions.push(pending_action);

                println!(
                    "Ship construction queued: {}. Resources spent: {}. Turns to complete: {}",
                    ship_id, cost, build_time
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
            CommandEffect::CreateFleet { name, ship_ids, location } => {
                let current_player_id = self.game_state.current_player().clone();
                let player = self.game_state.players.get_mut(&current_player_id)
                    .expect("Current player must exist");

                // Generate fleet ID
                let fleet_id = format!("fleet_{}", player.fleets.len() + 1);

                // Create fleet
                use crate::fleet::Fleet;
                let mut fleet = Fleet::new(fleet_id.clone(), name.clone(), location);

                // Add ships to fleet and update ship's fleet_id
                for ship_id in &ship_ids {
                    fleet.add_ship(ship_id.clone());
                    if let Some(ship) = player.ships.get_mut(ship_id) {
                        ship.fleet_id = Some(fleet_id.clone());
                    }
                }

                player.fleets.insert(fleet_id.clone(), fleet);

                println!(
                    "Fleet '{}' ({}) created with {} ship(s)",
                    name, fleet_id, ship_ids.len()
                );
            }
            CommandEffect::AddToFleet { fleet_id, ship_ids } => {
                let current_player_id = self.game_state.current_player().clone();
                let player = self.game_state.players.get_mut(&current_player_id)
                    .expect("Current player must exist");

                // Update ship's fleet_id
                for ship_id in &ship_ids {
                    if let Some(ship) = player.ships.get_mut(ship_id) {
                        ship.fleet_id = Some(fleet_id.clone());
                    }
                }

                // Add ships to fleet
                if let Some(fleet) = player.fleets.get_mut(&fleet_id) {
                    for ship_id in &ship_ids {
                        fleet.add_ship(ship_id.clone());
                    }
                    println!(
                        "Added {} ship(s) to fleet '{}'",
                        ship_ids.len(), fleet.name
                    );
                }
            }
            CommandEffect::RemoveFromFleet { fleet_id, ship_ids } => {
                let current_player_id = self.game_state.current_player().clone();
                let player = self.game_state.players.get_mut(&current_player_id)
                    .expect("Current player must exist");

                // Clear ship's fleet_id
                for ship_id in &ship_ids {
                    if let Some(ship) = player.ships.get_mut(ship_id) {
                        ship.fleet_id = None;
                    }
                }

                // Remove ships from fleet
                if let Some(fleet) = player.fleets.get_mut(&fleet_id) {
                    for ship_id in &ship_ids {
                        fleet.remove_ship(ship_id);
                    }
                    println!(
                        "Removed {} ship(s) from fleet '{}'",
                        ship_ids.len(), fleet.name
                    );
                }
            }
            CommandEffect::DisbandFleet { fleet_id } => {
                let current_player_id = self.game_state.current_player().clone();
                let player = self.game_state.players.get_mut(&current_player_id)
                    .expect("Current player must exist");

                // Get fleet info before removing
                let fleet_name = player.fleets.get(&fleet_id)
                    .map(|f| f.name.clone())
                    .unwrap_or_default();
                let ship_ids: Vec<_> = player.fleets.get(&fleet_id)
                    .map(|f| f.ships.clone())
                    .unwrap_or_default();

                // Clear fleet_id from all ships in the fleet
                for ship_id in &ship_ids {
                    if let Some(ship) = player.ships.get_mut(ship_id) {
                        ship.fleet_id = None;
                    }
                }

                // Remove fleet
                player.fleets.remove(&fleet_id);

                println!(
                    "Fleet '{}' disbanded. {} ship(s) are now standalone.",
                    fleet_name, ship_ids.len()
                );
            }
            CommandEffect::EndTurn { player_name } => {
                println!("{} ends their turn.", player_name);

                // Rotate player order - move current player to back of queue
                self.game_state.players_order.rotate_left(1);
                self.game_state.players_remaining_this_turn -= 1;

                // Check if all players have played this turn
                if self.game_state.players_remaining_this_turn == 0 {
                    // Process pending actions for ALL players at end of turn
                    let completion_messages = self.process_all_pending_actions();
                    if !completion_messages.is_empty() {
                        println!("\n=== Turn {} Processing ===", self.game_state.turn);
                        for message in completion_messages {
                            println!("{}", message);
                        }
                    }

                    // Increment turn and reset counter
                    self.game_state.turn += 1;
                    self.game_state.players_remaining_this_turn = self.game_state.players_order.len();

                    println!("\n=== Turn {} Begins ===", self.game_state.turn);
                }

                let next_player_id = self.game_state.current_player();
                let next_player = self.game_state.players.get(next_player_id)
                    .expect("Player in rotation must exist in players map");
                println!("{}'s turn.", next_player.name);
            }
            CommandEffect::None { message } => {
                println!("{message}")
            }
        }

        Ok(())
    }

    /// Process pending actions for ALL players at the end of a full turn.
    /// Returns messages describing completed actions.
    fn process_all_pending_actions(&mut self) -> Vec<String> {
        let mut completion_messages = Vec::new();

        // Collect all player IDs to iterate over
        let player_ids: Vec<_> = self.game_state.players.keys().cloned().collect();

        for player_id in player_ids {
            // Tick and collect completed actions for this player
            let completed_actions = {
                let player = self.game_state.players.get_mut(&player_id)
                    .expect("Player must exist");

                // Decrement all cooldowns
                for action in player.pending_actions.iter_mut() {
                    action.tick();
                }

                // Collect completed actions (cooldown reached 0)
                let mut completed = Vec::new();
                player.pending_actions.retain(|action| {
                    if action.is_complete() {
                        completed.push(action.clone());
                        false // Remove from pending
                    } else {
                        true // Keep in pending
                    }
                });
                completed
            };

            // Execute completed actions for this player
            for action in completed_actions {
                use crate::pending_action::ActionType;

                match action.action_type {
                    ActionType::BuildStructure(structure_id) => {
                        let planet = self.game_state.map.planets.get_mut(&action.planet_id)
                            .expect("Planet must exist for pending action");

                        match planet.complete_build_structure(structure_id.clone(), &self.game_state.structure_config) {
                            Ok(()) => {
                                planet.recalculate_from_structures();
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
                                planet.recalculate_from_structures();
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

                    ActionType::BuildShip(ship_type) => {
                        let planet = self.game_state.map.planets.get(&action.planet_id)
                            .expect("Planet must exist for pending action");
                        let planet_name = planet.name.clone();
                        let planet_id = action.planet_id.clone();

                        let player = self.game_state.players.get_mut(&player_id)
                            .expect("Player must exist");
                        let ship_instance_id = player.add_ship(ship_type.clone(), planet_id);

                        completion_messages.push(format!(
                            "Ship built: {} ({}) at planet {}",
                            ship_instance_id, ship_type, planet_name
                        ));
                    }
                }
            }
        }

        // Produce resources on all colonized planets
        for planet in self.game_state.map.planets.values_mut() {
            if planet.get_owner().is_some() {
                planet.produce_resources();
            }
        }

        completion_messages
    }
}
