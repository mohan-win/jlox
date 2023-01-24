use super::interpreter_error::EarlyReturnReason;
use super::{environment::Environment, runtime_value::LoxCallable};
use super::{interpreter_error::RuntimeResult, runtime_value::RuntimeValue, Interpreter};
use crate::ast::Fun;
use std::fmt;
use std::rc::Rc;

#[derive(Debug)]
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

        let result = interpreter.execute_block(&self.function.body, environment);
        if let Err(err) = result {
            if let Some(EarlyReturnReason::ReturnFromFunction { return_value }) =
                err.early_return_reason()
            {
                Ok(return_value)
            } else {
                Err(err)
            }
        } else {
            Ok(RuntimeValue::Nil)
        }
    }
}
