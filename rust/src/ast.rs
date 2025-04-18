use std::rc::Rc;
use crate::common::Position;
use crate::errors::CompilerError;
use crate::lexer::{BinaryOperator, Number, UnaryOperator};
use crate::variable_resolution::VariableResolutionVisitor;

pub trait Visitor {
    fn visit_program(
        &mut self,
        line_number: &Rc<Position>,
        function_declaration: &mut Box<ASTNode>,
    ) -> Result<(), CompilerError>;
    fn visit_function(
        &mut self,
        line_number: &Rc<Position>,
        identifier: &mut Rc<String>,
        body: &mut Option<Box<ASTNode>>,
    ) -> Result<(), CompilerError>;
    fn visit_declaration(
        &mut self,
        line_number: &Rc<Position>,
        identifier: &mut Rc<String>,
        expression: &mut Option<Box<ASTNode>>,
    ) -> Result<(), CompilerError>;
    fn visit_assignment(
        &mut self,
        line_number: &Rc<Position>,
        left: &mut Box<ASTNode>,
        right: &mut Box<ASTNode>,
    ) -> Result<(), CompilerError>;
    fn visit_return(
        &mut self,
        line_number: &Rc<Position>,
        expression: &mut Option<Box<ASTNode>>,
    ) -> Result<(), CompilerError>;
    fn visit_block(
        &mut self,
        line_number: &Rc<Position>,
        body: &mut Vec<Box<ASTNode>>,
    ) -> Result<(), CompilerError>;
    fn visit_unary(
        &mut self,
        line_number: &Rc<Position>,
        op: &mut UnaryOperator,
        expression: &mut Box<ASTNode>,
    ) -> Result<(), CompilerError>;
    fn visit_binary(
        &mut self,
        line_number: &Rc<Position>,
        op: &mut BinaryOperator,
        left: &mut Box<ASTNode>,
        right: &mut Box<ASTNode>,
    ) -> Result<(), CompilerError>;
    fn visit_condition(
        &mut self,
        line_number: &Rc<Position>,
        condition: &mut Box<ASTNode>,
        if_true: &mut Option<Box<ASTNode>>,
        if_false: &mut Option<Box<ASTNode>>,
    ) -> Result<(), CompilerError>;
    fn visit_while(
        &mut self,
        line_number: &Rc<Position>,
        condition: &mut Box<ASTNode>,
        body: &mut Option<Box<ASTNode>>,
        label: &mut Rc<String>,
        is_do_while: &mut bool,
    ) -> Result<(), CompilerError>;
    fn visit_break(
        &mut self,
        line_number: &Rc<Position>,
        label: &mut Rc<String>,
    ) -> Result<(), CompilerError>;
    fn visit_continue(
        &mut self,
        line_number: &Rc<Position>,
        label: &mut Rc<String>,
        is_for: &mut bool,
    ) -> Result<(), CompilerError>;
    fn visit_for(
        &mut self,
        line_number: &Rc<Position>,
        init: &mut Option<Box<ASTNode>>,
        condition: &mut Option<Box<ASTNode>>,
        increment: &mut Option<Box<ASTNode>>,
        body: &mut Option<Box<ASTNode>>,
        label: &mut Rc<String>,
    ) -> Result<(), CompilerError>;
    fn visit_const(
        &mut self,
        line_number: &Rc<Position>,
        value: &mut Number,
    ) -> Result<(), CompilerError>;
    fn visit_variable(
        &mut self,
        line_number: &Rc<Position>,
        identifier: &mut Rc<String>,
    ) -> Result<(), CompilerError>;
    fn visit_prefix(
        &mut self,
        line_number: &Rc<Position>,
        variable: &mut Box<ASTNode>,
        operator: &mut UnaryOperator,
    ) -> Result<(), CompilerError>;
    fn visit_postfix(
        &mut self,
        line_number: &Rc<Position>,
        variable: &mut Box<ASTNode>,
        operator: &mut UnaryOperator,
    ) -> Result<(), CompilerError>;
}

