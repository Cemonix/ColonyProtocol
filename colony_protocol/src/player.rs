use std::collections::HashMap;

use super::configs::ship_config::ShipId;
use super::fleet::Fleet;
use super::planet::PlanetId;
use super::pending_action::PendingAction;
use super::ship::{FleetId, Ship, ShipInstanceId};

pub type PlayerId = String;

pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub planets: Vec<PlanetId>,
    pub pending_actions: Vec<PendingAction>,
    pub ships: HashMap<ShipInstanceId, Ship>,
    pub fleets: HashMap<FleetId, Fleet>,
    ship_id_counters: HashMap<ShipId, u32>,
}

impl Player {
    pub fn new(id: PlayerId, name: String) -> Self {
        Self {
            id,
            name,
            planets: Vec::new(),
            pending_actions: Vec::new(),
            ships: HashMap::new(),
            fleets: HashMap::new(),
            ship_id_counters: HashMap::new(),
        }
    }

    /// Checks if the player has a pending action on the specified planet.
    /// Since only one action per planet is allowed, this returns true if any action exists for that planet.
    pub fn has_pending_action_on_planet(&self, planet_id: &PlanetId) -> bool {
        self.pending_actions
            .iter()
            .any(|action| &action.planet_id == planet_id)
    }

    /// Finds an immutable reference to the pending action on the specified planet.
    pub fn find_pending_action_on_planet(&self, planet_id: &PlanetId) -> Option<&PendingAction> {
        self.pending_actions
            .iter()
            .find(|action| &action.planet_id == planet_id)
    }

    /// Finds a mutable reference to the pending action on the specified planet.
    pub fn find_pending_action_on_planet_mut(
        &mut self,
        planet_id: &PlanetId,
    ) -> Option<&mut PendingAction> {
        self.pending_actions
            .iter_mut()
            .find(|action| &action.planet_id == planet_id)
    }

    /// Removes and returns the pending action on the specified planet, if it exists.
    pub fn remove_pending_action_on_planet(&mut self, planet_id: &PlanetId) -> Option<PendingAction> {
        self.pending_actions
            .iter()
            .position(|action| &action.planet_id == planet_id)
            .map(|index| self.pending_actions.remove(index))
    }

    /// Generates a unique ship instance ID for the given ship type.
    /// IDs follow the pattern: interceptor_1, interceptor_2, ravager_1, etc.
    fn generate_ship_id(&mut self, ship_type: &ShipId) -> ShipInstanceId {
        let counter = self.ship_id_counters.entry(ship_type.clone()).or_insert(0);
        *counter += 1;
        format!("{}_{}", ship_type, counter)
    }

    /// Creates a new ship of the given type at the specified location and adds it to the player's ships.
    /// Returns the generated ship instance ID.
    pub fn add_ship(&mut self, ship_type: ShipId, location: PlanetId) -> ShipInstanceId {
        let ship_id = self.generate_ship_id(&ship_type);
        let ship = Ship::new(ship_id.clone(), ship_type, location);
        self.ships.insert(ship_id.clone(), ship);
        ship_id
    }
}