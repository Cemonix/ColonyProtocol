// Configuration modules

pub mod structure_config;
pub mod planet_names;
pub mod player_names;

pub use structure_config::{StructureConfigError, StructureConfig};
pub use planet_names::{PlanetNamesConfigError, PlanetNameParts};
