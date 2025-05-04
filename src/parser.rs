use crate::ast::BlockItem::{D, S};
use crate::ast::Expression::{
    Assignment, Condition, Constant, FunctionCall, Postfix, Prefix, Unary, Variable,
};
use crate::ast::ForInit::{InitDecl, InitExp};
use crate::ast::Statement::{Compound, For, If, Null, Return, While};
use crate::ast::{
    ASTNode, Block, BlockItem, Declaration, Expression, ForInit, FuncType, FunctionDeclaration,
    Program, Statement, VariableDeclaration, extract_base_variable, is_lvalue_node,
};
use crate::common::Position;
use crate::errors::CompilerError;
use crate::errors::CompilerError::{SemanticError, SyntaxError};
use crate::lexer::BinaryOperator::Assign;
use crate::lexer::Symbol::{Ambiguous, Binary};
use crate::lexer::{
    BinaryOperator, Keyword, StorageClass, Symbol, Token, Type, UnaryOperator, UnaryOrBinaryOp,
};
use std::collections::{HashSet, VecDeque};
use std::rc::Rc;

macro_rules! match_and_consume {
    ($parser:expr, $pattern:pat) => {{
        let token = $parser.peek_token();
        if matches!(token, $pattern) {
            $parser.tokens.pop_front();
            true
        } else {
            false
        }
    }};

    ($parser:expr, $pattern:pat => $replacement:expr) => {{
        let token = $parser.peek_token();
        if let $pattern = token {
            $parser.tokens.pop_front();
            $replacement
        } else {
            None
        }
    }};
}

