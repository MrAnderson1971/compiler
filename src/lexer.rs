use crate::common::Const;
use crate::common::Const::{ConstInt, ConstLong, ConstUInt, ConstULong};
use crate::lexer::Symbol::{Ambiguous, Binary, Unary};
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum BinaryOperator {
    Addition,
    Subtraction,

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
pub(crate) enum UnaryOperator {
    Increment,
    Decrement,
    LogicalNot,
    BitwiseNot,
    Negate,
    UnaryAdd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum UnaryOrBinaryOp {
    Addition,
    Subtraction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Symbol {
    Binary(BinaryOperator),
    Unary(UnaryOperator),
    Ambiguous(UnaryOrBinaryOp),
    OpenParenthesis,
    CloseParenthesis,
    OpenBrace,
    CloseBrace,
    Colon,
    Semicolon,
    Comma,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum StorageClass {
    Static,
    Extern,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Type {
    Void,
    Int,
    Long,
    Unsigned,
    Signed,
    UInt,
    ULong,
}

impl Type {
    pub(crate) fn size(&self) -> i32 {
        match self {
            Type::Void => 0,
            Type::Int | Type::UInt => 4,
            Type::Long | Type::ULong => 8,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Keyword {
    Return,
    If,
    Else,
    Do,
    While,
    For,
    Continue,
    Break,
    Type(Type),
    StorageClass(StorageClass),
}

#[derive(Debug, Clone, PartialEq)] // String prevents Copy. PartialEq is useful for tests.
pub(crate) enum Token {
    Keyword(Keyword),
    Symbol(Symbol),
    Name(String),
    NumberLiteral(Const),
    Invalid,
    Overflow,
    EOF,
}

fn match_keyword(string: &str) -> Option<Keyword> {
    match string {
        "return" => Some(Keyword::Return),
        "int" => Some(Keyword::Type(Type::Int)),
        "if" => Some(Keyword::If),
        "else" => Some(Keyword::Else),
        "do" => Some(Keyword::Do),
        "while" => Some(Keyword::While),
        "for" => Some(Keyword::For),
        "continue" => Some(Keyword::Continue),
        "break" => Some(Keyword::Break),
        "static" => Some(Keyword::StorageClass(StorageClass::Static)),
        "extern" => Some(Keyword::StorageClass(StorageClass::Extern)),
        "long" => Some(Keyword::Type(Type::Long)),
        "unsigned" => Some(Keyword::Type(Type::Unsigned)),
        "signed" => Some(Keyword::Type(Type::Signed)),
        _ => None,
    }
}

pub(crate) fn lex(source: String) -> VecDeque<Token> {
    let mut tokens: VecDeque<Token> = VecDeque::new();
    let mut chars = source.chars().peekable();

    'main_loop: while let Some(c) = chars.next() {
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
            ',' => Token::Symbol(Symbol::Comma),
            '0'..='9' => {
                let mut number_string = String::new();
                number_string.push(c);
                while let Some(char) = chars.peek() {
                    if !char.is_ascii_digit() {
                        break;
                    }
                    number_string.push(*char);
                    chars.next();
                }
                let mut is_long = false;
                let mut is_unsigned = false;
                for _ in 0..2 {
                    match chars.peek() {
                        Some(char) if *char == 'l' || *char == 'L' => {
                            chars.next();
                            if is_long {
                                tokens.push_back(Token::Invalid);
                                continue 'main_loop;
                            }
                            is_long = true;
                        }
                        Some(char) if *char == 'u' || *char == 'U' => {
                            chars.next();
                            if is_unsigned {
                                tokens.push_back(Token::Invalid);
                                continue 'main_loop;
                            }
                            is_unsigned = true;
                        }
                        _ => break,
                    }
                }
                if is_long {
                    match number_string.parse::<u64>() {
                        Ok(num) => {
                            if is_unsigned {
                                Token::NumberLiteral(ConstULong(num))
                            } else {
                                Token::NumberLiteral(ConstLong(num as i64))
                            }
                        }
                        Err(_) => Token::Overflow,
                    }
                } else {
                    match number_string.parse::<u32>() {
                        Ok(num) => {
                            if is_unsigned {
                                Token::NumberLiteral(ConstUInt(num))
                            } else {
                                Token::NumberLiteral(ConstInt(num as i32))
                            }
                        }
                        Err(_) => match number_string.parse::<u64>() {
                            // fallback in case of overflow
                            Ok(num) => {
                                if is_unsigned {
                                    Token::NumberLiteral(ConstULong(num))
                                } else {
                                    Token::NumberLiteral(ConstLong(num as i64))
                                }
                            }
                            Err(_) => Token::Overflow,
                        },
                    }
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
                    None => Token::Name(identifier),
                }
            }
            ' ' | '\n' | '\t' => continue,
            _ => Token::Invalid,
        };
        tokens.push_back(next);
    }
    tokens.push_back(Token::EOF);
    tokens
}
