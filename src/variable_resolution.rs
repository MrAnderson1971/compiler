use crate::ast::{
    ASTNode, Block, Declaration, Expression, ForInit, FunAttr, InitialValue, Statement, StaticAttr,
    VariableDeclaration, Visitor,
};
use crate::common::Position;
use crate::errors::CompilerError;
use crate::errors::CompilerError::SemanticError;
use crate::lexer::{StorageClass, Type};
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

#[derive(Debug, Clone)]
struct ScopeEntry {
    layer: i32,
    is_extern: bool,
    unique_name: Rc<String>,
}

pub(crate) struct VariableResolutionVisitor<'map> {
    layer: i32,
    function: Rc<String>,
    variable_scopes: HashMap<String, VecDeque<ScopeEntry>>,
    loop_labels: VecDeque<(Rc<String>, bool)>,
    functions_map: &'map HashMap<String, FunAttr>,
    global_variables_map: &'map mut HashMap<String, StaticAttr>,
}

impl<'map> VariableResolutionVisitor<'map> {
    pub(crate) fn new(
        function: Rc<String>,
        functions_map: &'map HashMap<String, FunAttr>,
        global_variables_map: &'map mut HashMap<String, StaticAttr>,
    ) -> Self {
        Self {
            layer: 0,
            function,
            variable_scopes: HashMap::new(),
            loop_labels: VecDeque::new(),
            functions_map,
            global_variables_map,
        }
    }

    fn pop_stack(&mut self) {
        for scopes in self.variable_scopes.values_mut() {
            while !scopes.is_empty() && scopes.back().unwrap().layer == self.layer {
                scopes.pop_back();
            }
        }
    }

    fn resolve_variable(&self, original_name: &str) -> Option<Rc<String>> {
        if let Some(scopes) = self.variable_scopes.get(original_name) {
            if !scopes.is_empty() {
                let scope = scopes.back().unwrap();
                return if scope.is_extern {
                    Some(Rc::new(original_name.to_string()))
                } else {
                    Some(scope.unique_name.clone())
                };
            }
        }

        if self.global_variables_map.contains_key(original_name) {
            return Some(Rc::new(original_name.to_string()));
        }

        None
    }
}

