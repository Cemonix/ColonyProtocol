// Planet name generation configuration

use serde::Deserialize;

#[cfg(not(test))]
const PLANET_NAMES_CONFIG_PATH: &str = "data/planet_names.json";

#[cfg(test)]
const PLANET_NAMES_CONFIG_PATH: &str = "../data/planet_names.json";

#[derive(thiserror::Error, Debug)]
pub enum PlanetNamesConfigError {
    #[error("Failed to read config file: {0}")]
    FileReadError(#[from] std::io::Error),

    #[error("Failed to parse JSON: {0}")]
    JsonParseError(#[from] serde_json::Error),
}

#[derive(Debug, Deserialize)]
pub struct PlanetNameParts {
    pub prefixes: Vec<String>,
    pub suffixes: Vec<String>,
}

impl PlanetNameParts {
    pub fn load() -> Result<Self, PlanetNamesConfigError> {
        let json_content = std::fs::read_to_string(PLANET_NAMES_CONFIG_PATH)?;
        let name_parts: PlanetNameParts = serde_json::from_str(&json_content)?;
        Ok(name_parts)
    }
}
