use crate::{
    ast::Expr,
    ast::{Fun, LitralValue, Stmt},
    token::{Token, TokenType},
};
use std::error::Error;
use std::fmt;

/// ToDo:: refractor using parser combinators
pub struct Parser<'a> {
    tokens: &'a Vec<Token>,
    current: usize,
    num_of_parser_errs: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &Vec<Token>) -> Parser {
        Parser {
            tokens,
            current: 0,
            num_of_parser_errs: 0,
        }
    }

    pub fn get_num_of_parser_errors(&self) -> usize {
        self.num_of_parser_errs
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut statements: Vec<Stmt> = Vec::new();
        while !self.is_at_end() {
            let stmt = self.declaration();
            if let Some(stmt) = stmt {
                statements.push(stmt);
            }
        }
        statements
    }

    fn error(&mut self, err: ParserError) {
        self.num_of_parser_errs += 1;
        crate::error::error_at_compiler(&err);
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

    fn declaration(&mut self) -> Option<Stmt> {
        let stmt = if self.matches(&[TokenType::CLASS]) {
            self.class_declaration()
        } else if self.matches(&[TokenType::FUN]) {
            self.function("function")
        } else if self.matches(&[TokenType::VAR]) {
            self.var_declaration()
        } else {
            self.statement()
        };

        if let Err(parse_err) = stmt {
            self.error(parse_err);
            self.synchronize();
            None
        } else {
            stmt.ok()
        }
    }

    fn class_declaration(&mut self) -> ParserResult<Stmt> {
        let name = self
            .consume(&TokenType::IDENTIFIER, "Expect class name")?
            .clone();
        let mut super_class = None;
        if self.matches(&[TokenType::LESS]) {
            self.consume(&TokenType::IDENTIFIER, "Expect super class name after '<'")?;
            super_class = Some(Expr::Variable {
                name: self.previous().clone(),
                depth: None,
            })
        }

        self.consume(&TokenType::LEFT_BRACE, "Expect '{' after the class name")?;
        let mut methods = Vec::new();
        while !self.check(&TokenType::RIGHT_BRACE) && !self.is_at_end() {
            if let Stmt::Function(fun) = self.function("method")? {
                methods.push(fun)
            }
        }
        self.consume(&TokenType::RIGHT_BRACE, "End class definition with '}'")?;

        Ok(Stmt::Class {
            name,
            super_class,
            methods,
        })
    }

    fn function(&mut self, kind: &str) -> ParserResult<Stmt> {
        let name = self
            .consume(
                &TokenType::IDENTIFIER,
                format!("Expect {} name", kind).as_str(),
            )?
            .clone();
        self.consume(
            &TokenType::LEFT_PARAN,
            format!("Expect '(' after {} name", kind).as_str(),
        )?;
        let mut params = Vec::new();
        if !self.check(&TokenType::RIGHT_PARAN) {
            loop {
                if params.len() >= 255 {
                    self.error(ParserError::new(
                        self.peek(),
                        format!("Can't allow more than 255 params for a {}", kind).as_str(),
                    ))
                }
                let param = self.consume(
                    &TokenType::IDENTIFIER,
                    format!("Expect {} parameter", kind).as_str(),
                )?;
                params.push(param.clone());
                if !self.matches(&[TokenType::COMMA]) {
                    break;
                }
            }
        }
        self.consume(
            &TokenType::RIGHT_PARAN,
            format!("Expect ')' after {} parameters", kind).as_str(),
        )?;
        self.consume(
            &TokenType::LEFT_BRACE,
            format!("Expect '{{' before start of a {} body", kind).as_str(),
        )?;
        let body = self.block()?;
        Ok(Stmt::Function(Fun { name, params, body }))
    }

    fn var_declaration(&mut self) -> ParserResult<Stmt> {
        self.consume(&TokenType::IDENTIFIER, "Expect a variable name")?;
        let name = self.previous().clone();

        let mut expression: Option<Expr> = None;
        if self.matches(&[TokenType::EQUAL]) {
            expression = Some(*self.expression()?);
        }
        self.consume(
            &TokenType::SEMICOLON,
            "Expect ';' after variable declaration",
        )?;
        Ok(Stmt::Var { name, expression })
    }

    fn statement(&mut self) -> ParserResult<Stmt> {
        if self.matches(&[TokenType::PRINT]) {
            self.print_statement()
        } else if self.matches(&[TokenType::IF]) {
            self.if_statement()
        } else if self.matches(&[TokenType::WHILE]) {
            self.while_statement()
        } else if self.matches(&[TokenType::FOR]) {
            self.for_statement()
        } else if self.matches(&[TokenType::RETURN]) {
            self.return_statement()
        } else if self.matches(&[TokenType::LEFT_BRACE]) {
            let statements = self.block()?;
            Ok(Stmt::Block { statements })
        } else {
            self.expression_statement()
        }
    }

    fn if_statement(&mut self) -> ParserResult<Stmt> {
        self.consume(&TokenType::LEFT_PARAN, "Expect ( after if")?;
        let condition = self.expression()?;
        self.consume(&TokenType::RIGHT_PARAN, "Expect ) after if condition")?;
        let then_branch = self.statement()?;

        let mut else_branch = None;
        if self.matches(&[TokenType::ELSE]) {
            else_branch = Some(Box::new(self.statement()?));
        }
        Ok(Stmt::IfStmt {
            condition: *condition,
            then_branch: Box::new(then_branch),
            else_branch,
        })
    }

    fn while_statement(&mut self) -> ParserResult<Stmt> {
        self.consume(&TokenType::LEFT_PARAN, "Expect '(' after while")?;
        let condition = self.expression()?;
        self.consume(&TokenType::RIGHT_PARAN, "Expect ')' after condition")?;
        let body = self.statement()?;
        Ok(Stmt::WhileStmt {
            condition: *condition,
            body: Box::new(body),
        })
    }

    fn for_statement(&mut self) -> ParserResult<Stmt> {
        self.consume(&TokenType::LEFT_PARAN, "Expect '(' after for")?;
        let initializer;
        if self.matches(&[TokenType::SEMICOLON]) {
            initializer = None;
        } else if self.matches(&[TokenType::VAR]) {
            initializer = Some(self.var_declaration()?);
        } else {
            initializer = Some(self.expression_statement()?);
        }

        let mut condition = None;
        if !self.check(&TokenType::SEMICOLON) {
            condition = Some(*self.expression()?);
        }
        self.consume(&TokenType::SEMICOLON, "Expect ';' after for condition")?;
        let mut increment = None;
        if !self.check(&TokenType::RIGHT_PARAN) {
            increment = Some(*self.expression()?);
        }
        self.consume(&TokenType::RIGHT_PARAN, "Expect  matching ')' in for loop")?;
        let mut body = self.statement()?;

        if let Some(increment) = increment {
            body = Stmt::Block {
                statements: vec![
                    body,
                    Stmt::ExpressionStmt {
                        expression: increment,
                    },
                ],
            }
        }
        if let None = condition {
            condition = Some(Expr::Litral(LitralValue::True));
        }
        body = Stmt::WhileStmt {
            condition: condition.unwrap(),
            body: Box::new(body),
        };

        if let Some(initializer) = initializer {
            body = Stmt::Block {
                statements: vec![initializer, body],
            }
        }

        Ok(body)
    }

    fn return_statement(&mut self) -> ParserResult<Stmt> {
        let keyword = self.previous().clone();
        let mut value = None;
        if !self.check(&TokenType::SEMICOLON) {
            value = Some(*self.expression()?);
        }
        self.consume(&TokenType::SEMICOLON, "Expect ';' after return value")?;

        Ok(Stmt::Return { keyword, value })
    }

    fn block(&mut self) -> ParserResult<Vec<Stmt>> {
        let mut statements: Vec<Stmt> = Vec::new();
        while !self.check(&TokenType::RIGHT_BRACE) && !self.is_at_end() {
            if let Some(stmt) = self.declaration() {
                statements.push(stmt);
            }
        }
        self.consume(&TokenType::RIGHT_BRACE, "Expect '}' after block")?;
        Ok(statements)
    }

    fn print_statement(&mut self) -> ParserResult<Stmt> {
        let expr = self.expression()?;
        self.consume(&TokenType::SEMICOLON, "Expect ; after expression")?;
        Ok(Stmt::PrintStmt { expression: *expr })
    }

    fn expression_statement(&mut self) -> ParserResult<Stmt> {
        let expr = self.expression()?;
        self.consume(&TokenType::SEMICOLON, "Expect ; after expression")?;
        Ok(Stmt::ExpressionStmt { expression: *expr })
    }

    fn expression(&mut self) -> ParserBoxdResult<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> ParserBoxdResult<Expr> {
        let expr = self.or()?;

        if self.matches(&[TokenType::EQUAL]) {
            let equals = self.previous().clone();
            let value = self.assignment()?;

            if let Expr::Variable { name: token, .. } = *expr {
                return Ok(Box::new(Expr::Assign {
                    name: token,
                    depth: None,
                    value,
                }));
            } else if let Expr::Get { object, name } = *expr {
                return Ok(Box::new(Expr::Set {
                    object,
                    name,
                    value,
                }));
            } else {
                self.error(ParserError::new(&equals, "Invalid assignment target"));
            }
        }

        return Ok(expr);
    }

    fn or(&mut self) -> ParserBoxdResult<Expr> {
        let left = self.and()?;

        if self.matches(&[TokenType::OR]) {
            let operator = self.previous().clone();
            let right = self.and()?;
            Ok(Box::new(Expr::Logical {
                left,
                operator,
                right,
            }))
        } else {
            Ok(left)
        }
    }

    fn and(&mut self) -> ParserBoxdResult<Expr> {
        let left = self.equality()?;

        if self.matches(&[TokenType::AND]) {
            let operator = self.previous().clone();
            let right = self.equality()?;
            Ok(Box::new(Expr::Logical {
                left,
                operator,
                right,
            }))
        } else {
            Ok(left)
        }
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
            Ok(Box::new(Expr::Unary { operator, right }))
        } else {
            self.call()
        }
    }

    fn call(&mut self) -> ParserBoxdResult<Expr> {
        let mut expr = self.primary()?;

        loop {
            if self.matches(&[TokenType::LEFT_PARAN]) {
                expr = self.finish_call(expr)?;
            } else if self.matches(&[TokenType::DOT]) {
                let name = self
                    .consume(&TokenType::IDENTIFIER, "Expect property name after '.'")?
                    .clone();
                expr = Box::new(Expr::Get { object: expr, name })
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Box<Expr>) -> ParserBoxdResult<Expr> {
        let mut arguments = Vec::new();

        if !self.check(&TokenType::RIGHT_PARAN) {
            loop {
                if arguments.len() >= 255 {
                    self.error(ParserError::new(
                        self.peek(),
                        "Function can't have more than 255 arguments",
                    ));
                }
                arguments.push(*self.expression()?);
                if !self.matches(&[TokenType::COMMA]) {
                    break;
                }
            }
        }
        self.consume(
            &TokenType::RIGHT_PARAN,
            "Expect ')' at the end of function call",
        )?;

        Ok(Box::new(Expr::Call {
            callee,
            paran: self.previous().clone(),
            arguments,
        }))
    }

    fn primary(&mut self) -> ParserBoxdResult<Expr> {
        use Expr::*;
        use TokenType::*;

        let expr: Option<Expr> = match &self.peek().token_type {
            FALSE => Some(Litral(LitralValue::False)),
            TRUE => Some(Litral(LitralValue::True)),
            NIL => Some(Litral(LitralValue::Nil)),
            NUMBER { litral } => Some(Litral(LitralValue::NUMBER(litral.clone()))),
            STRING { litral } => Some(Litral(LitralValue::STRING(litral.clone()))),
            IDENTIFIER => Some(Expr::Variable {
                name: self.peek().clone(),
                depth: None,
            }),
            THIS => Some(Expr::This {
                keyword: self.peek().clone(),
                depth: None,
            }),
            _ => None,
        };
        if let Some(e) = expr {
            self.advance(); // Important: comsume token & advance
            Ok(Box::new(e))
        } else if self.matches(&[TokenType::SUPER]) {
            let keyword = self.previous().clone();
            self.consume(&TokenType::DOT, "Expect '.' after 'super' keyword")?;
            self.consume(
                &TokenType::IDENTIFIER,
                "Expect super class method name after '.'",
            )?;
            Ok(Box::new(Expr::Super {
                keyword,
                depth: None,
                method: self.previous().clone(),
            }))
        } else if let LEFT_PARAN = self.peek().token_type {
            self.advance(); // Important: comsume token & advance
            let expr = self.expression()?;
            self.consume(&TokenType::RIGHT_PARAN, "Expect ) after expression")?;
            Ok(Box::new(Expr::Grouping { expression: expr }))
        } else {
            self.advance();
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
    pub token_type: TokenType,
    pub line: usize,
    pub message: String,
}

impl ParserError {
    fn new(token: &Token, message: &str) -> ParserError {
        ParserError {
            token_type: token.token_type.clone(),
            line: token.line,
            message: String::from(message),
        }
    }
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Parser error: line {} at [{:?}] {}",
            self.line, self.token_type, self.message
        )
    }
}

impl Error for ParserError {}

pub type ParserResult<T> = Result<T, ParserError>;
pub type ParserBoxdResult<T> = ParserResult<Box<T>>;
