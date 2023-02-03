use crate::{
    ast::{Expr, Stmt},
    token::{Token, TokenType},
};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub mod environment;
pub mod interpreter_error;
pub mod lox_class;
pub mod lox_function;
pub mod native_functions;
pub mod runtime_value;

use self::{
    environment::Environment,
    interpreter_error::{
        EarlyReturn, EarlyReturnReason, InterpreterError, RuntimeError, RuntimeResult,
    },
    lox_class::{LoxClass, LoxInstance},
    lox_function::LoxFunction,
    native_functions::NativeFnClock,
    runtime_value::RuntimeValue,
};

pub struct Interpreter {
    globals: Rc<RefCell<Environment>>,
    environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        let globals = Interpreter::define_globals();
        let globals_clone = Rc::clone(&globals);
        Interpreter {
            globals,
            environment: globals_clone,
        }
    }

    pub fn interpret(&mut self, statements: &Vec<Stmt>) -> RuntimeResult<()> {
        statements
            .iter()
            .try_for_each(|statement| self.execute(statement))
    }

    fn define_globals() -> Rc<RefCell<Environment>> {
        let environment = Rc::new(RefCell::new(Environment::new()));
        let clock = Rc::new(NativeFnClock {});
        (*environment)
            .borrow_mut()
            .define("clock", RuntimeValue::Callable(clock));
        environment
    }

    /*
       Helper methods for environment.
    */

    /// Execute statement
    fn execute(&mut self, statement: &Stmt) -> RuntimeResult<()> {
        match statement {
            Stmt::Class { name, methods } => {
                self.environment
                    .borrow_mut()
                    .define(&name.lexeme, RuntimeValue::Nil);

                let mut methods_map: HashMap<String, Rc<LoxFunction>> = HashMap::new();
                methods.iter().for_each(|method| {
                    methods_map.insert(
                        method.name.lexeme.clone(),
                        Rc::new(LoxFunction::new(
                            method,
                            &self.environment,
                            method.name.lexeme.eq("init"),
                        )),
                    );
                });

                let kclass = Rc::new(LoxClass::new(&name.lexeme, methods_map));

                self.environment
                    .borrow_mut()
                    .assign(name, RuntimeValue::Callable(kclass))?;
            }
            Stmt::Var { name, expression } => {
                let mut value = RuntimeValue::Nil;
                if let Some(expression) = expression {
                    value = self.evaluate(expression)?;
                }
                self.environment.borrow_mut().define(&name.lexeme, value)
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
            Stmt::Return { keyword, value } => {
                if let Some(value) = value {
                    let return_value = self.evaluate(value)?;
                    return Err(EarlyReturn::new(
                        keyword,
                        EarlyReturnReason::ReturnFromFunction { return_value },
                    ));
                } else {
                    return Err(EarlyReturn::new(
                        keyword,
                        EarlyReturnReason::ReturnFromFunction {
                            return_value: RuntimeValue::Nil,
                        },
                    ));
                }
            }
            Stmt::Block { statements } => {
                let existing_environment = Rc::clone(&self.environment);
                self.execute_block(statements, Environment::new_with(existing_environment))?;
            }
            Stmt::Function(fun) => {
                let function = Rc::new(LoxFunction::new(fun, &self.environment, false));
                self.environment
                    .borrow_mut()
                    .define(fun.name.lexeme.as_str(), RuntimeValue::Callable(function))
            }
        }
        Ok(())
    }

    /// Helper for executing block
    fn execute_block(
        &mut self,
        statements: &Vec<Stmt>,
        block_environment: Environment,
    ) -> RuntimeResult<()> {
        // set block environment
        let old_environment = Rc::clone(&self.environment);
        self.environment = Rc::new(RefCell::new(block_environment));

        let result = statements
            .iter()
            .try_for_each(|statement| self.execute(statement));

        // restore old environment;
        self.environment = old_environment;

        result
    }

    /// Helper for evaluating expression
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
                    _ => Err(RuntimeError::new(operator, "Unsupported operator")
                        as Box<dyn InterpreterError>),
                };
                result.map_err(|err| {
                    // Note: add the token to the runtime error, so that error message
                    // can include the line.
                    RuntimeError::new(operator, err.message().unwrap_or_default())
                        as Box<dyn InterpreterError>
                })
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
            Expr::Variable { name, depth } => match depth {
                Some(depth) => self.environment.borrow().get_at(&name.lexeme, *depth),
                None => self.globals.borrow().get(name),
            },
            Expr::This { keyword, depth } => match depth {
                Some(depth) => self.environment.borrow().get_at(&keyword.lexeme, *depth),
                None => {
                    panic!("'this' can't be in global scope")
                }
            },
            Expr::Assign { name, value, depth } => {
                let value = self.evaluate(value)?;
                match depth {
                    Some(depth) => {
                        self.environment
                            .borrow_mut()
                            .assign_at(&name.lexeme, value, *depth)
                    }
                    None => self.globals.borrow_mut().assign(name, value),
                }
            }
            Expr::Call {
                callee,
                paran,
                arguments,
            } => self.evaluate_function_call(callee, paran, arguments),
            Expr::Get { object, name } => {
                if let RuntimeValue::Instance(instance) = self.evaluate(object)? {
                    let value = LoxInstance::get(&instance, name);
                    match value {
                        Some(value) => Ok(value),
                        None => Err(RuntimeError::new(
                            name,
                            format!("Property {} not found in the object", name.lexeme).as_str(),
                        )),
                    }
                } else {
                    Err(RuntimeError::new(name, "Only instance can have properties"))
                }
            }
            Expr::Set {
                object,
                name,
                value,
            } => {
                let object = self.evaluate(object)?;
                if let RuntimeValue::Instance(instance) = object {
                    let value = self.evaluate(value)?;
                    Ok(instance.as_ref().borrow_mut().set(name, value))
                } else {
                    Err(RuntimeError::new(
                        name,
                        "Left of a '.' expression should be an instance",
                    ))
                }
            }
        }
    }

    /// Helper for evaluating function call
    fn evaluate_function_call(
        &mut self,
        callee: &Box<Expr>,
        paran: &Token,
        arguments: &Vec<Expr>,
    ) -> RuntimeResult {
        if let RuntimeValue::Callable(function) = self.evaluate(callee)? {
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
