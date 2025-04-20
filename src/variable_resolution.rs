use crate::ast::{ASTNode, Block, Declaration, Expression, ForInit, Program, Statement, Visitor};
use crate::common::{Identifier, Position};
use crate::errors::CompilerError;
use crate::errors::CompilerError::SemanticError;
use crate::lexer::{BinaryOperator, Number, UnaryOperator};
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

pub(crate) struct VariableResolutionVisitor {
    layer: i32,
    function: Rc<String>,
    variable_map: HashMap<String, VecDeque<i32>>,
    loop_labels: VecDeque<(Rc<String>, bool)>,
}

impl VariableResolutionVisitor {
    pub(crate) fn new(function: Rc<String>) -> Self {
        Self {
            layer: 0,
            function,
            variable_map: HashMap::new(),
            loop_labels: VecDeque::new(),
        }
    }
}

/*
*resolve_declaration(Declaration(name, init), variable_map):
1 if name is in variable_map:
fail("Duplicate variable declaration!")
unique_name = make_temporary()
2 variable_map.add(name, unique_name)
3 if init is not null:
init = resolve_exp(init, variable_map)
4 return Declaration(unique_name, init)
*/
impl Visitor for VariableResolutionVisitor {
    fn visit_program(
        &mut self,
        _line_number: &Rc<Position>,
        _function_declaration: &mut Program,
    ) -> Result<(), CompilerError> {
        panic!("Should not be called")
    }

    fn visit_function(
        &mut self,
        _line_number: &Rc<Position>,
        _identifier: &mut Rc<Identifier>,
        _params: &mut Vec<Rc<Identifier>>,
        body: &mut ASTNode<Block>,
    ) -> Result<(), CompilerError> {
        body.accept(self)
    }

    fn visit_declaration(
        &mut self,
        line_number: &Rc<Position>,
        declaration: &mut Declaration,
    ) -> Result<(), CompilerError> {
        match declaration {
            Declaration::VariableDeclaration(d) => {
                let (identifier, expression) = (&mut d.kind.name, &mut d.kind.init);
                if !self
                    .variable_map
                    .contains_key(&(*Rc::clone(&identifier)).clone())
                {
                    let mut stack = VecDeque::new();
                    stack.push_back(self.layer);
                    self.variable_map
                        .insert((*Rc::clone(&identifier)).clone(), stack);
                } else {
                    let stack = self
                        .variable_map
                        .get_mut(&(*Rc::clone(&identifier)).clone())
                        .unwrap();
                    if !stack.is_empty() && *stack.back().unwrap() == self.layer {
                        return Err(SemanticError(format!(
                            "Duplicate variable declaration {} at {:?}",
                            identifier, line_number
                        )));
                    }
                    stack.push_back(self.layer);
                }
                *identifier = Rc::new(format!("{}::{}::{}", self.function, identifier, self.layer));
                if let Some(expression) = expression {
                    expression.accept(self)
                } else {
                    Ok(())
                }
            }
            Declaration::FunctionDeclaration(_) => todo!(),
        }
    }

    fn visit_assignment(
        &mut self,
        _line_number: &Rc<Position>,
        left: &mut Box<ASTNode<Expression>>,
        right: &mut Box<ASTNode<Expression>>,
    ) -> Result<(), CompilerError> {
        left.accept(self)?;
        right.accept(self)
    }

    fn visit_return(
        &mut self,
        _line_number: &Rc<Position>,
        expression: &mut ASTNode<Expression>,
    ) -> Result<(), CompilerError> {
        expression.accept(self)
    }

    fn visit_block(
        &mut self,
        _line_number: &Rc<Position>,
        body: &mut Block,
    ) -> Result<(), CompilerError> {
        self.layer += 1;
        for node in body {
            node.accept(self)?;
        }
        self.pop_stack();
        self.layer -= 1;
        Ok(())
    }

    fn visit_unary(
        &mut self,
        _line_number: &Rc<Position>,
        _op: &mut UnaryOperator,
        expression: &mut Box<ASTNode<Expression>>,
    ) -> Result<(), CompilerError> {
        expression.accept(self)
    }

    fn visit_binary(
        &mut self,
        _line_number: &Rc<Position>,
        _op: &mut BinaryOperator,
        left: &mut Box<ASTNode<Expression>>,
        right: &mut Box<ASTNode<Expression>>,
    ) -> Result<(), CompilerError> {
        left.accept(self)?;
        right.accept(self)
    }

    fn visit_condition(
        &mut self,
        _line_number: &Rc<Position>,
        condition: &mut Box<ASTNode<Expression>>,
        if_true: &mut Box<ASTNode<Expression>>,
        if_false: &mut Box<ASTNode<Expression>>,
    ) -> Result<(), CompilerError> {
        condition.accept(self)?;
        if_true.accept(self)?;
        if_false.accept(self)?;
        Ok(())
    }

