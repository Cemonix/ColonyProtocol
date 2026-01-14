use super::parse::{ParseError, TokenParser};

/// Help command - displays available commands
#[derive(Debug, PartialEq)]
pub enum HelpCommand {
    /// Show general help
    General,
}

impl HelpCommand {
    /// Parse help command from tokens
    ///
    /// Syntax: `help`
    pub fn parse(_parser: &mut TokenParser) -> Result<Self, ParseError> {
        // Help takes no arguments for now
        // Could extend later: `help planet`, `help fleet`, etc.
        Ok(HelpCommand::General)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_help() {
        let tokens = vec![];
        let mut parser = TokenParser::new(&tokens);
        let result = HelpCommand::parse(&mut parser);
        assert_eq!(result, Ok(HelpCommand::General));
    }
}
