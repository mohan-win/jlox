use crate::{ast::Fun, error::error_in_token, interpreter::Interpreter, token::Token};

use super::ast::{Expr, Stmt};
use std::collections::HashMap;

pub struct Resolver<'a> {
    scopes: Vec<HashMap<String, bool>>,
    interpreter: &'a mut Interpreter,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Resolver<'a> {
        Resolver {
            scopes: Vec::new(),
            interpreter,
        }
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) {
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
                self.declare(&fun.name);
                self.define(&fun.name);
                self.resolve_function(fun)
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
                else_branch.as_ref().and_then(|else_branch| {
                    self.resolve_stmt(else_branch);
                    Some(())
                });
            }
            Stmt::WhileStmt { condition, body } => {
                self.resolve_expr(condition);
                self.resolve_stmt(&body);
            }
            Stmt::Return { keyword: _, value } => {
                value.as_ref().and_then(|value| {
                    self.resolve_expr(value);
                    Some(())
                });
            }
        }
    }

    fn resolve_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Variable(name) => {
                if !self.scopes.is_empty() {
                    if let Some(false) = self.scopes.last().unwrap().get(&name.lexeme) {
                        error_in_token(name, "Can't read local variable in its own initializer")
                    }
                }
                self.resolve_local(expr, name)
            }
            expr @ Expr::Assign { name, value } => {
                self.resolve_expr(value);
                self.resolve_local(expr, name)
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
                    .iter()
                    .for_each(|argument| self.resolve_expr(argument));
            }
        }
    }

    fn resolve_function(&mut self, fun: &Fun) {
        self.begin_scope();
        fun.params.iter().for_each(|param| {
            self.declare(param);
            self.define(param);
        });
        self.resolve_stmts(&fun.body);
        self.end_scope();
    }

    fn resolve_local(&mut self, expr: &Expr, name: &Token) {
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
            self.interpreter.resolve(expr, depth)
        }
    }

    fn resolve_stmts(&mut self, stmts: &Vec<Stmt>) {
        stmts.iter().for_each(|stmt| self.resolve_stmt(stmt))
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::<String, bool>::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }

        self.scopes
            .last_mut()
            .unwrap()
            .insert(name.lexeme.clone(), false);
    }

    fn define(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }

        self.scopes
            .last_mut()
            .unwrap()
            .insert(name.lexeme.clone(), true);
    }
}
