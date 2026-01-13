use std::collections::HashMap;

use thiserror::Error;

use crate::resources::Resources;
use crate::configs::structure_config::StructureConfig;
use crate::structure::{ StructureId, Structure, StructureState, StructureError };
use crate::player::PlayerId;

pub type PlanetId = String;

#[derive(Debug, Error)]
pub enum PlanetError {
    #[error("Structure {structure} was not found on planet {planet}")]
    StructureNotFound {
        structure: StructureId,
        planet: PlanetId
    },

    #[error("Structure {structure} on planet {planet} is already upgrading")]
    AlreadyUpgrading {
        structure: StructureId,
        planet: PlanetId
    },

    #[error("Structure {structure} on planet {planet} has reached maximum level")]
    MaxLevelReached {
        structure: StructureId,
        planet: PlanetId
    },

    #[error("Planet {name} has not enough resources. Resources needed: {cost}")]
    NotEnoughResources {
        name: String,
        cost: Resources
    },

    #[error("Structure {structure} already exists on planet {planet}")]
    StructureAlreadyExists {
        structure: StructureId,
        planet: PlanetId
    },

    #[error("Structure definition '{structure}' not found in configuration")]
    StructureDefinitionNotFound {
        structure: StructureId
    },

    #[error(transparent)]
    StructureError(#[from] StructureError),
}

pub struct Planet {
    pub id: PlanetId,
    pub name: String,
    pub connections: Vec<PlanetId>,
    owner: Option<PlayerId>,
    structures: HashMap<StructureId, Structure>,
    production_rate: Resources,
    available_resources: Resources,
    storage_capacity: Resources,
}

impl Planet {
    pub fn new(id: PlanetId, name: String, connections: Vec<PlanetId>) -> Self {
        Planet {
            id,
            name,
            connections,
            owner: None, // Nobody owns planet when it is created
            structures: HashMap::new(), // No structure is build on created planet
            production_rate: Resources::default(),
            available_resources: Resources::default(),
            storage_capacity: Resources::default()
        }
    }

    pub fn create_structure(
        &mut self,
        structure_id: StructureId,
        structure_config: &StructureConfig
    ) -> Result<Resources, PlanetError> {
        // Check if structure already exists
        if self.structures.contains_key(&structure_id) {
            return Err(PlanetError::StructureAlreadyExists {
                structure: structure_id.clone(),
                planet: self.id.clone()
            });
        }

        // Get structure definition from config
        let structure_definition = structure_config.get(&structure_id)
            .ok_or(PlanetError::StructureDefinitionNotFound {
                structure: structure_id.clone()
            })?;

        // Create new structure instance
        let structure = Structure::new(structure_definition);

        // Calculate build cost
        let build_cost = structure.cost_to_upgrade()?.clone();

        // Check if planet has enough resources
        if !self.available_resources.has_enough(&build_cost) {
            return Err(PlanetError::NotEnoughResources {
                name: self.name.clone(),
                cost: build_cost.clone()
            });
        }

        // Deduct resources from planet
        self.available_resources -= &build_cost;

        // Insert structure into planet's structures
        self.structures.insert(structure_id, structure);

        Ok(build_cost)
    }

    pub fn upgrade_structure(&mut self, structure_id: StructureId) -> Result<Resources, PlanetError> {
        // Get mutable reference to the structure, return error if not found
        let structure = self.structures.get_mut(&structure_id)
            .ok_or(PlanetError::StructureNotFound { structure: structure_id.clone(), planet: self.id.clone() })?;

        // Check if structure is already upgrading
        if let StructureState::Upgrading { .. } = structure.state {
            return Err(PlanetError::AlreadyUpgrading {
                structure: structure_id.clone(),
                planet: self.id.clone()
            });
        }

        // Check if structure has reached maximum level
        if structure.is_max_level() {
            return Err(PlanetError::MaxLevelReached {
                structure: structure_id.clone(),
                planet: self.id.clone()
            });
        }

        // Calculate upgrade cost
        let cost_to_upgrade = structure.cost_to_upgrade()?.clone();
        
        // Check if planet has enough resources for upgrade
        if !self.available_resources.has_enough(&cost_to_upgrade) {
            return Err(PlanetError::NotEnoughResources {
                name: self.name.clone(),
                cost: cost_to_upgrade.clone()
            })
        }

        // Deduct resources from planet
        self.available_resources -= &cost_to_upgrade;

        // Initiate structure upgrade
        structure.upgrade();

        Ok(cost_to_upgrade)
    }

    pub fn process_turn(&mut self) {
        // Calculate storage capacity and production rate
        self.storage_capacity = Resources::default();
        self.production_rate = Resources::default();

        for structure in self.structures.values() {
            // All structures provide storage (even while upgrading)
            self.storage_capacity += &structure.storage;

            // Only operational structures contribute to production rate
            if matches!(structure.state, StructureState::Operational) {
                self.production_rate += &structure.production;
            }
        }

        // Consume energy, add production (capped at storage), process turns
        for structure in self.structures.values_mut() {
            // Consume energy (only operational structures consume energy)
            // TODO: What happens when we have no energy left?
            self.available_resources.energy -= structure.energy_consumption();

            // Add production, capping each resource at storage capacity
            self.available_resources.minerals = self.available_resources.minerals
                .saturating_add(structure.production.minerals)
                .min(self.storage_capacity.minerals);

            self.available_resources.gas = self.available_resources.gas
                .saturating_add(structure.production.gas)
                .min(self.storage_capacity.gas);

            self.available_resources.energy = self.available_resources.energy
                .saturating_add(structure.production.energy)
                .min(self.storage_capacity.energy);

            // Process structure turn
            structure.process_turn();
        }
    }

    pub fn colonize(&mut self) {
        todo!()
    }
}