use thiserror::Error;

/// Errors that can occur during command parsing
#[derive(Debug, Error, PartialEq)]
pub enum ParseError {
    #[error("Empty command - type 'help' for available commands")]
    EmptyCommand,

    #[error("Unknown command: '{0}'. Type 'help' for available commands")]
    UnknownCommand(String),

    #[error("Invalid syntax for '{command}': {reason}")]
    InvalidSyntax { command: String, reason: String },

    #[error("Missing required argument: {0}")]
    MissingArgument(String),
}

/// Helper type for working with command tokens
/// Wraps a slice of string tokens with utility methods
pub struct TokenParser<'a> {
    tokens: &'a [&'a str],
    position: usize,
}

impl<'a> TokenParser<'a> {
    pub fn new(tokens: &'a [&'a str]) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    /// Peek at the current token without consuming it
    pub fn peek(&self) -> Option<&'a str> {
        self.tokens.get(self.position).copied()
    }

    /// Consume and return the next token
    pub fn next(&mut self) -> Option<&'a str> {
        let token = self.peek();
        if token.is_some() {
            self.position += 1;
        }
        token
    }

    /// Consume a token and return it, or error if missing
    pub fn expect(&mut self, arg_name: &str) -> Result<&'a str, ParseError> {
        self.next()
            .ok_or_else(|| ParseError::MissingArgument(arg_name.to_string()))
    }

    /// Check if there are remaining tokens
    pub fn has_remaining(&self) -> bool {
        self.position < self.tokens.len()
    }

    /// Get all remaining tokens as a slice
    pub fn remaining(&self) -> &'a [&'a str] {
        &self.tokens[self.position..]
    }
}
