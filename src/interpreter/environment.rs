use std::rc::Rc;
use std::{cell::RefCell, collections::HashMap};

use crate::token::Token;

use super::{
    runtime_error::{RuntimeError, RuntimeResult},
    runtime_value::RuntimeValue,
};

pub struct Environment {
    values: HashMap<String, RuntimeValue>,
    enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            values: HashMap::new(),
            enclosing: None,
        }
    }
    /// instantate environment with an `outer_scope` environment
    pub fn new_with(outer_scope: Rc<RefCell<Environment>>) -> Environment {
        Environment {
            values: HashMap::new(),
            enclosing: Some(outer_scope),
        }
    }
    pub fn define(&mut self, name: &str, value: RuntimeValue) {
        self.values.insert(String::from(name), value);
    }
    pub fn get(&self, name: &Token) -> RuntimeResult {
        if let Some(value) = self.values.get(&name.lexeme) {
            Ok(value.clone())
        } else {
            self.enclosing.as_ref().map_or(
                Err(RuntimeError::new(
                    name,
                    format!("Undefined variable \"{}\".", name.lexeme).as_str(),
                )),
                |enclosing| (**enclosing).borrow().get(name),
            )
        }
    }
    pub fn assign(&mut self, name: &Token, value: RuntimeValue) -> RuntimeResult {
        if let Some(_) = self.values.get(name.lexeme.as_str()) {
            self.values.insert(name.lexeme.clone(), value.clone());
            Ok(value)
        } else {
            self.enclosing.as_mut().map_or(
                Err(RuntimeError::new(
                    name,
                    format!("Variable {} is not declared", name.lexeme).as_str(),
                )),
                |enclosing| enclosing.borrow_mut().assign(name, value),
            )
        }
    }

    pub fn take_enclosing(&mut self) -> Option<Rc<RefCell<Environment>>> {
        self.enclosing.take()
    }
}
