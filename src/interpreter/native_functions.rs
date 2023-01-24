use std::fmt;
use std::time::SystemTime;

use super::{
    interpreter_error::{RuntimeError, RuntimeResult},
    runtime_value::{LoxCallable, RuntimeValue},
    Interpreter,
};

#[derive(Debug)]
pub struct NativeFnClock;

impl LoxCallable for NativeFnClock {
    fn arity(&self) -> usize {
        0
    }
    fn call(&self, _interpreter: &mut Interpreter, arguments: Vec<RuntimeValue>) -> RuntimeResult {
        if arguments.len() != 0 {
            Err(RuntimeError::new_with_message(
                "calling native clock requires zero arguments",
            ))
        } else {
            match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
                Ok(time) => Ok(RuntimeValue::Number(time.as_secs_f64())),
                Err(err) => Err(RuntimeError::new_with_message(
                    format!("{:?}", err).as_str(),
                )),
            }
        }
    }
}

impl fmt::Display for NativeFnClock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<native fn clock>")
    }
}
