use crate::ast::ASTNodeType::{BlockNode, BreakNode, ConditionNode, ContinueNode, DeclarationNode, ForNode, FunctionNode, ProgramNode, ReturnNode, WhileNode};
use crate::ast::{ASTNode, ASTNodeType};
use crate::common::Position;
use crate::errors::CompilerError;
use crate::errors::CompilerError::SyntaxError;
use crate::lexer::BinaryOperator::Assign;
use crate::lexer::Symbol::Binary;
use crate::lexer::{Keyword, Symbol, Token};
use std::collections::VecDeque;

macro_rules! expect_token {
    ($parser:expr, $expected_token:expr) => {{
        let expected = $expected_token; // Evaluate expected token once
        // Peek first to check without consuming
        match $parser.peek_token() {
            // Check if peek succeeded AND the token matches
            Ok(ref peeked_token) if peeked_token == &expected => {
                // Use 'ref' to borrow the peeked token for comparison without moving it.
                // If it matches, consume the actual token from the deque.
                // Map Ok(consumed_token) to Ok(()) as we don't need the token value again.
                // If consuming fails (unexpected EOF after peek), propagate the error.
                $parser.get_token_and_advance().map(|_| ())
            }
            Ok(other_token) => {
                // Peeked successfully, but the token doesn't match.
                // `other_token` is an owned Token because peek_token clones.
                let line = $parser.line_number.clone(); // Clone Position for the error
                // Use the correct CompilerError variant constructor
                Err(CompilerError::SyntaxError(format!(
                    "Expected {:?} but got {:?} at {:?}", // Use Debug format for Position
                    expected, other_token, line
                )))
            }
            Err(_) => {
                // Failed to peek (likely EOF based on peek_token implementation)
                let line = $parser.line_number.clone(); // Clone Position for the error
                // Use the correct CompilerError variant constructor
                Err(CompilerError::SyntaxError(format!(
                    "Expected {:?} but got EOF at {:?}", // Use Debug format for Position
                    expected, line
                )))
            }
        }
    }};
}

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

    fn parse_declaration(&mut self) -> Result<Box<ASTNode>, CompilerError> {
        let current = self.get_token_and_advance()?;
        let identifier = match current {
            Token::Identifier(name) => name,
            _ => {
                return Err(SyntaxError(format!(
                    "Expected identifier but got {:?} at {:?}",
                    current, self.line_number
                )));
            }
        };
        if let Token::Symbol(symbol) = self.peek_token()? {
            if let Binary(Assign) = symbol {
                let expression = self.parse_expression()?;
                Ok(self.make_node(DeclarationNode {identifier, expression: Some(expression)}))
            } else {
                Err(SyntaxError(format!("Expected = or ; but got {:?} at {:?}", symbol, self.line_number)))
            }
        } else {
            Ok(self.make_node(DeclarationNode {identifier, expression: None}))
        }
    }

    fn parse_expression(&mut self) -> Result<Box<ASTNode>, CompilerError> {
        Ok(Box::new(ASTNode::new(
            self.line_number.clone(),
            ASTNodeType::ConstNode { value: 0 },
        )))
    }

    fn parse_statement(&mut self) -> Result<Option<Box<ASTNode>>, CompilerError> {
        let token = self.peek_token()?;
        match token {
            Token::Keyword(keyword) => {
                self.tokens.pop_front();
                match keyword {
                    Keyword::Return => {
                        let expression = self.parse_expression()?;
                        Ok(Some(self.make_node(ReturnNode {
                            expression: Some(expression),
                        })))
                    }
                    Keyword::If => {
                        expect_token!(self, Token::Symbol(Symbol::OpenParenthesis))?;
                        let condition = self.parse_expression()?;
                        expect_token!(self, Token::Symbol(Symbol::CloseParenthesis))?;
                        let body = self.parse_statement()?;
                        if let Token::Keyword(Keyword::Else) = self.peek_token()? {
                            self.tokens.pop_front();
                            let else_body = self.parse_statement()?;
                            Ok(Some(self.make_node(ConditionNode {
                                condition,
                                if_true: body,
                                if_false: else_body,
                            })))
                        } else {
                            Ok(Some(self.make_node(ConditionNode {
                                condition,
                                if_true: body,
                                if_false: None,
                            })))
                        }
                    }
                    Keyword::Else => Err(SyntaxError(format!(
                        "Unexpected else at {:?}",
                        self.line_number
                    ))),
                    Keyword::While => {
                        let label = self.loop_label_counter.to_string();
                        self.loop_label_counter += 1;
                        expect_token!(self, Token::Symbol(Symbol::OpenParenthesis))?;
                        let condition = self.parse_expression()?;
                        expect_token!(self, Token::Symbol(Symbol::CloseParenthesis))?;
                        let body = self.parse_statement()?;
                        Ok(Some(self.make_node(WhileNode {
                            condition,
                            body,
                            label,
                            is_do_while: false,
                        })))
                    }
                    Keyword::Break => {
                        let node = self.make_node(BreakNode {
                            label: "".to_string(),
                        });
                        self.end_line()?;
                        Ok(Some(node))
                    }
                    Keyword::Continue => {
                        let node = self.make_node(ContinueNode {
                            label: "".to_string(),
                        });
                        self.end_line()?;
                        Ok(Some(node))
                    }
                    Keyword::Do => {
                        let label = self.loop_label_counter.to_string();
                        self.loop_label_counter += 1;
                        let body = self.parse_statement()?;
                        expect_token!(self, Token::Keyword(Keyword::While))?;
                        expect_token!(self, Token::Symbol(Symbol::OpenParenthesis))?;
                        let condition = self.parse_expression()?;
                        expect_token!(self, Token::Symbol(Symbol::CloseParenthesis))?;
                        Ok(Some(self.make_node(WhileNode {
                            condition,
                            body: body,
                            label,
                            is_do_while: true,
                        })))
                    }
                    Keyword::For => {
                        let label = self.loop_label_counter.to_string();
                        self.loop_label_counter += 1;
                        let init = self.parse_block_item()?;
                        let condition = self.parse_statement()?;
                        let increment =
                            if let Token::Symbol(Symbol::CloseParenthesis) = self.peek_token()? {
                                Some(self.parse_expression()?)
                            } else {
                                None
                            };
                        expect_token!(self, Token::Symbol(Symbol::CloseParenthesis))?;
                        let body = self.parse_statement()?;
                        Ok(Some(self.make_node(ForNode {
                            init,
                            condition,
                            increment,
                            body,
                            label,
                        })))
                    }
                    _ => Err(SyntaxError(format!(
                        "Unexpected keyword {:?} at {:?}",
                        keyword, self.line_number
                    ))),
                }
            }
            Token::Symbol(Symbol::OpenBrace) => {
                let mut block_items = Vec::<Box<ASTNode>>::new();
                let mut next_token = self.peek_token()?;
                loop {
                    match next_token {
                        Token::Symbol(Symbol::CloseBrace) => break,
                        _ => {
                            if let Some(block) = self.parse_block_item()? {
                                block_items.push(block);
                            }
                        }
                    }
                    next_token = self.peek_token()?;
                }
                Ok(Some(self.make_node(BlockNode { body: block_items })))
            }
            Token::Symbol(Symbol::Semicolon) => {
                self.end_line()?;
                Ok(None)
            }
            _ => {
                let out = self.parse_expression()?;
                self.end_line()?;
                Ok(Some(out))
            }
        }
    }

    fn parse_block_item(&mut self) -> Result<Option<Box<ASTNode>>, CompilerError> {
        let token = self.peek_token()?;
        match token {
            Token::Keyword(keyword) => match keyword {
                Keyword::Int => {
                    self.tokens.pop_front();
                    let out = self.parse_declaration()?;
                    Ok(Some(out))
                }
                _ => self.parse_statement(),
            },
            _ => self.parse_statement(),
        }
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

    fn end_line(&mut self) -> Result<(), CompilerError> {
        let current = self.get_token_and_advance()?;
        match current {
            Token::Symbol(Symbol::Semicolon) => {
                self.line_number.0 += 1;
                Ok(())
            }
            _ => Err(SyntaxError(format!(
                "Expected semicolon but got {:?} at {:?}",
                current, self.line_number
            )))?,
        }
    }

    fn make_node(&self, kind: ASTNodeType) -> Box<ASTNode> {
        Box::new(ASTNode::new(self.line_number.clone(), kind))
    }
}
