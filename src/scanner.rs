use core::str;
use std::collections::HashMap;

use crate::{
    error::error,
    token::{Token, TokenType},
};
use unicode_segmentation::UnicodeSegmentation;

struct ScanPosition {
    start: usize,
    current: usize,
    line: usize,
}

pub struct Scanner {
    source_graphemes: Vec<String>,
    pos: ScanPosition,
    tokens: Vec<Token>,
    keywords: HashMap<&'static str, TokenType>,
}

impl Scanner {
    pub fn new(source: String) -> Scanner {
        Scanner {
            source_graphemes: source
                .graphemes(true)
                .map(|str| String::from(str))
                .collect(),
            pos: ScanPosition {
                start: 0,
                current: 0,
                line: 1,
            },
            tokens: Vec::new(),
            keywords: KEYWORDS(),
        }
    }

    pub fn scan_tokens(&mut self) -> &Vec<Token> {
        while !self.is_at_end() {
            self.pos.start = self.pos.current;
            self.scan_token()
        }

        self.add_token(TokenType::EOF);
        &self.tokens
    }

    fn is_at_end(&self) -> bool {
        self.pos.current >= self.source_graphemes.len()
    }

    fn scan_token(&mut self) {
        let c = self.advance().to_string();
        match c.as_str() {
            "(" => self.add_token(TokenType::LEFT_PARAN),
            ")" => self.add_token(TokenType::RIGHT_PARAN),
            "{" => self.add_token(TokenType::LEFT_BRACE),
            "}" => self.add_token(TokenType::RIGHT_BRACE),
            "," => self.add_token(TokenType::COMMA),
            "." => self.add_token(TokenType::DOT),
            "-" => self.add_token(TokenType::MINUS),
            "+" => self.add_token(TokenType::PLUS),
            ";" => self.add_token(TokenType::SEMICOLON),
            "*" => self.add_token(TokenType::STAR),
            "!" => {
                let token = if self.advance_if_matched("=") {
                    TokenType::BANG_EQUAL
                } else {
                    TokenType::BANG
                };
                self.add_token(token)
            }
            "=" => {
                let token = if self.advance_if_matched("=") {
                    TokenType::EQUAL_EQUAL
                } else {
                    TokenType::EQUAL
                };
                self.add_token(token)
            }
            "<" => {
                let token = if self.advance_if_matched("=") {
                    TokenType::LESS_EQUAL
                } else {
                    TokenType::LESS
                };
                self.add_token(token)
            }
            ">" => {
                let token = if self.advance_if_matched("=") {
                    TokenType::GREATER_EQUAL
                } else {
                    TokenType::GREATER
                };
                self.add_token(token)
            }
            "/" => {
                self.handle_slash();
            }
            " " | "\r" | "\t" => (), // Ignoring whitespaces.
            "\n" => self.pos.line += 1,
            "\"" => self.string_litral(),

            c => {
                if Self::is_digit(c) {
                    self.number();
                } else if Self::is_alpha(c) {
                    self.identifier();
                } else {
                    error(self.pos.line, &format!("Unexpected charactor {}", c));
                }
            }
        }
    }

    fn advance(&mut self) -> &str {
        let s = &self.source_graphemes[self.pos.current];
        self.pos.current += 1;
        s
    }

    fn advance_if_matched(&mut self, expected: &str) -> bool {
        if self.is_at_end() {
            return false;
        } else if self.source_graphemes[self.pos.current] != expected {
            return false;
        } else {
            self.pos.current += 1;
            return true;
        }
    }

    fn peek(&self) -> &str {
        if self.is_at_end() {
            "\0"
        } else {
            &self.source_graphemes[self.pos.current]
        }
    }

    fn peek_next(&self) -> &str {
        if self.pos.current + 1 >= self.source_graphemes.len() {
            "\0"
        } else {
            &self.source_graphemes[self.pos.current + 1]
        }
    }

    fn add_token(&mut self, token_type: TokenType) {
        let text = self.source_graphemes[self.pos.start..self.pos.current].join("");
        self.tokens
            .push(Token::new(token_type, text, self.pos.line))
    }

