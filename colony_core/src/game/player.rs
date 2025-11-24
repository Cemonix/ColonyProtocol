use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlayerId(pub u32);

impl Display for PlayerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Player{}", self.0)
    }
}

#[derive(Debug)]
pub struct Player {
    pub id: PlayerId,
    pub name: String,
}

impl Player {
    pub fn new(id: PlayerId, name: String) -> Self {
        Self { id, name }
    }
}