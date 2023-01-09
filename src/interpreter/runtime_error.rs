use crate::token::Token;
use std::error::Error;
use std::fmt;

use super::runtime_value::RuntimeValue;

#[derive(Debug)]
pub struct RuntimeError {
    pub token: Option<Token>,
    pub message: String,
}

impl RuntimeError {
    pub fn new(token: &Token, message: &str) -> RuntimeError {
        RuntimeError {
            token: Some(token.clone()),
            message: String::from(message),
        }
    }
    pub fn new_with_message(message: &str) -> RuntimeError {
        RuntimeError {
            token: None,
            message: String::from(message),
        }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(token) = &self.token {
            write!(f, "[line {}]: {}", token.line, self.message)
        } else {
            write!(f, "[line unknown]: {}", self.message)
        }
    }
}

impl Error for RuntimeError {}

pub type RuntimeResult = Result<RuntimeValue, RuntimeError>;
