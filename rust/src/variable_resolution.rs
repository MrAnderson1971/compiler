use crate::ast::{ASTNode, Visitor};
use crate::errors::CompilerError;
use crate::errors::CompilerError::SemanticError;
use crate::lexer::{BinaryOperator, Number, UnaryOperator};
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;
use crate::common::Position;

pub struct VariableResolutionVisitor {
    layer: i32,
    function: Rc<String>,
    variable_map: HashMap<Rc<String>, VecDeque<i32>>,
    loop_labels: VecDeque<(Rc<String>, bool)>,
}

impl VariableResolutionVisitor {
    pub fn new(function: Rc<String>) -> Self {
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
        _function_declaration: &mut Box<ASTNode>,
    ) -> Result<(), CompilerError> {
        panic!("Should not be called")
    }

    fn visit_function(
        &mut self,
        _line_number: &Rc<Position>,
        _identifier: &mut Rc<String>,
        body: &mut Option<Box<ASTNode>>,
    ) -> Result<(), CompilerError> {
        if let Some(body) = body {
            body.accept(self)
        } else {
            Ok(())
        }
    }

    fn visit_declaration(
        &mut self,
        line_number: &Rc<Position>,
        identifier: &mut Rc<String>,
        expression: &mut Option<Box<ASTNode>>,
    ) -> Result<(), CompilerError> {
        if !self.variable_map.contains_key(identifier) {
            let mut stack = VecDeque::new();
            stack.push_back(self.layer);
            self.variable_map.insert(identifier.clone(), stack);
        } else {
            let stack = self.variable_map.get_mut(identifier).unwrap();
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

    fn visit_assignment(
        &mut self,
        _line_number: &Rc<Position>,
        left: &mut Box<ASTNode>,
        right: &mut Box<ASTNode>,
    ) -> Result<(), CompilerError> {
        left.accept(self)?;
        right.accept(self)
    }

    fn visit_return(
        &mut self,
        _line_number: &Rc<Position>,
        expression: &mut Option<Box<ASTNode>>,
    ) -> Result<(), CompilerError> {
        if let Some(expression) = expression {
            expression.accept(self)
        } else {
            Ok(())
        }
    }

    fn visit_block(
        &mut self,
        _line_number: &Rc<Position>,
        body: &mut Vec<Box<ASTNode>>,
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
        expression: &mut Box<ASTNode>,
    ) -> Result<(), CompilerError> {
        expression.accept(self)
    }

    fn visit_binary(
        &mut self,
        _line_number: &Rc<Position>,
        _op: &mut BinaryOperator,
        left: &mut Box<ASTNode>,
        right: &mut Box<ASTNode>,
    ) -> Result<(), CompilerError> {
        left.accept(self)?;
        right.accept(self)
    }

    fn visit_condition(
        &mut self,
        _line_number: &Rc<Position>,
        condition: &mut Box<ASTNode>,
        if_true: &mut Option<Box<ASTNode>>,
        if_false: &mut Option<Box<ASTNode>>,
    ) -> Result<(), CompilerError> {
        condition.accept(self)?;
        if let Some(if_true) = if_true {
            if_true.accept(self)?;
        }
        if let Some(if_false) = if_false {
            if_false.accept(self)?;
        }
        Ok(())
    }

    fn visit_while(
        &mut self,
        _line_number: &Rc<Position>,
        condition: &mut Box<ASTNode>,
        body: &mut Option<Box<ASTNode>>,
        _label: &mut Rc<String>,
        _is_do_while: &mut bool,
    ) -> Result<(), CompilerError> {
        condition.accept(self)?;
        if let Some(body) = body {
            body.accept(self)?;
        }
        Ok(())
    }

    fn visit_break(
        &mut self,
        line_number: &Rc<Position>,
        label: &mut Rc<String>,
    ) -> Result<(), CompilerError> {
        if self.loop_labels.is_empty() {
            Err(SemanticError(format!("Break outside loop at {:?}", line_number)))
        } else {
            *label = self.loop_labels.back().unwrap().0.clone();
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
            Err(SemanticError(format!("Continue outside loop at {:?}", line_number)))
        } else {
            *label = self.loop_labels.back().unwrap().0.clone();
            *is_for = self.loop_labels.back().unwrap().1;
            Ok(())
        }
    }

    fn visit_for(
        &mut self,
        _line_number: &Rc<Position>,
        init: &mut Option<Box<ASTNode>>,
        condition: &mut Option<Box<ASTNode>>,
        increment: &mut Option<Box<ASTNode>>,
        body: &mut Option<Box<ASTNode>>,
        label: &mut Rc<String>,
    ) -> Result<(), CompilerError> {
        if let Some(init) = init { // the init adds a scope
            init.accept(self)?;
        }
        self.loop_labels.push_back((label.clone(), true));
        if let Some(condition) = condition {
            condition.accept(self)?;
        }
        if let Some(increment) = increment {
            increment.accept(self)?;
        }
        if let Some(body) = body {
            body.accept(self)?;
        }
        self.loop_labels.pop_back();
        if let Some(..) = init {
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
        match self.variable_map.get_mut(identifier) {
            None => Err(SemanticError(format!("Undefined variable {} at {:?}", identifier, line_number))),
            Some(stack) => {
                if stack.is_empty() {
                    Err(SemanticError(format!("Variable {} at {:?} out of scope", identifier, line_number)))
                } else {
                    let variable = stack.back().unwrap();
                    *identifier = Rc::new(format!("{}::{}::{}", self.function, identifier, variable));
                    Ok(())
                }
            }
        }
    }

    fn visit_prefix(
        &mut self,
        _line_number: &Rc<Position>,
        variable: &mut Box<ASTNode>,
        _operator: &mut UnaryOperator,
    ) -> Result<(), CompilerError> {
        variable.accept(self)
    }

    fn visit_postfix(
        &mut self,
        _line_number: &Rc<Position>,
        variable: &mut Box<ASTNode>,
        _operator: &mut UnaryOperator,
    ) -> Result<(), CompilerError> {
        variable.accept(self)
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
