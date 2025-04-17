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
    PrefixNode {
        variable: Box<ASTNode>,
        operator: BinaryOperator,
    },
    PostfixNode {
        variable: Box<ASTNode>,
        operator: BinaryOperator,
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
        is_do_while: bool
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
