use crate::CompilerError;
use crate::CompilerError::SemanticError;
use crate::common::{Const, Position};
use crate::lexer::{BinaryOperator, StorageClass, Type, UnaryOperator};
use crate::tac::{FunctionBody, TACInstruction};
use crate::tac_generator::TacVisitor;
use crate::type_check::TypeCheckVisitor;
use crate::variable_resolution::VariableResolutionVisitor;
use std::cmp::PartialEq;
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
        _line_number: &Rc<Position>,
        left: &mut Box<ASTNode<Expression>>,
        right: &mut Box<ASTNode<Expression>>,
        _type_: &mut Type,
    ) -> Result<(), CompilerError>
    where
        Self: Sized,
    {
        left.accept(self)?;
        right.accept(self)
    }
    fn visit_return(
        &mut self,
        _line_number: &Rc<Position>,
        expression: &mut ASTNode<Expression>,
    ) -> Result<(), CompilerError>
    where
        Self: Sized,
    {
        expression.accept(self)
    }
    fn visit_block(
        &mut self,
        _line_number: &Rc<Position>,
        body: &mut Block,
    ) -> Result<(), CompilerError>
    where
        Self: Sized,
    {
        for item in body {
            item.accept(self)?;
        }
        Ok(())
    }
    fn visit_unary(
        &mut self,
        _line_number: &Rc<Position>,
        _op: &mut UnaryOperator,
        expression: &mut Box<ASTNode<Expression>>,
        _type_: &mut Type,
    ) -> Result<(), CompilerError>
    where
        Self: Sized,
    {
        expression.accept(self)
    }
    fn visit_binary(
        &mut self,
        _line_number: &Rc<Position>,
        _op: &mut BinaryOperator,
        left: &mut Box<ASTNode<Expression>>,
        right: &mut Box<ASTNode<Expression>>,
        _type_: &mut Type,
    ) -> Result<(), CompilerError>
    where
        Self: Sized,
    {
        left.accept(self)?;
        right.accept(self)
    }
    fn visit_condition(
        &mut self,
        _line_number: &Rc<Position>,
        condition: &mut Box<ASTNode<Expression>>,
        if_true: &mut Box<ASTNode<Expression>>,
        if_false: &mut Box<ASTNode<Expression>>,
        _type_: &mut Type,
    ) -> Result<(), CompilerError>
    where
        Self: Sized,
    {
        condition.accept(self)?;
        if_true.accept(self)?;
        if_false.accept(self)
    }
    fn visit_while(
        &mut self,
        _line_number: &Rc<Position>,
        condition: &mut ASTNode<Expression>,
        body: &mut Box<ASTNode<Statement>>,
        _label: &mut Rc<String>,
        _is_do_while: &mut bool,
    ) -> Result<(), CompilerError>
    where
        Self: Sized,
    {
        condition.accept(self)?;
        body.accept(self)
    }
    fn visit_break(
        &mut self,
        _line_number: &Rc<Position>,
        _label: &mut Rc<String>,
    ) -> Result<(), CompilerError>
    where
        Self: Sized,
    {
        Ok(())
    }
    fn visit_continue(
        &mut self,
        _line_number: &Rc<Position>,
        _label: &mut Rc<String>,
        _is_for: &mut bool,
    ) -> Result<(), CompilerError> {
        Ok(())
    }
    fn visit_for(
        &mut self,
        _line_number: &Rc<Position>,
        init: &mut ASTNode<ForInit>,
        condition: &mut Option<ASTNode<Expression>>,
        increment: &mut Option<ASTNode<Expression>>,
        body: &mut Box<ASTNode<Statement>>,
        _label: &mut Rc<String>,
    ) -> Result<(), CompilerError>
    where
        Self: Sized,
    {
        init.accept(self)?;
        if let Some(condition) = condition {
            condition.accept(self)?;
        }
        if let Some(increment) = increment {
            increment.accept(self)?;
        }
        body.accept(self)
    }
    fn visit_const(
        &mut self,
        _line_number: &Rc<Position>,
        _value: &mut Const,
        _type_: &mut Type,
    ) -> Result<(), CompilerError> {
        Ok(())
    }
    fn visit_variable(
        &mut self,
        _line_number: &Rc<Position>,
        _identifier: &mut Rc<String>,
        _type_: &mut Type,
    ) -> Result<(), CompilerError> {
        Ok(())
    }
    fn visit_function_call(
        &mut self,
        _line_number: &Rc<Position>,
        _identifier: &mut Rc<String>,
        arguments: &mut Box<Vec<ASTNode<Expression>>>,
        _ret_type: &mut Type,
    ) -> Result<(), CompilerError>
    where
        Self: Sized,
    {
        for argument in arguments.iter_mut() {
            argument.accept(self)?;
        }
        Ok(())
    }
    fn visit_prefix(
        &mut self,
        _line_number: &Rc<Position>,
        variable: &mut Box<ASTNode<Expression>>,
        _operator: &mut UnaryOperator,
        _type_: &mut Type,
    ) -> Result<(), CompilerError>
    where
        Self: Sized,
    {
        variable.accept(self)
    }
    fn visit_postfix(
        &mut self,
        _line_number: &Rc<Position>,
        variable: &mut Box<ASTNode<Expression>>,
        _operator: &mut UnaryOperator,
        _type_: &mut Type,
    ) -> Result<(), CompilerError>
    where
        Self: Sized,
    {
        variable.accept(self)
    }
    fn visit_if_else(
        &mut self,
        _line_number: &Rc<Position>,
        expression: &mut ASTNode<Expression>,
        if_true: &mut Box<ASTNode<Statement>>,
        if_false: &mut Option<Box<ASTNode<Statement>>>,
    ) -> Result<(), CompilerError>
    where
        Self: Sized,
    {
        expression.accept(self)?;
        if_true.accept(self)?;
        if let Some(if_false) = if_false {
            if_false.accept(self)
        } else {
            Ok(())
        }
    }
    fn visit_cast(
        &mut self,
        _line_number: &Rc<Position>,
        _target_type: &mut Type,
        exp: &mut Box<ASTNode<Expression>>,
    ) -> Result<(), CompilerError>
    where
        Self: Sized,
    {
        exp.accept(self)
    }
}

