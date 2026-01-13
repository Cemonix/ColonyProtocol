use std::collections::HashMap;

use thiserror::Error;

use super::configs::structure_config::{ StructureConfig, StructureConfigError };
use super::planet::{ PlanetId, Planet };
use super::player::{ PlayerId, Player };

#[derive(Debug, Error)]
pub enum GameStateError {
    #[error(transparent)]
    StructureConfigError(#[from] StructureConfigError)
}

pub struct GameState {
    pub players: HashMap<PlayerId, Player>,
    pub planets: HashMap<PlanetId, Planet>,
    pub turn: u32,
    pub structure_config: StructureConfig
}

impl GameState {
    pub fn new() -> Result<Self, GameStateError> {
        Ok(
            GameState {
                players: HashMap::new(),
                planets: HashMap::new(),
                turn: 0,
                structure_config: StructureConfig::load()?
            }
        )
    }
}