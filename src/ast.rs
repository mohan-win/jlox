use crate::token::Token;

#[derive(Debug)]
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
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
}
