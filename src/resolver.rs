use crate::ast::VarPosition;
use crate::{ast::Fun, token::Token};

use super::ast::{Expr, Stmt};
use std::collections::HashMap;
use std::error;
use std::fmt;

#[derive(Clone)]
pub struct VarResolutionInfo {
    local_index: usize,
    is_initialized: bool,
    num_of_usage: usize,
    line: usize,
}

impl VarResolutionInfo {
    fn new(
        local_index: usize,
        is_initialized: bool,
        num_of_usage: usize,
        line: usize,
    ) -> VarResolutionInfo {
        VarResolutionInfo {
            local_index,
            is_initialized,
            num_of_usage,
            line,
        }
    }
}

pub struct Resolver {
    scopes: Vec<HashMap<String, VarResolutionInfo>>,
    current_function: Option<FunctionType>,
    current_var_index: usize,
    num_of_resolver_errs: usize,
}

impl Resolver {
    pub fn new() -> Resolver {
        Resolver {
            scopes: Vec::new(),
            current_function: None,
            current_var_index: 0,
            num_of_resolver_errs: 0,
        }
    }

    pub fn get_num_of_resolver_errs(&self) -> usize {
        self.num_of_resolver_errs
    }

    pub fn resolve(&mut self, stmts: &mut Vec<Stmt>) {
        self.begin_scope(); // Native functions scope
        self.add_native_functions_info();

        self.begin_scope(); // Global scope!
        self.resolve_stmts(stmts);
        self.end_scope(); // End Global scope

        self.end_scope(); // End Native functions scope
    }

    /// Add native functions resolution info
    fn add_native_functions_info(&mut self) {
        self.scopes.last_mut().unwrap().insert(
            String::from("clock"),
            VarResolutionInfo {
                local_index: 0,
                is_initialized: true,
                num_of_usage: 1, // Note: set the usage to > 0 so that if a program doesn't use native functions it is not warned
                line: 0,
            },
        );
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
            Expr::Variable { name, pos } => {
                if !self.scopes.is_empty() {
                    if let Some(var_info) = self.scopes.last().unwrap().get(&name.lexeme) {
                        if !var_info.is_initialized {
                            self.error(&ResolverError::new(
                                name,
                                "Can't read local variable in its own initializer",
                            ))
                        }
                    }
                }
                self.increment_usage(name);
                *pos = self.resolve_local_index_depth(name);
            }
            Expr::Assign { name, value, pos } => {
                self.resolve_expr(value);
                self.increment_usage(name);
                *pos = self.resolve_local_index_depth(name);
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

    /// resolve local index and depth for the given token
    fn resolve_local_index_depth(&self, name: &Token) -> Option<VarPosition> {
        let result =
            self.scopes
                .iter()
                .rev()
                .enumerate()
                .try_for_each(|(depth, scope)| match scope.get(&name.lexeme) {
                    Some(var_info) => Err(VarPosition {
                        depth,
                        index: var_info.local_index,
                    }),
                    None => Ok(()),
                });
        if let Err(var_info) = result {
            Some(var_info)
        } else {
            None
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
        self.current_var_index = 0; // reset the local variable index on a new scope
    }

    fn end_scope(&mut self) {
        if let Some(scope_popped) = self.scopes.pop() {
            Resolver::warn_unused_variables(&scope_popped)
        }
        if let Some(scope) = self.scopes.last() {
            self.current_var_index = scope.len();
        }
    }

    fn warn_unused_variables(scope: &HashMap<String, VarResolutionInfo>) {
        for (var_name, var_info) in scope {
            if var_info.num_of_usage == 0 {
                Resolver::warn(&ResolverError::new_with_message(
                    format!(
                        "[line: {}] Variable {} is not used",
                        var_info.line, var_name
                    )
                    .as_str(),
                ))
            }
        }
    }

    fn declare(&mut self, name: &Token) {
        assert!(
            self.scopes.is_empty() == false,
            "There should be a scope available"
        );
        let scope = self.scopes.last_mut().unwrap();

        if !scope.contains_key(&name.lexeme) {
            scope.insert(
                name.lexeme.clone(),
                VarResolutionInfo::new(self.current_var_index, false, 0, name.line),
            );
            self.current_var_index += 1;
        } else {
            self.error(&ResolverError::new(
                name,
                "Already a variable with this name in this scope",
            ))
        }
    }

    fn define(&mut self, name: &Token) {
        assert!(
            self.scopes.is_empty() == false,
            "There should be a scope available"
        );

        let var_info = self
            .scopes
            .last_mut()
            .unwrap()
            .get_mut(name.lexeme.as_str())
            .expect("Var should be declared before its definition");
        var_info.is_initialized = true;
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
                let var_info: &mut VarResolutionInfo = scope
                    .get_mut(name.lexeme.as_str())
                    .expect("variable should be present");
                var_info.num_of_usage += 1;
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
