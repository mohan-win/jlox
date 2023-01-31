use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use crate::ast::VarPosition;

use super::{interpreter_error::RuntimeResult, runtime_value::RuntimeValue};

#[derive(Debug)]
pub struct Environment {
    values: Vec<RuntimeValue>,
    enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            values: Vec::new(),
            enclosing: None,
        }
    }
    /// instantate environment with an `outer_scope` environment
    pub fn new_with(outer_scope: Rc<RefCell<Environment>>) -> Environment {
        Environment {
            values: Vec::new(),
            enclosing: Some(outer_scope),
        }
    }
    pub fn define(&mut self, value: RuntimeValue) {
        self.values.push(value);
    }

    pub fn get_at(&self, pos: &VarPosition) -> RuntimeResult {
        let value = self.env_at_depth(pos.depth, |env| {
            env.values
                .get(pos.index)
                .expect("vars should be found in the environment at exact depth & index")
                .clone()
        });
        Ok(value)
    }

    pub fn assign_at(&mut self, value: RuntimeValue, pos: &VarPosition) -> RuntimeResult {
        self.env_mut_at_depth(pos.depth, |env| {
            env.values[pos.index] = value.clone();
        });

        Ok(value)
    }

    pub fn take_enclosing(&mut self) -> Option<Rc<RefCell<Environment>>> {
        self.enclosing.take()
    }

    fn ancestor(&self, depth: usize) -> Rc<RefCell<Environment>> {
        let mut environment = Some(Rc::clone(self.enclosing.as_ref().unwrap()));
        let mut i = 1;
        while i < depth {
            i += 1;
            let t = Rc::clone(
                environment
                    .take()
                    .unwrap()
                    .as_ref()
                    .borrow()
                    .enclosing
                    .as_ref()
                    .unwrap(),
            );
            environment = Some(t);
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
            f(&self.ancestor(depth).as_ref().borrow())
        }
    }
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Length = {}", self.values.len())?;
        self.values
            .iter()
            .try_for_each(|value| write!(f, "{:?}", value))
    }
}
