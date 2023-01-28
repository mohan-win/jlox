use std::fmt;
use std::rc::Rc;
use std::{cell::RefCell, collections::HashMap};

use crate::token::Token;

use super::{
    interpreter_error::{RuntimeError, RuntimeResult},
    runtime_value::RuntimeValue,
};

#[derive(Debug)]
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

    pub fn get_at(&self, name: &Token, depth: usize) -> RuntimeResult {
        let runtime_value = self.env_at_depth(depth, |env| {
            env.values
                .get(&name.lexeme)
                .expect("Local names should be found in the environment at exact depth")
                .clone()
        });
        Ok(runtime_value)
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

    pub fn assign_at(&mut self, name: &Token, value: RuntimeValue, depth: usize) -> RuntimeResult {
        self.env_mut_at_depth(depth, |env| {
            env.values.insert(name.lexeme.clone(), value.clone());
        });

        Ok(value)
    }

    pub fn take_enclosing(&mut self) -> Option<Rc<RefCell<Environment>>> {
        self.enclosing.take()
    }

    fn ancestor(&self, depth: usize) -> &Rc<RefCell<Environment>> {
        let mut environment = None;
        let mut i = 0;
        while i < depth {
            environment = Some(
                self.enclosing
                    .as_ref()
                    .expect("There should be an environment at exact depth"),
            );
            i += 1;
        }
        environment.unwrap()
    }

    fn env_mut_at_depth<F, R>(&mut self, depth: usize, f: F) -> R
    where
        F: FnOnce(&mut Environment) -> R,
    {
        if depth == 0 {
            f(self)
        } else {
            f(&mut self.ancestor(depth).borrow_mut())
        }
    }
    fn env_at_depth<F, R>(&self, depth: usize, f: F) -> R
    where
        F: FnOnce(&Environment) -> R,
    {
        if depth == 0 {
            f(self)
        } else {
            f(&self.ancestor(depth).borrow())
        }
    }
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.values
            .iter()
            .try_for_each(|value| write!(f, "{} {}", value.0, value.1))
    }
}
