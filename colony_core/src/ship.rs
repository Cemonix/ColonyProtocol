use crate::configs::ship_config::ShipId;
use crate::planet::PlanetId;

pub type ShipInstanceId = String;
pub type FleetId = String;

#[derive(Debug, Clone)]
pub struct Ship {
    pub id: ShipInstanceId,
    pub ship_type: ShipId,
    pub location: PlanetId,
    pub fleet_id: Option<FleetId>,
}

impl Ship {
    pub fn new(id: ShipInstanceId, ship_type: ShipId, location: PlanetId) -> Self {
        Self {
            id,
            ship_type,
            location,
            fleet_id: None,
        }
    }

    pub fn is_in_fleet(&self) -> bool {
        self.fleet_id.is_some()
    }
}
