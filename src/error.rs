use crate::interpreter::interpreter_error::InterpreterError;
use std::error::Error;

pub fn error(line: usize, message: &str) {
    eprintln!("[Line {}] Error: {}", line, message)
}

pub fn error_at_compiler(err: &dyn Error) {
    eprintln!("{}", err);
}

pub fn error_at_runtime(err: Box<dyn InterpreterError>) {
    eprintln!("Runtime error: {}", err);
}

pub fn report(line: usize, where_in: &str, message: &str) {
    eprintln!("[Line {}] Error {}: {}", line, where_in, message)
}
