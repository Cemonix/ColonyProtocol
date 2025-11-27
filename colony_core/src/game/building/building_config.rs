use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use thiserror::Error;

use super::resource::Resources;

#[derive(Debug, Error)]
pub enum BuildingConfigError {
    #[error("Failed to read building config file: {0}")]
    FileReadError(#[from] std::io::Error),

    #[error("Failed to parse building config JSON: {0}")]
    JsonParseError(#[from] serde_json::Error),

    #[error("Building '{building_id}' has invalid array length: {field} has {actual} elements but max_level is {expected}")]
    InvalidArrayLength {
        building_id: String,
        field: String,
        expected: usize,
        actual: usize,
    },

    #[error("Building '{building_id}' has duplicate ID in config")]
    DuplicateBuildingId { building_id: String },

    #[error("Building '{building_id}' has prerequisite '{prereq_id}' that doesn't exist")]
    InvalidPrerequisite {
        building_id: String,
        prereq_id: String,
    },

    #[error("Building '{building_id}' has prerequisite '{prereq_id}' with invalid level requirements (length mismatch)")]
    InvalidPrerequisiteLevels {
        building_id: String,
        prereq_id: String,
    },

    #[error("Building '{building_id}' has prerequisite '{prereq_id}' requiring level {required_level} but that building's max level is {max_level}")]
    PrerequisiteLevelTooHigh {
        building_id: String,
        prereq_id: String,
        required_level: u8,
        max_level: u8,
    },

    #[error("Circular dependency detected in building prerequisites: {cycle}")]
    CircularDependency { cycle: String },
}

#[derive(Debug, Deserialize)]
struct PrerequisiteConfig {
    building_id: String,
    required_levels: Vec<u8>,
}

#[derive(Debug, Deserialize)]
struct ResourceConfig {
    minerals: Vec<u32>,
    gas: Vec<u32>,
    energy: Vec<u32>,
}

#[derive(Debug, Deserialize)]
struct BuildingConfig {
    id: String,
    name: String,
    description: String,
    max_level: u8,
    costs: Vec<Resources>,
    build_time: Vec<u8>,
    energy_consumption: Vec<u32>,
    hitpoints: Vec<u32>,
    production: ResourceConfig,
    storage_capacity: ResourceConfig,
    prerequisites: Vec<PrerequisiteConfig>,
}

#[derive(Debug, Deserialize)]
struct BuildingConfigJson {
    buildings: Vec<BuildingConfig>,
}

#[derive(Debug, Clone)]
pub struct Prerequisite {
    pub building_id: String,
    pub required_levels: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct BuildingDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub max_level: u8,
    pub costs: Vec<Resources>,
    pub build_time: Vec<u8>,
    pub energy_consumption: Vec<u32>,
    pub hitpoints: Vec<u32>,
    pub production: Vec<Resources>,
    pub storage_capacity: Vec<Resources>,
    pub prerequisites: Vec<Prerequisite>,
}

impl BuildingDefinition {
    pub fn cost_for_level(&self, level: u8) -> Option<&Resources> {
        if level == 0 || level > self.max_level {
            return None;
        }
        self.costs.get((level - 1) as usize)
    }

    pub fn build_time_for_level(&self, level: u8) -> Option<u8> {
        if level == 0 || level > self.max_level {
            return None;
        }
        self.build_time.get((level - 1) as usize).copied()
    }

    pub fn energy_consumption_for_level(&self, level: u8) -> Option<u32> {
        if level == 0 || level > self.max_level {
            return None;
        }
        self.energy_consumption.get((level - 1) as usize).copied()
    }

    pub fn hitpoints_for_level(&self, level: u8) -> Option<u32> {
        if level == 0 || level > self.max_level {
            return None;
        }
        self.hitpoints.get((level - 1) as usize).copied()
    }

    pub fn production_for_level(&self, level: u8) -> Option<&Resources> {
        if level == 0 || level > self.max_level {
            return None;
        }
        self.production.get((level - 1) as usize)
    }

    pub fn storage_capacity_for_level(&self, level: u8) -> Option<&Resources> {
        if level == 0 || level > self.max_level {
            return None;
        }
        self.storage_capacity.get((level - 1) as usize)
    }
}

#[derive(Debug)]
pub struct BuildingRegistry {
    definitions: HashMap<String, Arc<BuildingDefinition>>,
}

impl BuildingRegistry {
    pub fn load_from_file(path: &str) -> Result<Self, BuildingConfigError> {
        let json_content = std::fs::read_to_string(path)?;
        Self::load_from_string(&json_content)
    }

    pub fn load_from_string(json: &str) -> Result<Self, BuildingConfigError> {
        let config: BuildingConfigJson = serde_json::from_str(json)?;
        Self::validate_and_build(config)
    }

    fn validate_and_build(config: BuildingConfigJson) -> Result<Self, BuildingConfigError> {
        let mut definitions = HashMap::new();

        for building_json in &config.buildings {
            // Check for duplicate IDs
            if definitions.contains_key(&building_json.id) {
                return Err(BuildingConfigError::DuplicateBuildingId {
                    building_id: building_json.id.clone(),
                });
            }

            let expected_len = building_json.max_level as usize;
            Self::validate_array_length(
                &building_json.id,
                "costs",
                expected_len,
                building_json.costs.len(),
            )?;
            Self::validate_array_length(
                &building_json.id,
                "build_time",
                expected_len,
                building_json.build_time.len(),
            )?;
            Self::validate_array_length(
                &building_json.id,
                "energy_consumption",
                expected_len,
                building_json.energy_consumption.len(),
            )?;
            Self::validate_array_length(
                &building_json.id,
                "hitpoints",
                expected_len,
                building_json.hitpoints.len(),
            )?;
            Self::validate_array_length(
                &building_json.id,
                "production.minerals",
                expected_len,
                building_json.production.minerals.len(),
            )?;
            Self::validate_array_length(
                &building_json.id,
                "production.gas",
                expected_len,
                building_json.production.gas.len(),
            )?;
            Self::validate_array_length(
                &building_json.id,
                "production.energy",
                expected_len,
                building_json.production.energy.len(),
            )?;
            Self::validate_array_length(
                &building_json.id,
                "storage_capacity.minerals",
                expected_len,
                building_json.storage_capacity.minerals.len(),
            )?;
            Self::validate_array_length(
                &building_json.id,
                "storage_capacity.gas",
                expected_len,
                building_json.storage_capacity.gas.len(),
            )?;
            Self::validate_array_length(
                &building_json.id,
                "storage_capacity.energy",
                expected_len,
                building_json.storage_capacity.energy.len(),
            )?;

            let definition = Self::convert_to_definition(building_json)?;
            definitions.insert(building_json.id.clone(), Arc::new(definition));
        }

        for building_json in &config.buildings {
            Self::validate_prerequisites(&building_json.id, &building_json.prerequisites, &definitions)?;
        }

        Self::check_circular_dependencies(&definitions)?;

        Ok(Self { definitions })
    }

    fn validate_array_length(
        building_id: &str,
        field: &str,
        expected: usize,
        actual: usize,
    ) -> Result<(), BuildingConfigError> {
        if expected != actual {
            return Err(BuildingConfigError::InvalidArrayLength {
                building_id: building_id.to_string(),
                field: field.to_string(),
                expected,
                actual,
            });
        }
        Ok(())
    }

    fn convert_to_definition(
        json: &BuildingConfig,
    ) -> Result<BuildingDefinition, BuildingConfigError> {
        let production: Vec<Resources> = json
            .production
            .minerals
            .iter()
            .zip(&json.production.gas)
            .zip(&json.production.energy)
            .map(|((&minerals, &gas), &energy)| Resources::new(minerals, gas, energy))
            .collect();

        let storage_capacity: Vec<Resources> = json
            .storage_capacity
            .minerals
            .iter()
            .zip(&json.storage_capacity.gas)
            .zip(&json.storage_capacity.energy)
            .map(|((&minerals, &gas), &energy)| Resources::new(minerals, gas, energy))
            .collect();

        let prerequisites: Vec<Prerequisite> = json
            .prerequisites
            .iter()
            .map(|p| Prerequisite {
                building_id: p.building_id.clone(),
                required_levels: p.required_levels.clone(),
            })
            .collect();

        Ok(BuildingDefinition {
            id: json.id.clone(),
            name: json.name.clone(),
            description: json.description.clone(),
            max_level: json.max_level,
            costs: json.costs.clone(),
            build_time: json.build_time.clone(),
            energy_consumption: json.energy_consumption.clone(),
            hitpoints: json.hitpoints.clone(),
            production,
            storage_capacity,
            prerequisites,
        })
    }

    fn validate_prerequisites(
        building_id: &str,
        prerequisites: &[PrerequisiteConfig],
        definitions: &HashMap<String, Arc<BuildingDefinition>>,
    ) -> Result<(), BuildingConfigError> {
        let building_def = definitions.get(building_id).unwrap();

        for prereq in prerequisites {
            let prereq_def = definitions.get(&prereq.building_id).ok_or_else(|| {
                BuildingConfigError::InvalidPrerequisite {
                    building_id: building_id.to_string(),
                    prereq_id: prereq.building_id.clone(),
                }
            })?;

            if prereq.required_levels.len() != building_def.max_level as usize {
                return Err(BuildingConfigError::InvalidPrerequisiteLevels {
                    building_id: building_id.to_string(),
                    prereq_id: prereq.building_id.clone(),
                });
            }

            for &required_level in &prereq.required_levels {
                if required_level > prereq_def.max_level {
                    return Err(BuildingConfigError::PrerequisiteLevelTooHigh {
                        building_id: building_id.to_string(),
                        prereq_id: prereq.building_id.clone(),
                        required_level,
                        max_level: prereq_def.max_level,
                    });
                }
            }
        }

        Ok(())
    }

    fn check_circular_dependencies(
        definitions: &HashMap<String, Arc<BuildingDefinition>>,
    ) -> Result<(), BuildingConfigError> {
        for building_id in definitions.keys() {
            let mut visited = HashSet::new();
            let mut path = Vec::new();
            Self::detect_cycle(building_id, definitions, &mut visited, &mut path)?;
        }
        Ok(())
    }

    fn detect_cycle(
        current: &str,
        definitions: &HashMap<String, Arc<BuildingDefinition>>,
        visited: &mut HashSet<String>,
        path: &mut Vec<String>,
    ) -> Result<(), BuildingConfigError> {
        if path.contains(&current.to_string()) {
            // Cycle detected
            path.push(current.to_string());
            let cycle = path.join(" -> ");
            return Err(BuildingConfigError::CircularDependency { cycle });
        }

        if visited.contains(current) {
            return Ok(()); // Already fully explored
        }

        path.push(current.to_string());

        let def = definitions.get(current).unwrap();
        for prereq in &def.prerequisites {
            Self::detect_cycle(&prereq.building_id, definitions, visited, path)?;
        }

        path.pop();
        visited.insert(current.to_string());

        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<Arc<BuildingDefinition>> {
        self.definitions.get(id).cloned()
    }

    pub fn all_buildings(&self) -> impl Iterator<Item = &Arc<BuildingDefinition>> {
        self.definitions.values()
    }

    pub fn len(&self) -> usize {
        self.definitions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_valid_config() {
        let json = r#"
        {
            "buildings": [
                {
                    "id": "command_center",
                    "name": "Command Center",
                    "description": "Main building",
                    "max_level": 2,
                    "costs": [
                        {"minerals": 0, "gas": 0, "energy": 0},
                        {"minerals": 100, "gas": 0, "energy": 0}
                    ],
                    "build_time": [0, 2],
                    "energy_consumption": [0, 0],
                    "hitpoints": [1000, 1500],
                    "production": {
                        "minerals": [10, 20],
                        "gas": [5, 10],
                        "energy": [10, 20]
                    },
                    "storage_capacity": {
                        "minerals": [500, 1000],
                        "gas": [250, 500],
                        "energy": [0, 0]
                    },
                    "prerequisites": []
                }
            ]
        }
        "#;

        let registry = BuildingRegistry::load_from_string(json).expect("Should parse valid config");
        assert_eq!(registry.len(), 1);

        let building = registry.get("command_center").expect("Should find command center");
        assert_eq!(building.name, "Command Center");
        assert_eq!(building.max_level, 2);
    }

    #[test]
    fn test_invalid_array_length() {
        let json = r#"
        {
            "buildings": [
                {
                    "id": "test_building",
                    "name": "Test",
                    "description": "Test",
                    "max_level": 2,
                    "costs": [
                        {"minerals": 0, "gas": 0, "energy": 0}
                    ],
                    "build_time": [0, 2],
                    "energy_consumption": [0, 0],
                    "hitpoints": [1000, 1500],
                    "production": {
                        "minerals": [10, 20],
                        "gas": [5, 10],
                        "energy": [10, 20]
                    },
                    "storage_capacity": {
                        "minerals": [500, 1000],
                        "gas": [250, 500],
                        "energy": [0, 0]
                    },
                    "prerequisites": []
                }
            ]
        }
        "#;

        let result = BuildingRegistry::load_from_string(json);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            BuildingConfigError::InvalidArrayLength { .. }
        ));
    }

    #[test]
    fn test_building_definition_helper_methods() {
        let json = r#"
        {
            "buildings": [
                {
                    "id": "test",
                    "name": "Test",
                    "description": "Test",
                    "max_level": 2,
                    "costs": [
                        {"minerals": 100, "gas": 0, "energy": 0},
                        {"minerals": 200, "gas": 0, "energy": 0}
                    ],
                    "build_time": [1, 2],
                    "energy_consumption": [10, 20],
                    "hitpoints": [1000, 1500],
                    "production": {
                        "minerals": [10, 20],
                        "gas": [5, 10],
                        "energy": [10, 20]
                    },
                    "storage_capacity": {
                        "minerals": [500, 1000],
                        "gas": [250, 500],
                        "energy": [0, 0]
                    },
                    "prerequisites": []
                }
            ]
        }
        "#;

        let registry = BuildingRegistry::load_from_string(json).unwrap();
        let building = registry.get("test").unwrap();

        // Test cost_for_level
        let cost = building.cost_for_level(1).expect("Level 1 should exist");
        assert_eq!(cost.minerals, 100);

        // Test build_time_for_level
        let time = building.build_time_for_level(2).expect("Level 2 should exist");
        assert_eq!(time, 2);

        // Test energy_consumption_for_level
        let energy = building.energy_consumption_for_level(1).expect("Level 1 should exist");
        assert_eq!(energy, 10);

        // Test invalid level
        assert!(building.cost_for_level(0).is_none());
        assert!(building.cost_for_level(3).is_none());
    }

    #[test]
    fn test_load_game_config() {
        // This test loads the actual buildings.json config file
        let result = BuildingRegistry::load_from_file("../data/buildings.json");

        assert!(result.is_ok(), "Failed to load buildings.json: {:?}", result.err());
        let registry = result.unwrap();

        // Check that we have all 6 buildings
        assert_eq!(registry.len(), 6);

        // Verify Command Center exists and has correct properties
        let cc = registry.get("command_center").expect("Command Center should exist");
        assert_eq!(cc.name, "Command Center");
        assert_eq!(cc.max_level, 5);
        assert_eq!(cc.cost_for_level(1).unwrap().minerals, 0); // Free initial build
        assert_eq!(cc.cost_for_level(2).unwrap().minerals, 300);
        assert_eq!(cc.hitpoints_for_level(1).unwrap(), 1000);
        assert_eq!(cc.production_for_level(1).unwrap().minerals, 20);
        assert!(cc.prerequisites.is_empty());

        // Verify Power Plant
        let pp = registry.get("power_plant").expect("Power Plant should exist");
        assert_eq!(pp.name, "Power Plant");
        assert_eq!(pp.max_level, 5);
        assert_eq!(pp.cost_for_level(1).unwrap().minerals, 150);
        assert_eq!(pp.production_for_level(1).unwrap().energy, 50);
        assert!(pp.prerequisites.is_empty());

        // Verify Warehouse has prerequisites
        let warehouse = registry.get("warehouse").expect("Warehouse should exist");
        assert_eq!(warehouse.name, "Warehouse");
        assert_eq!(warehouse.prerequisites.len(), 1);
        assert_eq!(warehouse.prerequisites[0].building_id, "command_center");
        assert_eq!(warehouse.prerequisites[0].required_levels[0], 2); // Level 1 warehouse requires CC level 2

        // Verify Mineral Extractor
        let extractor = registry.get("mineral_extractor").expect("Mineral Extractor should exist");
        assert_eq!(extractor.max_level, 5);
        assert_eq!(extractor.production_for_level(1).unwrap().minerals, 40);
        assert_eq!(extractor.prerequisites.len(), 1);
        assert_eq!(extractor.prerequisites[0].building_id, "warehouse");

        // Verify Shipyard has multiple prerequisites
        let shipyard = registry.get("shipyard").expect("Shipyard should exist");
        assert_eq!(shipyard.name, "Shipyard");
        assert_eq!(shipyard.prerequisites.len(), 2);
        assert!(shipyard.prerequisites.iter().any(|p| p.building_id == "power_plant"));
        assert!(shipyard.prerequisites.iter().any(|p| p.building_id == "gas_refinery"));

        // Verify Gas Refinery
        let refinery = registry.get("gas_refinery").expect("Gas Refinery should exist");
        assert_eq!(refinery.production_for_level(1).unwrap().gas, 30);
    }
}
