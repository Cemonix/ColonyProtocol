use super::planet::PlanetId;

pub type PlayerId = String;

pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub planets: Vec<PlanetId>
}