pub(crate) struct FunAttr {
    pub(crate) defined: bool,
    pub(crate) global: bool,
    pub(crate) func_type: Rc<FuncType>,
}

pub(crate) struct StaticAttr {
    pub(crate) init: InitialValue,
    pub(crate) global: bool,
    pub(crate) type_: Type,
}

#[derive(Debug)]
pub(crate) enum InitialValue {
    Tentative,
    Initial(Const),
    NoInitializer,
}

#[derive(Debug)]
pub(crate) struct FuncType {
    pub(crate) params: Vec<Type>,
    pub(crate) ret: Type,
}

#[derive(Debug)]
pub(crate) struct ASTNode<T> {
    pub(crate) line_number: Rc<Position>,
    pub(crate) type_: Type,
    pub(crate) kind: T,
}

pub(crate) type Program = Vec<ASTNode<Declaration>>;

#[derive(Debug)]
pub(crate) struct FunctionDeclaration {
    pub(crate) name: Rc<String>,
    pub(crate) params: Vec<String>,
    pub(crate) body: Option<ASTNode<Block>>,
    pub(crate) storage_class: Option<StorageClass>,
    pub(crate) func_type: Rc<FuncType>,
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
    pub(crate) name: Rc<String>,
    pub(crate) init: Option<ASTNode<Expression>>,
    pub(crate) storage_class: Option<StorageClass>,
    pub(crate) var_type: Type,
}

#[derive(Debug)]
pub(crate) enum Expression {
    Constant(Const),
    Variable(Rc<String>),
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
    FunctionCall(Rc<String>, Box<Vec<ASTNode<Expression>>>),
    Prefix(UnaryOperator, Box<ASTNode<Expression>>),
    Postfix(UnaryOperator, Box<ASTNode<Expression>>),
    Cast(Type, Box<ASTNode<Expression>>),
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

pub(crate) fn extract_base_variable(node: &Expression) -> Rc<String> {
    match node {
        Expression::Variable(v) => Rc::clone(v),
        Expression::Prefix(_, v) => extract_base_variable(&v.kind),
        _ => panic!("Not a variable"),
    }
}

impl PartialEq for FuncType {
    fn eq(&self, other: &Self) -> bool {
        self.params == other.params && self.ret == other.ret
    }
}

impl ASTNode<Program> {
    pub(crate) fn generate(&mut self, out: &mut String) -> Result<(), CompilerError> {
        let mut shared_functions_map: HashMap<String, FunAttr> = HashMap::new();
        let mut shared_variables_map: HashMap<String, StaticAttr> = HashMap::new();

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
                let mut visitor =
                    TypeCheckVisitor::new(&shared_functions_map, &shared_variables_map);
                visitor.visit_declaration(&declaration.line_number, &mut declaration.kind)?;
                println!("{:#?}", declaration);
                declaration.generate(out)?;
            }
        }

        for (name, static_attr) in shared_variables_map.iter() {
            let tac = match &static_attr.init {
                InitialValue::Tentative => TACInstruction::StaticVariable {
                    name: Rc::from(name.clone()),
                    global: static_attr.global,
                    init: match static_attr.type_ {
                        Type::Int => Const::ConstInt(0),
                        Type::Long => Const::ConstLong(0),
                        Type::UInt => Const::ConstUInt(0),
                        Type::ULong => Const::ConstULong(0),
                        _ => unreachable!(),
                    },
                },
                InitialValue::Initial(i) => TACInstruction::StaticVariable {
                    name: Rc::from(name.clone()),
                    global: static_attr.global,
                    init: i.clone(),
                },
                InitialValue::NoInitializer => continue,
            };
            tac.make_assembly(out, &FunctionBody::new());
        }

