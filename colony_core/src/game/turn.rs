use super::state::GameState;

pub trait TurnSystem {
    fn process(&mut self, state: &mut GameState);
}