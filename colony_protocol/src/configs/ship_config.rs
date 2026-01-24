use std::collections::HashMap;
use std::sync::Arc;

use thiserror::Error;

use crate::resources::Resources;

pub type ShipId = String;

const SHIP_CONFIG_PATH: &str = "data/ships.json";

#[derive(Debug, Error)]
pub enum ShipConfigError {
    #[error("Failed to read config file: {0}")]
    FileReadError(#[from] std::io::Error),

    #[error("Failed to parse JSON: {0}")]
    JsonParseError(#[from] serde_json::Error),

    #[error("Ship '{ship_name}' has invalid counter reference: '{counter_id}'")]
    InvalidCounterReference {
        ship_name: String,
        counter_id: ShipId,
    },
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct ShipDefinition {
    pub id: ShipId,
    pub name: String,
    pub description: String,
    pub attack: u32,
    pub shield: u32,
    pub bombardment: u32,
    pub cost: Resources,
    pub build_time: u32,
    pub counters: Vec<ShipId>,
    pub required_shipyard_level: u16,
}

#[derive(Debug)]
pub struct ShipConfig {
    ships: HashMap<ShipId, Arc<ShipDefinition>>,
}

impl ShipConfig {
    pub fn load() -> Result<Self, ShipConfigError> {
        let json_content = std::fs::read_to_string(SHIP_CONFIG_PATH)?;
        Self::load_from_string(&json_content)
    }

    pub fn load_from_string(json: &str) -> Result<Self, ShipConfigError> {
        let definitions: Vec<ShipDefinition> = serde_json::from_str(json)?;

        let mut ships: HashMap<ShipId, Arc<ShipDefinition>> = HashMap::new();
        for ship in definitions {
            let ship_id = ship.id.clone();
            let arc_def = Arc::new(ship);
            ships.insert(ship_id, arc_def);
        }

        // Validate counter references after all ships are loaded
        Self::validate_counters(&ships)?;

        Ok(ShipConfig { ships })
    }

    pub fn get(&self, id: &ShipId) -> Option<Arc<ShipDefinition>> {
        self.ships.get(id).cloned()
    }

    fn validate_counters(ships: &HashMap<ShipId, Arc<ShipDefinition>>) -> Result<(), ShipConfigError> {
        for ship in ships.values() {
            for counter_id in &ship.counters {
                if !ships.contains_key(counter_id) {
                    return Err(ShipConfigError::InvalidCounterReference {
                        ship_name: ship.name.clone(),
                        counter_id: counter_id.clone(),
                    });
                }
            }
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
                "id": "interceptor",
                "name": "Interceptor",
                "description": "Fast fighter for fleet combat",
                "attack": 10,
                "shield": 5,
                "bombardment": 0,
                "cost": {"minerals": 100, "gas": 50, "energy": 0},
                "build_time": 2,
                "counters": ["interceptor", "ravager"],
                "required_shipyard_level": 1
            },
            {
                "id": "ravager",
                "name": "Ravager",
                "description": "Heavy bomber for planetary bombardment",
                "attack": 5,
                "shield": 15,
                "bombardment": 25,
                "cost": {"minerals": 200, "gas": 100, "energy": 0},
                "build_time": 3,
                "counters": [],
                "required_shipyard_level": 2
            }
        ]"#;

        let result = ShipConfig::load_from_string(json);
        assert!(result.is_ok());

        let config = result.unwrap();
        assert!(config.get(&"interceptor".to_string()).is_some());
        assert!(config.get(&"ravager".to_string()).is_some());
    }

    #[test]
    fn test_invalid_counter_reference() {
        let json = r#"[
            {
                "id": "interceptor",
                "name": "Interceptor",
                "description": "Fast fighter",
                "attack": 10,
                "shield": 5,
                "bombardment": 0,
                "cost": {"minerals": 100, "gas": 50, "energy": 0},
                "build_time": 2,
                "counters": ["nonexistent_ship"],
                "required_shipyard_level": 1
            }
        ]"#;

        let result = ShipConfig::load_from_string(json);
        assert!(result.is_err());

        match result.unwrap_err() {
            ShipConfigError::InvalidCounterReference { ship_name, counter_id } => {
                assert_eq!(ship_name, "Interceptor");
                assert_eq!(counter_id, "nonexistent_ship");
            }
            err => panic!("Expected InvalidCounterReference, got {:?}", err),
        }
    }

    #[test]
    fn test_invalid_json() {
        let json = r#"{ this is not valid json }"#;

        let result = ShipConfig::load_from_string(json);
        assert!(result.is_err());

        match result.unwrap_err() {
            ShipConfigError::JsonParseError(_) => {}
            err => panic!("Expected JsonParseError, got {:?}", err),
        }
    }
}
