use crate::CompilerError;
use crate::CompilerError::SemanticError;
use crate::common::{Identifier, Position};
use crate::lexer::{BinaryOperator, Number, StorageClass, Type, UnaryOperator};
use crate::tac::{FunctionBody, TACInstruction};
use crate::tac_visitor::TacVisitor;
use crate::variable_resolution::VariableResolutionVisitor;
use std::collections::HashMap;
use std::ops::DerefMut;
use std::rc::Rc;

pub(crate) trait Visitor {
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
    fn visit_function_call(
        &mut self,
        line_number: &Rc<Position>,
        identifier: &mut Rc<Identifier>,
        arguments: &mut Box<Vec<ASTNode<Expression>>>,
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

pub(crate) struct FunAttr {
    pub(crate) defined: bool,
    pub(crate) global: bool,
    pub(crate) param_count: usize,
}

pub(crate) struct StaticAttr {
    pub(crate) init: InitialValue,
    pub(crate) global: bool,
    pub(crate) type_: Type,
}

#[derive(Debug)]
pub(crate) enum InitialValue {
    Tentative,
    Initial(Number),
    NoInitializer,
}

#[derive(Debug)]
pub(crate) enum ASTType {
    Type(Type),
    FuncType {
        params: Box<Vec<ASTType>>,
        ret: Box<ASTType>,
    },
}

#[derive(Debug)]
pub(crate) struct ASTNode<T> {
    pub(crate) line_number: Rc<Position>,
    pub(crate) kind: T,
}

pub(crate) type Program = Vec<ASTNode<Declaration>>;

#[derive(Debug)]
pub(crate) struct FunctionDeclaration {
    pub(crate) name: Rc<Identifier>,
    pub(crate) params: Vec<Identifier>,
    pub(crate) body: Option<ASTNode<Block>>,
    pub(crate) storage_class: Option<StorageClass>,
    pub(crate) type_: Rc<ASTType>,
}

pub(crate) type Block = Vec<ASTNode<BlockItem>>;

#[derive(Debug)]
pub(crate) enum BlockItem {
    D(ASTNode<Declaration>),
    S(Box<ASTNode<Statement>>),
}

#[derive(Debug)]
pub(crate) enum Declaration {
    FunctionDeclaration(FunctionDeclaration),
    VariableDeclaration(VariableDeclaration),
}

#[derive(Debug)]
pub(crate) struct VariableDeclaration {
    pub(crate) name: Rc<Identifier>,
    pub(crate) init: Option<ASTNode<Expression>>,
    pub(crate) storage_class: Option<StorageClass>,
    pub(crate) type_: Rc<ASTType>,
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

#[derive(Debug)]
pub(crate) enum ForInit {
    InitDecl(Declaration),
    InitExp(Option<ASTNode<Expression>>),
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

impl ASTNode<Program> {
    pub(crate) fn generate(&mut self, out: &mut String) -> Result<(), CompilerError> {
        let mut shared_functions_map: HashMap<Identifier, FunAttr> = HashMap::new();
        let mut shared_variables_map: HashMap<Identifier, StaticAttr> = HashMap::new();

        // first pass: register declarations
        for declaration in self.kind.iter_mut() {
            match &mut declaration.kind {
                Declaration::FunctionDeclaration(func) => {
                    if let Some(value) = Self::typecheck_function_declaration(
                        &mut shared_functions_map,
                        &mut shared_variables_map,
                        &func,
                    ) {
                        return value;
                    }
                }
                Declaration::VariableDeclaration(var) => {
                    if let Some(value) = Self::typecheck_file_scope_variable_declaration(
                        &mut shared_functions_map,
                        &mut shared_variables_map,
                        &var,
                    ) {
                        return value;
                    }
                }
            }
        }

        // second: regular
        for declaration in &mut self.kind {
            if let Declaration::FunctionDeclaration(func) = &declaration.kind {
                let func_name = Rc::clone(&func.name);
                let mut visitor = VariableResolutionVisitor::new(
                    func_name,
                    &shared_functions_map,
                    &mut shared_variables_map,
                );
                visitor.visit_declaration(&declaration.line_number, &mut declaration.kind)?;
                declaration.generate(out, &shared_functions_map, &mut shared_variables_map)?;
            }
        }

        for (name, static_attr) in shared_variables_map.iter() {
            let tac = match static_attr.init {
                InitialValue::Tentative => TACInstruction::StaticVariable {
                    name: Rc::from(name.clone()),
                    global: static_attr.global,
                    init: 0,
                },
                InitialValue::Initial(i) => TACInstruction::StaticVariable {
                    name: Rc::from(name.clone()),
                    global: static_attr.global,
                    init: i,
                },
                InitialValue::NoInitializer => continue,
            };
            tac.make_assembly(out, &FunctionBody::new());
        }

        Ok(())
    }

    fn typecheck_file_scope_variable_declaration(
        shared_functions_map: &mut HashMap<Identifier, FunAttr>,
        shared_variables_map: &mut HashMap<Identifier, StaticAttr>,
        var: &&mut VariableDeclaration,
    ) -> Option<Result<(), CompilerError>> {
        let mut initial_value = if let Some(init) = &var.init {
            if let Expression::Constant(i) = init.kind {
                InitialValue::Initial(i)
            } else {
                return Some(Err(SemanticError(format!(
                    "Initial value {:?} of {} is non-constant",
                    init.kind, var.name
                ))));
            }
        } else {
            if var.storage_class == Some(StorageClass::Extern) {
                InitialValue::NoInitializer
            } else {
                InitialValue::Tentative
            }
        };
        let mut global = var.storage_class != Some(StorageClass::Static);
        let identifier = (*var.name).clone();

        if shared_functions_map.contains_key(&identifier) {
            return Some(Err(SemanticError(format!(
                "Function {} redeclared as variable",
                identifier
            ))));
        }

        if let Some(StaticAttr {
            global: old_global,
            init: old_init,
            ..
        }) = shared_variables_map.get(&identifier)
        {
            if var.storage_class == Some(StorageClass::Extern) {
                global = *old_global;
            } else if *old_global != global {
                return Some(Err(SemanticError(format!(
                    "Conflicting variable linkage of {}",
                    identifier
                ))));
            }
            if let InitialValue::Initial(i) = old_init {
                if let Some(_) = var.init {
                    return Some(Err(SemanticError(format!(
                        "Conflict file scope variable definitions of {}",
                        identifier
                    ))));
                } else {
                    initial_value = InitialValue::Initial(*i);
                }
            } else if !matches!(initial_value, InitialValue::Initial(_))
                && matches!(old_init, InitialValue::Tentative)
            {
                initial_value = InitialValue::Tentative;
            }
        }
        shared_variables_map.insert(
            identifier,
            StaticAttr {
                init: initial_value,
                global,
                type_: Type::Int,
            },
        );
        None
    }

    fn typecheck_function_declaration(
        shared_functions_map: &mut HashMap<Identifier, FunAttr>,
        shared_variables_map: &mut HashMap<Identifier, StaticAttr>,
        func: &&mut FunctionDeclaration,
    ) -> Option<Result<(), CompilerError>> {
        let name = Rc::clone(&func.name);
        let param_count = func.params.len();
        let has_body = func.body.is_some();
        let identifier = (*name).clone();
        if shared_variables_map.contains_key(&identifier) {
            return Some(Err(SemanticError(format!(
                "Variable {} redeclared as function",
                identifier
            ))));
        }
        if let Some(old_decl) = shared_functions_map.get(&identifier) {
            if old_decl.defined && has_body {
                // Error if duplicate definition (duplicate prototypes are fine)
                return Some(Err(SemanticError(format!(
                    "Duplicate definition of {}",
                    name
                ))));
            }
            if old_decl.global && func.storage_class == Some(StorageClass::Static) {
                return Some(Err(SemanticError(format!(
                    "Static function declaration of {} follows non-static",
                    name
                ))));
            }
            if old_decl.param_count != param_count {
                return Some(Err(SemanticError(format!(
                    "Incompatible function declaration of {}",
                    name
                ))));
            }
        }
        shared_functions_map.insert(
            identifier,
            FunAttr {
                defined: func.body.is_some(),
                global: func.storage_class != Some(StorageClass::Static),
                param_count,
            },
        );
        None
    }
}

impl ASTNode<Declaration> {
    pub(crate) fn generate(
        &mut self,
        out: &mut String,
        shared_functions_map: &HashMap<Identifier, FunAttr>,
        shared_variables_map: &mut HashMap<Identifier, StaticAttr>,
    ) -> Result<(), CompilerError> {
        if let Declaration::FunctionDeclaration(func) = &mut self.kind {
            let identifier = Rc::clone(&func.name);

            let mut variable_resolution_visitor = VariableResolutionVisitor::new(
                Rc::clone(&identifier),
                shared_functions_map,
                shared_variables_map,
            );

            self.accept(&mut variable_resolution_visitor as &mut dyn Visitor)?;

            let mut function_body = FunctionBody::new();
            let mut tac_visitor = TacVisitor::new(Rc::clone(&identifier), &mut function_body);
            self.accept(&mut tac_visitor as &mut dyn Visitor)?;
            println!("{:#?}", function_body);

            if identifier.as_str() == "main" {
                function_body.add_default_return_to_main();
            }

            for instruction in &function_body.instructions {
                instruction.make_assembly(out, &function_body);
            }

            return Ok(());
        }

        unimplemented!();
    }
}

impl ASTNode<Block> {
    pub(crate) fn accept(&mut self, visitor: &mut dyn Visitor) -> Result<(), CompilerError> {
        for block_item in &mut self.kind {
            block_item.accept(visitor)?;
        }
        Ok(())
    }
}

impl ASTNode<BlockItem> {
    pub(crate) fn accept(&mut self, visitor: &mut dyn Visitor) -> Result<(), CompilerError> {
        match &mut self.kind {
            BlockItem::D(declaration) => declaration.accept(visitor),
            BlockItem::S(statement) => statement.deref_mut().accept(visitor),
        }
    }
}

impl ASTNode<Declaration> {
    fn accept(&mut self, visitor: &mut dyn Visitor) -> Result<(), CompilerError> {
        visitor.visit_declaration(&self.line_number, &mut self.kind)
    }
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
            Expression::FunctionCall(identifier, arguments) => {
                visitor.visit_function_call(&self.line_number, identifier, arguments)
            }
            Expression::Prefix(op, exp) => visitor.visit_prefix(&self.line_number, exp, op),
            Expression::Postfix(op, exp) => visitor.visit_postfix(&self.line_number, exp, op),
        }
    }
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

impl ASTNode<ForInit> {
    pub(crate) fn accept(&mut self, visitor: &mut dyn Visitor) -> Result<(), CompilerError> {
        match &mut self.kind {
            ForInit::InitDecl(v) => visitor.visit_declaration(&self.line_number, v),
            ForInit::InitExp(v) => match v {
                Some(e) => e.accept(visitor),
                None => Ok(()),
            },
        }
    }
}
