use super::PlayerId;

pub enum ActionType {
    Build,
    Research,
    Explore,
}

pub struct PendingAction {
    action_type: ActionType,
    target: String,
    turns_remaining: u32,
    player_id: PlayerId,
}