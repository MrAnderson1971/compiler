use crate::CompilerError;
use crate::common::{Identifier, Position};
use crate::lexer::{BinaryOperator, Number, UnaryOperator};
use crate::tac::FunctionBody;
use crate::tac_visitor::TacVisitor;
use crate::variable_resolution::VariableResolutionVisitor;
use std::ops::DerefMut;
use std::rc::Rc;

pub(crate) trait Visitor {
    fn visit_program(
        &mut self,
        line_number: &Rc<Position>,
        function_declaration: &mut Program,
    ) -> Result<(), CompilerError>;
    fn visit_function(
        &mut self,
        line_number: &Rc<Position>,
        identifier: &mut Rc<Identifier>,
        params: &mut Vec<Rc<Identifier>>,
        body: &mut ASTNode<Block>,
    ) -> Result<(), CompilerError>;
    fn visit_declaration(
        &mut self,
        line_number: &Rc<Position>,
        declaration: &mut Declaration,
    ) -> Result<(), CompilerError>;
    fn visit_assignment(
        &mut self,
        line_number: &Rc<Position>,
        left: &mut Box<ASTNode<Expression>>,
        right: &mut Box<ASTNode<Expression>>,
    ) -> Result<(), CompilerError>;
    fn visit_return(
        &mut self,
        line_number: &Rc<Position>,
        expression: &mut ASTNode<Expression>,
    ) -> Result<(), CompilerError>;
    fn visit_block(
        &mut self,
        line_number: &Rc<Position>,
        body: &mut Block,
    ) -> Result<(), CompilerError>;
    fn visit_unary(
        &mut self,
        line_number: &Rc<Position>,
        op: &mut UnaryOperator,
        expression: &mut Box<ASTNode<Expression>>,
    ) -> Result<(), CompilerError>;
    fn visit_binary(
        &mut self,
        line_number: &Rc<Position>,
        op: &mut BinaryOperator,
        left: &mut Box<ASTNode<Expression>>,
        right: &mut Box<ASTNode<Expression>>,
    ) -> Result<(), CompilerError>;
    fn visit_condition(
        &mut self,
        line_number: &Rc<Position>,
        condition: &mut Box<ASTNode<Expression>>,
        if_true: &mut Box<ASTNode<Expression>>,
        if_false: &mut Box<ASTNode<Expression>>,
    ) -> Result<(), CompilerError>;
    fn visit_while(
        &mut self,
        line_number: &Rc<Position>,
        condition: &mut ASTNode<Expression>,
        body: &mut Box<ASTNode<Statement>>,
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
        init: &mut ASTNode<ForInit>,
        condition: &mut Option<ASTNode<Expression>>,
        increment: &mut Option<ASTNode<Expression>>,
        body: &mut Box<ASTNode<Statement>>,
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
        identifier: &mut Rc<Identifier>,
    ) -> Result<(), CompilerError>;
    fn visit_prefix(
        &mut self,
        line_number: &Rc<Position>,
        variable: &mut Box<ASTNode<Expression>>,
        operator: &mut UnaryOperator,
    ) -> Result<(), CompilerError>;
    fn visit_postfix(
        &mut self,
        line_number: &Rc<Position>,
        variable: &mut Box<ASTNode<Expression>>,
        operator: &mut UnaryOperator,
    ) -> Result<(), CompilerError>;
    fn visit_if_else(
        &mut self,
        line_number: &Rc<Position>,
        expression: &mut ASTNode<Expression>,
        if_true: &mut Box<ASTNode<Statement>>,
        if_false: &mut Option<Box<ASTNode<Statement>>>,
    ) -> Result<(), CompilerError>;
}

impl ASTNode<Program> {
    fn accept(&mut self, visitor: &mut dyn Visitor) -> Result<(), CompilerError> {
        for function_declaration in &mut self.kind {
            function_declaration.accept(visitor)?;
        }
        Ok(())
    }

    pub(crate) fn generate(&mut self, out: &mut String) -> Result<(), CompilerError> {
        for function_declaration in &mut self.kind {
            function_declaration.generate(out)?;
        }
        Ok(())
    }
}

impl ASTNode<FunctionDeclaration> {
    fn accept(&mut self, visitor: &mut dyn Visitor) -> Result<(), CompilerError> {
        let f = &mut self.kind;
        visitor.visit_function(&self.line_number, &mut f.name, &mut f.params, &mut f.body)
    }