impl<'map> Visitor for VariableResolutionVisitor<'map> {
    fn visit_declaration(
        &mut self,
        line_number: &Rc<Position>,
        declaration: &mut Declaration,
    ) -> Result<(), CompilerError> {
        match declaration {
            Declaration::VariableDeclaration(d) => self.handle_variable_declaration(line_number, d),
            Declaration::FunctionDeclaration(f) => {
                for param in &mut f.params {
                    let original_name = param.clone();
                    let unique_name = Rc::new(format!(
                        "{}::{}::{}",
                        self.function, original_name, self.layer
                    ));

                    let entry = ScopeEntry {
                        layer: self.layer,
                        is_extern: false,
                        unique_name: Rc::clone(&unique_name),
                    };

                    self.variable_scopes
                        .entry(original_name)
                        .or_insert_with(VecDeque::new)
                        .push_back(entry);

                    *param = unique_name.to_string();
                }

                if let Some(body) = &mut f.body {
                    self.layer += 1;
                    body.accept(self)?;
                    self.pop_stack();
                    self.layer -= 1;
                }

                self.pop_stack();

                Ok(())
            }
        }
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

    fn visit_variable(
        &mut self,
        line_number: &Rc<Position>,
        identifier: &mut Rc<String>,
        _node: &mut Type,
    ) -> Result<(), CompilerError> {
        let original_name = identifier.as_ref().to_string();

        // Try to resolve the variable
        if let Some(resolved_name) = self.resolve_variable(&original_name) {
            *identifier = resolved_name;
            Ok(())
        } else {
            // Variable not found in any scope
            Err(SemanticError(format!(
                "Undefined variable {} at {:?}",
                original_name, line_number
            )))
        }
    }

    fn visit_function_call(
        &mut self,
        line_number: &Rc<Position>,
        identifier: &mut Rc<String>,
        arguments: &mut Box<Vec<ASTNode<Expression>>>,
        _ret_type: &mut Type,
    ) -> Result<(), CompilerError> {
        let original_name = identifier.as_ref().to_string();
        if let Some(func) = self.functions_map.get(&original_name) {
            if arguments.len() != (*func.func_type).params.len() {
                return Err(SemanticError(format!(
                    "Function {} called with {} parameters but expected {} at {:?}",
                    original_name,
                    arguments.len(),
                    (*func.func_type).params.len(),
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
                original_name, line_number
            )))
        }
    }
}

impl<'map> VariableResolutionVisitor<'map> {
    fn handle_variable_declaration(
        &mut self,
        line_number: &Rc<Position>,
        d: &mut VariableDeclaration,
    ) -> Result<(), CompilerError> {
        let original_name = d.name.as_ref().to_string();

        if self.functions_map.contains_key(&original_name) {
            return Err(SemanticError(format!(
                "Function {} redeclared as variable at {:?}",
                original_name, line_number
            )));
        }

        let scopes = self
            .variable_scopes
            .entry(original_name.clone())
            .or_insert_with(VecDeque::new);

        if !scopes.is_empty() && scopes.back().unwrap().layer == self.layer {
            return Err(SemanticError(format!(
                "Duplicate variable declaration {} at {:?}",
                original_name, line_number
            )));
        }
        match d.storage_class {
            Some(StorageClass::Extern) => {
                if d.init.is_some() {
                    return Err(SemanticError(format!(
                        "Extern variable cannot be initialized at {:?}",
                        line_number
                    )));
                }

                if let Some(attr) = self.global_variables_map.get(&original_name) {
                    if attr.type_ != d.var_type {
                        return Err(SemanticError(format!(
                            "Extern variable {} redeclared with incompatible type at {:?}",
                            d.name, line_number
                        )));
                    }
                } else {
                    self.global_variables_map.insert(
                        original_name.clone(),
                        StaticAttr {
                            init: InitialValue::NoInitializer,
                            global: true,
                            type_: Type::Int,
                        },
                    );
                }

                let entry = ScopeEntry {
                    layer: self.layer,
                    is_extern: true,
                    unique_name: Rc::clone(&d.name),
                };

                self.variable_scopes
                    .entry(original_name)
                    .or_insert_with(VecDeque::new)
                    .push_back(entry);

                Ok(())
            }

            Some(StorageClass::Static) => {
                let initial_value = if let Some(init) = &d.init {
                    if let Expression::Constant(i) = &init.kind {
                        InitialValue::Initial(i.clone())
                    } else {
                        return Err(SemanticError(format!(
                            "Non-constant initializer of static variable {} at {:?}",
                            original_name, line_number
                        )));
                    }
                } else {
                    InitialValue::Initial(0u32.into())
                };

                let unique_name = Rc::from(format!("{}.{}", self.function, d.name));
                d.name = Rc::clone(&unique_name);

                self.global_variables_map.insert(
                    d.name.to_string(),
                    StaticAttr {
                        init: initial_value,
                        global: false,
                        type_: Type::Int,
                    },
                );

                let entry = ScopeEntry {
                    layer: self.layer,
                    is_extern: false,
                    unique_name: Rc::clone(&unique_name),
                };

                self.variable_scopes
                    .entry(original_name)
                    .or_insert_with(VecDeque::new)
                    .push_back(entry);

                Ok(())
            }

            None => {
                let unique_name = Rc::new(format!(
                    "{}::{}::{}",
                    self.function, original_name, self.layer
                ));

                d.name = Rc::clone(&unique_name);

                let entry = ScopeEntry {
                    layer: self.layer,
                    is_extern: false,
                    unique_name,
                };
                scopes.push_back(entry);

                if let Some(expr) = &mut d.init {
                    expr.accept(self)?;
                }

                Ok(())
            }
        }
    }
}
