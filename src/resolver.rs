use crate::{ast::Fun, token::Token};

use super::ast::{Expr, Stmt};
use std::collections::HashMap;
use std::error;
use std::fmt;

pub struct Resolver {
    scopes: Vec<HashMap<String, (bool, usize, usize)>>, // Vec<HashMap<variable_name, (is_initialized, num_of_usage)>
    current_function: Option<FunctionType>,
    num_of_resolver_errs: usize,
}

impl Resolver {
    pub fn new() -> Resolver {
        Resolver {
            scopes: Vec::new(),
            current_function: None,
            num_of_resolver_errs: 0,
        }
    }

    pub fn get_num_of_resolver_errs(&self) -> usize {
        self.num_of_resolver_errs
    }

    pub fn resolve(&mut self, stmts: &mut Vec<Stmt>) {
        self.resolve_stmts(stmts);
    }

    fn resolve_stmts(&mut self, stmts: &mut Vec<Stmt>) {
        stmts.iter_mut().for_each(|stmt| self.resolve_stmt(stmt));
    }

    fn resolve_stmt(&mut self, stmt: &mut Stmt) {
        match stmt {
            Stmt::Block { statements } => {
                self.begin_scope();
                self.resolve_stmts(statements);
                self.end_scope();
            }
            Stmt::Var { name, expression } => {
                self.declare(name);
                if let Some(expression) = expression {
                    self.resolve_expr(expression)
                }
                self.define(name)
            }
            Stmt::Function(fun) => {
                let enclosing_function = self.current_function.take();
                self.current_function = Some(FunctionType::Function);

                self.declare(&fun.name);
                self.define(&fun.name);
                self.resolve_function(fun);

                self.current_function = enclosing_function;
            }
            Stmt::PrintStmt { expression } => self.resolve_expr(expression),
            Stmt::ExpressionStmt { expression } => self.resolve_expr(expression),
            Stmt::IfStmt {
                condition,
                then_branch,
                else_branch,
            } => {
                self.resolve_expr(condition);
                self.resolve_stmt(then_branch);
                else_branch.as_mut().and_then(|else_branch| {
                    self.resolve_stmt(else_branch);
                    Some(())
                });
            }
            Stmt::WhileStmt { condition, body } => {
                self.resolve_expr(condition);
                self.resolve_stmt(body.as_mut());
            }
            Stmt::Return { keyword, value } => {
                if let Some(FunctionType::Function) = self.current_function {
                    value.as_mut().and_then(|value| {
                        self.resolve_expr(value);
                        Some(())
                    });
                } else {
                    self.error(&ResolverError::new(
                        &keyword,
                        "Return statement allowed only inside a function",
                    ))
                }
            }
        }
    }

    fn resolve_expr(&mut self, expr: &mut Expr) {
        match expr {
            Expr::Variable { name, depth } => {
                if !self.scopes.is_empty() {
                    if let Some((false, ..)) = self.scopes.last().unwrap().get(&name.lexeme) {
                        self.error(&ResolverError::new(
                            name,
                            "Can't read local variable in its own initializer",
                        ))
                    }
                }
                self.increment_usage(name);
                *depth = self.resolve_local_depth(name)
            }
            Expr::Assign { name, value, depth } => {
                self.resolve_expr(value);
                self.increment_usage(name);
                *depth = self.resolve_local_depth(name)
            }
            Expr::Litral(_) => {}
            Expr::Unary { operator: _, right } => {
                self.resolve_expr(right);
            }
            Expr::Binary {
                left,
                operator: _,
                right,
            } => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }
            Expr::Logical {
                left,
                operator: _,
                right,
            } => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }
            Expr::Grouping { expression } => {
                self.resolve_expr(expression);
            }
            Expr::Call {
                callee,
                paran: _,
                arguments,
            } => {
                self.resolve_expr(callee);
                arguments
                    .iter_mut()
                    .for_each(|argument| self.resolve_expr(argument));
            }
        }
    }

    fn resolve_function(&mut self, fun: &mut Fun) {
        self.begin_scope();
        fun.params.iter().for_each(|param| {
            self.declare(param);
            self.define(param);
        });
        self.resolve_stmts(fun.body.as_mut());
        self.end_scope();
    }

    fn resolve_local_depth(&self, name: &Token) -> Option<usize> {
        let result = self
            .scopes
            .iter()
            .rev()
            .enumerate()
            .try_for_each(|(i, scope)| {
                if scope.contains_key(&name.lexeme) {
                    Err(i)
                } else {
                    Ok(())
                }
            });
        if let Err(depth) = result {
            Some(depth)
        } else {
            None
        }
    }

    fn begin_scope(&mut self) {
        self.scopes
            .push(HashMap::<String, (bool, usize, usize)>::new());
    }

    fn end_scope(&mut self) {
        if let Some(scope_popped) = self.scopes.pop() {
            Resolver::warn_unused_variables(&scope_popped)
        }
    }

    fn warn_unused_variables(scope: &HashMap<String, (bool, usize, usize)>) {
        for (var_name, (_, num_of_usage, line)) in scope {
            if *num_of_usage == 0 {
                Resolver::warn(&ResolverError::new_with_message(
                    format!("[line: {}] Variable {} is not used", line, var_name).as_str(),
                ))
            }
        }
    }

    fn declare(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }
        let scope = self.scopes.last_mut().unwrap();

        if !scope.contains_key(&name.lexeme) {
            scope.insert(name.lexeme.clone(), (false, 0, name.line));
        } else {
            self.error(&ResolverError::new(
                name,
                "Already a variable with this name in this scope",
            ))
        }
    }

    fn define(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }

        self.scopes
            .last_mut()
            .unwrap()
            .insert(name.lexeme.clone(), (true, 0, name.line));
    }

    fn increment_usage(&mut self, name: &Token) {
        let scope = self.scopes.iter_mut().rev().try_for_each(|scope| {
            if scope.contains_key(name.lexeme.as_str()) {
                Err(scope)
            } else {
                Ok(())
            }
        });

        match scope {
            Err(scope) => {
                let (is_defined, num_of_usage, line) = scope
                    .get(name.lexeme.as_str())
                    .expect("variable should be present");

                scope.insert(name.lexeme.clone(), (*is_defined, *num_of_usage + 1, *line));
            }
            Ok(()) => {
                self.error(&ResolverError::new(
                    name,
                    "Variable is used before it's declaration",
                ));
            }
        }
    }

    fn error(&mut self, err: &ResolverError) {
        self.num_of_resolver_errs += 1;
        crate::error::error_at_compiler(err)
    }

    fn warn(err: &ResolverError) {
        crate::error::error_at_compiler(err)
    }
}

enum FunctionType {
    Function,
}

#[derive(Debug)]
struct ResolverError {
    token: Option<Token>,
    message: String,
}

impl ResolverError {
    fn new(token: &Token, message: &str) -> ResolverError {
        ResolverError {
            token: Some(token.clone()),
            message: String::from(message),
        }
    }
    fn new_with_message(message: &str) -> ResolverError {
        ResolverError {
            token: None,
            message: String::from(message),
        }
    }
}

impl<'a> fmt::Display for ResolverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.token.as_ref() {
            Some(token) => write!(
                f,
                "Resolver error: line {} at [{:?}] {}",
                token.line, token.token_type, self.message
            ),
            None => write!(f, "Resolver error: {}", self.message),
        }
    }
}

impl<'a> error::Error for ResolverError {}
