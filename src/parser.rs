use crate::{
    ast::Expr,
    ast::VAL,
    error::report,
    token::{Token, TokenType},
};
use std::error::Error;
use std::fmt;

/// ToDo:: refractor using parser combinators
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> ParserBoxdResult<Expr> {
        self.expression()
    }

    fn is_at_end(&self) -> bool {
        match self.peek().token_type {
            TokenType::EOF => true,
            _ => false,
        }
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        assert!(self.current > 0);
        &self.tokens[self.current - 1]
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            &self.peek().token_type == token_type
        }
    }

    fn matches(&mut self, token_types: &[TokenType]) -> bool {
        for token_type in token_types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }

        return false;
    }

    fn consume(&mut self, token_type: &TokenType, message: &str) -> ParserResult<&Token> {
        if self.check(token_type) {
            Ok(self.advance())
        } else {
            Err(ParserError::new(self.peek(), message))
        }
    }

    fn expression(&mut self) -> ParserBoxdResult<Expr> {
        self.equality()
    }

    fn equality(&mut self) -> ParserBoxdResult<Expr> {
        let mut expr = self.comparison()?;

        while self.matches(&[TokenType::BANG_EQUAL, TokenType::EQUAL_EQUAL]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = Box::new(Expr::Binary {
                left: expr,
                operator,
                right,
            })
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> ParserBoxdResult<Expr> {
        use TokenType::*;

        let mut expr = self.term()?;
        while self.matches(&[GREATER, GREATER_EQUAL, LESS, LESS_EQUAL]) {
            let operator = self.previous().clone();
            let right = self.term()?;
            expr = Box::new(Expr::Binary {
                left: expr,
                operator,
                right,
            })
        }
        Ok(expr)
    }

    fn term(&mut self) -> ParserBoxdResult<Expr> {
        let mut expr = self.factor()?;

        while self.matches(&[TokenType::MINUS, TokenType::PLUS]) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = Box::new(Expr::Binary {
                left: expr,
                operator,
                right,
            })
        }

        Ok(expr)
    }

    fn factor(&mut self) -> ParserBoxdResult<Expr> {
        let mut expr = self.unary()?;

        while self.matches(&[TokenType::STAR, TokenType::SLASH]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = Box::new(Expr::Binary {
                left: expr,
                operator,
                right,
            })
        }
        Ok(expr)
    }

    fn unary(&mut self) -> ParserBoxdResult<Expr> {
        if self.matches(&[TokenType::BANG, TokenType::MINUS]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            Ok(Box::new(Expr::Unary {
                operator,
                right: right,
            }))
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> ParserBoxdResult<Expr> {
        use Expr::*;
        use TokenType::*;

        let expr: Option<Expr> = match &self.peek().token_type {
            FALSE => Some(Litral(VAL::False)),
            TRUE => Some(Litral(VAL::True)),
            NIL => Some(Litral(VAL::Nil)),
            NUMBER { litral } => Some(Litral(VAL::NUMBER(litral.clone()))),
            STRING { litral } => Some(Litral(VAL::STRING(litral.clone()))),
            _ => None,
        };
        if let Some(e) = expr {
            self.advance(); // Important: comsume token & advance
            return Ok(Box::new(e));
        } else if let LEFT_PARAN = self.peek().token_type {
            self.advance(); // Important: comsume token & advance
            let expr = self.expression()?;
            self.consume(&TokenType::RIGHT_PARAN, "Expect ) after expression")?;
            Ok(Box::new(Expr::Grouping { expression: expr }))
        } else {
            Err(ParserError::new(
                self.previous(),
                "Unsupported primary token",
            ))
        }
    }

    fn synchronize(&mut self) {
        use TokenType::*;
        self.advance();

        while !self.is_at_end() {
            if let SEMICOLON = self.previous().token_type {
                return;
            }

            match self.peek().token_type {
                CLASS | FUN | VAR | FOR | IF | WHILE | PRINT | RETURN => return,
                _ => {
                    self.advance();
                    continue;
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct ParserError {
    token_type: TokenType,
    line: usize,
    message: String,
}

impl ParserError {
    fn new(token: &Token, message: &str) -> ParserError {
        Self::error(token, message);
        ParserError {
            token_type: token.token_type.clone(),
            line: token.line,
            message: String::from(message),
        }
    }

    fn error(token: &Token, message: &str) {
        if token.token_type == TokenType::EOF {
            report(token.line, " at end", message)
        } else {
            report(
                token.line,
                format!("at '{}'", token.lexeme).as_str(),
                message,
            )
        }
    }
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Parser error: line {} at {:?} message {}",
            self.line, self.token_type, self.message
        )
    }
}

impl Error for ParserError {}

pub type ParserResult<T> = Result<T, ParserError>;
pub type ParserBoxdResult<T> = ParserResult<Box<T>>;