        Ok(())
    }

    fn typecheck_file_scope_variable_declaration(
        shared_functions_map: &mut HashMap<String, FunAttr>,
        shared_variables_map: &mut HashMap<String, StaticAttr>,
        var: &&mut VariableDeclaration,
    ) -> Option<Result<(), CompilerError>> {
        let mut initial_value = if let Some(init) = &var.init {
            if let Expression::Constant(i) = &init.kind {
                InitialValue::Initial(i.clone())
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
            type_: old_type,
        }) = shared_variables_map.get(&identifier)
        {
            if var.var_type != *old_type {
                return Some(Err(SemanticError(format!(
                    "Conflicting variable type definitions of {}",
                    var.name
                ))));
            }
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
                    initial_value = InitialValue::Initial(i.clone());
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
                type_: var.var_type,
            },
        );
        None
    }

    fn typecheck_function_declaration(
        shared_functions_map: &mut HashMap<String, FunAttr>,
        shared_variables_map: &mut HashMap<String, StaticAttr>,
        func: &&mut FunctionDeclaration,
    ) -> Option<Result<(), CompilerError>> {
        let name = Rc::clone(&func.name);
        let func_type = Rc::clone(&func.func_type);
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
            if *old_decl.func_type != *func_type {
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
                func_type,
            },
        );
        None
    }
}

impl ASTNode<Declaration> {
    pub(crate) fn generate(&mut self, out: &mut String) -> Result<(), CompilerError> {
        if let Declaration::FunctionDeclaration(func) = &mut self.kind {
            let identifier = Rc::clone(&func.name);

            let mut function_body = FunctionBody::new();
            let mut tac_visitor = TacVisitor::new(Rc::clone(&identifier), &mut function_body);
            self.accept(&mut tac_visitor)?;
            println!("{:#?}", function_body);

            function_body.add_default_return();

            for instruction in &function_body.instructions {
                *out += "\n";
                instruction.make_assembly(out, &function_body);
            }

            return Ok(());
        }

        unimplemented!();
    }
}

impl ASTNode<Block> {
    pub(crate) fn accept<V: Visitor>(&mut self, visitor: &mut V) -> Result<(), CompilerError> {
        for block_item in &mut self.kind {
            block_item.accept(visitor)?;
        }
        Ok(())
    }
}

impl ASTNode<BlockItem> {
    pub(crate) fn accept<V: Visitor>(&mut self, visitor: &mut V) -> Result<(), CompilerError> {
        match &mut self.kind {
            BlockItem::D(declaration) => declaration.accept(visitor),
            BlockItem::S(statement) => statement.deref_mut().accept(visitor),
        }
    }
}

impl ASTNode<Declaration> {
    fn accept<V: Visitor>(&mut self, visitor: &mut V) -> Result<(), CompilerError> {
        visitor.visit_declaration(&self.line_number, &mut self.kind)
    }
}

impl ASTNode<Expression> {
    pub(crate) fn accept<V: Visitor>(&mut self, visitor: &mut V) -> Result<(), CompilerError> {
        match &mut self.kind {
            Expression::Constant(value) => {
                visitor.visit_const(&self.line_number, value, &mut self.type_)
            }
            Expression::Variable(v) => {
                visitor.visit_variable(&self.line_number, v, &mut self.type_)
            }
            Expression::Unary(op, exp) => {
                visitor.visit_unary(&self.line_number, op, exp, &mut self.type_)
            }
            Expression::Binary { op, left, right } => {
                visitor.visit_binary(&self.line_number, op, left, right, &mut self.type_)
            }
            Expression::Assignment { left, right } => {
                visitor.visit_assignment(&self.line_number, left, right, &mut self.type_)
            }
            Expression::Condition {
                condition,
                if_true,
                if_false,
            } => visitor.visit_condition(
                &self.line_number,
                condition,
                if_true,
                if_false,
                &mut self.type_,
            ),
            Expression::FunctionCall(identifier, arguments) => visitor.visit_function_call(
                &self.line_number,
                identifier,
                arguments,
                &mut self.type_,
            ),
            Expression::Prefix(op, exp) => {
                visitor.visit_prefix(&self.line_number, exp, op, &mut self.type_)
            }
            Expression::Postfix(op, exp) => {
                visitor.visit_postfix(&self.line_number, exp, op, &mut self.type_)
            }
            Expression::Cast(type_, exp) => visitor.visit_cast(&self.line_number, type_, exp),
        }
    }
}

impl ASTNode<Statement> {
    pub(crate) fn accept<V: Visitor>(&mut self, visitor: &mut V) -> Result<(), CompilerError> {
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
    pub(crate) fn accept<V: Visitor>(&mut self, visitor: &mut V) -> Result<(), CompilerError> {
        match &mut self.kind {
            ForInit::InitDecl(v) => visitor.visit_declaration(&self.line_number, v),
            ForInit::InitExp(v) => match v {
                Some(e) => e.accept(visitor),
                None => Ok(()),
            },
        }
    }
}