    pub(crate) fn generate(&mut self, out: &mut String) -> Result<(), CompilerError> {
        let identifier = Rc::clone(&self.kind.name);
        let mut variable_resolution_visitor =
            VariableResolutionVisitor::new(Rc::clone(&identifier));
        self.accept(&mut variable_resolution_visitor as &mut dyn Visitor)?;

        let mut function_body = FunctionBody::new();

        let mut tac_visitor = TacVisitor::new(Rc::clone(&identifier), &mut function_body);
        self.accept(&mut tac_visitor as &mut dyn Visitor)?;

        // Default return statement in the main method
        if identifier.as_str() == "main" {
            function_body.add_default_return_to_main();
        }

        println!("{:#?}", function_body);

        for instruction in &function_body.instructions {
            instruction.make_assembly(out, &function_body);
        }
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct ASTNode<T> {
    pub(crate) line_number: Rc<Position>,
    pub(crate) kind: T,
}

pub(crate) type Program = Vec<ASTNode<FunctionDeclaration>>;

#[derive(Debug)]
pub(crate) struct FunctionDeclaration {
    pub(crate) name: Rc<Identifier>,
    pub(crate) params: Vec<Rc<Identifier>>,
    pub(crate) body: ASTNode<Block>,
}

pub(crate) type Block = Vec<ASTNode<BlockItem>>;

impl ASTNode<Block> {
    pub(crate) fn accept(&mut self, visitor: &mut dyn Visitor) -> Result<(), CompilerError> {
        for block_item in &mut self.kind {
            block_item.accept(visitor)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) enum BlockItem {
    D(ASTNode<Declaration>),
    S(Box<ASTNode<Statement>>),
}

impl ASTNode<BlockItem> {
    pub(crate) fn accept(&mut self, visitor: &mut dyn Visitor) -> Result<(), CompilerError> {
        match &mut self.kind {
            BlockItem::D(declaration) => declaration.accept(visitor),
            BlockItem::S(statement) => statement.deref_mut().accept(visitor),
        }
    }
}

#[derive(Debug)]
pub(crate) enum Declaration {
    FunctionDeclaration(ASTNode<FunctionDeclaration>),
    VariableDeclaration(ASTNode<VariableDeclaration>),
}

impl ASTNode<Declaration> {
    fn accept(&mut self, visitor: &mut dyn Visitor) -> Result<(), CompilerError> {
        visitor.visit_declaration(&self.line_number, &mut self.kind)
    }
}

#[derive(Debug)]
pub(crate) struct VariableDeclaration {
    pub(crate) name: Rc<Identifier>,
    pub(crate) init: Option<ASTNode<Expression>>,
}

#[derive(Debug)]
pub(crate) enum Expression {
    Constant(Number),
    Variable(Rc<Identifier>),
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
    FunctionCall(Rc<Identifier>, Box<Vec<ASTNode<Expression>>>),
    Prefix(UnaryOperator, Box<ASTNode<Expression>>),
    Postfix(UnaryOperator, Box<ASTNode<Expression>>),
}

impl ASTNode<Expression> {
    pub(crate) fn accept(&mut self, visitor: &mut dyn Visitor) -> Result<(), CompilerError> {
        match &mut self.kind {
            Expression::Constant(value) => visitor.visit_const(&self.line_number, value),
            Expression::Variable(v) => visitor.visit_variable(&self.line_number, v),
            Expression::Unary(op, exp) => visitor.visit_unary(&self.line_number, op, exp),
            Expression::Binary { op, left, right } => {
                visitor.visit_binary(&self.line_number, op, left, right)
            }
            Expression::Assignment { left, right } => {
                visitor.visit_assignment(&self.line_number, left, right)
            }
            Expression::Condition {
                condition,
                if_true,
                if_false,
            } => visitor.visit_condition(&self.line_number, condition, if_true, if_false),
            Expression::FunctionCall(..) => todo!(),
            Expression::Prefix(op, exp) => visitor.visit_prefix(&self.line_number, exp, op),
            Expression::Postfix(op, exp) => visitor.visit_postfix(&self.line_number, exp, op),
        }
    }
}

#[derive(Debug)]
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

impl ASTNode<Statement> {
    pub(crate) fn accept(&mut self, visitor: &mut dyn Visitor) -> Result<(), CompilerError> {
        match &mut self.kind {
            Statement::Return(val) => visitor.visit_return(&self.line_number, val),
            Statement::Expression(exp) => exp.accept(visitor),
            Statement::If {
                condition,
                if_true,
                if_false,
            } => visitor.visit_if_else(&self.line_number, condition, if_true, if_false),
            Statement::Compound(block) => visitor.visit_block(&self.line_number, &mut block.kind),
            Statement::Break(label) => visitor.visit_break(&self.line_number, label),
            Statement::Continue { label, is_for } => {
                visitor.visit_continue(&self.line_number, label, is_for)
            }
            Statement::While {
                condition,
                body,
                label,
                is_do_while,
            } => visitor.visit_while(&self.line_number, condition, body, label, is_do_while),
            Statement::For {
                init,
                condition,
                increment,
                body,
                label,
            } => visitor.visit_for(&self.line_number, init, condition, increment, body, label),
            Statement::Null => Ok(()),
        }
    }
}

#[derive(Debug)]
pub(crate) enum ForInit {
    InitDecl(ASTNode<Declaration>),
    InitExp(Option<ASTNode<Expression>>),
}

impl ASTNode<ForInit> {
    pub(crate) fn accept(&mut self, visitor: &mut dyn Visitor) -> Result<(), CompilerError> {
        match &mut self.kind {
            ForInit::InitDecl(v) => visitor.visit_declaration(&self.line_number, &mut v.kind),
            ForInit::InitExp(v) => match v {
                Some(e) => e.accept(visitor),
                None => Ok(()),
            },
        }
    }
}

pub(crate) fn is_lvalue_node(node: &Expression) -> bool {
    match node {
        Expression::Prefix(_, _) | Expression::Variable(_) => true,
        _ => false,
    }
}

pub(crate) fn extract_base_variable(node: &Expression) -> Rc<Identifier> {
    match node {
        Expression::Variable(v) => Rc::clone(v),
        Expression::Prefix(_, v) => extract_base_variable(&v.kind),
        _ => panic!("Not a variable"),
    }
}
