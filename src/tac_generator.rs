use crate::ast::{ASTNode, Block, Declaration, Expression, ForInit, Statement, Visitor};
use crate::common::{Const, Identifier, Position};
use crate::errors::CompilerError;
use crate::errors::CompilerError::SemanticError;
use crate::lexer::{BinaryOperator, StorageClass, Type, UnaryOperator};
use crate::tac::TACInstruction::{
    AdjustStack, AllocateStackInstruction, BinaryOpInstruction,
    FunctionCall, FunctionInstruction, Jump, JumpIfNotZero, JumpIfZero, Label, PushArgument,
    ReturnInstruction, SignExtend, StoreValueInstruction, Truncate, UnaryOpInstruction,
};
use crate::tac::{FunctionBody, Operand, Pseudoregister};
use std::rc::Rc;

const FIRST_SIX_REGISTERS: [&str; 6] = ["edi", "esi", "edx", "ecx", "r8d", "r9d"];

pub(crate) struct TacVisitor<'a> {
    name: Rc<String>,
    body: &'a mut FunctionBody,
    result: Rc<Operand>,
    label_count: i32,
}

impl<'a> TacVisitor<'a> {
    pub(crate) fn new(name: Rc<String>, body: &'a mut FunctionBody) -> Self {
        Self {
            name,
            body,
            result: Rc::new(Operand::None),
            label_count: 0,
        }
    }
}

impl<'a> Visitor for TacVisitor<'a> {
    fn visit_declaration(
        &mut self,
        _line_number: &Rc<Position>,
        declaration: &mut Declaration,
    ) -> Result<(), CompilerError> {
        match declaration {
            Declaration::VariableDeclaration(v) => {
                if v.storage_class.is_some() {
                    return Ok(());
                }
                let (identifier, expression) = (&v.name, &mut v.init);
                let pseudoregister = Rc::from(Pseudoregister::new(self.body.variable_count));
                self.body
                    .variable_to_pseudoregister
                    .insert(identifier.as_ref().to_string(), Rc::clone(&pseudoregister));
                if let Some(expression) = expression {
                    expression.accept(self)?;
                    self.body.add_instruction(StoreValueInstruction {
                        dest: Rc::clone(&pseudoregister),
                        src: Rc::clone(&self.result),
                    });
                }
                self.body.variable_count += 1;
                Ok(())
            }
            Declaration::FunctionDeclaration(func) => {
                if let Some(body) = &mut func.body {
                    self.body.add_instruction(FunctionInstruction {
                        name: Rc::clone(&func.name),
                        global: func.storage_class != Some(StorageClass::Static),
                    });
                    self.body.add_instruction(AllocateStackInstruction);

                    for (i, param) in func.params.iter().enumerate() {
                        let param_register =
                            Rc::new(Pseudoregister::Pseudoregister(self.body.variable_count));
                        self.body.variable_count += 1;

                        self.body
                            .variable_to_pseudoregister
                            .insert(param.to_string(), Rc::clone(&param_register));

                        if i < 6 {
                            self.body.add_instruction(StoreValueInstruction {
                                dest: Rc::clone(&param_register),
                                src: Rc::from(Operand::Register(Pseudoregister::Register(
                                    FIRST_SIX_REGISTERS[i].to_string(),
                                ))),
                            });
                        } else {
                            let stack_offset = 16 + (i - 6) * 8;
                            // Option 1: Create a new MemoryReference variant
                            self.body.add_instruction(StoreValueInstruction {
                                dest: Rc::clone(&param_register),
                                src: Rc::from(Operand::MemoryReference(
                                    stack_offset,
                                    "rbp".to_string(),
                                )),
                            });
                        }
                    }

                    body.accept(self)?;
                    //self.body.add_instruction(DeallocateStackInstruction);
                }
                Ok(())
            }
        }
    }

