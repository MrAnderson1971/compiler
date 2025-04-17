use crate::common::Position;
use crate::errors::CompilerError;
use crate::lexer::Symbol::{Ambiguous, Binary, Unary};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinaryOperator {
    Multiply,
    Modulo,
    Divide,
    BitwiseXor,
    BitwiseAnd,
    BitwiseOr,
    BitwiseShiftLeft,
    BitwiseShiftRight,
    LogicalAnd,
    LogicalOr,
    Equals,
    NotEquals,
    LessThanOrEquals,
    GreaterThanOrEquals,
    LessThan,
    GreaterThan,
    Ternary, // ternary
    Assign,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnaryOperator {
    Increment,
    Decrement,
    LogicalNot,
    BitwiseNot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnaryOrBinaryOp {
    Addition,
    Subtraction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Symbol {
    Binary(BinaryOperator),
    Unary(UnaryOperator),
    Ambiguous(UnaryOrBinaryOp),
    OpenParenthesis,
    CloseParenthesis,
    OpenBrace,
    CloseBrace,
    Colon,
    Semicolon,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Keyword {
    Return,
    Int,
    If,
    Else,
    Do,
    While,
    For,
    Continue,
    Break,
}

pub type Number = u64;

#[derive(Debug, Clone, PartialEq)] // String prevents Copy. PartialEq is useful for tests.
pub enum Token {
    Keyword(Keyword),
    Symbol(Symbol),
    Identifier(String),
    NumberLiteral(Number),
    Invalid,
    EOF,
}

fn match_keyword(string: &str) -> Option<Keyword> {
    match string {
        "return" => Some(Keyword::Return),
        "int" => Some(Keyword::Int),
        "if" => Some(Keyword::If),
        "else" => Some(Keyword::Else),
        "do" => Some(Keyword::Do),
        "while" => Some(Keyword::While),
        "for" => Some(Keyword::For),
        "continue" => Some(Keyword::Continue),
        "break" => Some(Keyword::Break),
        _ => None,
    }
}

pub fn lex(source: String) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut chars = source.chars().peekable();
    while let Some(c) = chars.next() {
        let next: Token = match c {
            '{' => Token::Symbol(Symbol::OpenBrace),
            '}' => Token::Symbol(Symbol::CloseBrace),
            '(' => Token::Symbol(Symbol::OpenParenthesis),
            ')' => Token::Symbol(Symbol::CloseParenthesis),
            ';' => Token::Symbol(Symbol::Semicolon),
            ':' => Token::Symbol(Symbol::Colon),
            '?' => Token::Symbol(Binary(BinaryOperator::Ternary)),
            '~' => Token::Symbol(Unary(UnaryOperator::BitwiseNot)),
            '^' => Token::Symbol(Binary(BinaryOperator::BitwiseXor)),

            '-' => {
                if chars.peek() == Some(&'-') {
                    chars.next();
                    Token::Symbol(Unary(UnaryOperator::Decrement))
                } else {
                    Token::Symbol(Ambiguous(UnaryOrBinaryOp::Subtraction))
                }
            }
            '+' => {
                if chars.peek() == Some(&'+') {
                    chars.next();
                    Token::Symbol(Unary(UnaryOperator::Increment))
                } else {
                    Token::Symbol(Ambiguous(UnaryOrBinaryOp::Addition))
                }
            }
            '*' => Token::Symbol(Binary(BinaryOperator::Multiply)),
            '/' => {
                if chars.peek() == Some(&'/') {
                    // single line comment
                    while let Some(next) = chars.next() {
                        if next == '\n' {
                            break;
                        }
                    }
                    continue;
                } else {
                    Token::Symbol(Binary(BinaryOperator::Divide))
                }
            }
            '%' => Token::Symbol(Binary(BinaryOperator::Modulo)),

            '=' => {
                if chars.peek() == Some(&'=') {
                    chars.next();
                    Token::Symbol(Binary(BinaryOperator::Equals))
                } else {
                    Token::Symbol(Binary(BinaryOperator::Assign))
                }
            }
            '!' => {
                if chars.peek() == Some(&'=') {
                    chars.next();
                    Token::Symbol(Binary(BinaryOperator::NotEquals))
                } else {
                    Token::Symbol(Unary(UnaryOperator::LogicalNot))
                }
            }
            '>' => {
                if chars.peek() == Some(&'=') {
                    chars.next();
                    Token::Symbol(Binary(BinaryOperator::GreaterThanOrEquals))
                } else if chars.peek() == Some(&'>') {
                    chars.next();
                    Token::Symbol(Binary(BinaryOperator::BitwiseShiftRight))
                } else {
                    Token::Symbol(Binary(BinaryOperator::GreaterThan))
                }
            }
            '<' => {
                if chars.peek() == Some(&'=') {
                    chars.next();
                    Token::Symbol(Binary(BinaryOperator::LessThanOrEquals))
                } else if chars.peek() == Some(&'<') {
                    chars.next();
                    Token::Symbol(Binary(BinaryOperator::BitwiseShiftLeft))
                } else {
                    Token::Symbol(Binary(BinaryOperator::LessThan))
                }
            }
            '|' => {
                if chars.peek() == Some(&'|') {
                    chars.next();
                    Token::Symbol(Binary(BinaryOperator::LogicalOr))
                } else {
                    Token::Symbol(Binary(BinaryOperator::BitwiseOr))
                }
            }
            '&' => {
                if chars.peek() == Some(&'&') {
                    chars.next();
                    Token::Symbol(Binary(BinaryOperator::LogicalAnd))
                } else {
                    Token::Symbol(Binary(BinaryOperator::BitwiseAnd))
                }
            }
            '0'..='9' => {
                let mut number_string = String::new();
                number_string.push(c);
                while let Some(&next) = chars.peek() {
                    if next.is_ascii_digit() {
                        number_string.push(next);
                        chars.next();
                    } else {
                        break;
                    }
                }
                match number_string.parse::<u64>() {
                    Ok(num) => Token::NumberLiteral(num),
                    Err(_) => Token::Invalid, // Handle parsing error
                }
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut identifier = String::new();
                identifier.push(c);
                while let Some(&next) = chars.peek() {
                    if next.is_ascii_alphanumeric() || next == '_' {
                        identifier.push(next);
                        chars.next();
                    } else {
                        break;
                    }
                }
                match match_keyword(identifier.as_str()) {
                    Some(key) => Token::Keyword(key),
                    None => Token::Identifier(identifier),
                }
            }
            ' ' | '\n' | '\t' => continue,
            _ => Token::Invalid,
        };
        tokens.push(next);
    }
    tokens.push(Token::EOF);
    tokens
}

trait TryFrom<T> {
    fn try_from(self, line_number: Position) -> Result<T, CompilerError>;
}

macro_rules! impl_token_try_from {
    ($target_type:ty, $variant:path, $type_name:expr) => {
        impl TryFrom<Token> for $target_type {
            fn try_from(token: Token, line_number: Position) -> Result<Self, CompilerError> {
                match token {
                    $variant(value) => Ok(value),
                    _ => Err(SyntaxError(
                        format!("Expected {}, got {:?} at {:?}", $type_name, token, line_number),
                    ))
                }
            }
        }
    };
}
