use crate::CompilerError;
use crate::CompilerError::SemanticError;
use crate::ast::{
    ASTNode, Block, Declaration, Expression, ForInit, FunAttr, Statement, StaticAttr, Visitor,
};
use crate::common::{Const, Identifier, Position};
use crate::lexer::{BinaryOperator, Type, UnaryOperator};
use std::collections::HashMap;
use std::rc::Rc;

pub(crate) struct TypeCheckVisitor<'map> {
    variables_map: HashMap<Identifier, Type>,
    functions_map: &'map HashMap<Identifier, FunAttr>,
    global_variables_map: &'map HashMap<Identifier, StaticAttr>,
    current_return_type: Type,
}

fn get_common_type(type1: &Type, type2: &Type) -> Type {
    if type1 == type2 {
        type1.clone()
    } else {
        Type::Long
    }
}

fn convert_to(line_number: &Rc<Position>, e: &mut ASTNode<Expression>, t: &Type) {
    if e.type_ == *t {
        return;
    }

    let original_expr = std::mem::replace(
        e,
        ASTNode {
            kind: Expression::Constant(Const::ConstInt(0)), // Temporary placeholder
            type_: Type::Void,
            line_number: Rc::clone(line_number),
        },
    );

    let cast_node = ASTNode {
        kind: Expression::Cast(t.clone(), Box::from(original_expr)),
        type_: t.clone(),
        line_number: Rc::clone(line_number),
    };

    *e = cast_node;
}

impl<'map> TypeCheckVisitor<'map> {
    pub(crate) fn new(
        functions_map: &'map HashMap<Identifier, FunAttr>,
        global_variables_map: &'map HashMap<Identifier, StaticAttr>,
    ) -> Self {
        Self {
            variables_map: HashMap::new(),
            functions_map,
            global_variables_map,
            current_return_type: Type::Void,
        }
    }
}

impl<'map> Visitor for TypeCheckVisitor<'map> {
    fn visit_declaration(
        &mut self,
        line_number: &Rc<Position>,
        declaration: &mut Declaration,
    ) -> Result<(), CompilerError> {
        match declaration {
            Declaration::VariableDeclaration(decl) => {
                if decl.var_type == Type::Void {
                    Err(SemanticError(format!(
                        "Cannot declare variable {} of type 'void' at {:?}",
                        decl.name, line_number
                    )))
                } else {
                    self.variables_map
                        .insert(decl.name.to_string(), decl.var_type);
                    Ok(())
                }
            }
            Declaration::FunctionDeclaration(decl) => {
                for (param_name, param_type) in decl.params.iter().zip(decl.func_type.params.iter())
                {
                    self.variables_map
                        .insert(param_name.clone(), param_type.clone());
                }
                self.current_return_type = decl.func_type.ret.clone();
                if let Some(body) = &mut decl.body {
                    body.accept(self)
                } else {
                    Ok(())
                }
            }
        }
    }

    fn visit_assignment(
        &mut self,
        line_number: &Rc<Position>,
        left: &mut Box<ASTNode<Expression>>,
        right: &mut Box<ASTNode<Expression>>,
        type_: &mut Type,
    ) -> Result<(), CompilerError> {
        left.accept(self)?;
        right.accept(self)?;
        let left_type = &left.type_;
        convert_to(line_number, right, left_type);
        *type_ = left_type.clone();
        Ok(())
    }

    fn visit_return(
        &mut self,
        line_number: &Rc<Position>,
        expression: &mut ASTNode<Expression>,
    ) -> Result<(), CompilerError> {
        expression.accept(self)?;
        convert_to(line_number, expression, &self.current_return_type);
        Ok(())
    }

    fn visit_block(
        &mut self,
        _line_number: &Rc<Position>,
        body: &mut Block,
    ) -> Result<(), CompilerError> {
        for line in body.iter_mut() {
            line.accept(self)?;
        }
        Ok(())
    }

    fn visit_unary(
        &mut self,
        _line_number: &Rc<Position>,
        op: &mut UnaryOperator,
        expression: &mut Box<ASTNode<Expression>>,
        type_: &mut Type,
    ) -> Result<(), CompilerError> {
        expression.accept(self)?;
        *type_ = match op {
            UnaryOperator::LogicalNot => Type::Int,
            _ => expression.type_,
        };
        Ok(())
    }