macro_rules! expect_token {
    ($parser:expr, $expected_token:pat) => {{
        // Use a pattern instead of an expression
        if let Some(token) = $parser.tokens.front() {
            if matches!(token, $expected_token) {
                $parser.tokens.pop_front();
                Ok(())
            } else {
                let line = Rc::clone(&$parser.line_number);
                Err(CompilerError::SyntaxError(format!(
                    "Expected token matching pattern but got {:?} at {:?}",
                    token, line
                )))
            }
        } else {
            let line = Rc::clone(&$parser.line_number);
            Err(CompilerError::SyntaxError(format!(
                "Unexpected end of tokens at {:?}",
                line
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
    fn parse_params(&mut self) -> Result<(Vec<String>, Vec<Type>), CompilerError> {
        expect_token!(self, Token::Symbol(Symbol::OpenParenthesis))?;
        let mut params = vec![];
        let mut types = vec![];

        // Handle empty parameter list
        if match_and_consume!(self, Token::Symbol(Symbol::CloseParenthesis)) {
            return Ok((params, types));
        }

        // Process parameters
        loop {
            // Parse type specifiers
            let mut specifiers = vec![];
            while let Token::Keyword(spec @ Keyword::Type(..)) = self.peek_token() {
                self.tokens.pop_front();
                specifiers.push(spec);
            }

            if specifiers.is_empty() {
                return Err(SyntaxError(format!(
                    "Expected type specifier but got {:?} at {:?}",
                    self.peek_token(),
                    self.line_number
                )));
            }

            let (type_, _) = self.parse_type_and_storage_class(specifiers)?;

            // Parse parameter name
            if let Token::Name(name) = self.peek_token() {
                self.tokens.pop_front();
                params.push(name);
                types.push(type_);
            } else {
                return Err(SyntaxError(format!(
                    "Expected parameter name but got {:?} at {:?}",
                    self.peek_token(),
                    self.line_number
                )));
            }

            // Check for end of parameter list or more parameters
            if match_and_consume!(self, Token::Symbol(Symbol::CloseParenthesis)) {
                return Ok((params, types));
            }

            expect_token!(self, Token::Symbol(Symbol::Comma))?;
        }
    }

    fn parse_type_specifier(&self, types: Vec<Type>) -> Result<Type, CompilerError> {
        if types.is_empty() {
            return Err(SyntaxError(format!(
                "Invalid type specifier {:?} at {:?}",
                types, self.line_number
            )));
        }
        if types == vec![Type::Double] {
            return Ok(Type::Double);
        }
        let mut seen = HashSet::new();
        for item in types.iter() {
            if !seen.insert(*item) {
                return Err(SyntaxError(format!(
                    "Invalid type specifier {:?} at {:?}",
                    types, self.line_number
                )));
            }
        }
        if seen.contains(&Type::Double) {
            return Err(SyntaxError(format!(
                "Double cannot be combined with other specifiers, at {:?}",
                self.line_number
            )));
        }
        if seen.contains(&Type::Signed) && seen.contains(&Type::Unsigned) {
            return Err(SyntaxError(format!(
                "Invalid type specifier {:?} at {:?}",
                types, self.line_number
            )));
        }
        if seen.contains(&Type::Unsigned) && seen.contains(&Type::Long) {
            Ok(Type::ULong)
        } else if seen.contains(&Type::Unsigned) {
            Ok(Type::UInt)
        } else if seen.contains(&Type::Long) {
            Ok(Type::Long)
        } else {
            Ok(Type::Int)
        }
    }

    /*
    parse_type_and_storage_class(specifier_list):
         types = []
         storage_classes = []
         1 for specifier in specifier_list:
            if specifier is "int":
                types.append(specifier)
            else:
                storage_classes.append(specifier)
         if length(types) != 1:
            fail("Invalid type specifier")
         if length(storage_classes) > 1:
            fail("Invalid storage class")

         2 type = Int

        if length(storage_classes) == 1:
            3 storage_class = parse_storage_class(storage_classes[0])
        else:
            storage_class = null
         return (type, storage_class)
     */
    fn parse_type_and_storage_class(
        &mut self,
        specifier_list: Vec<Keyword>,
    ) -> Result<(Type, Option<StorageClass>), CompilerError> {
        let mut types = vec![];
        let mut storage_classes = vec![];
        for specifier in specifier_list.iter() {
            if let Keyword::Type(type_) = specifier {
                types.push(*type_);
            } else if let Keyword::StorageClass(class) = specifier {
                storage_classes.push(class);
            }
        }

        let type_ = self.parse_type_specifier(types)?;
        if storage_classes.len() > 1 {
            return Err(SyntaxError(format!(
                "Invalid storage class {:?} at {:?}",
                storage_classes, self.line_number
            )));
        };

        let storage_class = if storage_classes.len() == 1 {
            Some(*storage_classes[0])
        } else {
            None
        };
        Ok((type_, storage_class))
    }

    fn parse_top_level(&mut self) -> Result<ASTNode<Declaration>, CompilerError> {
        let mut specifiers = vec![];
        while let Token::Keyword(spec @ (Keyword::Type(..) | Keyword::StorageClass(..))) =
            self.peek_token()
        {
            self.tokens.pop_front();
            specifiers.push(spec);
        }
        let (type_, storage_class) = self.parse_type_and_storage_class(specifiers)?;
        let function_name =
            if let Some(name) = match_and_consume!(self, Token::Name(name) => Some(name)) {
                name
            } else {
                return Err(SyntaxError(format!(
                    "Expected identifier but got {:?} at {:?}",
                    self.peek_token(),
                    self.line_number
                )));
            };
        self.line_number = Rc::from((0, function_name.clone()));
        let mut block_items: Vec<ASTNode<BlockItem>> = Vec::new();
        let next = self.peek_token();
        match next {
            Token::Symbol(Symbol::OpenParenthesis) => {} // function
            Token::Symbol(Binary(Assign)) | Token::Symbol(Symbol::Semicolon) => {
                // top level variable
                let declaration =
                    self.parse_declaration((type_, storage_class), Some(function_name))?;
                self.tokens.pop_front(); // consume semicolon
                return Ok(self.make_node(Declaration::VariableDeclaration(declaration.kind)));
            }
            _ => {
                return Err(SyntaxError(format!(
                    "Unexpected token {:?} at {:?}",
                    self.peek_token(),
                    self.line_number
                )));
            }
        }

        let (params, types) = self.parse_params()?;

        // function prototype
        if match_and_consume!(self, Token::Symbol(Symbol::Semicolon)) {
            return Ok(
                self.make_node(Declaration::FunctionDeclaration(FunctionDeclaration {
                    name: Rc::from(function_name),
                    params,
                    body: None,
                    storage_class,
                    func_type: Rc::from(FuncType {
                        params: types,
                        ret: type_,
                    }),
                })),
            );
        }

        // full definition
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
        Ok(
            self.make_node(Declaration::FunctionDeclaration(FunctionDeclaration {
                name: Rc::from(function_name),
                params,
                body: Some(function_body),
                storage_class,
                func_type: Rc::from(FuncType {
                    params: types,
                    ret: type_,
                }),
            })),
        )
    }

    fn parse_declaration(
        &mut self,
        specifiers: (Type, Option<StorageClass>),
        name: Option<String>,
    ) -> Result<ASTNode<VariableDeclaration>, CompilerError> {
        let identifier = if let Some(name) = name {
            name
        } else {
            let current = self.consume_and_pop();
            match current {
                Token::Name(name) => name,
                _ => {
                    return Err(SyntaxError(format!(
                        "Expected identifier but got {:?} at {:?}",
                        current, self.line_number
                    )));
                }
            }
        };
        if match_and_consume!(self, Token::Symbol(Binary(Assign))) {
            let expression = self.parse_binary_op(0)?;
            Ok(self.make_node(VariableDeclaration {
                name: Rc::from(identifier),
                init: Some(expression),
                storage_class: specifiers.1,
                var_type: specifiers.0,
            }))
        } else {
            Ok(self.make_node(VariableDeclaration {
                name: Rc::from(identifier),
                init: None,
                storage_class: specifiers.1,
                var_type: specifiers.0,
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
            if match_and_consume!(self, Token::Symbol(Symbol::CloseParenthesis)) {
                return Ok(Box::new(params));
            }
            expect_token!(self, Token::Symbol(Symbol::Comma))?;
            params.push(self.parse_binary_op(0)?);
        }
    }

    fn parse_primary(&mut self, token: Token) -> Result<ASTNode<Expression>, CompilerError> {
        match token {
            Token::NumberLiteral(value) => {
                self.tokens.pop_front();
                Ok(self.make_node::<Expression>(Constant(value)))
            }
            Token::Symbol(..) => {
                expect_token!(self, Token::Symbol(Symbol::OpenParenthesis))?;
                let expression = if let Some(t) =
                    match_and_consume!(self, Token::Keyword(Keyword::Type(t)) => Some(t))
                {
                    let mut types = vec![t];
                    while let Some(t) =
                        match_and_consume!(self, Token::Keyword(Keyword::Type(t)) => Some(t))
                    {
                        types.push(t);
                    }
                    expect_token!(self, Token::Symbol(Symbol::CloseParenthesis))?;
                    let type_ = self.parse_type_specifier(types)?;
                    let exp = self.parse_unary_or_primary()?;
                    Ok(self.make_node(Expression::Cast(type_, Box::from(exp))))
                } else {
                    let expression = match self.parse_binary_op(0) {
                        Ok(expression) => Ok(expression),
                        Err(err) => return Err(err),
                    };
                    expect_token!(self, Token::Symbol(Symbol::CloseParenthesis))?;
                    expression
                };
                expression
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
        if let Some(token) = match_and_consume!(self, op @ Token::Symbol(Symbol::Unary(_) | Ambiguous(_)) => Some(op))
        {
            match token {
                Token::Symbol(Symbol::Unary(op)) => {
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
                    let expression = self.parse_unary_or_primary()?;
                    return Ok(
                        self.make_node(Unary(UnaryOperator::UnaryAdd, Box::from(expression)))
                    );
                }
                Token::Symbol(Ambiguous(UnaryOrBinaryOp::Subtraction)) => {
                    let expression = self.parse_unary_or_primary()?;
                    return Ok(self.make_node(Unary(UnaryOperator::Negate, Box::from(expression))));
                }
                _ => unreachable!(),
            }
        }

        let primary = self.parse_primary(self.peek_token())?;
        if let Some(op) = match_and_consume!(self,Token::Symbol(Symbol::Unary(
                op @ (UnaryOperator::Increment | UnaryOperator::Decrement),
            )) => Some(op))
        {
            self.parse_increment_decrement(primary, op, false)
        } else {
            Ok(primary)
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
            if match_and_consume!(self, Token::Symbol(Binary(Assign))) {
                // compound assignment
                if is_lvalue_node(&left.kind) {
                    /*
                    Turn x ?= rhs into x = (x ? rhs)
                    */
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
            Token::Keyword(spec @ Keyword::Type(_)) => {
                let mut specifiers = vec![spec];
                self.tokens.pop_front();
                while let Token::Keyword(spec @ (Keyword::Type(_) | Keyword::StorageClass(_))) =
                    self.peek_token()
                {
                    specifiers.push(spec);
                    self.tokens.pop_front();
                }
                let (type_, storage_class) = self.parse_type_and_storage_class(specifiers)?;
                let variable_declaration = self.parse_declaration((type_, storage_class), None)?;
                let declaration =
                    self.make_node(Declaration::VariableDeclaration(variable_declaration.kind));
                Ok(self.make_node(InitDecl(declaration.kind)))
            }
            Token::Symbol(Symbol::Semicolon) => Ok(self.make_node(InitExp(None))),
            _ => {
                let exp = self.parse_binary_op(0)?;
                Ok(self.make_node(InitExp(Some(exp))))
            }
        }
    }

    fn parse_statement(&mut self) -> Result<ASTNode<Statement>, CompilerError> {
        if let Some(keyword) = match_and_consume!(self, Token::Keyword(keyword) => Some(keyword)) {
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
                    let condition = if let Token::Symbol(Symbol::Semicolon) = self.peek_token() {
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
        } else {
            match self.peek_token() {
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
    }

    fn parse_block_item(&mut self) -> Result<ASTNode<BlockItem>, CompilerError> {
        if let Some(spec) = match_and_consume!(self, Token::Keyword(spec @ (Keyword::Type(_) | Keyword::StorageClass(_))) => Some(spec))
        {
            let mut specifiers = vec![spec];
            while let Token::Keyword(spec @ (Keyword::Type(_) | Keyword::StorageClass(_))) =
                self.peek_token()
            {
                self.tokens.pop_front();
                specifiers.push(spec);
            }
            let (type_, storage_class) = self.parse_type_and_storage_class(specifiers)?;
            let out = self.parse_declaration((type_, storage_class), None)?;
            if let Token::Symbol(Symbol::OpenParenthesis) = self.peek_token() {
                return Err(SemanticError(format!(
                    "Inner function declaration of {} at {:?}",
                    out.kind.name, self.line_number
                )));
            }
            self.end_line()?;
            Ok(self.make_node(D(self.make_node(Declaration::VariableDeclaration(out.kind)))))
        } else {
            let statement = self.parse_statement()?;
            Ok(self.make_node(S(Box::from(statement))))
        }
    }

    pub(crate) fn parse_program(&mut self) -> Result<ASTNode<Program>, CompilerError> {
        let mut declarations = Vec::new();

        while !matches!(self.tokens.front().unwrap(), Token::EOF) {
            let declaration = self.parse_top_level()?;
            declarations.push(declaration);
        }

        expect_token!(self, Token::EOF)?;

        Ok(self.make_node(declarations))
    }

    fn peek_token(&self) -> Token {
        self.tokens.front().unwrap().clone()
    }

    fn end_line(&mut self) -> Result<(), CompilerError> {
        if match_and_consume!(self, Token::Symbol(Symbol::Semicolon)) {
            self.line_number = Rc::from((self.line_number.0 + 1, self.line_number.1.clone()));
            Ok(())
        } else {
            Err(SyntaxError(format!(
                "Expected semicolon but got {:?} at {:?}",
                self.peek_token(),
                self.line_number
            )))
        }
    }

    fn make_node<T>(&self, kind: T) -> ASTNode<T> {
        ASTNode {
            line_number: Rc::clone(&self.line_number),
            kind,
            type_: Type::Void, // placeholder
        }
    }

    fn consume_and_pop(&mut self) -> Token {
        self.tokens.pop_front().unwrap()
    }
}
