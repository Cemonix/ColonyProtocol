use std::collections::{HashMap, VecDeque};

use rand::Rng;
use rand::seq::SliceRandom;

use crate::commands::command::{CommandEffect, CommandError};
use crate::commands::parser;
use crate::configs::ship_config::{ShipConfig, ShipConfigError, ShipId};
use crate::configs::structure_config::{StructureConfig, StructureConfigError};
use crate::game_configuration::{GameConfigurationError, GameConfiguration};
use crate::game_state::{GameState, GameStateError};
use crate::map::{MapSize, Map, MapError};
use crate::planet::{PlanetError, PlanetId};
use crate::planet_name_generator::{PlanetNameGenerator, PlanetNameGeneratorError};
use crate::player::{PlayerId, Player};
use crate::ship::{FleetId, ShipInstanceId};
use crate::utils;

/// Counter bonus multiplier for ships attacking their counter-type
const COUNTER_BONUS_MULTIPLIER: f32 = 1.5;

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

struct CombatResult {
    attacker_wins: bool,
    attacker_strength: u32,
    defender_strength: u32,
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
            CommandEffect::MoveFleet { fleet_id, target_planet, distance } => {
                let current_player_id = self.game_state.current_player().clone();
                let player = self.game_state.players.get(&current_player_id)
                    .expect("Current player must exist");

                // Get fleet info
                let fleet = player.fleets.get(&fleet_id)
                    .expect("Fleet must exist (validated by command)");
                let source_planet = fleet.location.clone();
                let fleet_name = fleet.name.clone();

                // Get planet names for display
                let source_name = self.game_state.map.planets.get(&source_planet)
                    .map(|p| p.name.clone())
                    .unwrap_or_else(|| source_planet.clone());
                let target_name = self.game_state.map.planets.get(&target_planet)
                    .map(|p| p.name.clone())
                    .unwrap_or_else(|| target_planet.clone());

                // Create pending action (no resource cost for movement)
                use crate::pending_action::{PendingAction, ActionType};
                let pending_action = PendingAction::new(
                    ActionType::MoveFleet(fleet_id.clone(), target_planet.clone()),
                    source_planet,
                    distance as u32,
                    crate::resources::Resources::default(),
                );

                // Add to player's pending actions
                let player = self.game_state.players.get_mut(&current_player_id)
                    .expect("Current player must exist");
                player.pending_actions.push(pending_action);

                println!(
                    "Fleet '{}' ({}) ordered to move from {} to {}. Arrival in {} turn(s).",
                    fleet_name, fleet_id, source_name, target_name, distance
                );
            }
            CommandEffect::BombardPlanet { fleet_id, target_planet, bombardment_power } => {
                let current_player_id = self.game_state.current_player().clone();
                let player = self.game_state.players.get(&current_player_id)
                    .expect("Current player must exist");

                // Get fleet info
                let fleet = player.fleets.get(&fleet_id)
                    .expect("Fleet must exist (validated by command)");
                let fleet_name = fleet.name.clone();

                // Get planet name for display
                let target_name = self.game_state.map.planets.get(&target_planet)
                    .map(|p| p.name.clone())
                    .unwrap_or_else(|| target_planet.clone());

                // Create pending action (no resource cost for bombardment, per-turn action)
                use crate::pending_action::{PendingAction, ActionType};
                let pending_action = PendingAction::new(
                    ActionType::BombardPlanet(fleet_id.clone(), target_planet.clone()),
                    target_planet.clone(),
                    u32::MAX, // Bombardment continues indefinitely until shields are down
                    crate::resources::Resources::default(),
                );

                // Add to player's pending actions
                let player = self.game_state.players.get_mut(&current_player_id)
                    .expect("Current player must exist");
                player.pending_actions.push(pending_action);

                println!(
                    "Fleet '{}' ({}) begins bombarding {} with {} bombardment power.",
                    fleet_name, fleet_id, target_name, bombardment_power
                );
            }
            CommandEffect::CancelBombard { fleet_id } => {
                let current_player_id = self.game_state.current_player().clone();
                let player = self.game_state.players.get_mut(&current_player_id)
                    .expect("Current player must exist");

                // Remove the bombardment action
                use crate::pending_action::ActionType;
                player.pending_actions.retain(|action| {
                    !matches!(&action.action_type,
                        ActionType::BombardPlanet(fid, _)
                        if fid == &fleet_id)
                });

                println!("Fleet '{}' bombardment cancelled.", fleet_id);
            }
            CommandEffect::ColonizePlanet { fleet_id, planet_id } => {
                let current_player_id = self.game_state.current_player().clone();

                // Get planet name before mutating
                let planet_name = self.game_state.map.planets.get(&planet_id)
                    .map(|p| p.name.clone())
                    .unwrap_or_else(|| planet_id.clone());

                // Colonize the planet
                let planet = self.game_state.map.planets.get_mut(&planet_id)
                    .expect("Planet must exist (validated by command)");

                match planet.colonize(&self.game_state.structure_config) {
                    Ok(()) => {
                        // Set ownership
                        planet.set_owner(current_player_id.clone());

                        // Add planet to player's planet list
                        let player = self.game_state.players.get_mut(&current_player_id)
                            .expect("Current player must exist");
                        player.planets.push(planet_id.clone());

                        println!(
                            "Fleet '{}' has colonized {}! Planet now belongs to {}.",
                            fleet_id, planet_name, player.name
                        );
                    }
                    Err(e) => {
                        println!("Failed to colonize {}: {}", planet_name, e);
                    }
                }
            }
            CommandEffect::EndTurn { player_name } => {
                println!("{} ends their turn.", player_name);

                // Rotate player order - move current player to back of queue
                self.game_state.players_order.rotate_left(1);
                self.game_state.players_remaining_this_turn -= 1;

                // Check if all players have played this turn
                if self.game_state.players_remaining_this_turn == 0 {
                    // Process bombardments first (happens every turn for ongoing bombardments)
                    let bombardment_messages = self.process_bombardments();

                    // Then process pending actions for ALL players at end of turn
                    let completion_messages = self.process_all_pending_actions();

                    // Display all turn processing messages
                    let all_messages: Vec<_> = bombardment_messages.into_iter()
                        .chain(completion_messages.into_iter())
                        .collect();

                    if !all_messages.is_empty() {
                        println!("\n=== Turn {} Processing ===", self.game_state.turn);
                        for message in all_messages {
                            println!("{}", message);
                        }
                    }

                    // Check for win condition
                    if let Some(winner_id) = self.check_win_condition() {
                        let winner = self.game_state.players.get(&winner_id)
                            .expect("Winner must exist");
                        println!("\n🎉 VICTORY! {} has conquered the entire system!", winner.name);
                        println!("Game Over - {} wins on Turn {}", winner.name, self.game_state.turn);
                        return Ok(());
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

    /// Checks if any player has won by owning all planets.
    /// Returns the winner's PlayerId if there is one, None otherwise.
    fn check_win_condition(&self) -> Option<PlayerId> {
        let total_planets = self.game_state.map.planets.len();

        for player in self.game_state.players.values() {
            if player.planets.len() == total_planets {
                return Some(player.id.clone());
            }
        }

        None
    }

    /// Processes a fleet arriving at a destination planet.
    /// Handles combat resolution and conquest.
    /// Returns messages describing what happened.
    fn process_fleet_arrival(
        &mut self,
        attacker_id: &PlayerId,
        fleet_id: &FleetId,
        destination: &PlanetId,
    ) -> Vec<String> {
        let mut messages = Vec::new();

        // Get destination planet info
        let planet_owner = self.game_state.map.planets
            .get(destination)
            .and_then(|p| p.get_owner().clone());
        let planet_name = self.game_state.map.planets
            .get(destination)
            .map(|p| p.name.clone())
            .unwrap_or_else(|| destination.clone());

        // Check if this is a friendly arrival (same owner) or potential combat
        let needs_combat = planet_owner.as_ref() != Some(attacker_id);

        if !needs_combat {
            // Friendly arrival - just move the fleet
            self.move_fleet_to_planet(attacker_id, fleet_id, destination);
            messages.push(format!(
                "Fleet {} arrived at {} (friendly territory)",
                fleet_id, planet_name
            ));
            return messages;
        }

        // Combat needed - get defending ships at the planet
        let defender_id = planet_owner;
        let defending_ship_ids = self.get_defending_ships(destination, &defender_id);

        if defending_ship_ids.is_empty() {
            // Undefended planet - move fleet there
            self.move_fleet_to_planet(attacker_id, fleet_id, destination);

            if defender_id.is_some() {
                messages.push(format!(
                    "Fleet {} arrived at undefended enemy planet {}. Use bombardment to weaken defenses, then colonize.",
                    fleet_id, planet_name
                ));
            } else {
                messages.push(format!(
                    "Fleet {} arrived at neutral planet {}. Use colonize command to claim it.",
                    fleet_id, planet_name
                ));
            }
            return messages;
        }

        // Defended planet - resolve combat
        let combat_result = self.resolve_combat(
            attacker_id,
            fleet_id,
            &defender_id,
            &defending_ship_ids,
        );

        messages.push(format!(
            "⚔ BATTLE at {}! {} vs {}",
            planet_name,
            self.game_state.players.get(attacker_id).map(|p| &p.name).unwrap_or(&String::from("Unknown")),
            defender_id.as_ref()
                .and_then(|id| self.game_state.players.get(id))
                .map(|p| &p.name)
                .unwrap_or(&String::from("Unknown"))
        ));
        messages.push(format!(
            "  Attack: {} | Defense: {}",
            combat_result.attacker_strength,
            combat_result.defender_strength
        ));

        if combat_result.attacker_wins {
            messages.push(format!("  Victory! Attacker wins!"));

            // Destroy defending ships
            self.destroy_ships(&defender_id, &defending_ship_ids);
            messages.push(format!(
                "  {} defending ship(s) destroyed",
                defending_ship_ids.len()
            ));

            // Move attacker fleet to planet
            self.move_fleet_to_planet(attacker_id, fleet_id, destination);
            messages.push(format!(
                "  Fleet {} now orbits {}. Use bombardment to weaken defenses, then colonize.",
                fleet_id, planet_name
            ));
        } else {
            messages.push(format!("  Defeat! Defender wins!"));

            // Destroy attacking fleet
            let attacker_fleet = self.game_state.players
                .get(attacker_id)
                .and_then(|p| p.fleets.get(fleet_id))
                .map(|f| f.ships.clone())
                .unwrap_or_default();

            self.destroy_ships(&Some(attacker_id.clone()), &attacker_fleet);
            messages.push(format!(
                "  {} attacking ship(s) destroyed",
                attacker_fleet.len()
            ));

            // Disband the empty fleet
            if let Some(player) = self.game_state.players.get_mut(attacker_id) {
                player.fleets.remove(fleet_id);
            }
        }

        messages
    }

    /// Gets all ships defending a planet (ships belonging to planet owner at that location).
    fn get_defending_ships(
        &self,
        planet_id: &PlanetId,
        owner_id: &Option<PlayerId>,
    ) -> Vec<ShipInstanceId> {
        let Some(owner) = owner_id else {
            return Vec::new();
        };

        let Some(player) = self.game_state.players.get(owner) else {
            return Vec::new();
        };

        player.ships
            .values()
            .filter(|ship| &ship.location == planet_id)
            .map(|ship| ship.id.clone())
            .collect()
    }

    /// Moves a fleet to a new planet location.
    fn move_fleet_to_planet(
        &mut self,
        player_id: &PlayerId,
        fleet_id: &FleetId,
        destination: &PlanetId,
    ) {
        // Update fleet location
        if let Some(player) = self.game_state.players.get_mut(player_id) {
            if let Some(fleet) = player.fleets.get_mut(fleet_id) {
                fleet.location = destination.clone();

                // Update all ship locations
                for ship_id in &fleet.ships.clone() {
                    if let Some(ship) = player.ships.get_mut(ship_id) {
                        ship.location = destination.clone();
                    }
                }
            }
        }
    }

    /// Destroys a list of ships belonging to a player.
    fn destroy_ships(&mut self, player_id: &Option<PlayerId>, ship_ids: &[ShipInstanceId]) {
        let Some(owner_id) = player_id else {
            return;
        };

        let Some(player) = self.game_state.players.get_mut(owner_id) else {
            return;
        };

        for ship_id in ship_ids {
            // Remove ship from any fleet
            if let Some(ship) = player.ships.get(ship_id) {
                if let Some(fleet_id) = &ship.fleet_id {
                    if let Some(fleet) = player.fleets.get_mut(fleet_id) {
                        fleet.remove_ship(ship_id);
                    }
                }
            }

            // Remove ship from player
            player.ships.remove(ship_id);
        }

        // Clean up empty fleets
        let empty_fleet_ids: Vec<_> = player.fleets
            .iter()
            .filter(|(_, f)| f.is_empty())
            .map(|(id, _)| id.clone())
            .collect();

        for fleet_id in empty_fleet_ids {
            player.fleets.remove(&fleet_id);
        }
    }

    /// Resolves combat between an attacking fleet and defending ships.
    fn resolve_combat(
        &self,
        attacker_id: &PlayerId,
        attacker_fleet_id: &FleetId,
        defender_id: &Option<PlayerId>,
        defending_ship_ids: &[ShipInstanceId],
    ) -> CombatResult {
        let attacker_strength = self.calculate_fleet_attack(attacker_id, attacker_fleet_id, defending_ship_ids);
        let defender_strength = self.calculate_defense(defender_id, defending_ship_ids, attacker_fleet_id, attacker_id);

        CombatResult {
            attacker_wins: attacker_strength > defender_strength,
            attacker_strength,
            defender_strength,
        }
    }

    /// Calculates total attack strength of a fleet with counter bonuses.
    fn calculate_fleet_attack(
        &self,
        player_id: &PlayerId,
        fleet_id: &FleetId,
        defender_ship_ids: &[ShipInstanceId],
    ) -> u32 {
        let Some(player) = self.game_state.players.get(player_id) else {
            return 0;
        };

        let Some(fleet) = player.fleets.get(fleet_id) else {
            return 0;
        };

        let mut total_attack = 0;

        for ship_id in &fleet.ships {
            if let Some(ship) = player.ships.get(ship_id) {
                if let Some(ship_def) = self.game_state.ship_config.get(&ship.ship_type) {
                    let mut attack = ship_def.attack;

                    // Apply counter bonuses
                    if self.has_counter_advantage(&ship.ship_type, defender_ship_ids) {
                        attack = (attack as f32 * COUNTER_BONUS_MULTIPLIER) as u32;
                    }

                    total_attack += attack;
                }
            }
        }

        total_attack
    }

    /// Calculates total defense strength of defending ships with counter bonuses.
    fn calculate_defense(
        &self,
        defender_id: &Option<PlayerId>,
        defender_ship_ids: &[ShipInstanceId],
        attacker_fleet_id: &FleetId,
        attacker_id: &PlayerId,
    ) -> u32 {
        let Some(owner_id) = defender_id else {
            return 0;
        };

        let Some(player) = self.game_state.players.get(owner_id) else {
            return 0;
        };

        let mut total_defense = 0;

        for ship_id in defender_ship_ids {
            if let Some(ship) = player.ships.get(ship_id) {
                if let Some(ship_def) = self.game_state.ship_config.get(&ship.ship_type) {
                    let mut defense = ship_def.shield;

                    // Apply counter bonuses
                    if self.has_counter_advantage_against_fleet(&ship.ship_type, attacker_fleet_id, attacker_id) {
                        defense = (defense as f32 * COUNTER_BONUS_MULTIPLIER) as u32;
                    }

                    total_defense += defense;
                }
            }
        }

        total_defense
    }

    /// Checks if a ship type has counter advantage against any of the defender ships.
    fn has_counter_advantage(&self, ship_type: &ShipId, defender_ship_ids: &[ShipInstanceId]) -> bool {
        let Some(ship_def) = self.game_state.ship_config.get(ship_type) else {
            return false;
        };

        // Get all defender ship types
        for ship_id in defender_ship_ids {
            for player in self.game_state.players.values() {
                if let Some(defender_ship) = player.ships.get(ship_id) {
                    if ship_def.counters.contains(&defender_ship.ship_type) {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Checks if a ship type has counter advantage against any ship in the attacking fleet.
    fn has_counter_advantage_against_fleet(
        &self,
        ship_type: &ShipId,
        attacker_fleet_id: &FleetId,
        attacker_id: &PlayerId,
    ) -> bool {
        let Some(ship_def) = self.game_state.ship_config.get(ship_type) else {
            return false;
        };

        let Some(player) = self.game_state.players.get(attacker_id) else {
            return false;
        };

        let Some(fleet) = player.fleets.get(attacker_fleet_id) else {
            return false;
        };

        // Check if this ship counters any ship in the attacking fleet
        for ship_id in &fleet.ships {
            if let Some(attacker_ship) = player.ships.get(ship_id) {
                if ship_def.counters.contains(&attacker_ship.ship_type) {
                    return true;
                }
            }
        }

        false
    }

    /// Process bombardment actions for ALL players at the end of a full turn.
    /// Bombardments deal damage each turn until shields are destroyed.
    /// Returns messages describing bombardment results.
    fn process_bombardments(&mut self) -> Vec<String> {
        use crate::pending_action::ActionType;
        let mut bombardment_messages = Vec::new();

        // Collect all player IDs to iterate over
        let player_ids: Vec<_> = self.game_state.players.keys().cloned().collect();

        for player_id in player_ids {
            // Collect bombardment actions for this player
            let bombardment_actions: Vec<_> = {
                let player = self.game_state.players.get(&player_id)
                    .expect("Player must exist");

                player.pending_actions.iter()
                    .filter_map(|action| {
                        if let ActionType::BombardPlanet(fleet_id, planet_id) = &action.action_type {
                            Some((fleet_id.clone(), planet_id.clone()))
                        } else {
                            None
                        }
                    })
                    .collect()
            };

            // Process each bombardment
            for (fleet_id, planet_id) in bombardment_actions {
                // Calculate bombardment power
                let bombardment_power = self.game_state.calculate_fleet_bombardment(&player_id, &fleet_id);

                // Get planet name for messages
                let planet_name = self.game_state.map.planets.get(&planet_id)
                    .map(|p| p.name.clone())
                    .unwrap_or_else(|| planet_id.clone());

                // Apply damage to shields
                let planet = self.game_state.map.planets.get_mut(&planet_id)
                    .expect("Planet must exist");

                let shields_before = planet.get_shield_hp();
                let _overflow_damage = planet.take_shield_damage(bombardment_power);

                if shields_before > 0 {
                    let shields_after = planet.get_shield_hp();

                    if shields_after == 0 {
                        // Shields destroyed!
                        bombardment_messages.push(format!(
                            "Fleet {} bombards {}. Shields destroyed! ({} → 0 HP). Planet ready for colonization.",
                            fleet_id, planet_name, shields_before
                        ));

                        // Remove the bombardment action since shields are down
                        let player = self.game_state.players.get_mut(&player_id)
                            .expect("Player must exist");
                        player.pending_actions.retain(|action| {
                            !matches!(&action.action_type,
                                ActionType::BombardPlanet(fid, pid)
                                if fid == &fleet_id && pid == &planet_id)
                        });
                    } else {
                        // Shields still up
                        bombardment_messages.push(format!(
                            "Fleet {} bombards {}. Shields damaged: {} → {} HP.",
                            fleet_id, planet_name, shields_before, shields_after
                        ));
                    }
                }
            }
        }

        bombardment_messages
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

                    ActionType::MoveFleet(fleet_id, destination) => {
                        // Handle fleet arrival and potential combat
                        let messages = self.process_fleet_arrival(&player_id, &fleet_id, &destination);
                        completion_messages.extend(messages);
                    }

                    ActionType::BombardPlanet(_, _) => {
                        // Bombardment actions complete when shields hit 0, handled in process_bombardments
                        // This case should not be reached since bombardments are removed when shields hit 0
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
