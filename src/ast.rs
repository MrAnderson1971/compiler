use crate::common::Position;
use crate::lexer::{BinaryOperator, Number, UnaryOperator};
use std::rc::Rc;

#[derive(Debug)]
pub(crate) struct ASTNode<T> {
    pub(crate) line_number: Rc<Position>,
    pub(crate) kind: T,
}

pub(crate) type Program = Vec<ASTNode<FunctionDeclaration>>;

pub(crate) struct FunctionDeclaration {
    pub(crate) identifier: Rc<String>,
    pub(crate) params: Vec<Rc<String>>,
    pub(crate) body: ASTNode<Block>,
}

pub(crate) type Block = Vec<ASTNode<BlockItem>>;

pub(crate) enum BlockItem {
    D(ASTNode<Declaration>),
    S(Box<ASTNode<Statement>>),
}

pub(crate) enum Declaration {
    FunctionDeclaration(ASTNode<FunctionDeclaration>),
    VariableDeclaration(ASTNode<VariableDeclaration>),
}

pub(crate) struct VariableDeclaration {
    pub(crate) identifier: Rc<String>,
    pub(crate) init: Option<ASTNode<Expression>>,
}

#[derive(Debug)]
pub(crate) enum Expression {
    Constant(Number),
    Variable(Rc<String>),
    Unary(UnaryOperator, Box<ASTNode<Expression>>),
    Binary {
        op: BinaryOperator,
        left: Box<ASTNode<Expression>>,
        right: Box<ASTNode<Expression>>,
    },
    Assignment {
        left: Box<ASTNode<Expression>>,
        right: Box<ASTNode<Expression>>,
    },
    Condition {
        condition: Box<ASTNode<Expression>>,
        if_true: Box<ASTNode<Expression>>,
        if_false: Box<ASTNode<Expression>>,
    },
    FunctionCall(Rc<String>, Vec<Box<ASTNode<Expression>>>),
    Prefix(UnaryOperator, Box<ASTNode<Expression>>),
    Postfix(UnaryOperator, Box<ASTNode<Expression>>),
}

pub(crate) enum Statement {
    Return(ASTNode<Expression>),
    Expression(ASTNode<Expression>),
    If {
        condition: ASTNode<Expression>,
        if_true: Box<ASTNode<Statement>>,
        if_false: Option<Box<ASTNode<Statement>>>,
    },
    Compound(ASTNode<Block>),
    Break(Rc<String>),
    Continue {
        label: Rc<String>,
        is_for: bool,
    },
    While {
        condition: ASTNode<Expression>,
        body: Box<ASTNode<Statement>>,
        label: Rc<String>,
        is_do_while: bool,
    },
    For {
        init: ASTNode<ForInit>,
        condition: Option<ASTNode<Expression>>,
        increment: Option<ASTNode<Expression>>,
        body: Box<ASTNode<Statement>>,
        label: Rc<String>,
    },
    Null,
}

pub(crate) enum ForInit {
    InitDecl(ASTNode<VariableDeclaration>),
    InitExp(Option<ASTNode<Expression>>),
}

pub(crate) fn is_lvalue_node(node: &Expression) -> bool {
    match node {
        Expression::Prefix(_, _) | Expression::Variable(_) => true,
        _ => false,
    }
}

pub(crate) fn extract_base_variable(node: &Expression) -> Rc<String> {
    match node {
        Expression::Variable(v) => Rc::clone(v),
        Expression::Prefix(_, v) => extract_base_variable(&v.kind),
        _ => panic!("Not a variable"),
    }
}
