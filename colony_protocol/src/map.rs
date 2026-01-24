use std::collections::HashMap;

use rand::Rng;

use crate::planet::{Connection, Planet, PlanetId};
use crate::planet_name_generator::{PlanetNameGenerator, PlanetNameGeneratorError};
use crate::player::PlayerId;
use crate::utils;

static GRID_HEIGHT: u8 = 40;
static GRID_WIDTH: u8 = 120;
static MAX_DISTANCE: u8 = 5;
static PLANET_ICON: char = '◉';

#[derive(Debug, thiserror::Error)]
pub enum MapError {
    #[error(transparent)]
    PlanetNameGeneratorError(#[from] PlanetNameGeneratorError),
}

pub enum MapSize {
    Small,
    Medium,
    Large
}

impl MapSize {
    pub fn num_planets(&self) -> u32 {
        match self {
            MapSize::Small => 10,
            MapSize::Medium => 20,
            MapSize::Large => 30,
        }
    }
}

pub struct Map {
    pub planets: HashMap<PlanetId, Planet>,
    pub planet_positions: HashMap<PlanetId, (u8, u8)>,
    pub size: MapSize
}

impl Map {
    pub fn generate(size: MapSize, name_generator: &mut PlanetNameGenerator) -> Result<Self, MapError> {
        let num_planets = size.num_planets();

        let mut positions: HashMap<PlanetId, (u8, u8)> = HashMap::with_capacity(num_planets as usize);
        let mut planets: HashMap<PlanetId, Planet> = HashMap::with_capacity(num_planets as usize);
        let mut rng = rand::rng();

        // Generate first planet (root of tree)
        let root_name = name_generator.generate()?;
        let root_id = utils::name_to_id(&root_name);
        let root = Planet::new(root_id.clone(), root_name, None, Vec::new());
        
        let (rand_pos_x, rand_pos_y) = loop {
            let x = rng.random_range(1..GRID_WIDTH - 1);
            let y = rng.random_range(1..GRID_HEIGHT - 1);
            if !positions.values().any(|&pos| pos == (x, y)) {
                break (x, y);
            }
        };
        positions.insert(root_id.clone(), (rand_pos_x, rand_pos_y));

        planets.insert(root_id, root);

        let norm = (GRID_HEIGHT + GRID_WIDTH) / MAX_DISTANCE;

        // Generate remaining planets, connecting each to a random existing planet
        for _ in 1..num_planets {
            let planet_name = name_generator.generate()?;
            let planet_id = utils::name_to_id(&planet_name);

            // Pick random existing planet to connect to
            let keys: Vec<_> = planets.keys().collect();
            let parent_id = keys[rng.random_range(0..keys.len())].clone();
            let parent_position = positions.get(&parent_id)
                .expect("parent_id was just selected from planets.keys()");
            let (parent_x, parent_y) = *parent_position;
            
            let rand_pos_x = rng.random_range(0..GRID_WIDTH);
            let rand_pos_y = rng.random_range(0..GRID_HEIGHT);
            positions.insert(planet_id.clone(), (rand_pos_x, rand_pos_y));

            let distance = rand_pos_x.abs_diff(parent_x) + rand_pos_y.abs_diff(parent_y);
            let distance_scaled = ((distance + norm - 1) / norm).clamp(1, MAX_DISTANCE);

            let connection_to_parent = Connection { 
                to: parent_id.clone(), 
                distance: distance_scaled
            };
            
            // Create new planet with connection to parent
            let new_planet = Planet::new(
                planet_id.clone(),
                planet_name,
                None,
                vec![connection_to_parent.clone()],
            );
            planets.insert(planet_id.clone(), new_planet);

            let connection_to_child = Connection { 
                to: planet_id, 
                distance: connection_to_parent.distance
            };

            // Add bidirectional edge: parent also connects to new planet
            planets.get_mut(&parent_id)
                .expect("parent_id was just selected from planets.keys()")
                .add_connection(connection_to_child);
        }

        Ok(Map {
            planets,
            planet_positions: positions,
            size
        })
    }

    pub fn render_full(&self, player_names: &HashMap<PlayerId, String>) -> String {
        let width = GRID_WIDTH as usize;
        let height = GRID_HEIGHT as usize;

        let mut grid: Vec<char> = vec![' '; width * height];

        // Helper to convert (x, y) to flat index
        let idx = |x: usize, y: usize| y * width + x;

        // Draw borders
        for x in 0..width {
            grid[idx(x, 0)] = '#';
            grid[idx(x, height - 1)] = '#';
        }
        for y in 0..height {
            grid[idx(0, y)] = '#';
            grid[idx(width - 1, y)] = '#';
        }

        // Draw connection lines between planets
        for (planet_id, planet) in &self.planets {
            let Some(&(x1, y1)) = self.planet_positions.get(planet_id) else { continue };

            for connection in planet.get_connections() {
                let Some(&(x2, y2)) = self.planet_positions.get(&connection.to) else { continue };
                Self::draw_line(&mut grid, width, x1 as i32, y1 as i32, x2 as i32, y2 as i32);
            }
        }

        // Draw planets on top of lines
        for (_, &(x, y)) in &self.planet_positions {
            grid[idx(x as usize, y as usize)] = PLANET_ICON;
        }

        // Draw labels on top of everything (so they don't get interrupted by edges)
        for (planet_id, &(x, y)) in &self.planet_positions {
            let planet = self.planets.get(planet_id).expect("planet_id exists in planet_positions");
            let label = if let Some(owner_id) = planet.get_owner() {
                let owner_name = player_names.get(owner_id).map(|s| s.as_str()).unwrap_or("Unknown");
                format!(" {} ({})", planet_id, owner_name)
            } else {
                format!(" {}", planet_id)
            };

            // Write label chars into grid, overwriting everything except borders
            let label_start_x = x as usize + 1;
            for (i, ch) in label.chars().enumerate() {
                let label_x = label_start_x + i;
                if label_x < width - 1 {
                    let current_char = grid[idx(label_x, y as usize)];
                    // Don't overwrite borders (#) or planet icons (◉)
                    if current_char != '#' && current_char != PLANET_ICON {
                        grid[idx(label_x, y as usize)] = ch;
                    }
                }
            }
        }

        // Convert grid to string
        let mut map = String::with_capacity((width + 1) * height);
        for y in 0..height {
            for x in 0..width {
                map.push(grid[idx(x, y)]);
            }
            map.push('\n');
        }
        map
    }

    /// Draw a line between two points using Bresenham's algorithm
    fn draw_line(grid: &mut [char], width: usize, mut x1: i32, mut y1: i32, x2: i32, y2: i32) {
        let dx = (x2 - x1).abs();
        let dy = -(y2 - y1).abs();
        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };
        let mut err = dx + dy;

        loop {
            let idx = y1 as usize * width + x1 as usize;

            // Choose line character based on direction
            let ch = if dx == 0 {
                '│'
            } else if dy == 0 {
                '─'
            } else if (sx > 0 && sy > 0) || (sx < 0 && sy < 0) {
                '\\'
            } else {
                '/'
            };

            // Only draw if cell is empty (don't overwrite planets or borders)
            if grid[idx] == ' ' {
                grid[idx] = ch;
            }

            if x1 == x2 && y1 == y2 {
                break;
            }

            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x1 += sx;
            }
            if e2 <= dx {
                err += dx;
                y1 += sy;
            }
        }
    }
}