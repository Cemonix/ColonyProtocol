use std::fmt::Display;

use super::player::PlayerId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlanetId(pub u32);

impl Display for PlanetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Planet{}", self.0)
    }
}

pub struct Planet {
    pub id: PlanetId,
    pub owner_id: PlayerId,
    pub name: String,
}