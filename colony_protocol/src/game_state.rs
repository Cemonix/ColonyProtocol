use std::collections::{HashMap, VecDeque};
use std::collections::hash_map::Entry;

use thiserror::Error;

use super::configs::structure_config::{ StructureConfig, StructureConfigError };
use super::planet::{ PlanetId, Planet };
use super::player::{ PlayerId, Player };

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
    pub planets: HashMap<PlanetId, Planet>,
    pub turn: u32,
    pub structure_config: StructureConfig
}

impl GameState {
    pub fn new(
        players: HashMap<PlayerId, Player>,
        players_order: VecDeque<PlayerId>,
        planets: HashMap<PlanetId, Planet>, 
    ) -> Result<Self, GameStateError> {
        Ok(
            GameState {
                players,
                players_order,
                planets,
                turn: 0,
                structure_config: StructureConfig::load()?
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
        match self.planets.entry(planet.id.clone()) {
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