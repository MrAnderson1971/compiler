use crate::ast::BlockItem::{D, S};
use crate::ast::Expression::{
    Assignment, Condition, Constant, FunctionCall, Postfix, Prefix, Unary, Variable,
};
use crate::ast::ForInit::{InitDecl, InitExp};
use crate::ast::Statement::{Compound, For, If, Null, Return, While};
use crate::ast::{
    ASTNode, Block, BlockItem, Declaration, Expression, ForInit, FunctionDeclaration, Program,
    Statement, VariableDeclaration, extract_base_variable, is_lvalue_node,
};
use crate::common::{Identifier, Position};
use crate::errors::CompilerError;
use crate::errors::CompilerError::{SemanticError, SyntaxError};
use crate::lexer::BinaryOperator::Assign;
use crate::lexer::Symbol::{Ambiguous, Binary};
use crate::lexer::{BinaryOperator, Keyword, Symbol, Token, UnaryOperator, UnaryOrBinaryOp};
use std::collections::VecDeque;
use std::rc::Rc;

macro_rules! expect_token {
    ($parser:expr, $expected_token:expr) => {{
        let expected = $expected_token; // Evaluate expected token once
        // Peek first to check without consuming
        let peeked_token = $parser.peek_token();
        // Check if peek succeeded AND the token matches
        if peeked_token == expected {
            $parser.tokens.pop_front();
            Ok(())
        } else {
            // Peeked successfully, but the token doesn't match.
            let line = Rc::clone(&$parser.line_number);
            Err(CompilerError::SyntaxError(format!(
                "Expected {:?} but got {:?} at {:?}",
                expected, peeked_token, line
            )))
        }
    }};
}

pub(crate) struct Parser {
    loop_label_counter: i32,
    tokens: VecDeque<Token>,
    line_number: Rc<Position>,
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
            Assign => 1,
        },
        _ => -1,
    }
}

impl Parser {
    pub(crate) fn new(tokens: VecDeque<Token>) -> Self {
        Parser {
            loop_label_counter: 0,
            tokens,
            line_number: Rc::from((0, "".to_string())),
        }
    }

    #[allow(unused_variables)]
    fn parse_params(&mut self) -> Result<Vec<Identifier>, CompilerError> {
        expect_token!(self, Token::Symbol(Symbol::OpenParenthesis))?;
        let mut params = vec![];

        let next = self.tokens.pop_front().unwrap();
        match next {
            Token::Symbol(Symbol::CloseParenthesis) => return Ok(params),
            Token::Keyword(Keyword::Int) => {
                if let Token::Name(name) = self.tokens.pop_front().unwrap() {
                    params.push(name);
                } else {
                    return Err(SyntaxError(format!(
                        "Expected identifier but got {:?}",
                        next
                    )));
                }
            }
            _ => {
                return Err(SyntaxError(format!(
                    "Expected identifier but got {:?}",
                    next
                )));
            }
        }

        loop {
            let next = self.tokens.pop_front().unwrap();
            match next {
                Token::Symbol(Symbol::CloseParenthesis) => return Ok(params),
                Token::Symbol(Symbol::Comma) => {
                    expect_token!(self, Token::Keyword(Keyword::Int))?;
                    if let Token::Name(name) = self.tokens.pop_front().unwrap() {
                        params.push(name);
                    } else {
                        return Err(SyntaxError(format!(
                            "Expected identifier but got {:?}",
                            next
                        )));
                    }
                }
                _ => {
                    return Err(SyntaxError(format!(
                        "Expected identifier but got {:?}",
                        next
                    )));
                }
            }
        }
    }