    fn visit_assignment(
        &mut self,
        line_number: &Rc<Position>,
        left: &mut Box<ASTNode<Expression>>,
        right: &mut Box<ASTNode<Expression>>,
        _type_: &mut Type,
    ) -> Result<(), CompilerError> {
        left.accept(self)?;
        let dest = Rc::clone(&self.result);
        right.accept(self)?;
        let src = Rc::clone(&self.result);
        match dest.as_ref() {
            Operand::Register(variable) => {
                let dest_registry: Rc<Pseudoregister> = Rc::new((*variable).clone());
                self.body.add_instruction(StoreValueInstruction {
                    dest: dest_registry,
                    src,
                });
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
        _line_number: &Rc<Position>,
        expression: &mut ASTNode<Expression>,
    ) -> Result<(), CompilerError> {
        expression.accept(self)?;
        self.body.add_instruction(ReturnInstruction {
            val: Rc::clone(&self.result),
        });
        Ok(())
    }

    fn visit_block(
        &mut self,
        _line_number: &Rc<Position>,
        body: &mut Block,
    ) -> Result<(), CompilerError> {
        for item in body {
            item.accept(self)?;
        }
        Ok(())
    }

    fn visit_unary(
        &mut self,
        _line_number: &Rc<Position>,
        op: &mut UnaryOperator,
        expression: &mut Box<ASTNode<Expression>>,
        _type_: &mut Type,
    ) -> Result<(), CompilerError> {
        expression.accept(self)?;
        if *op == UnaryOperator::UnaryAdd {
            return Ok(());
        }
        let src = Rc::clone(&self.result);
        let dest = Rc::new(Pseudoregister::new(self.body.variable_count));
        self.body.variable_count += 1;
        self.body.add_instruction(UnaryOpInstruction {
            dest: Rc::clone(&dest),
            op: *op,
            operand: src,
        });
        self.result = Rc::from(Operand::Register((*dest).clone()));
        Ok(())
    }

    fn visit_binary(
        &mut self,
        _line_number: &Rc<Position>,
        op: &mut BinaryOperator,
        left: &mut Box<ASTNode<Expression>>,
        right: &mut Box<ASTNode<Expression>>,
        _type_: &mut Type,
    ) -> Result<(), CompilerError> {
        match op {
            BinaryOperator::LogicalAnd => {
                let false_label: Rc<String> =
                    Rc::from(format!(".{}{}_false", self.name, self.label_count));
                self.label_count += 1;
                let end_label: Rc<String> =
                    Rc::from(format!(".{}{}_end", self.name, self.label_count));
                self.label_count += 1;

                // Short-circuiting
                left.accept(self)?;
                let left_operand = Rc::clone(&self.result);
                self.body.add_instruction(JumpIfZero {
                    label: Rc::clone(&false_label),
                    operand: left_operand,
                }); // goto false

                right.accept(self)?;
                let right_operand = Rc::clone(&self.result);
                self.body.add_instruction(JumpIfZero {
                    label: Rc::clone(&false_label),
                    operand: right_operand,
                }); // goto false

                let dest = Rc::new(Pseudoregister::new(self.body.variable_count));
                self.body.add_instruction(StoreValueInstruction {
                    dest: Rc::clone(&dest),
                    src: Rc::new(Operand::Immediate(1.into())),
                });
                self.body.add_instruction(Jump {
                    label: Rc::clone(&end_label),
                }); // goto end

                // false label
                self.body.add_instruction(Label {
                    label: Rc::clone(&false_label),
                });
                self.body.add_instruction(StoreValueInstruction {
                    dest: Rc::clone(&dest),
                    src: Rc::new(Operand::Immediate(0.into())),
                });

                // end
                self.body.add_instruction(Label {
                    label: Rc::clone(&end_label),
                });
                self.result = Rc::from(Operand::Register((*dest).clone()));
                Ok(())
            }
            BinaryOperator::LogicalOr => {
                let true_label: Rc<String> =
                    Rc::from(format!(".{}{}_true", self.name, self.label_count));
                self.label_count += 1;
                let end_label: Rc<String> =
                    Rc::from(format!(".{}{}_end", self.name, self.label_count));
                self.label_count += 1;

                left.accept(self)?;
                let left_operand = Rc::clone(&self.result);
                self.body.add_instruction(JumpIfNotZero {
                    // goto true
                    label: Rc::clone(&true_label),
                    operand: left_operand,
                });

                right.accept(self)?;
                let right_operand = Rc::clone(&self.result);
                self.body.add_instruction(JumpIfNotZero {
                    label: Rc::clone(&true_label),
                    operand: right_operand,
                }); // goto true

                let dest = Rc::new(Pseudoregister::new(self.body.variable_count));
                self.body.add_instruction(StoreValueInstruction {
                    dest: Rc::clone(&dest),
                    src: Rc::new(Operand::Immediate(0.into())),
                });
                self.body.add_instruction(
                    // goto end
                    Jump {
                        label: Rc::clone(&end_label),
                    },
                );

                self.body.add_instruction(Label {
                    label: Rc::clone(&true_label),
                }); // true
                self.body.add_instruction(StoreValueInstruction {
                    dest: Rc::clone(&dest),
                    src: Rc::new(Operand::Immediate(1.into())),
                });
                //end
                self.body.add_instruction(Label {
                    label: Rc::clone(&end_label),
                });
                self.body.variable_count += 1;
                self.result = Rc::from(Operand::Register((*dest).clone()));
                Ok(())
            }
            _ => {
                left.accept(self)?;
                let left = Rc::clone(&self.result);

                right.accept(self)?;
                let right = Rc::clone(&self.result);

                let dest = Rc::new(Pseudoregister::new(self.body.variable_count));
                self.body.variable_count += 1;
                self.body.add_instruction(BinaryOpInstruction {
                    dest: Rc::clone(&dest),
                    op: *op,
                    left,
                    right,
                });
                self.result = Rc::from(Operand::Register((*dest).clone()));
                Ok(())
            }
        }
    }

    fn visit_condition(
        &mut self,
        _line_number: &Rc<Position>,
        condition: &mut Box<ASTNode<Expression>>,
        if_true: &mut Box<ASTNode<Expression>>,
        if_false: &mut Box<ASTNode<Expression>>,
        _type_: &mut Type,
    ) -> Result<(), CompilerError> {
        condition.accept(self)?;
        let else_label: Rc<String> = Rc::from(format!(".{}{}_else", self.name, self.label_count));
        self.label_count += 1;
        let end_label: Rc<String> = Rc::from(format!(".{}{}_end", self.name, self.label_count));
        self.label_count += 1;
        let dest = Rc::new(Pseudoregister::new(self.body.variable_count));
        self.body.add_instruction(JumpIfZero {
            // if false goto else
            label: Rc::clone(&else_label),
            operand: Rc::clone(&self.result),
        });
        if_true.accept(self)?;
        self.body.add_instruction(StoreValueInstruction {
            dest: Rc::clone(&dest),
            src: Rc::clone(&self.result),
        });
        self.body.add_instruction(Jump {
            label: Rc::clone(&end_label),
        }); // goto end
        self.body.add_instruction(Label {
            label: Rc::clone(&else_label),
        }); // else
        if_false.accept(self)?;
        self.body.add_instruction(StoreValueInstruction {
            dest: Rc::clone(&dest),
            src: Rc::clone(&self.result),
        });
        self.body.add_instruction(Label {
            label: Rc::clone(&end_label),
        });
        self.result = Rc::from(Operand::Register((*dest).clone()));
        Ok(())
    }

    fn visit_while(
        &mut self,
        _line_number: &Rc<Position>,
        condition: &mut ASTNode<Expression>,
        body: &mut Box<ASTNode<Statement>>,
        label: &mut Rc<String>,
        is_do_while: &mut bool,
    ) -> Result<(), CompilerError> {
        let start_label: Rc<String> = Rc::from(format!(".{}{}_start.loop", self.name, label));
        let end_label: Rc<String> = Rc::from(format!(".{}{}_end.loop", self.name, label));
        if !*is_do_while {
            self.body.add_instruction(
                // start
                Label {
                    label: Rc::clone(&start_label),
                },
            );
            condition.accept(self)?;
            self.body.add_instruction(JumpIfZero {
                // if false goto end
                label: Rc::clone(&end_label),
                operand: Rc::clone(&self.result),
            });
            body.accept(self)?;
            self.body.add_instruction(Jump {
                label: Rc::clone(&start_label),
            }); // goto start
            self.body.add_instruction(Label {
                label: Rc::clone(&end_label),
            }); // end
            self.result = Rc::from(Operand::None);
        } else {
            self.body.add_instruction(Label {
                label: Rc::clone(&start_label),
            }); // start
            body.accept(self)?;
            condition.accept(self)?;
            self.body.add_instruction(JumpIfZero {
                label: Rc::clone(&end_label),
                operand: Rc::clone(&self.result),
            }); // if false goto end
            self.body.add_instruction(Jump {
                label: Rc::clone(&start_label),
            }); // goto start
            self.body.add_instruction(Label {
                label: Rc::clone(&end_label),
            });
        }
        Ok(())
    }

    fn visit_break(
        &mut self,
        _line_number: &Rc<Position>,
        label: &mut Rc<String>,
    ) -> Result<(), CompilerError> {
        self.body.add_instruction(Jump {
            label: format!(".{}{}_end.loop", self.name, label).into(),
        });
        self.result = Rc::from(Operand::None);
        Ok(())
    }

    fn visit_continue(
        &mut self,
        _line_number: &Rc<Position>,
        label: &mut Rc<String>,
        is_for: &mut bool,
    ) -> Result<(), CompilerError> {
        if *is_for {
            self.body.add_instruction(Jump {
                label: format!(".{}{}_increment.loop", self.name, label).into(),
            });
        } else {
            self.body.add_instruction(Jump {
                label: format!(".{}{}_start.loop", self.name, label).into(),
            });
        }
        self.result = Rc::from(Operand::None);
        Ok(())
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
        let start_label: Rc<String> = Rc::from(format!(".{}{}_start.loop", self.name, label));
        let end_label: Rc<String> = Rc::from(format!(".{}{}_end.loop", self.name, label));
        let increment_label: Rc<String> =
            Rc::from(format!(".{}{}_increment.loop", self.name, label));
        init.accept(self)?;
        self.body.add_instruction(
            // start
            Label {
                label: Rc::clone(&start_label),
            },
        );
        if let Some(condition) = condition {
            condition.accept(self)?;
            self.body.add_instruction(JumpIfZero {
                // if false goto end
                label: Rc::clone(&end_label),
                operand: Rc::clone(&self.result),
            });
        }
        body.accept(self)?;
        self.body.add_instruction(Label {
            label: Rc::clone(&increment_label),
        }); // increment
        if let Some(increment) = increment {
            increment.accept(self)?;
        }
        self.body.add_instruction(Jump {
            label: Rc::clone(&start_label),
        }); // goto start
        self.body.add_instruction(Label {
            label: Rc::clone(&end_label),
        }); // end
        self.result = Rc::from(Operand::None);
        Ok(())
    }

    fn visit_const(
        &mut self,
        _line_number: &Rc<Position>,
        value: &mut Const,
        _type_: &mut Type,
    ) -> Result<(), CompilerError> {
        self.result = Rc::from(Operand::Immediate(value.clone()));
        Ok(())
    }

    fn visit_variable(
        &mut self,
        _line_number: &Rc<Position>,
        identifier: &mut Rc<Identifier>,
        _node: &mut Type,
    ) -> Result<(), CompilerError> {
        if let Some(pseudoregister) = self
            .body
            .variable_to_pseudoregister
            .get(&identifier.to_string())
        {
            self.result = Rc::from(Operand::Register((**pseudoregister).clone()));
            return Ok(());
        }

        // static
        self.result = Rc::from(Operand::Register(Pseudoregister::Data(Rc::clone(
            &identifier,
        ))));
        Ok(())
    }

    fn visit_function_call(
        &mut self,
        _line_number: &Rc<Position>,
        identifier: &mut Rc<Identifier>,
        arguments: &mut Box<Vec<ASTNode<Expression>>>,
        _ret_type: &mut Type,
    ) -> Result<(), CompilerError> {
        for i in (6..arguments.len()).rev() {
            arguments[i].accept(self)?;
            self.body
                .add_instruction(PushArgument(Rc::clone(&self.result)));
        }

        for i in 0..arguments.len().min(6) {
            arguments[i].accept(self)?;
            self.body.add_instruction(StoreValueInstruction {
                dest: Rc::from(Pseudoregister::Register(FIRST_SIX_REGISTERS[i].to_string())),
                src: Rc::clone(&self.result),
            });
        }

        self.body
            .add_instruction(FunctionCall(Rc::clone(&identifier)));

        if arguments.len() > 6 {
            let stack_cleanup_size = (arguments.len() - 6) * 4; // 4 bytes per arg
            self.body.add_instruction(AdjustStack(stack_cleanup_size));
        }

        let result_register = Rc::new(Pseudoregister::Pseudoregister(self.body.variable_count));
        self.body.variable_count += 1;

        self.body.add_instruction(StoreValueInstruction {
            dest: Rc::clone(&result_register),
            src: Rc::from(Operand::Register(Pseudoregister::Register(
                "eax".to_string(),
            ))),
        });

        self.result = Rc::from(Operand::Register((*result_register).clone()));

        Ok(())
    }

    fn visit_prefix(
        &mut self,
        line_number: &Rc<Position>,
        variable: &mut Box<ASTNode<Expression>>,
        operator: &mut UnaryOperator,
        _type_: &mut Type,
    ) -> Result<(), CompilerError> {
        let binary_operator = if *operator == UnaryOperator::Increment {
            BinaryOperator::Addition
        } else {
            BinaryOperator::Subtraction
        };
        variable.accept(self)?;
        match &*self.result {
            Operand::Register(pseudoregister) => {
                self.body.add_instruction(BinaryOpInstruction {
                    dest: Rc::from((*pseudoregister).clone()),
                    op: binary_operator,
                    left: Rc::clone(&self.result),
                    right: Rc::from(Operand::Immediate(1.into())),
                });
                self.body.variable_count += 1;
                Ok(())
            }
            _ => Err(SemanticError(format!(
                "Expected lvalue at {:?}",
                line_number
            ))),
        }
    }

    fn visit_postfix(
        &mut self,
        line_number: &Rc<Position>,
        variable: &mut Box<ASTNode<Expression>>,
        operator: &mut UnaryOperator,
        _type_: &mut Type,
    ) -> Result<(), CompilerError> {
        let binary_operator = if *operator == UnaryOperator::Increment {
            BinaryOperator::Addition
        } else {
            BinaryOperator::Subtraction
        };
        variable.accept(self)?;
        let dest = match &*self.result {
            Operand::Register(pseudoregister) => Rc::from((*pseudoregister).clone()),
            _ => {
                return Err(SemanticError(format!(
                    "Expected lvalue at {:?}",
                    line_number
                )));
            }
        };
        let temp1 = Rc::new(Pseudoregister::new(self.body.variable_count));
        self.body.variable_count += 1;
        self.body.add_instruction(StoreValueInstruction {
            dest: Rc::clone(&temp1),
            src: Rc::clone(&self.result),
        });
        self.body.add_instruction(BinaryOpInstruction {
            dest: Rc::clone(&dest),
            op: binary_operator,
            left: Rc::clone(&self.result),
            right: Rc::from(Operand::Immediate(1.into())),
        });
        self.result = Rc::from(Operand::Register((*temp1).clone()));
        Ok(())
    }

    fn visit_if_else(
        &mut self,
        _line_number: &Rc<Position>,
        condition: &mut ASTNode<Expression>,
        if_true: &mut Box<ASTNode<Statement>>,
        if_false: &mut Option<Box<ASTNode<Statement>>>,
    ) -> Result<(), CompilerError> {
        match if_false {
            None => {
                condition.accept(self)?;
                let end_label: Rc<String> =
                    Rc::from(format!(".{}{}_end", self.name, self.label_count));
                self.label_count += 1;
                self.body.add_instruction(JumpIfZero {
                    // if false goto end
                    label: Rc::clone(&end_label),
                    operand: Rc::clone(&self.result),
                });
                if_true.accept(self)?;
                self.body.add_instruction(Label {
                    label: Rc::clone(&end_label),
                });
            }
            Some(if_false) => {
                condition.accept(self)?;
                let else_label: Rc<String> =
                    Rc::from(format!(".{}{}_else", self.name, self.label_count));
                self.label_count += 1;
                let end_label: Rc<String> =
                    Rc::from(format!(".{}{}_end", self.name, self.label_count));
                self.label_count += 1;
                self.body.add_instruction(JumpIfZero {
                    // if false goto else
                    label: Rc::clone(&else_label),
                    operand: Rc::clone(&self.result),
                });
                if_true.accept(self)?;

                self.body.add_instruction(Jump {
                    label: Rc::clone(&end_label),
                }); // goto end
                self.body.add_instruction(Label {
                    label: Rc::clone(&else_label),
                }); // else
                if_false.accept(self)?;
                self.body.add_instruction(Label {
                    label: Rc::clone(&end_label),
                });
            }
        };
        self.result = Rc::from(Operand::None);
        Ok(())
    }

    fn visit_cast(
        &mut self,
        _line_number: &Rc<Position>,
        target_type: &mut Type,
        exp: &mut Box<ASTNode<Expression>>,
    ) -> Result<(), CompilerError> {
        exp.accept(self)?;
        if *target_type == exp.type_ {
            return Ok(());
        }
        let dest = Rc::from(Pseudoregister::new(self.body.variable_count));
        self.body.variable_count += 1;
        if *target_type == Type::Long {
            self.body.add_instruction(SignExtend {
                dest,
                src: Rc::clone(&self.result),
            });
        } else {
            self.body.add_instruction(Truncate {
                dest,
                src: Rc::clone(&self.result),
            })
        }
        Ok(())
    }
}
