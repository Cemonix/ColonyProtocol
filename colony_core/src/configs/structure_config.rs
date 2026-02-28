use std::collections::HashMap;
use std::sync::Arc;

use thiserror::Error;

use crate::resources::Resources;
use crate::structure::StructureId;

const STRUCTURE_CONFIG_PATH: &str = "data/structure.json";

#[derive(Debug, Error)]
pub enum StructureConfigError {
    #[error("Structure '{structure_name}': {field_name} has {actual} items but max_level is {expected}")]
    SizeMismatchError {
        structure_name: String,
        field_name: String,
        expected: usize,
        actual: usize
    },

    #[error("Failed to read config file: {0}")]
    FileReadError(#[from] std::io::Error),

    #[error("Failed to parse JSON: {0}")]
    JsonParseError(#[from] serde_json::Error),
}

#[derive(serde::Deserialize, Debug)]
pub struct Prerequisity {
    pub structure_id: StructureId,
    pub required_levels: Vec<u32>
}

impl Prerequisity {
    pub fn validate(&self, structure_name: &str, max_level: usize) -> Result<(), StructureConfigError> {
        let required_levels_len = self.required_levels.len();
        if required_levels_len > max_level {
            return Err(
                StructureConfigError::SizeMismatchError {
                    structure_name: structure_name.to_string(),
                    field_name: "prerequisity.required_levels".to_string(),
                    expected: max_level, 
                    actual: required_levels_len 
                }
            );
        }
        Ok(())
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct StructureDefinition {
    pub id: StructureId,
    pub name: String,
    pub description: String,
    pub max_level: u16,
    pub costs: Vec<Resources>,
    pub upgrade_time: Vec<u32>,
    pub energy_consumption: Vec<u32>,
    pub hitpoints: Vec<u32>,
    pub production: Vec<Resources>,
    pub storage_capacity: Vec<Resources>,
    pub prerequisites: Vec<Prerequisity>,
    /// Turns without attack required for shield regeneration (only for defense_shield)
    pub shield_regen_turns: Option<u32>,
}

#[derive(Debug)]
pub struct StructureConfig {
    structures: HashMap<StructureId, Arc<StructureDefinition>>
}

impl StructureConfig {
    pub fn load() -> Result<Self, StructureConfigError> {
        let json_content = std::fs::read_to_string(STRUCTURE_CONFIG_PATH)?;
        Self::load_from_string(&json_content)
    }

    pub fn load_from_string(json: &str) -> Result<Self, StructureConfigError> {
        let definitions: Vec<StructureDefinition> = serde_json::from_str(json)?;
        
        let mut structures: HashMap<StructureId, Arc<StructureDefinition>> = HashMap::new();
        for structure in definitions {
            // Validation of structure definitions
            StructureConfig::validate_arrays(&structure)?;
            StructureConfig::validate_prerequisities(&structure)?;
            
            let structure_id = structure.id.clone();
            let arc_def = Arc::new(structure);
            structures.insert(structure_id, arc_def);
        }
        Ok(StructureConfig { structures })
    }

    pub fn get(&self, id: &StructureId) -> Option<Arc<StructureDefinition>> {
        self.structures.get(id).cloned()
    }

    fn validate_arrays(definition: &StructureDefinition) -> Result<(), StructureConfigError> {
        let max_level = definition.max_level as usize;

        let sizes_to_check = [
            ("costs", definition.costs.len()),
            ("production", definition.production.len()),
            ("storage_capacity", definition.storage_capacity.len()),
            ("hitpoints", definition.hitpoints.len()),
            ("upgrade_time", definition.upgrade_time.len()),
            ("energy_consumption", definition.energy_consumption.len()),
        ];

        for (field_name, size) in sizes_to_check {
            if size > max_level {
                return Err(
                    StructureConfigError::SizeMismatchError { 
                        structure_name: definition.name.clone(),
                        field_name: field_name.to_string(),
                        expected: max_level, 
                        actual: size 
                    }
                );
            }
        }

        Ok(())
    }

    fn validate_prerequisities(definition: &StructureDefinition) -> Result<(), StructureConfigError> {
        for prerequisity in &definition.prerequisites {
            prerequisity.validate(
                definition.name.as_str(), definition.max_level as usize
            )?
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_valid_config() {
        let json = r#"[
            {
                "id": "metal_mine",
                "name": "Metal Mine",
                "description": "Produces metal",
                "max_level": 3,
                "costs": [
                    {"minerals": 100, "gas": 50, "energy": 0},
                    {"minerals": 200, "gas": 100, "energy": 0},
                    {"minerals": 400, "gas": 200, "energy": 0}
                ],
                "upgrade_time": [100, 200, 400],
                "energy_consumption": [10, 20, 40],
                "hitpoints": [1000, 2000, 4000],
                "production": [
                    {"minerals": 10, "gas": 0, "energy": 0},
                    {"minerals": 20, "gas": 0, "energy": 0},
                    {"minerals": 40, "gas": 0, "energy": 0}
                ],
                "storage_capacity": [
                    {"minerals": 0, "gas": 0, "energy": 0},
                    {"minerals": 0, "gas": 0, "energy": 0},
                    {"minerals": 0, "gas": 0, "energy": 0}
                ],
                "prerequisites": []
            }
        ]"#;

        let result = StructureConfig::load_from_string(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_costs_array_too_large() {
        let json = r#"[
            {
                "id": "metal_mine",
                "name": "Metal Mine",
                "description": "Produces metal",
                "max_level": 2,
                "costs": [
                    {"minerals": 100, "gas": 50, "energy": 0},
                    {"minerals": 200, "gas": 100, "energy": 0},
                    {"minerals": 400, "gas": 200, "energy": 0}
                ],
                "upgrade_time": [100, 200],
                "energy_consumption": [10, 20],
                "hitpoints": [1000, 2000],
                "production": [
                    {"minerals": 10, "gas": 0, "energy": 0},
                    {"minerals": 20, "gas": 0, "energy": 0}
                ],
                "storage_capacity": [
                    {"minerals": 0, "gas": 0, "energy": 0},
                    {"minerals": 0, "gas": 0, "energy": 0}
                ],
                "prerequisites": []
            }
        ]"#;

        let result = StructureConfig::load_from_string(json);
        assert!(result.is_err());

        let err = result.unwrap_err();
        match err {
            StructureConfigError::SizeMismatchError {
                structure_name, field_name, expected, actual
            } => {
                assert_eq!(structure_name, "Metal Mine");
                assert_eq!(field_name, "costs");
                assert_eq!(expected, 2);
                assert_eq!(actual, 3);
            },
            _ => panic!("Expected SizeMismatchError, got {:?}", err)
        }
    }

    #[test]
    fn test_production_array_too_large() {
        let json = r#"[
            {
                "id": "gas_extractor",
                "name": "Gas Extractor",
                "description": "Produces gas",
                "max_level": 1,
                "costs": [
                    {"minerals": 100, "gas": 50, "energy": 0}
                ],
                "upgrade_time": [100],
                "energy_consumption": [10],
                "hitpoints": [1000],
                "production": [
                    {"minerals": 0, "gas": 10, "energy": 0},
                    {"minerals": 0, "gas": 20, "energy": 0}
                ],
                "storage_capacity": [
                    {"minerals": 0, "gas": 0, "energy": 0}
                ],
                "prerequisites": []
            }
        ]"#;

        let result = StructureConfig::load_from_string(json);
        assert!(result.is_err());

        let err = result.unwrap_err();
        match err {
            StructureConfigError::SizeMismatchError { structure_name, field_name, expected, actual } => {
                assert_eq!(structure_name, "Gas Extractor");
                assert_eq!(field_name, "production");
                assert_eq!(expected, 1);
                assert_eq!(actual, 2);
            },
            _ => panic!("Expected SizeMismatchError, got {:?}", err)
        }
    }

    #[test]
    fn test_prerequisite_required_levels_too_large() {
        let json = r#"[
            {
                "id": "fusion_reactor",
                "name": "Fusion Reactor",
                "description": "Advanced energy source",
                "max_level": 2,
                "costs": [
                    {"minerals": 1000, "gas": 500, "energy": 100},
                    {"minerals": 2000, "gas": 1000, "energy": 200}
                ],
                "upgrade_time": [500, 1000],
                "energy_consumption": [0, 0],
                "hitpoints": [5000, 10000],
                "production": [
                    {"minerals": 0, "gas": 0, "energy": 0},
                    {"minerals": 0, "gas": 0, "energy": 0}
                ],
                "storage_capacity": [
                    {"minerals": 0, "gas": 0, "energy": 0},
                    {"minerals": 0, "gas": 0, "energy": 0}
                ],
                "prerequisites": [
                    {
                        "structure_id": "energy_plant",
                        "required_levels": [5, 10, 15]
                    }
                ]
            }
        ]"#;

        let result = StructureConfig::load_from_string(json);
        assert!(result.is_err());

        let err = result.unwrap_err();
        match err {
            StructureConfigError::SizeMismatchError { structure_name, field_name, expected, actual } => {
                assert_eq!(structure_name, "Fusion Reactor");
                assert_eq!(field_name, "prerequisity.required_levels");
                assert_eq!(expected, 2);
                assert_eq!(actual, 3);
            },
            _ => panic!("Expected SizeMismatchError, got {:?}", err)
        }
    }

    #[test]
    fn test_empty_arrays_are_valid() {
        let json = r#"[
            {
                "id": "storage",
                "name": "Storage",
                "description": "Stores resources",
                "max_level": 5,
                "costs": [],
                "upgrade_time": [],
                "energy_consumption": [],
                "hitpoints": [],
                "production": [],
                "storage_capacity": [],
                "prerequisites": []
            }
        ]"#;

        let result = StructureConfig::load_from_string(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_json() {
        let json = r#"{ this is not valid json }"#;

        let result = StructureConfig::load_from_string(json);
        assert!(result.is_err());

        match result.unwrap_err() {
            StructureConfigError::JsonParseError(_serde_json) => {},
            err => panic!("Expected JsonParseError, got {:?}", err)
        }
    }

    #[test]
    fn test_upgrade_time_array_too_large() {
        let json = r#"[
            {
                "id": "test_structure",
                "name": "Test Structure",
                "description": "Test",
                "max_level": 2,
                "costs": [
                    {"minerals": 100, "gas": 50, "energy": 0},
                    {"minerals": 200, "gas": 100, "energy": 0}
                ],
                "upgrade_time": [100, 200, 300],
                "energy_consumption": [10, 20],
                "hitpoints": [1000, 2000],
                "production": [
                    {"minerals": 10, "gas": 0, "energy": 0},
                    {"minerals": 20, "gas": 0, "energy": 0}
                ],
                "storage_capacity": [
                    {"minerals": 0, "gas": 0, "energy": 0},
                    {"minerals": 0, "gas": 0, "energy": 0}
                ],
                "prerequisites": []
            }
        ]"#;

        let result = StructureConfig::load_from_string(json);
        assert!(result.is_err());

        let err = result.unwrap_err();
        match err {
            StructureConfigError::SizeMismatchError {
                structure_name, field_name, expected, actual
            } => {
                assert_eq!(structure_name, "Test Structure");
                assert_eq!(field_name, "upgrade_time");
                assert_eq!(expected, 2);
                assert_eq!(actual, 3);
            },
            _ => panic!("Expected SizeMismatchError for upgrade_time, got {:?}", err)
        }
    }

    #[test]
    fn test_energy_consumption_array_too_large() {
        let json = r#"[
            {
                "id": "test_structure",
                "name": "Test Structure",
                "description": "Test",
                "max_level": 1,
                "costs": [
                    {"minerals": 100, "gas": 50, "energy": 0}
                ],
                "upgrade_time": [100],
                "energy_consumption": [10, 20],
                "hitpoints": [1000],
                "production": [
                    {"minerals": 10, "gas": 0, "energy": 0}
                ],
                "storage_capacity": [
                    {"minerals": 0, "gas": 0, "energy": 0}
                ],
                "prerequisites": []
            }
        ]"#;

        let result = StructureConfig::load_from_string(json);
        assert!(result.is_err());

        let err = result.unwrap_err();
        match err {
            StructureConfigError::SizeMismatchError {
                structure_name, field_name, expected, actual
            } => {
                assert_eq!(structure_name, "Test Structure");
                assert_eq!(field_name, "energy_consumption");
                assert_eq!(expected, 1);
                assert_eq!(actual, 2);
            },
            _ => panic!("Expected SizeMismatchError for energy_consumption, got {:?}", err)
        }
    }
}