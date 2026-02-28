use crate::planet::PlanetId;
use crate::ship::{FleetId, ShipInstanceId};

#[derive(Debug, Clone)]
pub struct Fleet {
    pub id: FleetId,
    pub name: String,
    pub ships: Vec<ShipInstanceId>,
    pub location: PlanetId,
}

impl Fleet {
    pub fn new(id: FleetId, name: String, location: PlanetId) -> Self {
        Self {
            id,
            name,
            ships: Vec::new(),
            location,
        }
    }

    pub fn add_ship(&mut self, ship_id: ShipInstanceId) {
        if !self.ships.contains(&ship_id) {
            self.ships.push(ship_id);
        }
    }

    pub fn remove_ship(&mut self, ship_id: &ShipInstanceId) -> bool {
        if let Some(pos) = self.ships.iter().position(|s| s == ship_id) {
            self.ships.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn is_empty(&self) -> bool {
        self.ships.is_empty()
    }

    pub fn ship_count(&self) -> usize {
        self.ships.len()
    }
}
