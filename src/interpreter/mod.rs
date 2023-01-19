use crate::{
    ast::{Expr, Stmt},
    token::TokenType,
};

pub mod environment;
pub mod interpreter_error;
pub mod runtime_value;

use self::{
    environment::Environment,
    interpreter_error::{
        EarlyReturn, EarlyReturnReason, InterpreterError, RuntimeError, RuntimeResult,
    },
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
        let result = match statement {
            Stmt::PrintStmt { expression } => {
                let value = self.evaluate(expression)?;
                println!("{}", value);
                Ok(())
            }
            Stmt::BreakStmt { token } => {
                Err(EarlyReturn::new(token, &EarlyReturnReason::BreakFromLoop)
                    as Box<dyn InterpreterError>)
            }
            Stmt::Var { name, expression } => {
                let mut value = RuntimeValue::Nil;
                if let Some(expression) = expression {
                    value = self.evaluate(expression)?;
                }
                self.environment
                    .as_mut()
                    .unwrap()
                    .define(&name.lexeme, value);
                Ok(())
            }
            Stmt::ExpressionStmt { expression } => {
                self.evaluate(expression)?;
                Ok(())
            }
            Stmt::IfStmt {
                condition,
                then_branch,
                else_branch,
            } => {
                if bool::from(self.evaluate(condition)?) {
                    self.execute(then_branch)?;
                } else if let Some(else_branch) = else_branch {
                    self.execute(else_branch)?;
                }
                Ok(())
            }
            Stmt::WhileStmt { condition, body } => {
                while bool::from(self.evaluate(condition)?) {
                    let result = self.execute(body);
                    if let Err(err) = result {
                        if let Some(EarlyReturnReason::BreakFromLoop) = err.early_return_reason() {
                            break;
                        } else {
                            return Err(err);
                        }
                    }
                }
                Ok(())
            }
            Stmt::Block { statements } => {
                let existing_environment = self.environment.take().unwrap();
                self.execute_block(statements, Environment::new_with(existing_environment))?;
                Ok(())
            }
        };
        result
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
                    _ => {
                        let err: Box<dyn InterpreterError> =
                            RuntimeError::new(operator, "Unsupported operator");
                        Err(err)
                    }
                };
                result
            }
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let left = self.evaluate(left)?;
                match (&operator.token_type, bool::from(&left)) {
                    (&TokenType::OR, true) | (&TokenType::AND, false) => Ok(left),
                    _ => {
                        let right = self.evaluate(right)?;
                        Ok(right)
                    }
                }
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
