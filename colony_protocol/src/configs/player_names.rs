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
    let player_data: PlayerNamesData = serde_json::from_str(&data)?;

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

    #[test]
    fn test_generate_unique_names() {
        let names = generate_random_names(3).unwrap();

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
        let names = generate_random_names(1).unwrap();
        assert!(!names[0].is_empty());
    }
}
