use super::interpreter_error::EarlyReturnReason;
use super::runtime_value::{LoxCallableInstance, LoxInstance};
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
    is_initializer: bool,
}

impl LoxFunction {
    pub fn new(
        declaration: &Fun,
        closure: &Rc<RefCell<Environment>>,
        is_initializer: bool,
    ) -> LoxFunction {
        LoxFunction {
            declaration: declaration.clone(),
            is_initializer,
            closure: Rc::clone(closure),
        }
    }

    pub fn bind<T: LoxInstance + Clone + 'static>(&self, instance: &T) -> LoxFunction {
        let mut environment = Environment::new_with(Rc::clone(&self.closure));
        environment.define("this", RuntimeValue::Instance(Rc::new(instance.clone())));
        LoxFunction::new(
            &self.declaration,
            &Rc::new(RefCell::new(environment)),
            self.is_initializer,
        )
    }

    pub fn bind_callable_instance<T: LoxCallableInstance + Clone + 'static>(
        &self,
        instance: &T,
    ) -> LoxFunction {
        let mut environment = Environment::new_with(Rc::clone(&self.closure));
        environment.define(
            "this",
            RuntimeValue::CallableInstance(Rc::new(instance.clone())),
        );
        LoxFunction::new(
            &self.declaration,
            &Rc::new(RefCell::new(environment)),
            self.is_initializer,
        )
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
        if self.is_initializer && result.is_ok() {
            // return 'this' from constructor
            self.closure.borrow().get_at("this", 0)
        } else if let Err(err) = result {
            if let Some(EarlyReturnReason::ReturnFromFunction { return_value }) =
                err.early_return_reason()
            {
                if self.is_initializer {
                    // return 'this' from constructor
                    assert!(
                        RuntimeValue::Nil == return_value,
                        "Return statement inside constructor can't have value"
                    );
                    self.closure.borrow().get_at("this", 0)
                } else {
                    Ok(return_value)
                }
            } else {
                Err(err)
            }
        } else {
            Ok(RuntimeValue::Nil)
        }
    }
}
