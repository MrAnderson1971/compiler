use crate::ast::{ASTNode, Visitor};
use crate::common::{Operand, Position, Pseudoregister};
use crate::errors::CompilerError;
use crate::errors::CompilerError::SemanticError;
use crate::lexer::{BinaryOperator, Number, UnaryOperator};
use crate::tac::FunctionBody;
use crate::tac::TACInstructionType::{
    BinaryOpInstruction, FunctionInstruction, Jump, JumpIfZero, Label, ReturnInstruction,
    StoreValueInstruction, UnaryOpInstruction,
};
use std::rc::Rc;

pub struct TacVisitor<'a> {
    name: Rc<String>,
    body: &'a mut FunctionBody,
    result: Rc<Operand>,
}

impl<'a> TacVisitor<'a> {
    pub fn new(name: Rc<String>, body: &'a mut FunctionBody) -> Self {
        Self {
            name,
            body,
            result: Rc::new(Operand::None),
        }
    }
}

impl<'a> Visitor for TacVisitor<'a> {
    fn visit_program(
        &mut self,
        _line_number: &Rc<Position>,
        _function_declaration: &mut Box<ASTNode>,
    ) -> Result<(), CompilerError> {
        panic!("Should not be called")
    }

    fn visit_function(
        &mut self,
        line_number: &Rc<Position>,
        identifier: &mut Rc<String>,
        body: &mut Option<Box<ASTNode>>,
    ) -> Result<(), CompilerError> {
        self.body.add_instruction(
            Rc::clone(&line_number),
            FunctionInstruction {
                name: identifier.clone(),
            },
        );
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
        let pseudoregister = Rc::new(Pseudoregister::new(
            (*self.name).clone(),
            self.body.variable_count,
        ));
        self.body
            .variable_to_pseudoregister
            .insert(identifier.clone(), Rc::clone(&pseudoregister));
        if let Some(expression) = expression {
            expression.accept(self)?;
            self.body.add_instruction(
                line_number.clone(),
                StoreValueInstruction {
                    dest: Rc::clone(&pseudoregister),
                    src: Rc::clone(&self.result),
                },
            );
        }
        self.body.variable_count += 1;
        Ok(())
    }

    fn visit_assignment(
        &mut self,
        line_number: &Rc<Position>,
        left: &mut Box<ASTNode>,
        right: &mut Box<ASTNode>,
    ) -> Result<(), CompilerError> {
        right.accept(self)?;
        let src = Rc::clone(&self.result);
        left.accept(self)?;
        let dest = Rc::clone(&self.result);
        match dest.as_ref() {
            Operand::Register(variable) => {
                let dest_registry: Rc<Pseudoregister> = Rc::clone(variable);
                self.body.add_instruction(
                    line_number.clone(),
                    StoreValueInstruction {
                        dest: dest_registry,
                        src,
                    },
                );
                Ok(())
            }
            _ => Err(SemanticError(format!(
                "Expected lvalue but got {:?} at {:?}",
                src, line_number
            ))),
        }
    }

    fn visit_return(
        &mut self,
        line_number: &Rc<Position>,
        expression: &mut Option<Box<ASTNode>>,
    ) -> Result<(), CompilerError> {
        if let Some(expression) = expression {
            expression.accept(self)?;
            self.body.add_instruction(
                line_number.clone(),
                ReturnInstruction {
                    val: Rc::clone(&self.result),
                },
            );
        } else {
            self.body.add_instruction(
                line_number.clone(),
                ReturnInstruction {
                    val: Rc::new(Operand::None),
                },
            );
        }
        Ok(())
    }

    fn visit_block(
        &mut self,
        _line_number: &Rc<Position>,
        body: &mut Vec<Box<ASTNode>>,
    ) -> Result<(), CompilerError> {
        for item in body {
            item.accept(self)?;
        }
        Ok(())
    }

    fn visit_unary(
        &mut self,
        line_number: &Rc<Position>,
        op: &mut UnaryOperator,
        expression: &mut Box<ASTNode>,
    ) -> Result<(), CompilerError> {
        expression.accept(self)?;
        if *op == UnaryOperator::UnaryAdd {
            return Ok(());
        }
        let src = Rc::clone(&self.result);
        let dest = Rc::new(Pseudoregister::new(
            (*self.name).clone(),
            self.body.variable_count,
        ));
        self.body.variable_count += 1;
        self.body.add_instruction(
            line_number.clone(),
            UnaryOpInstruction {
                dest: Rc::clone(&dest),
                op: *op,
                operand: src,
            },
        );
        self.result = Rc::from(Operand::Register(Rc::clone(&dest)));
        Ok(())
    }