    fn parse_function_declaration(
        &mut self,
    ) -> Result<ASTNode<FunctionDeclaration>, CompilerError> {
        expect_token!(self, Token::Keyword(Keyword::Int))?;
        let current = self.peek_token();
        let function_name = match current {
            Token::Name(name) => name,
            _ => {
                return Err(SyntaxError(format!(
                    "Expected identifier but got {:?} at {:?}",
                    current, self.line_number
                )));
            }
        };
        self.tokens.pop_front();
        self.line_number = Rc::from((0, function_name.clone()));
        let mut block_items: Vec<ASTNode<BlockItem>> = Vec::new();
        let params = self.parse_params()?;
        expect_token!(self, Token::Symbol(Symbol::OpenBrace))?;

        let mut next_token = self.peek_token();
        loop {
            match next_token {
                Token::Symbol(Symbol::CloseBrace) => break,
                Token::EOF => return Err(SyntaxError("Unexpected EOF".to_string())),
                _ => {
                    let item = self.parse_block_item()?;
                    block_items.push(item);
                }
            }
            next_token = self.peek_token();
        }
        let function_body = self.make_node::<Block>(block_items);
        expect_token!(self, Token::Symbol(Symbol::CloseBrace))?;
        Ok(self.make_node(FunctionDeclaration {
            name: Rc::from(function_name),
            params,
            body: function_body,
        }))
    }

    fn parse_declaration(&mut self) -> Result<ASTNode<VariableDeclaration>, CompilerError> {
        let current = self.peek_token();
        self.tokens.pop_front();
        let identifier = match current {
            Token::Name(name) => name,
            _ => {
                return Err(SyntaxError(format!(
                    "Expected identifier but got {:?} at {:?}",
                    current, self.line_number
                )));
            }
        };
        if let Token::Symbol(Binary(Assign)) = self.peek_token() {
            self.tokens.pop_front();
            let expression = self.parse_binary_op(0)?;
            Ok(self.make_node(VariableDeclaration {
                name: Rc::from(identifier),
                init: Some(expression),
            }))
        } else {
            Ok(self.make_node(VariableDeclaration {
                name: Rc::from(identifier),
                init: None,
            }))
        }
    }

    fn parse_increment_decrement(
        &mut self,
        expression: ASTNode<Expression>,
        symbol: UnaryOperator,
        is_prefix: bool,
    ) -> Result<ASTNode<Expression>, CompilerError> {
        if is_lvalue_node(&expression.kind) {
            let which = if is_prefix {
                Prefix(symbol, Box::from(expression))
            } else {
                Postfix(symbol, Box::from(expression))
            };
            Ok(self.make_node(which))
        } else {
            Err(SemanticError(format!(
                "Expected lvalue node at {:?} but got {:?}",
                expression, self.line_number
            )))
        }
    }

    fn parse_arguments(&mut self) -> Result<Box<Vec<ASTNode<Expression>>>, CompilerError> {
        let mut params = vec![];
        let next = self.peek_token();

        match next {
            Token::Symbol(Symbol::CloseParenthesis) => {
                self.tokens.pop_front();
                return Ok(Box::new(params));
            }
            _ => {
                params.push(self.parse_binary_op(0)?);
            }
        }

        loop {
            let next = self.peek_token();
            match next {
                Token::Symbol(Symbol::CloseParenthesis) => {
                    self.tokens.pop_front();
                    return Ok(Box::new(params));
                }
                _ => {
                    expect_token!(self, Token::Symbol(Symbol::Comma))?;
                    params.push(self.parse_binary_op(0)?);
                }
            }
        }
    }

    fn parse_primary(&mut self) -> Result<ASTNode<Expression>, CompilerError> {
        let token = self.peek_token();
        match token {
            Token::NumberLiteral(value) => {
                self.tokens.pop_front();
                Ok(self.make_node::<Expression>(Constant(value)))
            }
            Token::Symbol(..) => {
                expect_token!(self, Token::Symbol(Symbol::OpenParenthesis))?;
                let expression = self.parse_binary_op(0)?;
                expect_token!(self, Token::Symbol(Symbol::CloseParenthesis))?;
                Ok(expression)
            }
            Token::Name(identifier) => {
                self.tokens.pop_front();
                if let Token::Symbol(Symbol::OpenParenthesis) = self.peek_token() {
                    self.tokens.pop_front();
                    let params = self.parse_arguments()?;
                    Ok(self.make_node(FunctionCall(Rc::from(identifier), params)))
                } else {
                    Ok(self.make_node(Variable(Rc::from(identifier))))
                }
            }
            _ => Err(SyntaxError(format!(
                "Unexpected token {:?} at {:?}",
                token, self.line_number
            ))),
        }
    }

