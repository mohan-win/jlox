use std::rc::Rc;

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
    source: String,
    source_graphemes: Vec<String>, // ToDo:: replace it with Vec<&str> to save memory ??
    pos: ScanPosition,
    tokens: Vec<Token>,
}

impl<'a> Scanner {
    pub fn new(source: String) -> Scanner {
        let source = source;
        let source_graphemes: Vec<String> = source
            .clone()
            .graphemes(true)
            .map(|str| String::from(str))
            .collect();
        Scanner {
            source,
            source_graphemes,
            pos: ScanPosition {
                start: 0,
                current: 0,
                line: 1,
            },
            tokens: Vec::new(),
        }
    }

    pub fn scan_tokens(&mut self) {
        while !self.is_at_end() {
            self.pos.start = self.pos.current;
            self.scan_token()
        }
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
                if self.advance_if_matched("/") {
                    while self.peek() != "\n" && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::SLASH)
                }
            }
            " " | "\r" | "\t" => {
                println!("Ignoring whitespaces")
            }
            "\n" => self.pos.line += 1,
            "\"" => self.string_litral(),

            c => error(self.pos.line, &format!("Unexpected charactor {}", c)),
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
        }
        if &self.source_graphemes[self.pos.current] != expected {
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

    fn add_token(&mut self, token_type: TokenType) {
        let text = &self.source_graphemes[self.pos.start..self.pos.current].join("");
        self.tokens
            .push(Token::new(token_type, text.clone(), self.pos.line))
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

        let value = self.source_graphemes[self.pos.start + 1..self.pos.current].join("");
        self.add_token(TokenType::STRING { litral: value })
    }
}
