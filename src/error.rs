use crate::{interpreter::runtime_error::RuntimeError, parser::ParserError, token::TokenType};

pub fn error(line: usize, message: &str) {
    report(line, "", message)
}

pub fn error_in_parser(err: &ParserError) {
    if err.token_type == TokenType::EOF {
        report(err.line, " at end", err.message.as_str())
    } else {
        report(
            err.line,
            format!("at '{:?}'", err.token_type).as_str(),
            err.message.as_str(),
        )
    }
}

pub fn report(line: usize, where_in: &str, message: &str) {
    eprintln!("[Line {}] Error {}: {}", line, where_in, message)
}

pub fn error_at_runtime(err: &RuntimeError) {
    if let Some(token) = &err.token {
        eprintln!(
            "Runtime error: [Line {} on {:?}] {}",
            token.line, token.token_type, err.message
        );
    } else {
        eprintln!("Runtime error: {}", err.message);
    }
}