    fn parse_unary_or_primary(&mut self) -> Result<ASTNode<Expression>, CompilerError> {
        let token = self.peek_token();
        match token {
            Token::Symbol(Symbol::Unary(op)) => {
                self.tokens.pop_front();
                return match op {
                    UnaryOperator::Increment | UnaryOperator::Decrement => {
                        let expression = self.parse_unary_or_primary()?;
                        self.parse_increment_decrement(expression, op, true)
                    }
                    _ => {
                        let expression = self.parse_unary_or_primary()?;
                        Ok(self.make_node(Unary(op, Box::from(expression))))
                    }
                };
            }
            Token::Symbol(Ambiguous(UnaryOrBinaryOp::Addition)) => {
                self.tokens.pop_front();
                let expression = self.parse_unary_or_primary()?;
                return Ok(self.make_node(Unary(UnaryOperator::UnaryAdd, Box::from(expression))));
            }
            Token::Symbol(Ambiguous(UnaryOrBinaryOp::Subtraction)) => {
                self.tokens.pop_front();
                let expression = self.parse_unary_or_primary()?;
                return Ok(self.make_node(Unary(UnaryOperator::Negate, Box::from(expression))));
            }
            _ => {}
        }

        let primary = self.parse_primary()?;

        let token = self.peek_token();
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
    fn parse_condition(&mut self) -> Result<ASTNode<Expression>, CompilerError> {
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
    fn parse_binary_op(
        &mut self,
        min_precedence: i32,
    ) -> Result<ASTNode<Expression>, CompilerError> {
        let mut left = self.parse_unary_or_primary()?;
        loop {
            let token = self.peek_token();
            if !matches!(token, Token::Symbol(_)) {
                return Err(SyntaxError(format!(
                    "Unexpected token {:?} at {:?}",
                    token, self.line_number
                )));
            }
            let token = if let Token::Symbol(token @ (Binary(_) | Ambiguous(_))) = token {
                token
            } else {
                break;
            };
            if get_precedence(token) < min_precedence {
                break;
            }
            self.tokens.pop_front();
            if let Token::Symbol(Binary(Assign)) = self.peek_token() {
                // compound assignment
                if is_lvalue_node(&left.kind) {
                    /*
                    Turn x ?= rhs into x = (x ? rhs)
                    */
                    self.tokens.pop_front(); // remove the = operator
                    let right = self.parse_binary_op(get_precedence(Binary(Assign)))?;
                    let left_variable = self.make_node(Variable(extract_base_variable(&left.kind)));
                    let op = if let Binary(op) = token {
                        op
                    } else if token == Ambiguous(UnaryOrBinaryOp::Addition) {
                        BinaryOperator::Addition
                    } else {
                        BinaryOperator::Subtraction
                    };
                    let binary = self.make_node(Expression::Binary {
                        op,
                        left: Box::from(left_variable),
                        right: Box::from(right),
                    });
                    left = self.make_node(Assignment {
                        left: Box::from(left),
                        right: Box::from(binary),
                    });
                    continue;
                } else {
                    return Err(SemanticError(format!(
                        "Expected lvalue at {:?}",
                        self.line_number
                    )));
                }
            }
            match token {
                Binary(symbol) => match symbol {
                    Assign => {
                        if !is_lvalue_node(&left.kind) {
                            return Err(SemanticError(format!(
                                "Expected lvalue node at {:?}",
                                self.line_number
                            )));
                        }
                        let right = self.parse_binary_op(get_precedence(token))?;
                        left = self.make_node(Assignment {
                            left: Box::from(left),
                            right: Box::from(right),
                        });
                    }
                    BinaryOperator::Ternary => {
                        let middle = self.parse_condition()?;
                        let right = self.parse_binary_op(get_precedence(token))?;
                        left = self.make_node(Condition {
                            condition: Box::from(left),
                            if_true: Box::from(middle),
                            if_false: Box::from(right),
                        });
                    }

                    _ => {
                        let right = self.parse_binary_op(get_precedence(token) + 1)?;
                        left = self.make_node(Expression::Binary {
                            op: symbol,
                            left: Box::from(left),
                            right: Box::from(right),
                        });
                    }
                },
                Ambiguous(UnaryOrBinaryOp::Addition) => {
                    let right = self.parse_binary_op(get_precedence(token) + 1)?;
                    left = self.make_node(Expression::Binary {
                        op: BinaryOperator::Addition,
                        left: Box::from(left),
                        right: Box::from(right),
                    });
                }
                Ambiguous(UnaryOrBinaryOp::Subtraction) => {
                    let right = self.parse_binary_op(get_precedence(token) + 1)?;
                    left = self.make_node(Expression::Binary {
                        op: BinaryOperator::Subtraction,
                        left: Box::from(left),
                        right: Box::from(right),
                    });
                }
                _ => unreachable!(),
            }
        }
        Ok(left)
    }

    fn parse_for_init(&mut self) -> Result<ASTNode<ForInit>, CompilerError> {
        match self.peek_token() {
            Token::Keyword(Keyword::Int) => {
                self.tokens.pop_front();
                let variable_declaration = self.parse_declaration()?;
                let declaration =
                    self.make_node(Declaration::VariableDeclaration(variable_declaration));
                Ok(self.make_node(InitDecl(declaration)))
            }
            Token::Symbol(Symbol::Semicolon) => Ok(self.make_node(InitExp(None))),
            _ => {
                let exp = self.parse_binary_op(0)?;
                Ok(self.make_node(InitExp(Some(exp))))
            }
        }
    }

    fn parse_statement(&mut self) -> Result<ASTNode<Statement>, CompilerError> {
        let token = self.peek_token();
        match token {
            Token::Keyword(keyword) => {
                self.tokens.pop_front();
                match keyword {
                    Keyword::Return => {
                        let expression = self.parse_binary_op(0)?;
                        self.end_line()?;
                        Ok(self.make_node(Return(expression)))
                    }
                    Keyword::If => {
                        expect_token!(self, Token::Symbol(Symbol::OpenParenthesis))?;
                        let condition = self.parse_binary_op(0)?;
                        expect_token!(self, Token::Symbol(Symbol::CloseParenthesis))?;
                        let body = self.parse_statement()?;
                        if let Token::Keyword(Keyword::Else) = self.peek_token() {
                            self.tokens.pop_front();
                            let else_body = self.parse_statement()?;
                            Ok(self.make_node(If {
                                condition,
                                if_true: Box::from(body),
                                if_false: Some(Box::from(else_body)),
                            }))
                        } else {
                            Ok(self.make_node(If {
                                condition,
                                if_true: Box::from(body),
                                if_false: None,
                            }))
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
                        let condition = self.parse_binary_op(0)?;
                        expect_token!(self, Token::Symbol(Symbol::CloseParenthesis))?;
                        let body = Box::from(self.parse_statement()?);
                        Ok(self.make_node(While {
                            condition,
                            body,
                            label: Rc::from(label),
                            is_do_while: false,
                        }))
                    }
                    Keyword::Break => {
                        let node = self.make_node(Statement::Break(Rc::from("".to_string())));
                        Ok(node)
                    }
                    Keyword::Continue => {
                        let node = self.make_node(Statement::Continue {
                            label: Rc::from("".to_string()),
                            is_for: false,
                        });
                        Ok(node)
                    }
                    Keyword::Do => {
                        let label = self.loop_label_counter.to_string();
                        self.loop_label_counter += 1;
                        let body = Box::from(self.parse_statement()?);
                        expect_token!(self, Token::Keyword(Keyword::While))?;
                        expect_token!(self, Token::Symbol(Symbol::OpenParenthesis))?;
                        let condition = self.parse_binary_op(0)?;
                        expect_token!(self, Token::Symbol(Symbol::CloseParenthesis))?;
                        self.end_line()?;
                        Ok(self.make_node(While {
                            condition,
                            body,
                            label: Rc::from(label),
                            is_do_while: true,
                        }))
                    }
                    Keyword::For => {
                        expect_token!(self, Token::Symbol(Symbol::OpenParenthesis))?;
                        let label = self.loop_label_counter.to_string();
                        self.loop_label_counter += 1;
                        let init = self.parse_for_init()?;
                        self.end_line()?;
                        let condition = if let Token::Symbol(Symbol::Semicolon) = self.peek_token()
                        {
                            None
                        } else {
                            Some(self.parse_binary_op(0)?)
                        };
                        self.end_line()?;
                        let increment =
                            if let Token::Symbol(Symbol::CloseParenthesis) = self.peek_token() {
                                None
                            } else {
                                Some(self.parse_binary_op(0)?)
                            };
                        expect_token!(self, Token::Symbol(Symbol::CloseParenthesis))?;
                        let body = Box::from(self.parse_statement()?);
                        Ok(self.make_node(For {
                            init,
                            condition,
                            increment,
                            body,
                            label: Rc::from(label),
                        }))
                    }
                    _ => Err(SyntaxError(format!(
                        "Unexpected keyword {:?} at {:?}",
                        keyword, self.line_number
                    ))),
                }
            }
            Token::Symbol(Symbol::OpenBrace) => {
                self.tokens.pop_front();
                let mut block_items: Block = Vec::new();
                let mut next_token = self.peek_token();
                loop {
                    match next_token {
                        Token::Symbol(Symbol::CloseBrace) => {
                            self.tokens.pop_front();
                            break;
                        }
                        _ => {
                            let block = self.parse_block_item()?;
                            block_items.push(block);
                        }
                    }
                    next_token = self.peek_token();
                }
                Ok(self.make_node(Compound(self.make_node(block_items))))
            }
            Token::Symbol(Symbol::Semicolon) => {
                self.end_line()?;
                Ok(self.make_node(Null))
            }
            _ => {
                let out = self.parse_binary_op(0)?;
                self.end_line()?;
                Ok(self.make_node(Statement::Expression(out)))
            }
        }
    }

    fn parse_block_item(&mut self) -> Result<ASTNode<BlockItem>, CompilerError> {
        let token = self.peek_token();
        match token {
            Token::Keyword(keyword) => match keyword {
                Keyword::Int => {
                    self.tokens.pop_front();
                    let out = self.parse_declaration()?;
                    self.end_line()?;
                    Ok(self.make_node(D(self.make_node(Declaration::VariableDeclaration(out)))))
                }
                _ => {
                    let statement = self.parse_statement()?;
                    Ok(self.make_node(S(Box::from(statement))))
                }
            },
            _ => {
                let statement = self.parse_statement()?;
                Ok(self.make_node(S(Box::from(statement))))
            }
        }
    }

    pub(crate) fn parse_program(&mut self) -> Result<ASTNode<Program>, CompilerError> {
        let mut declarations = Vec::new();

        while !matches!(self.tokens.front().unwrap(), Token::EOF) {
            let function_declaration = self.parse_function_declaration()?;

            let declaration = Declaration::FunctionDeclaration(function_declaration);

            let declaration_node = self.make_node(declaration);

            declarations.push(declaration_node);
        }

        expect_token!(self, Token::EOF)?;

        Ok(self.make_node(declarations))
    }

    fn peek_token(&self) -> Token {
        self.tokens.front().unwrap().clone()
    }

    fn end_line(&mut self) -> Result<(), CompilerError> {
        let current = self.peek_token();
        match current {
            Token::Symbol(Symbol::Semicolon) => {
                self.line_number = Rc::from((self.line_number.0 + 1, self.line_number.1.clone()));
                self.tokens.pop_front();
                Ok(())
            }
            _ => Err(SyntaxError(format!(
                "Expected semicolon but got {:?} at {:?}",
                current, self.line_number
            )))?,
        }
    }

    fn make_node<T>(&self, kind: T) -> ASTNode<T> {
        ASTNode {
            line_number: Rc::clone(&self.line_number),
            kind,
        }
    }
}
