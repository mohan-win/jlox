use crate::{
    ast::{Expr, Stmt},
    token::TokenType,
};

pub mod environment;
pub mod runtime_error;
pub mod runtime_value;

use self::{
    environment::Environment,
    runtime_error::{RuntimeError, RuntimeResult},
    runtime_value::RuntimeValue,
};

pub struct Interpreter {
    environment: Option<Box<Environment>>,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            environment: Some(Box::new(Environment::new())),
        }
    }
    pub fn interpret(&mut self, statements: &Vec<Stmt>) -> RuntimeResult<()> {
        statements
            .iter()
            .try_for_each(|statement| self.execute(statement))
    }

    fn execute(&mut self, statement: &Stmt) -> RuntimeResult<()> {
        match statement {
            Stmt::Var { name, expression } => {
                let mut value = RuntimeValue::Nil;
                if let Some(expression) = expression {
                    value = self.evaluate(expression)?;
                }
                self.environment
                    .as_mut()
                    .unwrap()
                    .define(&name.lexeme, value)
            }
            Stmt::PrintStmt { expression } => {
                let value = self.evaluate(expression)?;
                println!("{}", value);
            }
            Stmt::ExpressionStmt { expression } => {
                self.evaluate(expression)?;
            }
            Stmt::Block { statements } => {
                let existing_environment = self.environment.take().unwrap();
                self.execute_block(statements, Environment::new_with(existing_environment))?;
            }
        }
        Ok(())
    }

    fn execute_block(
        &mut self,
        statements: &Vec<Stmt>,
        block_environment: Environment,
    ) -> RuntimeResult<()> {
        // set block environment
        self.environment = Some(Box::new(block_environment));

        let result = statements
            .iter()
            .try_for_each(|statement| self.execute(statement));

        // restore enclosing block;
        self.environment = self.environment.take().unwrap().take_enclosing();

        result
    }

    fn evaluate(&mut self, expr: &Expr) -> RuntimeResult {
        match expr {
            Expr::Grouping { expression } => self.evaluate(expression),
            Expr::Unary { operator, right } => {
                let right = self.evaluate(right)?;

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
                let left = self.evaluate(left)?;
                let right = self.evaluate(right)?;
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
            Expr::Variable(name) => self.environment.as_ref().unwrap().get(name),
            Expr::Assign { name, value } => {
                let value = self.evaluate(value)?;
                self.environment.as_mut().unwrap().assign(name, value)
            }
        }
    }
}
