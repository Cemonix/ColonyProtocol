// Planet graph generation

use std::collections::HashMap;

use rand::Rng;
use thiserror::Error;

use crate::{planet::{Planet, PlanetId}, utils};
use super::name_generator::{NameGenerator, NameGeneratorError};

#[derive(Debug, Error)]
pub enum GraphGeneratorError {
    #[error("Number of planets must be at least 1")]
    InvalidPlanetCount,

    #[error(transparent)]
    NameGeneratorError(#[from] NameGeneratorError),

}

pub struct GraphGenerator {
    num_planets: u32,
    name_generator: NameGenerator,
}

impl std::fmt::Debug for GraphGenerator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GraphGenerator")
            .field("num_planets", &self.num_planets)
            .finish_non_exhaustive()
    }
}

impl GraphGenerator {
    pub fn new(num_planets: u32, name_generator: NameGenerator) -> Result<Self, GraphGeneratorError> {
        if num_planets == 0 {
            return Err(GraphGeneratorError::InvalidPlanetCount);
        }

        Ok(GraphGenerator {
            num_planets,
            name_generator,
        })
    }

    /// Generate a connected graph of planets as a tree structure.
    /// Each planet has at least one bidirectional connection, ensuring all planets are reachable.
    pub fn generate(&mut self) -> Result<HashMap<PlanetId, Planet>, GraphGeneratorError> {
        let mut planets: HashMap<PlanetId, Planet> = HashMap::with_capacity(self.num_planets as usize);
        let mut rng = rand::rng();

        // Generate first planet (root of tree)
        let root_name = self.name_generator.generate()?;
        let root_id = utils::name_to_id(&root_name);
        let root = Planet::new(root_id.clone(), root_name, None, Vec::new());
        planets.insert(root_id, root);

        // Generate remaining planets, connecting each to a random existing planet
        for _ in 1..self.num_planets {
            let planet_name = self.name_generator.generate()?;
            let planet_id = utils::name_to_id(&planet_name);

            // Pick random existing planet to connect to
            let keys: Vec<_> = planets.keys().collect();
            let parent_id = keys[rng.random_range(0..keys.len())].clone();

            // Create new planet with connection to parent
            let new_planet = Planet::new(
                planet_id.clone(),
                planet_name,
                None,
                vec![parent_id.clone()],
            );
            planets.insert(planet_id.clone(), new_planet);

            // Add bidirectional edge: parent also connects to new planet
            planets.get_mut(&parent_id)
                .expect("parent_id was just selected from planets.keys()")
                .add_connection(planet_id);
        }

        Ok(planets)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_generator_rejects_zero_planets() {
        let name_gen = NameGenerator::new().unwrap();
        let result = GraphGenerator::new(0, name_gen);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GraphGeneratorError::InvalidPlanetCount));
    }

    #[test]
    fn test_generate_single_planet() {
        let name_gen = NameGenerator::new().unwrap();
        let mut graph_gen = GraphGenerator::new(1, name_gen).unwrap();

        let planets = graph_gen.generate().unwrap();
        assert_eq!(planets.len(), 1);

        // Single planet has no connections (it's the root)
        let planet = planets.values().next().unwrap();
        assert_eq!(planet.get_connections().len(), 0);
    }

    #[test]
    fn test_generate_tree_structure() {
        let name_gen = NameGenerator::new().unwrap();
        let mut graph_gen = GraphGenerator::new(10, name_gen).unwrap();

        let planets = graph_gen.generate().unwrap();
        assert_eq!(planets.len(), 10);

        // Verify the id field matches the key
        for (id, planet) in &planets {
            assert_eq!(id, &planet.id, "Key doesn't match planet.id");
        }

        // Verify bidirectional edges
        for planet in planets.values() {
            for neighbor_id in planet.get_connections() {
                let neighbor = planets.get(neighbor_id)
                    .expect(&format!("Neighbor {} not found", neighbor_id));
                assert!(
                    neighbor.get_connections().contains(&planet.id),
                    "Edge from {} to {} is not bidirectional",
                    planet.id,
                    neighbor_id
                );
            }
        }
    }

    #[test]
    fn test_all_planets_have_unique_names() {
        let name_gen = NameGenerator::new().unwrap();
        let mut graph_gen = GraphGenerator::new(20, name_gen).unwrap();

        let planets = graph_gen.generate().unwrap();

        let mut names = std::collections::HashSet::new();
        for planet in planets.values() {
            assert!(names.insert(planet.name.clone()), "Duplicate planet name: {}", planet.name);
        }
    }
}