    fn handle_slash(&mut self) {
        if self.advance_if_matched("/") {
            // single line comment
            while self.peek() != "\n" && !self.is_at_end() {
                self.advance();
            }
        } else if self.advance_if_matched("*") {
            // C-style multi line comment
            let mut comment_terminated = false;
            while !self.is_at_end() {
                match self.peek() {
                    "\n" => {
                        self.pos.line += 1;
                        self.advance();
                    }
                    "*" => {
                        if self.peek_next() != "/" {
                            self.advance();
                        } else {
                            // Advance twice to consume end of comment indication (*/)
                            self.advance();
                            self.advance();
                            comment_terminated = true;
                            break;
                        }
                    }
                    _ => {
                        self.advance();
                    }
                }
            }
            if !comment_terminated {
                error(self.pos.line, "Multi-line comment did't terminate!.");
            }
        } else {
            self.add_token(TokenType::SLASH)
        }
    }

    fn string_litral(&mut self) {
        while self.peek() != "\"" && !self.is_at_end() {
            if self.peek() == "\n" {
                self.pos.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            error(self.pos.line, "Unterminated string.")
        } else {
            self.advance(); // The closing ".
        }

        let value = self.source_graphemes[self.pos.start + 1..self.pos.current - 1].join("");
        self.add_token(TokenType::STRING { litral: value })
    }

    fn number(&mut self) {
        while Self::is_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == "." && Self::is_digit(self.peek_next()) {
            self.advance(); // consume "."
            while Self::is_digit(self.peek()) {
                self.advance();
            }
        }

        let num_str = self.source_graphemes[self.pos.start..self.pos.current].join("");
        let value: f64 = str::parse(&num_str).expect("This should be a valid number");
        self.add_token(TokenType::NUMBER { litral: value })
    }

    fn identifier(&mut self) {
        while Self::is_alpha_numeric(self.peek()) {
            self.advance();
        }

        let text = self.source_graphemes[self.pos.start..self.pos.current].join("");
        let token = match self.keywords.get(text.as_str()) {
            Some(token_type) => token_type.clone(),
            None => TokenType::IDENTIFIER,
        };
        self.add_token(token);
    }

    fn is_digit(c: &str) -> bool {
        match c {
            "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => true,
            _ => false,
        }
    }

    fn is_alpha(c: &str) -> bool {
        match c {
            "_" => true,
            "a" | "b" | "c" | "d" | "e" | "f" | "g" | "h" | "i" | "j" | "k" | "l" | "m" | "n"
            | "o" | "p" | "q" | "r" | "s" | "t" | "u" | "v" | "w" | "x" | "y" | "z" | "A" | "B"
            | "C" | "D" | "E" | "F" | "G" | "H" | "I" | "J" | "K" | "L" | "M" | "N" | "O" | "P"
            | "Q" | "R" | "S" | "T" | "U" | "V" | "W" | "X" | "Y" | "Z" => true,
            _ => false,
        }
    }

    fn is_alpha_numeric(c: &str) -> bool {
        Self::is_alpha(c) || Self::is_digit(c)
    }
}

#[allow(non_snake_case)]
fn KEYWORDS() -> HashMap<&'static str, TokenType> {
    let keywords: HashMap<&'static str, TokenType> = [
        ("and", TokenType::AND),
        ("class", TokenType::CLASS),
        ("else", TokenType::ELSE),
        ("extension", TokenType::EXTENSION),
        ("false", TokenType::FALSE),
        ("for", TokenType::FOR),
        ("fun", TokenType::FUN),
        ("if", TokenType::IF),
        ("nil", TokenType::NIL),
        ("or", TokenType::OR),
        ("print", TokenType::PRINT),
        ("return", TokenType::RETURN),
        ("super", TokenType::SUPER),
        ("this", TokenType::THIS),
        ("true", TokenType::TRUE),
        ("var", TokenType::VAR),
        ("while", TokenType::WHILE),
    ]
    .into_iter()
    .collect();
    keywords
}
