use std::fmt::Display;
#[derive(Debug, Clone)]
#[allow(non_camel_case_types)]
pub enum TokenType {
    // Single charactor token
    LEFT_PARAN,
    RIGHT_PARAN,
    LEFT_BRACE,
    RIGHT_BRACE,
    COMMA,
    DOT,
    MINUS,
    PLUS,
    SEMICOLON,
    SLASH,
    STAR,

    // On or two charactor token
    BANG,
    BANG_EQUAL,
    EQUAL,
    EQUAL_EQUAL,
    GREATER,
    GREATER_EQUAL,
    LESS,
    LESS_EQUAL,

    // Literals
    IDENTIFIER,
    STRING { litral: String },
    NUMBER { litral: f64 },

    // Keywords
    AND,
    CLASS,
    ELSE,
    FALSE,
    FUN,
    FOR,
    IF,
    NIL,
    OR,
    PRINT,
    RETURN,
    SUPER,
    THIS,
    TRUE,
    VAR,
    WHILE,

    EOF,
}

pub struct Token {
    token_type: TokenType,
    lexeme: String,
    line: usize,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, line: usize) -> Token {
        Token {
            token_type,
            lexeme,
            line,
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.token_type {
            TokenType::STRING { litral } => write!(f, "{:?} {}", self.token_type, litral),
            TokenType::NUMBER { litral } => write!(f, "{:?} {}", self.token_type, litral),
            _ => write!(f, "{:?}", self.token_type),
        }
    }
}
