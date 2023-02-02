use super::interpreter_error::EarlyReturnReason;
use super::lox_class::LoxInstance;
use super::{environment::Environment, runtime_value::LoxCallable};
use super::{interpreter_error::RuntimeResult, runtime_value::RuntimeValue, Interpreter};
use crate::ast::Fun;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

#[derive(Debug)]
pub struct LoxFunction {
    declaration: Fun,
    closure: Rc<RefCell<Environment>>,
}

impl LoxFunction {
    pub fn new(declaration: &Fun, closure: &Rc<RefCell<Environment>>) -> LoxFunction {
        LoxFunction {
            declaration: declaration.clone(),
            closure: Rc::clone(closure),
        }
    }
    pub fn bind(&self, instance: &Rc<RefCell<LoxInstance>>) -> LoxFunction {
        let mut environment = Environment::new_with(Rc::clone(&self.closure));
        environment.define("this", RuntimeValue::Instance(Rc::clone(instance)));
        LoxFunction::new(&self.declaration, &Rc::new(RefCell::new(environment)))
    }
}

impl fmt::Display for LoxFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<fn {}>", self.declaration.name.lexeme)
    }
}

impl LoxCallable for LoxFunction {
    fn arity(&self) -> usize {
        self.declaration.params.len()
    }

    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<RuntimeValue>) -> RuntimeResult {
        let mut environment = Environment::new_with(Rc::clone(&self.closure));
        for (i, arg) in arguments.into_iter().enumerate() {
            let param = &self.declaration.params[i];
            environment.define(param.lexeme.as_str(), arg)
        }

        let result = interpreter.execute_block(&self.declaration.body, environment);
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
