use crate::token::Token;
use std::error;
use std::fmt;

use super::runtime_value::RuntimeValue;

#[derive(Debug, Clone)]
pub enum EarlyReturnReason {
    ReturnFromFunction { return_value: RuntimeValue },
}

pub trait InterpreterError: error::Error {
    /// is this error used to facilitate early return ?
    fn is_early_return(&self) -> bool {
        false
    }
    fn early_return_reason(&self) -> Option<EarlyReturnReason> {
        None
    }
}

#[derive(Debug)]
pub struct RuntimeError {
    pub token: Option<Token>,
    pub message: String,
}

impl RuntimeError {
    pub fn new(token: &Token, message: &str) -> Box<RuntimeError> {
        Box::new(RuntimeError {
            token: Some(token.clone()),
            message: String::from(message),
        })
    }
    pub fn new_with_message(message: &str) -> Box<RuntimeError> {
        Box::new(RuntimeError {
            token: None,
            message: String::from(message),
        })
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

impl error::Error for RuntimeError {}
impl InterpreterError for RuntimeError {}

pub type RuntimeResult<T = RuntimeValue> = Result<T, Box<dyn InterpreterError>>;

#[derive(Debug)]
pub struct EarlyReturn {
    pub token: Token,
    pub reason: EarlyReturnReason,
}

impl EarlyReturn {
    pub fn new(token: &Token, reason: EarlyReturnReason) -> Box<EarlyReturn> {
        Box::new(EarlyReturn {
            token: token.clone(),
            reason,
        })
    }
}

impl fmt::Display for EarlyReturn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[line {}]: {:?}", self.token.line, self.reason)
    }
}

impl error::Error for EarlyReturn {}

impl InterpreterError for EarlyReturn {
    fn is_early_return(&self) -> bool {
        true
    }
    fn early_return_reason(&self) -> Option<EarlyReturnReason> {
        Some(self.reason.clone())
    }
}
