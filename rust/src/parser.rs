use crate::ast::ASTNode;
use crate::ast::ASTNodeType::{BlockNode, FunctionNode, ProgramNode};
use crate::common::Position;
use crate::errors::CompilerError;
use crate::errors::CompilerError::SyntaxError;
use crate::lexer::{Keyword, Symbol, Token};
use std::collections::VecDeque;
use std::fmt::Debug;

pub struct Parser {
    loop_label_counter: i32,
    tokens: VecDeque<Token>,
    line_number: Position,
}

impl Parser {
    pub(crate) fn new(tokens: VecDeque<Token>) -> Self {
        Parser {
            loop_label_counter: 0,
            tokens: tokens.clone(),
            line_number: (0, "".to_string()),
        }
    }

    pub(crate) fn parse(&mut self) -> Result<Box<ASTNode>, CompilerError> {
        Ok(Box::new(ASTNode::new(
            self.line_number.clone(),
            ProgramNode {
                function_declaration: self.parse_program()?,
            },
        )))
    }

    fn parse_function_declaration(&mut self) -> Result<Box<ASTNode>, CompilerError> {
        let current = self.get_token_and_advance()?;
        match current {
            Token::Keyword(Keyword::Int) => {}
            _ => {
                return Err(SyntaxError(format!(
                    "Expected keyword int but got {:?} at {:?}",
                    current, self.line_number
                )));
            }
        }
        let current = self.get_token_and_advance()?;
        let function_name = match current {
            Token::Identifier(name) => name,
            _ => {
                return Err(SyntaxError(format!(
                    "Expected identifier but got {:?} at {:?}",
                    current, self.line_number
                )));
            }
        };
        self.line_number = (0, function_name.clone());
        let mut function_body = Box::new(ASTNode::new(
            self.line_number.clone(),
            BlockNode { body: Vec::new() },
        ));
        let current = self.get_token_and_advance()?;
        match current {
            Token::Symbol(Symbol::OpenParenthesis) => {}
            _ => {
                return Err(SyntaxError(format!(
                    "Expected ( but got {:?} at {:?}",
                    current, self.line_number
                )));
            }
        }
        let current = self.get_token_and_advance()?;
        match current {
            Token::Symbol(Symbol::CloseParenthesis) => {}
            _ => {
                return Err(SyntaxError(format!(
                    "Expected ) but got {:?} at {:?}",
                    current, self.line_number
                )));
            }
        }
        let current = self.get_token_and_advance()?;
        match current {
            Token::Symbol(Symbol::OpenBrace) => {}
            _ => {
                return Err(SyntaxError(format!(
                    "Expected {{ but got {:?} at {:?}",
                    current, self.line_number
                )));
            }
        }

        let mut next_token = self.peek_token()?;
        loop {
            match next_token {
                Token::Symbol(Symbol::CloseBrace) => break,
                _ => {
                    if let Some(item) = self.parse_block_item()? {
                        if let BlockNode { ref mut body } = function_body.kind {
                            body.push(item);
                        }
                    }
                }
            }
            next_token = self.peek_token()?;
        }
        let current = self.get_token_and_advance()?;
        match current {
            Token::Symbol(Symbol::CloseBrace) => {}
            _ => {
                return Err(SyntaxError(format!(
                    "Expected }} but got {:?} at {:?}",
                    current, self.line_number
                )));
            }
        }
        let function_declaration = FunctionNode {
            identifier: function_name,
            body: function_body,
        };
        Ok(Box::new(ASTNode::new(
            self.line_number.clone(),
            function_declaration,
        )))
    }

    fn parse_block_item(&mut self) -> Result<Option<Box<ASTNode>>, CompilerError> {
        self.get_token_and_advance()?;
        Ok(None)
    }

    fn parse_program(&mut self) -> Result<Box<ASTNode>, CompilerError> {
        let function_declaration = self.parse_function_declaration()?;
        Ok(Box::new(ASTNode::new(
            self.line_number.clone(),
            ProgramNode {
                function_declaration,
            },
        )))
    }

    fn get_token_and_advance(&mut self) -> Result<Token, CompilerError> {
        if let Some(next) = self.tokens.pop_front() {
            Ok(next)
        } else {
            Err(SyntaxError("Unexpected EOF".to_string()))
        }
    }

    fn peek_token(&self) -> Result<Token, CompilerError> {
        if let Some(next) = self.tokens.front() {
            Ok(next.clone())
        } else {
            Err(SyntaxError("Unexpected EOF".to_string()))
        }
    }
}