    fn visit_binary(
        &mut self,
        line_number: &Rc<Position>,
        op: &mut BinaryOperator,
        left: &mut Box<ASTNode>,
        right: &mut Box<ASTNode>,
    ) -> Result<(), CompilerError> {
        match op {
            BinaryOperator::LogicalAnd => {
                let false_label: Rc<String> =
                    Rc::from(format!("{}{}_false", self.name, self.body.label_count));
                self.body.label_count += 1;
                let end_label: Rc<String> =
                    Rc::from(format!("{}{}_end", self.name, self.body.label_count));
                self.body.label_count += 1;

                // Short circuiting
                left.accept(self)?;
                let left_operand = Rc::clone(&self.result);
                self.body.add_instruction(
                    line_number.clone(),
                    JumpIfZero {
                        label: false_label.clone(),
                        operand: left_operand,
                    },
                ); // goto false

                right.accept(self)?;
                let right_operand = Rc::clone(&self.result);
                self.body.add_instruction(
                    line_number.clone(),
                    JumpIfZero {
                        label: false_label.clone(),
                        operand: right_operand,
                    },
                ); // goto false

                let dest = Rc::new(Pseudoregister::new(
                    (*self.name).clone(),
                    self.body.variable_count,
                ));
                self.body.add_instruction(
                    line_number.clone(),
                    StoreValueInstruction {
                        dest: Rc::clone(&dest),
                        src: Rc::new(Operand::Immediate(1)),
                    },
                );
                self.body.add_instruction(
                    line_number.clone(),
                    Jump {
                        label: end_label.clone(),
                    },
                ); // goto end

                // false label
                self.body.add_instruction(
                    line_number.clone(),
                    Label {
                        label: false_label.clone(),
                    },
                );
                self.body.add_instruction(
                    line_number.clone(),
                    StoreValueInstruction {
                        dest: Rc::clone(&dest),
                        src: Rc::new(Operand::Immediate(0)),
                    },
                );

                // end
                self.body.add_instruction(
                    line_number.clone(),
                    Label {
                        label: end_label.clone(),
                    },
                );
                self.result = Rc::from(Operand::Register(dest));
                Ok(())
            }
            BinaryOperator::LogicalOr => todo!(),
            _ => {
                left.accept(self)?;
                let left = Rc::clone(&self.result);

                right.accept(self)?;
                let right = Rc::clone(&self.result);

                let dest = Rc::new(Pseudoregister::new(
                    (*self.name).clone(),
                    self.body.variable_count,
                ));
                self.body.variable_count += 1;
                self.body.add_instruction(
                    line_number.clone(),
                    BinaryOpInstruction {
                        dest: Rc::clone(&dest),
                        op: *op,
                        left,
                        right,
                    },
                );
                self.result = Rc::from(Operand::Register(dest));
                Ok(())
            }
        }
    }

    fn visit_condition(
        &mut self,
        line_number: &Rc<Position>,
        condition: &mut Box<ASTNode>,
        if_true: &mut Option<Box<ASTNode>>,
        if_false: &mut Option<Box<ASTNode>>,
    ) -> Result<(), CompilerError> {
        todo!()
    }

    fn visit_while(
        &mut self,
        line_number: &Rc<Position>,
        condition: &mut Box<ASTNode>,
        body: &mut Option<Box<ASTNode>>,
        label: &mut Rc<String>,
        is_do_while: &mut bool,
    ) -> Result<(), CompilerError> {
        todo!()
    }

    fn visit_break(
        &mut self,
        line_number: &Rc<Position>,
        label: &mut Rc<String>,
    ) -> Result<(), CompilerError> {
        todo!()
    }

    fn visit_continue(
        &mut self,
        line_number: &Rc<Position>,
        label: &mut Rc<String>,
        is_for: &mut bool,
    ) -> Result<(), CompilerError> {
        todo!()
    }

    fn visit_for(
        &mut self,
        line_number: &Rc<Position>,
        init: &mut Option<Box<ASTNode>>,
        condition: &mut Option<Box<ASTNode>>,
        increment: &mut Option<Box<ASTNode>>,
        body: &mut Option<Box<ASTNode>>,
        label: &mut Rc<String>,
    ) -> Result<(), CompilerError> {
        todo!()
    }

    fn visit_const(
        &mut self,
        line_number: &Rc<Position>,
        value: &mut Number,
    ) -> Result<(), CompilerError> {
        todo!()
    }

    fn visit_variable(
        &mut self,
        line_number: &Rc<Position>,
        identifier: &mut Rc<String>,
    ) -> Result<(), CompilerError> {
        todo!()
    }

    fn visit_prefix(
        &mut self,
        line_number: &Rc<Position>,
        variable: &mut Box<ASTNode>,
        operator: &mut UnaryOperator,
    ) -> Result<(), CompilerError> {
        todo!()
    }

    fn visit_postfix(
        &mut self,
        line_number: &Rc<Position>,
        variable: &mut Box<ASTNode>,
        operator: &mut UnaryOperator,
    ) -> Result<(), CompilerError> {
        todo!()
    }
}
