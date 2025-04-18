use crate::ast::ASTNodeType::{
    AssignmentNode, BinaryNode, BlockNode, BreakNode, ConditionNode, ConstNode, ContinueNode,
    DeclarationNode, ForNode, FunctionNode, PostfixNode, PrefixNode, ProgramNode, ReturnNode,
    UnaryNode, VariableNode, WhileNode,
};
use crate::ast::{ASTNode, ASTNodeType, extract_base_variable, is_lvalue_node};
use crate::common::Position;
use crate::errors::CompilerError;
use crate::errors::CompilerError::{SemanticError, SyntaxError};
use crate::lexer::Symbol::{Ambiguous, Binary};
use crate::lexer::{BinaryOperator, Keyword, Symbol, Token, UnaryOperator, UnaryOrBinaryOp};
use std::collections::VecDeque;

macro_rules! expect_token {
    ($parser:expr, $expected_token:expr) => {{
        let expected = $expected_token; // Evaluate expected token once
        // Peek first to check without consuming
        match $parser.peek_token() {
            // Check if peek succeeded AND the token matches
            Ok(ref peeked_token) if peeked_token == &expected => {
                // Instead of returning an Option, convert to Result
                match $parser.tokens.pop_front() {
                    Some(_) => Ok(()),
                    None => {
                        let line = $parser.line_number.clone();
                        Err(CompilerError::SyntaxError(format!(
                            "Internal error: Token was peeked but couldn't be consumed at {:?}",
                            line
                        )))
                    }
                }
            }
            Ok(other_token) => {
                // Peeked successfully, but the token doesn't match.
                let line = $parser.line_number.clone();
                Err(CompilerError::SyntaxError(format!(
                    "Expected {:?} but got {:?} at {:?}",
                    expected, other_token, line
                )))
            }
            Err(err) => Err(err), // Propagate the original error
        }
    }};
}

pub(crate) struct Parser {
    loop_label_counter: i32,
    tokens: VecDeque<Token>,
    line_number: Position,
}

fn get_precedence(op: Symbol) -> i32 {
    match op {
        Ambiguous(..) => 45, // plus or minus
        Binary(op) => match op {
            BinaryOperator::Multiply | BinaryOperator::Divide | BinaryOperator::Modulo => 50,
            BinaryOperator::BitwiseShiftLeft | BinaryOperator::BitwiseShiftRight => 45,
            BinaryOperator::Addition | BinaryOperator::Subtraction => 45,
            BinaryOperator::LessThan
            | BinaryOperator::LessThanOrEquals
            | BinaryOperator::GreaterThan
            | BinaryOperator::GreaterThanOrEquals => 35,
            BinaryOperator::Equals | BinaryOperator::NotEquals => 30,
            BinaryOperator::BitwiseAnd => 25,
            BinaryOperator::BitwiseXor => 20,
            BinaryOperator::BitwiseOr => 15,
            BinaryOperator::LogicalAnd => 10,
            BinaryOperator::LogicalOr => 5,
            BinaryOperator::Ternary => 3,
            BinaryOperator::Assign => 1,
        },
        _ => -1,
    }
}

impl Parser {
    pub(crate) fn new(tokens: VecDeque<Token>) -> Self {
        Parser {
            loop_label_counter: 0,
            tokens: tokens.clone(),
            line_number: (0, "".to_string()),
        }
    }

    fn parse_function_declaration(&mut self) -> Result<Box<ASTNode>, CompilerError> {
        expect_token!(self, Token::Keyword(Keyword::Int))?;
        let current = self.peek_token()?;
        let function_name = match current {
            Token::Identifier(name) => name,
            _ => {
                return Err(SyntaxError(format!(
                    "Expected identifier but got {:?} at {:?}",
                    current, self.line_number
                )));
            }
        };
        self.tokens.pop_front();
        self.line_number = (0, function_name.clone());
        let mut function_body = self.make_node(BlockNode { body: Vec::new() });
        expect_token!(self, Token::Symbol(Symbol::OpenParenthesis))?;
        expect_token!(self, Token::Symbol(Symbol::CloseParenthesis))?;
        expect_token!(self, Token::Symbol(Symbol::OpenBrace))?;

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
        expect_token!(self, Token::Symbol(Symbol::CloseBrace))?;
        Ok(self.make_node(FunctionNode {
            identifier: function_name,
            body: function_body,
        }))
    }

