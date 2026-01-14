use super::parse::{ParseError, TokenParser};

/// Planet-related commands
#[derive(Debug, PartialEq)]
pub enum PlanetCommand {
    /// Build a structure on a planet
    /// Syntax: `planet build <planet_id> <structure_type>`
    Build {
        planet_id: String,
        structure_type: String,
    },

    /// View planet details
    /// Syntax: `planet view <planet_id>`
    View {
        planet_id: String,
    },
}

impl PlanetCommand {
    /// Parse planet subcommand from tokens
    ///
    /// Expected format:
    /// - `planet build <id> <structure>`
    /// - `planet view <id>`
    pub fn parse(parser: &mut TokenParser) -> Result<Self, ParseError> {
        // Next token should be the subcommand: build, view, etc.
        let subcommand = parser.expect("planet subcommand")?;

        match subcommand {
            "build" => {
                let planet_id = parser.expect("planet_id")?.to_string();
                let structure_type = parser.expect("structure_type")?.to_string();
                Ok(PlanetCommand::Build {
                    planet_id,
                    structure_type,
                })
            }
            "view" => {
                let planet_id = parser.expect("planet_id")?.to_string();
                Ok(PlanetCommand::View { planet_id })
            }
            unknown => Err(ParseError::UnknownCommand(format!(
                "planet {}",
                unknown
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_planet_build() {
        let tokens = vec!["build", "p123", "mine"];
        let mut parser = TokenParser::new(&tokens);
        let result = PlanetCommand::parse(&mut parser);
        assert_eq!(
            result,
            Ok(PlanetCommand::Build {
                planet_id: "p123".to_string(),
                structure_type: "mine".to_string(),
            })
        );
    }

    #[test]
    fn test_parse_planet_view() {
        let tokens = vec!["view", "p123"];
        let mut parser = TokenParser::new(&tokens);
        let result = PlanetCommand::parse(&mut parser);
        assert_eq!(
            result,
            Ok(PlanetCommand::View {
                planet_id: "p123".to_string(),
            })
        );
    }

    #[test]
    fn test_parse_planet_build_missing_args() {
        let tokens = vec!["build", "p123"];
        let mut parser = TokenParser::new(&tokens);
        let result = PlanetCommand::parse(&mut parser);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_unknown_planet_command() {
        let tokens = vec!["destroy", "p123"];
        let mut parser = TokenParser::new(&tokens);
        let result = PlanetCommand::parse(&mut parser);
        assert!(matches!(result, Err(ParseError::UnknownCommand(_))));
    }
}
