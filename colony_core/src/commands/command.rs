use super::{ ParseError, ValidationError };
use crate::game::GameState;
use crate::game::PlayerId;


pub trait Command {
    fn name() -> &'static str;
    fn parse(args: &[&str]) -> Result<Box<Self>, ParseError>;
    fn validate(&self, state: &GameState, player: PlayerId) -> Result<(), ValidationError>;
    fn execute(self, state: &mut GameState) -> CommandResult;
    fn completions(position: usize, args: &[&str], state: &GameState) -> Vec<String>;
}

#[derive(Debug)]
pub struct CommandResult {
    pub success: bool,
    pub message: String,
}

impl CommandResult {
    pub fn success(message: &str) -> Self {
        Self {
            success: true,
            message: message.to_string(),
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            message: message.to_string(),
        }
    }
}