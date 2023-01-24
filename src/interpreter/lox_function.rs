use super::{environment::Environment, runtime_value::LoxCallable};
use super::{runtime_error::RuntimeResult, runtime_value::RuntimeValue, Interpreter};
use crate::ast::Fun;
use std::fmt;
use std::rc::Rc;

pub struct LoxFunction {
    function: Fun,
}

impl LoxFunction {
    pub fn new(function: &Fun) -> LoxFunction {
        LoxFunction {
            function: function.clone(),
        }
    }
}

impl<'a> fmt::Display for LoxFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<fn {}>", self.function.name.lexeme)
    }
}

impl<'a> LoxCallable for LoxFunction {
    fn arity(&self) -> usize {
        self.function.params.len()
    }

    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<RuntimeValue>) -> RuntimeResult {
        let mut environment = Environment::new_with(Rc::clone(&interpreter.globals));
        for (i, arg) in arguments.into_iter().enumerate() {
            let param = &self.function.params[i];
            environment.define(param.lexeme.as_str(), arg)
        }

        interpreter.execute_block(&self.function.body, environment)?;
        Ok(RuntimeValue::Nil)
    }
}
