use std::sync::Arc;
use thiserror::Error;

use crate::resources::Resources;
use crate::configs::structure_config::StructureDefinition;

pub type StructureId = String;

#[derive(Debug, Error)]
pub enum StructureError {    
    #[error("Invalid level {level} for structure '{structure_name}' (max: {max_level})")]
    InvalidLevel {
        structure_name: String,
        level: u16,
        max_level: u16,
    },
}

pub enum StructureState {
    Operational,
    Upgrading {
        turns_remaining: u32,
        target_level: u16,
    },
    Damaged,
}

pub struct Structure {
    pub name: String,
    pub hitpoints: u32,
    pub level: u16,
    pub max_level: u16,
    pub production: Resources,
    pub storage: Resources,
    pub state: StructureState,
    structure_definition: Arc<StructureDefinition>,
}

impl Structure {
    pub fn new(definition: Arc<StructureDefinition>) -> Self {
        // Structure starts at level 0 (not yet built)
        // The upgrade_time[0] represents the build time from level 0 -> level 1
        let build_time = definition.upgrade_time.get(0).copied().unwrap_or(0);

        Structure {
            name: definition.name.clone(),
            hitpoints: 0, // No hitpoints until built
            level: 0,
            max_level: definition.max_level,
            production: Resources::default(), // No production until built
            storage: Resources::default(), // No storage until built
            state: StructureState::Upgrading {
                turns_remaining: build_time,
                target_level: 1
            },
            structure_definition: definition
        }
    }

    pub fn cost_to_upgrade(&self) -> Result<&Resources, StructureError> {
        let cost = self.structure_definition.costs.get(self.level as usize)
            .ok_or_else(|| StructureError::InvalidLevel {
                structure_name: self.structure_definition.name.clone(),
                level: self.level,
                max_level: self.structure_definition.max_level,
            })?;

        Ok(cost)
    }

    pub fn is_max_level(&self) -> bool {
        self.level >= self.max_level
    }

    pub fn upgrade(&mut self) {
        let curr_level_idx = self.level as usize;
        let upgrade_time = self.structure_definition.upgrade_time[curr_level_idx];

        self.state = StructureState::Upgrading {
            turns_remaining: upgrade_time,
            target_level: self.level + 1
        };
    }

    pub fn process_turn(&mut self) {
        if let StructureState::Upgrading { 
            turns_remaining, 
            target_level 
        } = &mut self.state {
            *turns_remaining -= 1;
            
            if *turns_remaining == 0 {
                self.level = *target_level;
                let level_idx = (self.level - 1) as usize;
                
                self.hitpoints = self.structure_definition.hitpoints[level_idx];
                self.production = self.structure_definition.production[level_idx].clone();
                self.storage = self.structure_definition.storage_capacity[level_idx].clone();
                
                self.state = StructureState::Operational;
            }
        }
    }

    pub fn energy_consumption(&self) -> u32 {
        if let StructureState::Upgrading { .. } = self.state {
            return 0;
        }
        self.structure_definition.energy_consumption[(self.level-1) as usize]
    }
}