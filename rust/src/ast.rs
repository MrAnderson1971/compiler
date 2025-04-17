use crate::lexer::{BinaryOperator, Number, UnaryOperator};

trait Visitor {}

#[derive(Debug)]
enum ASTNodeType {
    ProgramNode{function_declaration: Option<Box<ASTNode>>},
    FunctionNode{identifier: String, body: Option<Box<ASTNode>>},
    BlockNode{body: Vec<Box<ASTNode>>},
    DeclarationNode{identifier: String, expression: Option<Box<ASTNode>>},
    ReturnNode{expression: Option<Box<ASTNode>>},
    UnaryNode{op: UnaryOperator, expression: Box<ASTNode>},
    BinaryNode{op: BinaryOperator, left: Box<ASTNode>, right: Box<ASTNode>},
    ConstNode{value: Number},
    PrefixNode{variable: Box<ASTNode>, operator: BinaryOperator},
    PostfixNode{variable: Box<ASTNode>, operator: BinaryOperator},
    AssignmentNode{left: Box<ASTNode>, right: Box<ASTNode>},
    ConditionNode{condition: Box<ASTNode>, if_true: Box<ASTNode>, if_false: Option<Box<ASTNode>>},
    WhileNode{condition: Box<ASTNode>, body: Option<Box<ASTNode>>, label: String},
    BreakNode{label: String},
    ContinueNode{label: String},
    ForNode{init: Option<Box<ASTNode>>, condition: Option<Box<ASTNode>>,increment: Option<Box<ASTNode>>, label: String},
}

#[derive(Debug)]
enum ASTNode {
    LineNumber(i32, String),
    Kind(ASTNodeType),
}