    fn visit_while(
        &mut self,
        _line_number: &Rc<Position>,
        condition: &mut ASTNode<Expression>,
        body: &mut Box<ASTNode<Statement>>,
        label: &mut Rc<String>,
        _is_do_while: &mut bool,
    ) -> Result<(), CompilerError> {
        self.loop_labels.push_back((Rc::clone(&label), false));
        condition.accept(self)?;
        body.accept(self)?;
        self.loop_labels.pop_back();
        Ok(())
    }

    fn visit_break(
        &mut self,
        line_number: &Rc<Position>,
        label: &mut Rc<String>,
    ) -> Result<(), CompilerError> {
        if self.loop_labels.is_empty() {
            Err(SemanticError(format!(
                "Break outside loop at {:?}",
                line_number
            )))
        } else {
            *label = Rc::clone(&self.loop_labels.back().unwrap().0);
            Ok(())
        }
    }

    fn visit_continue(
        &mut self,
        line_number: &Rc<Position>,
        label: &mut Rc<String>,
        is_for: &mut bool,
    ) -> Result<(), CompilerError> {
        if self.loop_labels.is_empty() {
            Err(SemanticError(format!(
                "Continue outside loop at {:?}",
                line_number
            )))
        } else {
            *label = Rc::clone(&self.loop_labels.back().unwrap().0);
            *is_for = self.loop_labels.back().unwrap().1;
            Ok(())
        }
    }

    fn visit_for(
        &mut self,
        _line_number: &Rc<Position>,
        init: &mut ASTNode<ForInit>,
        condition: &mut Option<ASTNode<Expression>>,
        increment: &mut Option<ASTNode<Expression>>,
        body: &mut Box<ASTNode<Statement>>,
        label: &mut Rc<String>,
    ) -> Result<(), CompilerError> {
        if !matches!(init.kind, ForInit::InitExp(None)) {
            // the init adds a scope
            self.layer += 1;
            init.accept(self)?;
        }
        self.loop_labels.push_back((Rc::clone(&label), true));
        if let Some(condition) = condition {
            condition.accept(self)?;
        }
        if let Some(increment) = increment {
            increment.accept(self)?;
        }
        body.accept(self)?;

        self.loop_labels.pop_back();
        if !matches!(init.kind, ForInit::InitExp(None)) {
            self.pop_stack();
            self.layer -= 1;
        }
        Ok(())
    }

    fn visit_const(
        &mut self,
        _line_number: &Rc<Position>,
        _value: &mut Number,
    ) -> Result<(), CompilerError> {
        Ok(())
    }

    fn visit_variable(
        &mut self,
        line_number: &Rc<Position>,
        identifier: &mut Rc<String>,
    ) -> Result<(), CompilerError> {
        match self
            .variable_map
            .get_mut(&(*Rc::clone(&identifier)).clone())
        {
            None => Err(SemanticError(format!(
                "Undefined variable {} at {:?}",
                identifier, line_number
            ))),
            Some(stack) => {
                if stack.is_empty() {
                    Err(SemanticError(format!(
                        "Variable {} at {:?} out of scope",
                        identifier, line_number
                    )))
                } else {
                    let variable = stack.back().unwrap();
                    *identifier =
                        Rc::new(format!("{}::{}::{}", self.function, identifier, variable));
                    Ok(())
                }
            }
        }
    }

    fn visit_prefix(
        &mut self,
        _line_number: &Rc<Position>,
        variable: &mut Box<ASTNode<Expression>>,
        _operator: &mut UnaryOperator,
    ) -> Result<(), CompilerError> {
        variable.accept(self)
    }

    fn visit_postfix(
        &mut self,
        _line_number: &Rc<Position>,
        variable: &mut Box<ASTNode<Expression>>,
        _operator: &mut UnaryOperator,
    ) -> Result<(), CompilerError> {
        variable.accept(self)
    }

    fn visit_if_else(
        &mut self,
        _line_number: &Rc<Position>,
        expression: &mut ASTNode<Expression>,
        if_true: &mut Box<ASTNode<Statement>>,
        if_false: &mut Option<Box<ASTNode<Statement>>>,
    ) -> Result<(), CompilerError> {
        expression.accept(self)?;
        if let Some(if_false) = if_false {
            if_false.accept(self)?;
        }
        if_true.accept(self)
    }
}

impl VariableResolutionVisitor {
    fn pop_stack(&mut self) {
        for stack in self.variable_map.values_mut() {
            if !stack.is_empty() && stack.back().unwrap() == &self.layer {
                stack.pop_back();
            }
        }
    }
}
