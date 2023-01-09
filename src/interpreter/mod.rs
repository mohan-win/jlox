use crate::{ast::Expr, token::TokenType};

pub mod runtime_value;

use self::runtime_value::RuntimeValue;

pub fn evaluate(expr: &Expr) -> RuntimeValue {
    match expr {
        Expr::Grouping { expression } => evaluate(expression),
        Expr::Unary { operator, right } => {
            let right = evaluate(right);

            match operator.token_type {
                TokenType::MINUS => -right,
                TokenType::BANG => !right.is_truthy(),
                _ => panic!("Only - and ! are supported as a unary operator!"), // ToDo::
            }
        }
        Expr::Binary {
            left,
            operator,
            right,
        } => {
            let left = evaluate(left);
            let right = evaluate(right);
            match operator.token_type {
                TokenType::MINUS => left - right,
                TokenType::PLUS => left + right,
                TokenType::STAR => left * right,
                TokenType::SLASH => left / right,
                TokenType::GREATER => RuntimeValue::Boolean(left > right),
                TokenType::GREATER_EQUAL => RuntimeValue::Boolean(left >= right),
                TokenType::LESS => RuntimeValue::Boolean(left < right),
                TokenType::LESS_EQUAL => RuntimeValue::Boolean(left <= right),
                TokenType::BANG_EQUAL => !RuntimeValue::Boolean(left == right),
                TokenType::EQUAL_EQUAL => RuntimeValue::Boolean(left == right),
                _ => panic!("Unsupported operator"), // ToDo::
            }
        }
        Expr::Litral(litral) => litral.clone().into(),
    }
}