    fn visit_binary(
        &mut self,
        line_number: &Rc<Position>,
        op: &mut BinaryOperator,
        left: &mut Box<ASTNode<Expression>>,
        right: &mut Box<ASTNode<Expression>>,
        type_: &mut Type,
    ) -> Result<(), CompilerError> {
        left.accept(self)?;
        right.accept(self)?;
        if *op == BinaryOperator::LogicalAnd || *op == BinaryOperator::LogicalOr {
            *type_ = Type::Int;
            return Ok(());
        }
        let t1 = left.type_;
        let t2 = right.type_;
        let common_type = get_common_type(&t1, &t2);
        convert_to(line_number, left, &common_type);
        convert_to(line_number, right, &common_type);
        *type_ = match op {
            BinaryOperator::Addition
            | BinaryOperator::Subtraction
            | BinaryOperator::Multiply
            | BinaryOperator::Divide
            | BinaryOperator::Modulo => common_type,
            _ => Type::Int,
        };

        Ok(())
    }

    fn visit_condition(
        &mut self,
        line_number: &Rc<Position>,
        condition: &mut Box<ASTNode<Expression>>,
        if_true: &mut Box<ASTNode<Expression>>,
        if_false: &mut Box<ASTNode<Expression>>,
        type_: &mut Type,
    ) -> Result<(), CompilerError> {
        condition.accept(self)?;
        if_true.accept(self)?;
        if_false.accept(self)?;
        let common_type = get_common_type(&if_true.type_, &if_false.type_);
        convert_to(line_number, if_true, &common_type);
        convert_to(line_number, if_false, &common_type);
        *type_ = common_type;
        Ok(())
    }

    fn visit_while(
        &mut self,
        _line_number: &Rc<Position>,
        condition: &mut ASTNode<Expression>,
        body: &mut Box<ASTNode<Statement>>,
        _label: &mut Rc<String>,
        _is_do_while: &mut bool,
    ) -> Result<(), CompilerError> {
        condition.accept(self)?;
        body.accept(self)
    }

    fn visit_break(
        &mut self,
        _line_number: &Rc<Position>,
        _label: &mut Rc<String>,
    ) -> Result<(), CompilerError> {
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
    ) -> Result<(), CompilerError> {
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
        value: &mut Const,
        type_: &mut Type,
    ) -> Result<(), CompilerError> {
        match value {
            Const::ConstInt(_) => *type_ = Type::Int,
            Const::ConstLong(_) => *type_ = Type::Long,
        }
        Ok(())
    }

    fn visit_variable(
        &mut self,
        _line_number: &Rc<Position>,
        identifier: &mut Rc<Identifier>,
        node: &mut Type,
    ) -> Result<(), CompilerError> {
        if let Some(attr) = self.global_variables_map.get(&identifier.to_string()) {
            *node = attr.type_;
        } else {
            *node = self
                .variables_map
                .get(&identifier.to_string())
                .unwrap()
                .clone();
        }
        Ok(())
    }

    fn visit_function_call(
        &mut self,
        line_number: &Rc<Position>,
        identifier: &mut Rc<Identifier>,
        arguments: &mut Box<Vec<ASTNode<Expression>>>,
        ret_type: &mut Type,
    ) -> Result<(), CompilerError> {
        let func_type = Rc::clone(
            &self
                .functions_map
                .get(&identifier.to_string())
                .unwrap()
                .func_type,
        );
        if func_type.params.len() != arguments.len() {
            return Err(SemanticError(format!(
                "Function {} called with {} arguments but expected {} at {:?}",
                identifier,
                arguments.len(),
                func_type.params.len(),
                line_number
            )));
        }
        for (arg, param_type) in arguments.iter_mut().zip(func_type.params.iter()) {
            arg.accept(self)?;
            convert_to(line_number, arg, param_type);
        }
        *ret_type = func_type.ret.clone();
        Ok(())
    }

    fn visit_prefix(
        &mut self,
        _line_number: &Rc<Position>,
        variable: &mut Box<ASTNode<Expression>>,
        _operator: &mut UnaryOperator,
        type_: &mut Type,
    ) -> Result<(), CompilerError> {
        variable.accept(self)?;
        *type_ = variable.type_;
        Ok(())
    }

    fn visit_postfix(
        &mut self,
        _line_number: &Rc<Position>,
        variable: &mut Box<ASTNode<Expression>>,
        _operator: &mut UnaryOperator,
        type_: &mut Type,
    ) -> Result<(), CompilerError> {
        variable.accept(self)?;
        *type_ = variable.type_;
        Ok(())
    }

    fn visit_if_else(
        &mut self,
        _line_number: &Rc<Position>,
        expression: &mut ASTNode<Expression>,
        if_true: &mut Box<ASTNode<Statement>>,
        if_false: &mut Option<Box<ASTNode<Statement>>>,
    ) -> Result<(), CompilerError> {
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
        target_type: &mut Type,
        exp: &mut Box<ASTNode<Expression>>,
    ) -> Result<(), CompilerError> {
        exp.accept(self)?;
        *target_type = exp.type_.clone();
        Ok(())
    }
}
