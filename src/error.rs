use crate::interpreter::runtime_error::RuntimeError;

pub fn error(line: usize, message: &str) {
    report(line, "", message)
}

pub fn report(line: usize, where_in: &str, message: &str) {
    eprintln!("[Line {}] Error {}: {}", line, where_in, message)
}

pub fn runtime_error(err: &RuntimeError) {
    if let Some(token) = &err.token {
        eprintln!(
            "Runtime error: [Line {} on {:?}] {}",
            token.line, token.token_type, err.message
        );
    } else {
        eprintln!("Runtime error: {}", err.message);
    }
}
