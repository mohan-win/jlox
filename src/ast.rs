use crate::token::Token;

#[derive(Debug, Clone)]
pub enum LitralValue {
    NUMBER(f64),
    STRING(String),
    True,
    False,
    Nil,
}

#[derive(Debug)]
pub enum Expr {
    Litral(LitralValue),
    Variable(Token),
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Assign {
        name: Token,
        value: Box<Expr>,
    },
}

#[derive(Debug)]
pub enum Stmt {
    Var {
        name: Token,
        expression: Option<Expr>,
    },
    PrintStmt {
        expression: Expr,
    },
    ExpressionStmt {
        expression: Expr,
    },
    Block {
        statements: Vec<Stmt>,
    },
    IfStmt {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    WhileStmt {
        condition: Expr,
        body: Box<Stmt>,
    },
}
