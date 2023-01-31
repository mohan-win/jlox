use crate::token::Token;

#[derive(Debug, Clone)]
pub enum LitralValue {
    NUMBER(f64),
    STRING(String),
    True,
    False,
    Nil,
}

#[derive(Clone, Debug)]
pub enum Expr {
    Litral(LitralValue),
    Variable {
        name: Token,
        depth: Option<usize>,
    },
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
        depth: Option<usize>,
        value: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        paran: Token,
        arguments: Vec<Expr>,
    },
}

#[derive(Clone, Debug)]
pub struct Fun {
    pub name: Token,
    pub params: Vec<Token>,
    pub body: Vec<Stmt>,
}

#[derive(Clone, Debug)]
pub enum Stmt {
    Class {
        name: Token,
        methods: Vec<Fun>,
    },
    Function(Fun),
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
    Return {
        keyword: Token,
        value: Option<Expr>,
    },
}
