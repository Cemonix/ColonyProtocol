use serde::Deserialize;
use std::fs;
use rand::prelude::IndexedRandom;

#[derive(Debug, thiserror::Error)]
pub enum PlayerNameConfigError {
    #[error("Not enough names in configuration. Need {needed}, but only {available} available")]
    InsufficientNames { needed: usize, available: usize },
    
    #[error("Failed to read config file: {0}")]
    FileReadError(#[from] std::io::Error),

    #[error("Failed to parse JSON: {0}")]
    JsonParseError(#[from] serde_json::Error),
}

#[derive(Debug, Deserialize)]
struct PlayerNamesData {
    names: Vec<String>,
}

/// Loads player names from the JSON config file and returns a random selection
/// without duplicates.
///
/// # Arguments
/// * `count` - Number of unique names to generate
///
/// # Returns
/// A vector of unique player names
///
/// # Errors
/// Returns `PlayerNameError` if:
/// - The config file cannot be read
/// - The JSON is malformed
/// - There aren't enough names in the config for the requested count
pub fn generate_random_names(count: usize) -> Result<Vec<String>, PlayerNameConfigError> {
    let data = fs::read_to_string("data/player_names.json")?;
    generate_random_names_from_string(&data, count)
}

pub fn generate_random_names_from_string(json: &str, count: usize) -> Result<Vec<String>, PlayerNameConfigError> {
    let player_data: PlayerNamesData = serde_json::from_str(json)?;

    if player_data.names.len() < count {
        return Err(PlayerNameConfigError::InsufficientNames {
            needed: count,
            available: player_data.names.len()
        });
    }

    let mut rng = rand::rng();
    Ok(
        player_data.names
            .choose_multiple(&mut rng, count)
            .cloned()
            .collect()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_JSON: &str = r#"{"names": ["Alice", "Bob", "Charlie", "Diana", "Eve"]}"#;

    #[test]
    fn test_generate_unique_names() {
        let names = generate_random_names_from_string(TEST_JSON, 3).unwrap();

        assert_eq!(names.len(), 3);

        // Check all names are unique
        for i in 0..names.len() {
            for j in (i+1)..names.len() {
                assert_ne!(names[i], names[j], "Names should be unique");
            }
        }
    }

    #[test]
    fn test_names_are_not_empty() {
        let names = generate_random_names_from_string(TEST_JSON, 1).unwrap();
        assert!(!names[0].is_empty());
    }

    #[test]
    fn test_insufficient_names() {
        let result = generate_random_names_from_string(TEST_JSON, 10);
        assert!(result.is_err());

        match result.unwrap_err() {
            PlayerNameConfigError::InsufficientNames { needed, available } => {
                assert_eq!(needed, 10);
                assert_eq!(available, 5);
            }
            err => panic!("Expected InsufficientNames, got {:?}", err),
        }
    }
}
