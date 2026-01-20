use super::planet::PlanetId;
use super::pending_action::PendingAction;

pub type PlayerId = String;

pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub planets: Vec<PlanetId>,
    pub pending_actions: Vec<PendingAction>,
}

impl Player {
    /// Checks if the player has a pending action on the specified planet.
    /// Since only one action per planet is allowed, this returns true if any action exists for that planet.
    pub fn has_pending_action_on_planet(&self, planet_id: &PlanetId) -> bool {
        self.pending_actions
            .iter()
            .any(|action| &action.planet_id == planet_id)
    }

    /// Finds an immutable reference to the pending action on the specified planet.
    pub fn find_pending_action_on_planet(&self, planet_id: &PlanetId) -> Option<&PendingAction> {
        self.pending_actions
            .iter()
            .find(|action| &action.planet_id == planet_id)
    }

    /// Finds a mutable reference to the pending action on the specified planet.
    pub fn find_pending_action_on_planet_mut(
        &mut self,
        planet_id: &PlanetId,
    ) -> Option<&mut PendingAction> {
        self.pending_actions
            .iter_mut()
            .find(|action| &action.planet_id == planet_id)
    }

    /// Removes and returns the pending action on the specified planet, if it exists.
    pub fn remove_pending_action_on_planet(&mut self, planet_id: &PlanetId) -> Option<PendingAction> {
        self.pending_actions
            .iter()
            .position(|action| &action.planet_id == planet_id)
            .map(|index| self.pending_actions.remove(index))
    }
}