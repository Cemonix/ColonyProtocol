use super::Commands;

pub struct CommandSystem;

impl CommandSystem {
    pub fn new() -> Self {
        CommandSystem
    }

    pub fn parse_command(&self, input: &str) -> Option<String> {
        
    }
}

// Commands example
// Base structure:
// <entity> <action> <target> [options]
// planet   build    c418     metal-mine
// fleet    move     fleet-1  sector-5
// System: response with information about what command did - started building mine on planet c418 ready in 5 turns.