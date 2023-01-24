use crate::{
    ast::{Expr, Stmt},
    token::{Token, TokenType},
};
use std::{cell::RefCell, rc::Rc};

pub mod environment;
pub mod lox_function;
pub mod native_functions;
pub mod runtime_error;
pub mod runtime_value;

use self::{
    environment::Environment,
    lox_function::LoxFunction,
    native_functions::NativeFnClock,
    runtime_error::{RuntimeError, RuntimeResult},
    runtime_value::RuntimeValue,
};

pub struct Interpreter {
    globals: Rc<RefCell<Environment>>,
    environment: Option<Rc<RefCell<Environment>>>,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        let globals = Interpreter::define_globals();
        let globals_clone = Rc::clone(&globals);
        Interpreter {
            globals,
            environment: Some(globals_clone),
        }
    }

    fn define_globals() -> Rc<RefCell<Environment>> {
        let environment = Rc::new(RefCell::new(Environment::new()));
        let clock = Rc::new(NativeFnClock {});
        (*environment)
            .borrow_mut()
            .define("clock", RuntimeValue::Function(clock));
        environment
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
                    .as_ref()
                    .borrow_mut()
                    .define(&name.lexeme, value)
            }
            Stmt::ExpressionStmt { expression } => {
                self.evaluate(expression)?;
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
            }
            Stmt::WhileStmt { condition, body } => {
                while bool::from(self.evaluate(condition)?) {
                    self.execute(body)?;
                }
            }
            Stmt::PrintStmt { expression } => {
                let value = self.evaluate(expression)?;
                println!("{}", value);
            }
            Stmt::Block { statements } => {
                let existing_environment = self.environment.take().unwrap();
                self.execute_block(statements, Environment::new_with(existing_environment))?;
            }
            Stmt::Function(fun) => {
                let function = Rc::new(LoxFunction::new(fun));
                self.environment
                    .as_mut()
                    .unwrap()
                    .as_ref()
                    .borrow_mut()
                    .define(fun.name.lexeme.as_str(), RuntimeValue::Function(function))
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
        self.environment = Some(Rc::new(RefCell::new(block_environment)));

        let result = statements
            .iter()
            .try_for_each(|statement| self.execute(statement));

        // restore enclosing block;
        self.environment = self
            .environment
            .take()
            .unwrap()
            .as_ref()
            .borrow_mut()
            .take_enclosing();

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
            Expr::Variable(name) => self.environment.as_ref().unwrap().borrow().get(name),
            Expr::Assign { name, value } => {
                let value = self.evaluate(value)?;
                self.environment
                    .as_mut()
                    .unwrap()
                    .as_ref()
                    .borrow_mut()
                    .assign(name, value)
            }
            Expr::Call {
                callee,
                paran,
                arguments,
            } => self.evaluate_function_call(callee, paran, arguments),
        }
    }

    fn evaluate_function_call(
        &mut self,
        callee: &Box<Expr>,
        paran: &Token,
        arguments: &Vec<Expr>,
    ) -> RuntimeResult {
        if let RuntimeValue::Function(function) = self.evaluate(callee)? {
            let mut argument_vals = Vec::new();
            if function.arity() != arguments.len() {
                Err(RuntimeError::new(
                    paran,
                    format!(
                        "Expected: {} arguments, but given {} arguments",
                        function.arity(),
                        arguments.len()
                    )
                    .as_str(),
                ))
            } else {
                arguments
                    .iter()
                    .try_for_each(|argument| match self.evaluate(argument) {
                        Ok(argument_val) => {
                            argument_vals.push(argument_val);
                            Ok(())
                        }
                        Err(err) => Err(err),
                    })?;

                function.call(self, argument_vals)
            }
        } else {
            Err(RuntimeError::new(
                paran,
                "Only functions and classes are callable",
            ))
        }
    }
}
