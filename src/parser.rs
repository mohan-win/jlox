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
    fn new(tokens: Vec<Token>) -> Parser {
        Parser { tokens, current: 0 }
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

    fn expression(&mut self) -> ParserResult<Expr> {
        self.equality()
    }

    fn equality(&mut self) -> ParserResult<Expr> {
        let mut expr: Expr = self.comparison()?;

        while self.matches(&[TokenType::BANG_EQUAL, TokenType::EQUAL_EQUAL]) {
            let operator = self.previous().clone();
            let right: Expr = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> ParserResult<Expr> {
        use TokenType::*;

        let mut expr: Expr = self.term()?;
        while self.matches(&[GREATER, GREATER_EQUAL, LESS, LESS_EQUAL]) {
            let operator = self.previous().clone();
            let right: Expr = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    fn term(&mut self) -> ParserResult<Expr> {
        let mut expr: Expr = self.factor()?;

        while self.matches(&[TokenType::MINUS, TokenType::PLUS]) {
            let operator = self.previous().clone();
            let right: Expr = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        Ok(expr)
    }

    fn factor(&mut self) -> ParserResult<Expr> {
        let mut left: Expr = self.unary()?;

        while self.matches(&[TokenType::STAR, TokenType::SLASH]) {
            let operator = self.previous().clone();
            let right: Expr = self.unary()?;
            left = Expr::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            }
        }
        Ok(left)
    }

    fn unary(&mut self) -> ParserResult<Expr> {
        if self.matches(&[TokenType::BANG, TokenType::MINUS]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            })
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> ParserResult<Expr> {
        if self.matches(&[TokenType::FALSE]) {
            Ok(Expr::Litral(VAL::False))
        } else if self.matches(&[TokenType::TRUE]) {
            Ok(Expr::Litral(VAL::True))
        } else if self.matches(&[TokenType::NIL]) {
            Ok(Expr::Litral(VAL::Nil))
        } else if self.matches(&[TokenType::NUMBER { litral: 0.0 }]) {
            // Note the litral value (0.0) is ignored in match(..) only discreminant is considered
            if let TokenType::NUMBER { litral } = self.previous().token_type {
                Ok(Expr::Litral(VAL::NUMBER(litral)))
            } else {
                Err(ParserError::new(
                    self.previous(),
                    "token_type is expected to be NUMBER",
                ))
            }
        } else if self.matches(&[TokenType::STRING {
            litral: String::from(""),
        }]) {
            if let TokenType::STRING { litral } = &self.previous().token_type {
                return Ok(Expr::Litral(VAL::STRING(litral.clone())));
            } else {
                Err(ParserError::new(
                    self.previous(),
                    "token_type is expected to be STRING",
                ))
            }
        } else if self.matches(&[TokenType::LEFT_PARAN]) {
            let expr = self.expression()?;
            self.consume(&TokenType::RIGHT_PARAN, "Expect ) after expression");
            Ok(Expr::Grouping {
                expression: Box::new(expr),
            })
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
