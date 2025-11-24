use std::collections::HashMap;

use super::PendingAction;
use super::{ PlayerId, Player, };
use super::{ PlanetId, Planet };

pub struct GameState {
    players: HashMap<PlayerId, Player>,
    planets: HashMap<PlanetId, Planet>,
    pending_actions: Vec<PendingAction>,
    current_turn: u32,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
            planets: HashMap::new(),
            pending_actions: Vec::new(),
            current_turn: 0,
        }
    }

    pub fn get_current_turn(&self) -> u32 {
        self.current_turn
    }

    pub fn get_player(&self, player_id: PlayerId) -> Option<&Player> {
        self.players.get(&player_id)
    }

    pub fn get_planet(&self, planet_id: PlanetId) -> Option<&Planet> {
        self.planets.get(&planet_id)
    }

    pub fn add_player(&mut self, player: Player) {
        self.players.insert(player.id, player);
    }

    pub fn add_planet(&mut self, planet: Planet) {
        self.planets.insert(planet.id, planet);
    }

    pub fn next_turn(&mut self) {
        self.current_turn += 1;
    }
}