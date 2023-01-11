use std::collections::HashMap;

use crate::token::Token;

use super::{
    runtime_error::{RuntimeError, RuntimeResult},
    runtime_value::RuntimeValue,
};

pub struct Environment(HashMap<String, RuntimeValue>);

impl Environment {
    pub fn new() -> Environment {
        Environment(HashMap::new())
    }
    pub fn define(&mut self, name: &str, value: RuntimeValue) {
        self.0.insert(String::from(name), value);
    }
    pub fn get(&self, name: &Token) -> RuntimeResult {
        if let Some(value) = self.0.get(&name.lexeme) {
            Ok(value.clone())
        } else {
            Err(RuntimeError::new(
                name,
                format!("Undefined variable \"{}\".", name.lexeme).as_str(),
            ))
        }
    }
}
