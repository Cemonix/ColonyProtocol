use crate::configs::ship_config::ShipId;
use crate::planet::PlanetId;
use crate::resources::Resources;
use crate::ship::FleetId;
use crate::structure::StructureId;

#[derive(Debug, Clone)]
pub enum ActionType {
    BuildStructure(StructureId),
    UpgradeStructure(StructureId),
    BuildShip(ShipId),
    MoveFleet(FleetId, PlanetId),
    BombardPlanet(FleetId, PlanetId),
}

/// Represents an action pending completion (waiting for cooldown to reach 0)
#[derive(Debug, Clone)]
pub struct PendingAction {
    /// Type of action being performed
    pub action_type: ActionType,

    /// Planet where this action is happening
    pub planet_id: PlanetId,

    /// Number of turns remaining until completion (0 = completes this turn)
    pub cooldown_remaining: u32,

    /// Resources reserved for this action (for refund on cancel)
    pub reserved_resources: Resources,
}

impl PendingAction {
    /// Creates a new pending action
    pub fn new(
        action_type: ActionType,
        planet_id: PlanetId,
        cooldown: u32,
        cost: Resources,
    ) -> Self {
        Self {
            action_type,
            planet_id,
            cooldown_remaining: cooldown,
            reserved_resources: cost,
        }
    }

    /// Decrements the cooldown by 1 turn
    pub fn tick(&mut self) {
        self.cooldown_remaining = self.cooldown_remaining.saturating_sub(1);
    }

    /// Checks if the action is complete (cooldown reached 0)
    pub fn is_complete(&self) -> bool {
        self.cooldown_remaining == 0
    }
}
