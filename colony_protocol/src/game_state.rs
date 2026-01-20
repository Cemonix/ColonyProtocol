use std::collections::{HashMap, VecDeque};
use std::collections::hash_map::Entry;

use thiserror::Error;

use crate::map::Map;

use super::configs::structure_config::{StructureConfig, StructureConfigError};
use super::planet::Planet;
use super::player::{PlayerId, Player};

#[derive(Debug, Error)]
pub enum GameStateError {
    #[error("Player {0} is already present")]
    PlayerAlreadyExists(String),

    #[error("Planet {0} is already present")]
    PlanetAlreadyExists(String),

    #[error(transparent)]
    StructureConfigError(#[from] StructureConfigError)
}

pub struct GameState {
    pub players: HashMap<PlayerId, Player>,
    pub players_order: VecDeque<PlayerId>,
    pub map: Map,
    pub turn: u32,
    pub structure_config: StructureConfig
}

impl GameState {
    pub fn new(
        players: HashMap<PlayerId, Player>,
        players_order: VecDeque<PlayerId>,
        map: Map,
        structure_config: StructureConfig,
    ) -> Result<Self, GameStateError> {
        Ok(
            GameState {
                players,
                players_order,
                map,
                turn: 1,
                structure_config
            }
        )
    }

    pub fn current_player(&self) -> &PlayerId {
        self.players_order.front()
            .expect("Game has no players - invalid state")
    }
    
    pub fn add_player(&mut self, player: Player) -> Result<(), GameStateError> {
        match self.players.entry(player.id.clone()) {
            Entry::Vacant(e) => {
                e.insert(player);
                Ok(())
            }
            Entry::Occupied(_) => {
                Err(GameStateError::PlayerAlreadyExists(player.name))
            }
        }
    }

    pub fn add_planet(&mut self, planet: Planet) -> Result<(), GameStateError> {
        match self.map.planets.entry(planet.id.clone()) {
            Entry::Vacant(e) => {
                e.insert(planet);
                Ok(())
            }
            Entry::Occupied(_) => {
                Err(GameStateError::PlanetAlreadyExists(planet.name))
            }
        }
    }
}