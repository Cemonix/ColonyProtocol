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
    /// Current shield HP from defense_shield structure
    shield_hp: u32,
    /// Turns since last attack (shield regenerates when this reaches the configured threshold)
    shield_regen_timer: u32,
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
            storage_capacity: Resources::default(),
            shield_hp: 0,
            shield_regen_timer: 0,
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

    /// Returns the level of a structure on this planet, or 0 if not built.
    pub fn get_structure_level(&self, structure_id: &StructureId) -> u16 {
        self.structures
            .get(structure_id)
            .map(|s| s.level)
            .unwrap_or(0)
    }

    /// Returns a reference to all structures on this planet.
    pub fn get_structures(&self) -> &HashMap<StructureId, Structure> {
        &self.structures
    }

    pub fn get_shield_hp(&self) -> u32 {
        self.shield_hp
    }

    /// Returns max shield HP based on defense_shield structure level.
    /// Returns 0 if no defense shield is built.
    pub fn get_max_shield_hp(&self) -> u32 {
        self.structures
            .get("defense_shield")
            .map(|shield| shield.hitpoints)
            .unwrap_or(0)
    }

    /// Applies damage to the shield and resets the regeneration timer.
    /// Returns the amount of damage that passed through (overflow damage).
    pub fn take_shield_damage(&mut self, damage: u32) -> u32 {
        self.shield_regen_timer = 0;

        if damage >= self.shield_hp {
            let overflow = damage - self.shield_hp;
            self.shield_hp = 0;
            overflow
        } else {
            self.shield_hp -= damage;
            0
        }
    }

    /// Restores shield to maximum HP.
    fn regenerate_shield(&mut self) {
        self.shield_hp = self.get_max_shield_hp();
    }

    /// Returns the number of turns required for shield regeneration.
    /// Returns None if no defense shield is built.
    fn get_shield_regen_turns(&self) -> Option<u32> {
        self.structures
            .get("defense_shield")
            .and_then(|shield| shield.get_shield_regen_turns())
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

        // Create operational structure at level 1
        let structure = Structure::new_at_level(structure_definition, 1)?;

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

        // Shield logic: initialize when defense_shield becomes operational, or regenerate
        if let Some(regen_turns) = self.get_shield_regen_turns() {
            let max_shield = self.get_max_shield_hp();

            // Initialize shield if it's 0 but we have a defense_shield (just built)
            if self.shield_hp == 0 && max_shield > 0 {
                self.shield_hp = max_shield;
            } else if self.shield_hp < max_shield {
                // Regeneration logic
                self.shield_regen_timer += 1;
                if self.shield_regen_timer >= regen_turns {
                    self.regenerate_shield();
                    self.shield_regen_timer = 0;
                }
            }
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

        // Add the capital structure
        self.structures.insert(capital_id, capital);

        // Recalculate totals from structures
        self.recalculate_from_structures();

        // Fill resources to capacity
        self.available_resources = self.storage_capacity.clone();

        Ok(())
    }

    /// Recalculates production_rate and storage_capacity by summing all operational structures.
    pub fn recalculate_from_structures(&mut self) {
        self.production_rate = Resources::default();
        self.storage_capacity = Resources::default();

        for structure in self.structures.values() {
            // Only count operational structures
            if let StructureState::Operational = structure.state {
                self.production_rate += &structure.production;
                self.storage_capacity += &structure.storage;
            }
        }
    }

    /// Produces resources based on production_rate, capped at storage_capacity.
    pub fn produce_resources(&mut self) {
        self.available_resources += &self.production_rate;
        self.available_resources = self.available_resources.capped_at(&self.storage_capacity);
    }
}