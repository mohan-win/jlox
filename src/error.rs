use crate::{
    interpreter::interpreter_error::InterpreterError, parser::ParserError, token::Token,
    token::TokenType,
};

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

pub fn error_in_token(token: &Token, message: &str) {
    if token.token_type == TokenType::EOF {
        report(token.line, " at end", message)
    } else {
        report(
            token.line,
            format!("at '{:?}'", token.token_type).as_str(),
            message,
        )
    }
}

pub fn report(line: usize, where_in: &str, message: &str) {
    eprintln!("[Line {}] Error {}: {}", line, where_in, message)
}

pub fn error_at_runtime(err: Box<dyn InterpreterError>) {
    eprintln!("Runtime error: {}", err);
}
