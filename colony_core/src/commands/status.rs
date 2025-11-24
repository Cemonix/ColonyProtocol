use crate::game::{ GameState, PlanetId, PlayerId };
use crate::commands::{ Command, CommandResult };
use super::{ ParseError, ValidationError };

#[derive(Debug)]
pub struct StatusCommand {
    pub target: StatusTarget,
}

#[derive(Debug)]
pub enum StatusTarget {
    Planet { id: PlanetId },
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_planet_status() {
        // Test: "planet status c418" should parse correctly
        let args = vec!["planet", "c418"];
        let result = StatusCommand::parse(&args);

        assert!(result.is_ok());
        let command = result.unwrap();

        match command.target {
            StatusTarget::Planet { name } => {
                assert_eq!(name, "c418");
            }
        }
    }

    #[test]
    fn test_parse_missing_planet_name() {
        // Test: "planet status" without a name should error
        let args = vec!["planet"];
        let result = StatusCommand::parse(&args);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.message, "Missing planet name");
    }

    #[test]
    fn test_parse_unknown_target() {
        // Test: "status fleet" should error (not implemented yet)
        let args = vec!["fleet", "fleet-1"];
        let result = StatusCommand::parse(&args);

        assert!(result.is_err());
    }

    #[test]
    fn test_execute_planet_status() {
        // Test: executing planet status should return information
        let state = GameState::new();

        let command = StatusCommand {
            target: StatusTarget::Planet {
                name: "c418".to_string(),
            },
        };

        let result = command.execute(&state);
        assert!(result.success);
        assert!(result.message.contains("c418"));
    }
}
