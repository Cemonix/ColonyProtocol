use std::collections::HashMap;

use thiserror::Error;

use crate::player::PlayerId;
use crate::resources::Resources;
use crate::configs::structure_config::StructureConfig;
use crate::structure::{ StructureId, Structure, StructureState, StructureError };

pub type PlanetId = String;

pub struct BuildInfo {
    pub cost: Resources,
    pub turns: u32,
}

#[derive(Debug, Clone)]
pub struct Connection {
    pub to: PlanetId,
    pub distance: u8 // Distance in turns
}

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
    connections: Vec<Connection>,
    owner: Option<PlayerId>,
    structures: HashMap<StructureId, Structure>,
    production_rate: Resources,
    pub available_resources: Resources,
    pub storage_capacity: Resources,
}

impl Planet {
    pub fn new(
        id: PlanetId, name: String, owner: Option<PlayerId>, connections: Vec<Connection>
    ) -> Self {
        Planet {
            id,
            name,
            owner,
            connections,
            structures: HashMap::new(), // No structure is build on created planet
            production_rate: Resources::default(),
            available_resources: Resources::default(),
            storage_capacity: Resources::default()
        }
    }

    pub fn get_owner(&self) -> &Option<PlayerId> {
        &self.owner
    }

    pub fn set_owner(&mut self, new_owner: PlayerId) {
        self.owner = Some(new_owner);
    }

    pub fn get_connections(&self) -> &Vec<Connection> {
        &self.connections
    }

    pub fn add_connection(&mut self, connection: Connection) {
        self.connections.push(connection);
    }

    /// Validates that a structure can be built and returns the cost/time info.
    /// Does NOT deduct resources or add the structure - use complete_build_structure for that.
    pub fn validate_build_structure(
        &self,
        structure_id: &StructureId,
        structure_config: &StructureConfig
    ) -> Result<BuildInfo, PlanetError> {
        // Check if structure already exists
        if self.structures.contains_key(structure_id) {
            return Err(PlanetError::StructureAlreadyExists {
                structure: structure_id.clone(),
                planet: self.id.clone()
            });
        }

        // Get structure definition from config
        let structure_definition = structure_config.get(structure_id)
            .ok_or(PlanetError::StructureDefinitionNotFound {
                structure: structure_id.clone()
            })?;

        // Get build time
        let build_time = structure_definition.upgrade_time.get(0).copied()
            .expect("upgrade_time array validated during config load");

        // Create temporary structure to get cost
        let structure = Structure::new(structure_definition);
        let build_cost = structure.cost_to_upgrade()?.clone();

        // Check if planet has enough resources
        if !self.available_resources.has_enough(&build_cost) {
            return Err(PlanetError::NotEnoughResources {
                name: self.name.clone(),
                cost: build_cost.clone()
            });
        }

        Ok(BuildInfo {
            cost: build_cost,
            turns: build_time,
        })
    }

    /// Completes a structure build by adding it to the planet.
    /// Should be called when pending action's cooldown reaches 0.
    pub fn complete_build_structure(
        &mut self,
        structure_id: StructureId,
        structure_config: &StructureConfig
    ) -> Result<(), PlanetError> {
        // Get structure definition from config
        let structure_definition = structure_config.get(&structure_id)
            .ok_or(PlanetError::StructureDefinitionNotFound {
                structure: structure_id.clone()
            })?;

        // Create new structure instance
        let structure = Structure::new(structure_definition);

        // Insert structure into planet's structures
        self.structures.insert(structure_id, structure);

        Ok(())
    }

    /// Validates that a structure can be upgraded and returns the cost/time info.
    /// Does NOT deduct resources or upgrade the structure - use complete_upgrade_structure for that.
    pub fn validate_upgrade_structure(
        &self,
        structure_id: &StructureId
    ) -> Result<BuildInfo, PlanetError> {
        // Get reference to the structure, return error if not found
        let structure = self.structures.get(structure_id)
            .ok_or(PlanetError::StructureNotFound {
                structure: structure_id.clone(),
                planet: self.id.clone()
            })?;

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
            });
        }

        // Get upgrade time
        let upgrade_time = structure.get_upgrade_time();

        Ok(BuildInfo {
            cost: cost_to_upgrade,
            turns: upgrade_time,
        })
    }

    /// Completes a structure upgrade by increasing its level.
    /// Should be called when pending action's cooldown reaches 0.
    pub fn complete_upgrade_structure(
        &mut self,
        structure_id: &StructureId
    ) -> Result<(), PlanetError> {
        // Get mutable reference to the structure
        let structure = self.structures.get_mut(structure_id)
            .ok_or(PlanetError::StructureNotFound {
                structure: structure_id.clone(),
                planet: self.id.clone()
            })?;

        // Complete the upgrade
        structure.complete_upgrade();

        Ok(())
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

    /// Colonizes the planet by building a planetary capital and filling resources.
    pub fn colonize(&mut self, structure_config: &StructureConfig) -> Result<(), PlanetError> {
        let capital_id = String::from("planetary_capital");

        let capital_definition = structure_config.get(&capital_id)
            .ok_or(PlanetError::StructureDefinitionNotFound {
                structure: capital_id.clone()
            })?;

        let capital = Structure::new_at_level(capital_definition, 1)?;

        // Set storage capacity from the capital
        self.storage_capacity = capital.storage.clone();

        // Fill resources to capacity
        self.available_resources = self.storage_capacity.clone();

        // Add the capital structure
        self.structures.insert(capital_id, capital);

        Ok(())
    }
}