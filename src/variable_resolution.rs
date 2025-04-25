use crate::ast::IdentifierAttr::StaticAttr;
use crate::ast::{
    ASTNode, Block, Declaration, Expression, ForInit, FunAttr, IdentifierAttr, InitialValue,
    Statement, Visitor,
};
use crate::common::{Identifier, Position};
use crate::errors::CompilerError;
use crate::errors::CompilerError::SemanticError;
use crate::lexer::{BinaryOperator, Number, StorageClass, Type, UnaryOperator};
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

pub(crate) struct VariableResolutionVisitor<'map> {
    layer: i32,
    function: Rc<String>,
    variable_map: HashMap<Identifier, VecDeque<(i32, bool)>>, // (layer, has_linkage)
    loop_labels: VecDeque<(Rc<String>, bool)>,
    functions_map: &'map HashMap<Identifier, FunAttr>, // (has_body, is_global)
    global_variables_map: &'map mut HashMap<Identifier, IdentifierAttr>,
}

impl<'map> VariableResolutionVisitor<'map> {
    pub(crate) fn new(
        function: Rc<String>,
        functions_map: &'map HashMap<Identifier, FunAttr>,
        global_variables_map: &'map mut HashMap<Identifier, IdentifierAttr>,
    ) -> Self {
        Self {
            layer: 0,
            function,
            variable_map: HashMap::new(),
            loop_labels: VecDeque::new(),
            functions_map,
            global_variables_map,
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
impl<'map> Visitor for VariableResolutionVisitor<'map> {
    fn visit_declaration(
        &mut self,
        line_number: &Rc<Position>,
        declaration: &mut Declaration,
    ) -> Result<(), CompilerError> {
        match declaration {
            Declaration::VariableDeclaration(d) => {
                let (identifier, expression) = (&mut d.kind.name, &mut d.kind.init);
                let key = (*Rc::clone(&identifier)).clone();
                if self.functions_map.contains_key(&key) {
                    return Err(SemanticError(format!(
                        "Function {} redeclared as variable at {:?}",
                        identifier, line_number
                    )));
                }
                match d.kind.storage_class {
                    Some(StorageClass::Extern) => {
                        if !matches!(expression, None) {
                            Err(SemanticError(format!(
                                "Extern variable cannot be initialized at {:?}",
                                line_number
                            )))
                        } else {
                            self.global_variables_map.insert(
                                key,
                                StaticAttr {
                                    init: InitialValue::NoInitializer,
                                    global: true,
                                    type_: Type::Int,
                                },
                            );
                            Ok(())
                        }
                    }
                    Some(StorageClass::Static) => {
                        let initial_value = if let Some(init) = &d.kind.init {
                            if let Expression::Constant(i) = init.kind {
                                InitialValue::Initial(i)
                            } else {
                                return Err(SemanticError(format!(
                                    "Non-constant initializer of static variable {} at {:?}",
                                    identifier, line_number
                                )));
                            }
                        } else {
                            InitialValue::Initial(0)
                        };
                        self.global_variables_map.insert(
                            key,
                            StaticAttr {
                                init: initial_value,
                                global: false,
                                type_: Type::Int,
                            },
                        );
                        Ok(())
                    }
                    None => {
                        if !self
                            .variable_map
                            .contains_key(&key)
                        {
                            let mut stack = VecDeque::new();
                            stack.push_back((
                                self.layer,
                                d.kind.storage_class == Some(StorageClass::Extern),
                            ));
                            self.variable_map
                                .insert(key, stack);
                        } else {
                            let stack = self
                                .variable_map
                                .get_mut(&key)
                                .unwrap();
                            if !stack.is_empty() && (*stack.back().unwrap()).0 == self.layer {
                                return Err(SemanticError(format!(
                                    "Duplicate variable declaration {} at {:?}",
                                    identifier, line_number
                                )));
                            }
                            stack.push_back((
                                self.layer,
                                d.kind.storage_class == Some(StorageClass::Extern),
                            ));
                        }
                        *identifier =
                            Rc::new(format!("{}::{}::{}", self.function, identifier, self.layer));
                        if let Some(expression) = expression {
                            expression.accept(self)
                        } else {
                            Ok(())
                        }
                    }
                }
            }
            Declaration::FunctionDeclaration(f) => {
                self.layer += 1;
                for param in &mut f.kind.params {
                    let original_name = param.clone(); // Store original name
                    *param = format!("{}::{}::{}", self.function, param, self.layer);
                    let mut stack = VecDeque::new();
                    stack.push_back((self.layer, false));
                    self.variable_map.insert(original_name, stack); // Use original name as key
                }
                if let Some(body) = &mut f.kind.body {
                    body.accept(self)?;
                }
                self.variable_map.clear();
                self.layer -= 1;
                Ok(())
            }
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
                        Rc::new(format!("{}::{}::{}", self.function, identifier, variable.0));
                    Ok(())
                }
            }
        }
    }

    fn visit_function_call(
        &mut self,
        line_number: &Rc<Position>,
        identifier: &mut Rc<Identifier>,
        arguments: &mut Box<Vec<ASTNode<Expression>>>,
    ) -> Result<(), CompilerError> {
        if let Some(func) = self.functions_map.get(&(*Rc::clone(&identifier)).clone()) {
            if arguments.len() != func.param_count {
                return Err(SemanticError(format!(
                    "Function {} called with {} parameters but expected {} at {:?}",
                    identifier,
                    arguments.len(),
                    func.param_count,
                    line_number
                )));
            }
            for arg in (*arguments).iter_mut() {
                arg.accept(self)?;
            }
            Ok(())
        } else {
            Err(SemanticError(format!(
                "Undefined function {} called at {:?}",
                identifier, line_number
            )))
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

impl<'map> VariableResolutionVisitor<'map> {
    fn pop_stack(&mut self) {
        for stack in self.variable_map.values_mut() {
            if !stack.is_empty() && stack.back().unwrap().0 == self.layer {
                stack.pop_back();
            }
        }
    }
}