    fn parse_declaration(&mut self) -> Result<Box<ASTNode>, CompilerError> {
        let current = self.peek_token()?;
        self.tokens.pop_front();
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
            if let Binary(BinaryOperator::Assign) = symbol {
                let expression = self.parse_expression()?;
                Ok(self.make_node(DeclarationNode {
                    identifier,
                    expression: Some(expression),
                }))
            } else {
                Err(SyntaxError(format!(
                    "Expected = or ; but got {:?} at {:?}",
                    symbol, self.line_number
                )))
            }
        } else {
            Ok(self.make_node(DeclarationNode {
                identifier,
                expression: None,
            }))
        }
    }

    fn parse_increment_decrement(
        &mut self,
        expression: Box<ASTNode>,
        symbol: UnaryOperator,
        is_prefix: bool,
    ) -> Result<Box<ASTNode>, CompilerError> {
        if is_lvalue_node(&expression.kind) {
            let which = if is_prefix {
                PrefixNode {
                    variable: expression,
                    operator: symbol,
                }
            } else {
                PostfixNode {
                    variable: expression,
                    operator: symbol,
                }
            };
            Ok(self.make_node(which))
        } else {
            Err(SemanticError(format!(
                "Expected lvalue node at {:?} but got {:?}",
                expression, self.line_number
            )))
        }
    }

    fn parse_primary(&mut self) -> Result<Box<ASTNode>, CompilerError> {
        let token = self.peek_token()?;
        self.tokens.pop_front();
        match token {
            Token::NumberLiteral(value) => Ok(self.make_node(ConstNode { value })),
            Token::Symbol(..) => {
                expect_token!(self, Token::Symbol(Symbol::OpenParenthesis))?;
                let expression = self.parse_expression();
                expect_token!(self, Token::Symbol(Symbol::CloseParenthesis))?;
                expression
            }
            Token::Identifier(identifier) => Ok(self.make_node(VariableNode { identifier })),
            _ => Err(SemanticError(format!(
                "Unexpected token {:?} at {:?}",
                token, self.line_number
            ))),
        }
    }

    fn parse_unary_or_primary(&mut self) -> Result<Box<ASTNode>, CompilerError> {
        let token = self.peek_token()?;
        if let Token::Symbol(Symbol::Unary(op)) = token {
            self.tokens.pop_front();
            return match op {
                UnaryOperator::Increment | UnaryOperator::Decrement => {
                    self.tokens.pop_front();
                    let expression = self.parse_unary_or_primary()?;
                    self.parse_increment_decrement(expression, op, true)
                }
                _ => {
                    let expression = self.parse_unary_or_primary()?;
                    Ok(self.make_node(UnaryNode { op, expression }))
                }
            };
        }
        let primary = self.parse_primary()?;

        let token = self.peek_token()?;
        match token {
            Token::Symbol(Symbol::Unary(
                op @ (UnaryOperator::Increment | UnaryOperator::Decrement),
            )) => {
                self.tokens.pop_front();
                self.parse_increment_decrement(primary, op, false)
            }
            _ => Ok(primary),
        }
    }

    /*
    Parse the middle term of a ternary statement, keeps going until it hits a colon
    */
    fn parse_condition(&mut self) -> Result<Box<ASTNode>, CompilerError> {
        let middle = self.parse_binary_op(0);
        expect_token!(self, Token::Symbol(Symbol::Colon))?;
        middle
    }

    /*
    *parse_exp(tokens, min_prec):
    left = parse_factor(tokens)
    next_token = peek(tokens)
    while next_token is a binary operator and precedence(next_token) >= min_prec:
        if next_token is "=":
            take_token(tokens) // remove "=" from list of tokens
            right = parse_exp(tokens, precedence(next_token))
            left = Assignment(left, right)
        else if next_token is "?":
            middle = parse_conditional_middle(tokens)
            right = parse_exp(tokens, precedence(next_token))
            left = Conditional(left, middle, right)
        else:
            operator = parse_binop(tokens)
            right = parse_exp(tokens, precedence(next_token) + 1)
            left = Binary(operator, left, right)
        next_token = peek(tokens)
    return left
    */
    fn parse_binary_op(&mut self, min_precedence: i32) -> Result<Box<ASTNode>, CompilerError> {
        let mut left = self.parse_unary_or_primary()?;
        let mut token = self.peek_token()?;
        loop {
            match token {
                Token::Symbol(symbol @ Binary(op)) => {
                    if get_precedence(symbol) < min_precedence {
                        break;
                    }
                    self.tokens.pop_front();
                    match symbol {
                        Binary(BinaryOperator::Assign) => {
                            if !is_lvalue_node(&left.kind) {
                                return Err(SemanticError(format!(
                                    "Expected lvalue node at {:?}",
                                    self.line_number
                                )));
                            }
                            let right = self.parse_binary_op(get_precedence(symbol))?;
                            left = self.make_node(AssignmentNode { left, right });
                        }
                        Binary(BinaryOperator::Ternary) => {
                            let middle = self.parse_condition()?;
                            let right = self.parse_binary_op(get_precedence(symbol))?;
                            left = self.make_node(ConditionNode {
                                condition: left,
                                if_true: Some(middle),
                                if_false: Some(right),
                            });
                        }

                        _ => {
                            if let Token::Symbol(Binary(BinaryOperator::Assign)) =
                                self.peek_token()?
                            {
                                // compound assignment
                                if is_lvalue_node(&left.kind) {
                                    self.tokens.pop_front();
                                    let right = self.parse_binary_op(get_precedence(symbol))?;
                                    let left_variable = self.make_node(VariableNode {
                                        identifier: extract_base_variable(&left.kind).unwrap(),
                                    });
                                    let binary = self.make_node(BinaryNode {
                                        op,
                                        left: left_variable,
                                        right,
                                    });
                                    left = self.make_node(AssignmentNode {
                                        left,
                                        right: binary,
                                    });
                                } else {
                                    return Err(SemanticError(format!(
                                        "Expected lvalue at {:?}",
                                        self.line_number
                                    )));
                                }
                            } else {
                                let right = self.parse_binary_op(get_precedence(symbol) + 1)?;
                                left = self.make_node(BinaryNode { op, left, right });
                            }
                        }
                    }
                }
                Token::Symbol(symbol @ Ambiguous(UnaryOrBinaryOp::Addition)) => {
                    let right = self.parse_binary_op(get_precedence(symbol) + 1)?;
                    left = self.make_node(BinaryNode {
                        op: BinaryOperator::Addition,
                        left,
                        right,
                    });
                }
                Token::Symbol(symbol @ Ambiguous(UnaryOrBinaryOp::Subtraction)) => {
                    let right = self.parse_binary_op(get_precedence(symbol) + 1)?;
                    left = self.make_node(BinaryNode {
                        op: BinaryOperator::Subtraction,
                        left,
                        right,
                    });
                }
                Token::Symbol(..) => break,
                _ => {
                    return Err(SyntaxError(format!(
                        "Unexpected token {:?} at {:?}",
                        token, self.line_number
                    )));
                }
            }
            token = self.peek_token()?;
        }
        Ok(left)
    }

    fn parse_expression(&mut self) -> Result<Box<ASTNode>, CompilerError> {
        self.parse_binary_op(0)
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
                            body,
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

    pub(crate) fn parse_program(&mut self) -> Result<Box<ASTNode>, CompilerError> {
        let function_declaration = self.parse_function_declaration()?;
        Ok(Box::new(ASTNode::new(
            self.line_number.clone(),
            ProgramNode {
                function_declaration,
            },
        )))
    }

    fn peek_token(&self) -> Result<Token, CompilerError> {
        if let Some(next) = self.tokens.front() {
            Ok(next.clone())
        } else {
            Err(SyntaxError("Unexpected EOF".to_string()))
        }
    }

    fn end_line(&mut self) -> Result<(), CompilerError> {
        let current = self.peek_token()?;
        match current {
            Token::Symbol(Symbol::Semicolon) => {
                self.line_number.0 += 1;
                self.tokens.pop_front();
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
