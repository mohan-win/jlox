use crate::{
    ast::{Expr, Stmt},
    token::TokenType,
};

pub mod runtime_error;
pub mod runtime_value;

use self::{
    runtime_error::{RuntimeError, RuntimeResult},
    runtime_value::RuntimeValue,
};

pub fn interpret(statements: &Vec<Stmt>) -> RuntimeResult<()> {
    statements
        .iter()
        .try_for_each(|statement| execute(statement))
}

fn execute(statement: &Stmt) -> RuntimeResult<()> {
    match statement {
        Stmt::PrintStmt { expression } => {
            let value = evaluate(expression)?;
            println!("{}", value);
        }
        Stmt::ExpressionStmt { expression } => {
            evaluate(expression)?;
        }
    }
    Ok(())
}

fn evaluate(expr: &Expr) -> RuntimeResult {
    match expr {
        Expr::Grouping { expression } => evaluate(expression),
        Expr::Unary { operator, right } => {
            let right = evaluate(right)?;

            match operator.token_type {
                TokenType::MINUS => -right,
                TokenType::BANG => !right.is_truthy(),
                _ => Err(RuntimeError::new(
                    operator,
                    "Only - and ! are supported as a unary operator!",
                )),
            }
        }
        Expr::Binary {
            left,
            operator,
            right,
        } => {
            let left = evaluate(left)?;
            let right = evaluate(right)?;
            let result = match operator.token_type {
                TokenType::MINUS => left - right,
                TokenType::PLUS => left + right,
                TokenType::STAR => left * right,
                TokenType::SLASH => left / right,
                TokenType::GREATER => Ok(RuntimeValue::Boolean(left > right)),
                TokenType::GREATER_EQUAL => Ok(RuntimeValue::Boolean(left >= right)),
                TokenType::LESS => Ok(RuntimeValue::Boolean(left < right)),
                TokenType::LESS_EQUAL => Ok(RuntimeValue::Boolean(left <= right)),
                TokenType::BANG_EQUAL => !RuntimeValue::Boolean(left == right),
                TokenType::EQUAL_EQUAL => Ok(RuntimeValue::Boolean(left == right)),
                _ => Err(RuntimeError::new(operator, "Unsupported operator")),
            };
            result.map_err(|e| RuntimeError::new(operator, &e.message))
        }
        Expr::Litral(litral) => Ok(litral.clone().into()),
    }
}
