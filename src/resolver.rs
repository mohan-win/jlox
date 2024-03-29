use crate::{ast::Fun, token::Token};

use super::ast::{Expr, Stmt};
use std::collections::HashMap;
use std::error;
use std::fmt;

pub struct Resolver {
    scopes: Vec<HashMap<String, bool>>,
    current_function: Option<FunctionType>,
    current_class: Option<ClassType>,
    num_of_resolver_errs: usize,
}

impl Resolver {
    pub fn new() -> Resolver {
        Resolver {
            scopes: Vec::new(),
            current_function: None,
            current_class: None,
            num_of_resolver_errs: 0,
        }
    }

    pub fn get_num_of_resolver_errs(&self) -> usize {
        self.num_of_resolver_errs
    }

    pub fn resolve_stmts(&mut self, stmts: &mut Vec<Stmt>) {
        stmts.iter_mut().for_each(|stmt| self.resolve_stmt(stmt));
    }

    fn resolve_stmt(&mut self, stmt: &mut Stmt) {
        match stmt {
            Stmt::Class {
                name,
                super_class,
                methods,
            } => {
                let enclosing_class = self.current_class.take();
                self.current_class = Some(ClassType::Class);

                self.declare(name);
                self.define(name);

                super_class.as_ref().map(|super_class| {
                    if let Expr::Variable {
                        name: super_class_name,
                        ..
                    } = super_class
                    {
                        if super_class_name.lexeme.eq(&name.lexeme) {
                            self.error(&ResolverError::new(
                                super_class_name,
                                "A class can't interit from itself",
                            ))
                        }
                    }
                });

                if let Some(super_class) = super_class {
                    self.current_class = Some(ClassType::SubClass);
                    self.resolve_expr(super_class);

                    self.begin_scope(); // 'super' scope
                    self.scopes
                        .last_mut()
                        .unwrap()
                        .insert(String::from("super"), true);
                }

                self.begin_scope(); // 'this' scope
                self.scopes
                    .last_mut()
                    .unwrap()
                    .insert(String::from("this"), true);

                methods.iter_mut().for_each(|method| {
                    let mut declaration = FunctionType::Method;
                    if method.name.lexeme == "init" {
                        declaration = FunctionType::Initializer;
                    }
                    self.resolve_function(method, declaration);
                });

                self.end_scope(); // end of 'this' scope

                if let Some(_super_class) = super_class {
                    self.end_scope(); // end of 'super' scope
                }

                self.current_class = enclosing_class;
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
                self.resolve_function(fun, FunctionType::Function);
            }
            Stmt::Block { statements } => {
                self.begin_scope();
                self.resolve_stmts(statements);
                self.end_scope();
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
                if let Some(fun_type) = self.current_function {
                    value.as_mut().and_then(|value| {
                        if fun_type == FunctionType::Initializer {
                            self.error(&ResolverError::new(
                                keyword,
                                "Can't return a value from constructor",
                            ))
                        } else {
                            self.resolve_expr(value);
                        }

                        Some(())
                    });
                } else {
                    self.error(&ResolverError::new(
                        &keyword,
                        "Return statement allowed only inside a function or method",
                    ))
                }
            }
        }
    }

    fn resolve_expr(&mut self, expr: &mut Expr) {
        match expr {
            Expr::Variable { name, depth } => {
                if !self.scopes.is_empty() {
                    if let Some(false) = self.scopes.last().unwrap().get(&name.lexeme) {
                        self.error(&ResolverError::new(
                            name,
                            "Can't read local variable in its own initializer",
                        ))
                    }
                }
                *depth = self.resolve_local_depth(name)
            }
            Expr::Super {
                keyword,
                method: _,
                depth,
            } => match self.current_class {
                None => self.error(&ResolverError::new(
                    &keyword,
                    "Can't use 'super' outside a class",
                )),
                Some(ClassType::Class) => self.error(&ResolverError::new(
                    &keyword,
                    "Can't use 'super' keyword on a class without a super class",
                )),
                Some(ClassType::SubClass) => *depth = self.resolve_local_depth(&keyword),
            },
            Expr::This { keyword, depth } => {
                if let Some(_) = self.current_class {
                    *depth = self.resolve_local_depth(keyword);
                } else {
                    self.error(&ResolverError::new(
                        keyword,
                        "Can't use 'this' outside of a class",
                    ));
                }
            }
            Expr::Assign { name, value, depth } => {
                self.resolve_expr(value);
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
            Expr::Get { object, name: _ } => self.resolve_expr(object),
            Expr::Set {
                object,
                name: _,
                value,
            } => {
                self.resolve_expr(object);
                self.resolve_expr(value);
            }
        }
    }

    fn resolve_function(&mut self, fun: &mut Fun, fun_type: FunctionType) {
        let enclosing_function = self.current_function.take();
        self.current_function = Some(fun_type);

        self.begin_scope();
        fun.params.iter().for_each(|param| {
            self.declare(param);
            self.define(param);
        });
        self.resolve_stmts(fun.body.as_mut());
        self.end_scope();

        self.current_function = enclosing_function;
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
        self.scopes.push(HashMap::<String, bool>::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }
        let scope = self.scopes.last_mut().unwrap();

        if !scope.contains_key(&name.lexeme) {
            scope.insert(name.lexeme.clone(), false);
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
            .insert(name.lexeme.clone(), true);
    }

    fn error(&mut self, err: &ResolverError) {
        self.num_of_resolver_errs += 1;
        crate::error::error_at_compiler(err)
    }
}

#[derive(Copy, PartialEq, Clone)]
enum FunctionType {
    Function,
    Initializer,
    Method,
}

enum ClassType {
    Class,
    SubClass,
}

#[derive(Debug)]
struct ResolverError {
    token: Token,
    message: String,
}

impl ResolverError {
    fn new(token: &Token, message: &str) -> ResolverError {
        ResolverError {
            token: token.clone(),
            message: String::from(message),
        }
    }
}

impl<'a> fmt::Display for ResolverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Resolver error: line {} at [{:?}] {}",
            self.token.line, self.token.token_type, self.message
        )
    }
}

impl<'a> error::Error for ResolverError {}
