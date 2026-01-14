mod parse;
pub mod help;
pub mod planet;

pub use parse::ParseError;
use parse::TokenParser;
pub use help::HelpCommand;
pub use planet::PlanetCommand;

/// Top-level command enum representing all possible commands
#[derive(Debug, PartialEq)]
pub enum Command {
    /// Display help information
    Help(HelpCommand),

    /// End the current turn and process game state
    EndTurn,

    /// Planet-related commands (build, view, etc.)
    Planet(PlanetCommand),

    // Future commands:
    // Fleet(FleetCommand),
    // Status,
    // Save(String),
    // Load(String),
}

impl Command {
    /// Parse a command from a user input string
    ///
    /// # Arguments
    /// * `input` - Raw user input string (e.g., "planet build p123 mine")
    ///
    /// # Returns
    /// * `Ok(Command)` - Successfully parsed command
    /// * `Err(ParseError)` - Parsing failed with specific error
    ///
    /// # Examples
    /// ```
    /// let cmd = Command::parse("help")?;
    /// let cmd = Command::parse("planet build p123 mine")?;
    /// let cmd = Command::parse("end-turn")?;
    /// ```
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        // Split input into tokens by whitespace
        let tokens: Vec<&str> = input.split_whitespace().collect();

        // Handle empty input
        if tokens.is_empty() {
            return Err(ParseError::EmptyCommand);
        }

        // Create token parser to help with parsing
        let mut parser = TokenParser::new(&tokens);

        // First token is the main command
        let main_command = parser.next().unwrap(); // Safe: we checked tokens is not empty

        match main_command {
            "help" => Ok(Command::Help(HelpCommand::parse(&mut parser)?)),

            "end-turn" | "endturn" => Ok(Command::EndTurn),

            "planet" => Ok(Command::Planet(PlanetCommand::parse(&mut parser)?)),

            // Future commands would be added here:
            // "fleet" => Ok(Command::Fleet(FleetCommand::parse(&mut parser)?)),
            // "status" => Ok(Command::Status),

            unknown => Err(ParseError::UnknownCommand(unknown.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_help() {
        assert_eq!(Command::parse("help"), Ok(Command::Help(HelpCommand::General)));
    }

    #[test]
    fn test_parse_end_turn() {
        assert_eq!(Command::parse("end-turn"), Ok(Command::EndTurn));
        assert_eq!(Command::parse("endturn"), Ok(Command::EndTurn));
    }

    #[test]
    fn test_parse_planet_build() {
        let result = Command::parse("planet build p123 mine");
        assert!(matches!(result, Ok(Command::Planet(PlanetCommand::Build { .. }))));
    }

    #[test]
    fn test_parse_planet_view() {
        let result = Command::parse("planet view p123");
        assert!(matches!(result, Ok(Command::Planet(PlanetCommand::View { .. }))));
    }

    #[test]
    fn test_parse_empty_command() {
        assert_eq!(Command::parse(""), Err(ParseError::EmptyCommand));
        assert_eq!(Command::parse("   "), Err(ParseError::EmptyCommand));
    }

    #[test]
    fn test_parse_unknown_command() {
        let result = Command::parse("destroy p123");
        assert!(matches!(result, Err(ParseError::UnknownCommand(_))));
    }
}
