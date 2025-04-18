use crate::lexer::{BinaryOperator, Number, UnaryOperator};

trait Visitor {}

#[derive(Debug)]
pub enum ASTNodeType {
    ProgramNode {
        function_declaration: Box<ASTNode>,
    },
    FunctionNode {
        identifier: String,
        body: Box<ASTNode>,
    },
    BlockNode {
        body: Vec<Box<ASTNode>>,
    },
    DeclarationNode {
        identifier: String,
        expression: Option<Box<ASTNode>>,
    },
    ReturnNode {
        expression: Option<Box<ASTNode>>,
    },
    UnaryNode {
        op: UnaryOperator,
        expression: Box<ASTNode>,
    },
    BinaryNode {
        op: BinaryOperator,
        left: Box<ASTNode>,
        right: Box<ASTNode>,
    },
    ConstNode {
        value: Number,
    },
    VariableNode {
        identifier: String,
    },
    PrefixNode {
        variable: Box<ASTNode>,
        operator: UnaryOperator,
    },
    PostfixNode {
        variable: Box<ASTNode>,
        operator: UnaryOperator,
    },
    AssignmentNode {
        left: Box<ASTNode>,
        right: Box<ASTNode>,
    },
    ConditionNode {
        condition: Box<ASTNode>,
        if_true: Option<Box<ASTNode>>,
        if_false: Option<Box<ASTNode>>,
    },
    WhileNode {
        condition: Box<ASTNode>,
        body: Option<Box<ASTNode>>,
        label: String,
        is_do_while: bool,
    },
    BreakNode {
        label: String,
    },
    ContinueNode {
        label: String,
    },
    ForNode {
        init: Option<Box<ASTNode>>,
        condition: Option<Box<ASTNode>>,
        increment: Option<Box<ASTNode>>,
        body: Option<Box<ASTNode>>,
        label: String,
    },
}

#[derive(Debug)]
pub struct ASTNode {
    line_number: (i32, String),
    pub(crate) kind: ASTNodeType,
}

impl ASTNode {
    pub fn new(line_number: (i32, String), kind: ASTNodeType) -> ASTNode {
        ASTNode { line_number, kind }
    }
}

pub fn is_lvalue_node(node: &ASTNodeType) -> bool {
    match node {
        ASTNodeType::VariableNode { .. } | ASTNodeType::PrefixNode { .. } => true,
        _ => false,
    }
}

pub fn extract_base_variable(node: &ASTNodeType) -> Option<String> {
    match node {
        ASTNodeType::VariableNode {identifier} => Some(identifier.clone()),
        ASTNodeType::PrefixNode {variable, ..} => extract_base_variable(&variable.kind),
        _ => None,
    }
}