#[derive(Debug)]
pub enum ASTNodeType {
    ProgramNode {
        function_declaration: Box<ASTNode>,
    },
    FunctionNode {
        identifier: Rc<String>,
        body: Option<Box<ASTNode>>,
    },
    BlockNode {
        body: Vec<Box<ASTNode>>,
    },
    DeclarationNode {
        identifier: Rc<String>,
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
        identifier: Rc<String>,
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
        label: Rc<String>,
        is_do_while: bool,
    },
    BreakNode {
        label: Rc<String>,
    },
    ContinueNode {
        label: Rc<String>,
        is_for: bool,
    },
    ForNode {
        init: Option<Box<ASTNode>>,
        condition: Option<Box<ASTNode>>,
        increment: Option<Box<ASTNode>>,
        body: Option<Box<ASTNode>>,
        label: Rc<String>,
    },
}

#[derive(Debug)]
pub struct ASTNode {
    line_number: Rc<Position>,
    pub(crate) kind: ASTNodeType,
}

impl ASTNode {
    pub fn new(line_number: Rc<Position>, kind: ASTNodeType) -> ASTNode {
        ASTNode { line_number, kind }
    }

    pub fn accept(&mut self, visitor: &mut dyn Visitor) -> Result<(), CompilerError> {
        match &mut self.kind {
            ASTNodeType::ProgramNode {
                function_declaration,
            } => visitor.visit_program(&self.line_number, function_declaration),
            ASTNodeType::FunctionNode { identifier, body } => {
                visitor.visit_function(&self.line_number, identifier, body)
            }
            ASTNodeType::BlockNode { body } => visitor.visit_block(&self.line_number, body),
            ASTNodeType::DeclarationNode {
                identifier,
                expression,
            } => visitor.visit_declaration(&self.line_number, identifier, expression),
            ASTNodeType::ReturnNode { expression } => {
                visitor.visit_return(&self.line_number, expression)
            }
            ASTNodeType::UnaryNode { op, expression } => {
                visitor.visit_unary(&self.line_number, op, expression)
            }
            ASTNodeType::BinaryNode { op, left, right } => {
                visitor.visit_binary(&self.line_number, op, left, right)
            }
            ASTNodeType::ConditionNode {
                condition,
                if_true,
                if_false,
            } => visitor.visit_condition(&self.line_number, condition, if_true, if_false),
            ASTNodeType::WhileNode {
                condition,
                body,
                label,
                is_do_while,
            } => visitor.visit_while(&self.line_number, condition, body, label, is_do_while),
            ASTNodeType::BreakNode { label } => visitor.visit_break(&self.line_number, label),
            ASTNodeType::ContinueNode { label, is_for } => {
                visitor.visit_continue(&self.line_number, label, is_for)
            }
            ASTNodeType::ForNode {
                init,
                condition,
                increment,
                body,
                label,
            } => visitor.visit_for(&self.line_number, init, condition, increment, body, label),
            ASTNodeType::ConstNode { value } => visitor.visit_const(&self.line_number, value),
            ASTNodeType::VariableNode { identifier } => {
                visitor.visit_variable(&self.line_number, identifier)
            }
            ASTNodeType::PrefixNode { variable, operator } => {
                visitor.visit_prefix(&self.line_number, variable, operator)
            }
            ASTNodeType::AssignmentNode { left, right } => {
                visitor.visit_assignment(&self.line_number, left, right)
            }
            ASTNodeType::PostfixNode { variable, operator } => {
                visitor.visit_postfix(&self.line_number, variable, operator)
            }
        }
    }

    pub fn generate(&mut self, out: &mut String) -> Result<(), CompilerError> {
        match &mut self.kind {
            ASTNodeType::ProgramNode { function_declaration } => {
                function_declaration.generate(out)
            }
            ASTNodeType::FunctionNode { identifier, body } => {
                let mut variable_resolution_visitor = VariableResolutionVisitor::new(Rc::clone(identifier));
                if let Some(body) = body.as_mut() {
                    body.accept(&mut variable_resolution_visitor as &mut dyn Visitor)
                } else {
                    Ok(())
                }
                // let mut function_body = FunctionBody::new(identifier.clone());

            }
            _ => Ok(())
        }
    }
}

pub fn is_lvalue_node(node: &ASTNodeType) -> bool {
    match node {
        ASTNodeType::VariableNode { .. } | ASTNodeType::PrefixNode { .. } => true,
        _ => false,
    }
}

pub fn extract_base_variable(node: &ASTNodeType) -> Option<Rc<String>> {
    match node {
        ASTNodeType::VariableNode { identifier } => Some(identifier.clone()),
        ASTNodeType::PrefixNode { variable, .. } => extract_base_variable(&variable.kind),
        _ => None,
    }
}
