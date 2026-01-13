// Greek-style planet name generator

use std::collections::HashSet;

use rand::Rng;
use rand::rngs::ThreadRng;
use thiserror::Error;

use crate::configs::{PlanetNameParts, PlanetNamesConfigError};

#[derive(Debug, Error)]
pub enum NameGeneratorError {
    #[error("Failed to generate unique planet name: all variants exhausted for base name")]
    AllVariantsExhausted,

    #[error(transparent)]
    PlanetNamesConfigError(#[from] PlanetNamesConfigError),
}

pub struct NameGenerator {
    name_parts: PlanetNameParts,
    used_names: HashSet<String>,
    rng: ThreadRng,
}

impl NameGenerator {
    /// Load name parts from configuration
    pub fn new() -> Result<Self, NameGeneratorError> {
        let name_parts = PlanetNameParts::load()?;

        Ok(NameGenerator {
            name_parts,
            used_names: HashSet::new(),
            rng: rand::rng(),
        })
    }

    /// Generate a unique Greek-style planet name by combining a random prefix with a random suffix.
    /// If a collision occurs, appends Roman numerals (I-X) to ensure uniqueness.
    ///
    /// Examples: "Crimson Theta", "Void Kepler II", "Azure Prime V"
    ///
    /// Returns error if all 10 variants of a base name are exhausted.
    pub fn generate(&mut self) -> Result<String, NameGeneratorError> {
        // Pick random prefix and suffix for base name
        let prefix = &self.name_parts.prefixes[self.rng.random_range(0..self.name_parts.prefixes.len())];
        let suffix = &self.name_parts.suffixes[self.rng.random_range(0..self.name_parts.suffixes.len())];
        let base_name = format!("{} {}", prefix, suffix);

        // Try base name first (no Roman numeral)
        if !self.used_names.contains(&base_name) {
            self.used_names.insert(base_name.clone());
            return Ok(base_name);
        }

        // Base name taken, try Roman numerals I through X
        for variant in 1..=10 {
            let roman = Self::to_roman_numeral(variant);
            let candidate = format!("{} {}", base_name, roman);

            if !self.used_names.contains(&candidate) {
                self.used_names.insert(candidate.clone());
                return Ok(candidate);
            }
        }

        // All 10 variants exhausted for this base name
        Err(NameGeneratorError::AllVariantsExhausted)
    }

    /// Convert numbers 1-10 to Roman numerals
    fn to_roman_numeral(n: u8) -> &'static str {
        match n {
            1 => "I",
            2 => "II",
            3 => "III",
            4 => "IV",
            5 => "V",
            6 => "VI",
            7 => "VII",
            8 => "VIII",
            9 => "IX",
            10 => "X",
            _ => panic!("to_roman_numeral only supports 1-10"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_name_generator() {
        let generator = NameGenerator::new();
        assert!(generator.is_ok());
    }

    #[test]
    fn test_generate_unique_names() {
        let mut generator = NameGenerator::new().unwrap();

        // Generate 10 names and verify they're all unique
        let mut generated_names = HashSet::new();
        for _ in 0..10 {
            let name = generator.generate().unwrap();

            // Name should not be empty
            assert!(!name.is_empty());

            // Name should be unique
            assert!(generated_names.insert(name.clone()), "Generated duplicate name: {}", name);
        }
    }

    #[test]
    fn test_generated_name_format() {
        let mut generator = NameGenerator::new().unwrap();
        let name = generator.generate().unwrap();

        // Name should have 2-3 parts (prefix, suffix, optional roman numeral)
        let parts: Vec<&str> = name.split(' ').collect();
        assert!(parts.len() >= 2 && parts.len() <= 3, "Name '{}' has {} parts, expected 2-3", name, parts.len());

        let prefix = parts[0];
        let suffix = parts[1];

        // Verify prefix and suffix are from our JSON file
        assert!(generator.name_parts.prefixes.contains(&prefix.to_string()));
        assert!(generator.name_parts.suffixes.contains(&suffix.to_string()));

        // If there's a third part, it should be a Roman numeral
        if parts.len() == 3 {
            let roman = parts[2];
            assert!(
                ["I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X"].contains(&roman),
                "Invalid Roman numeral: {}", roman
            );
        }
    }

    #[test]
    fn test_collision_handling_with_roman_numerals() {
        let mut generator = NameGenerator::new().unwrap();

        // Force a collision by manually inserting a base name
        let test_base = format!("{} {}",
            &generator.name_parts.prefixes[0],
            &generator.name_parts.suffixes[0]
        );
        generator.used_names.insert(test_base.clone());

        // Generate a name - should get Roman numeral variant
        let name = generator.generate().unwrap();

        // If it's the same base combo, should have Roman numeral
        if name.starts_with(&test_base) {
            assert!(name.ends_with(" I"), "Expected Roman numeral I, got: {}", name);
        }
    }
}
