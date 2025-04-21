use crate::ast::{ASTNode, Block, Declaration, Expression, ForInit, Program, Statement, Visitor};
use crate::common::{Identifier, Operand, Position, Pseudoregister};
use crate::errors::CompilerError;
use crate::errors::CompilerError::SemanticError;
use crate::lexer::{BinaryOperator, Number, UnaryOperator};
use crate::tac::FunctionBody;
use crate::tac::TACInstruction::{
    AllocateStackInstruction, BinaryOpInstruction, FunctionInstruction, Jump, JumpIfNotZero,
    JumpIfZero, Label, ReturnInstruction, StoreValueInstruction, UnaryOpInstruction,
};
use std::rc::Rc;

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
    fn visit_program(
        &mut self,
        _line_number: &Rc<Position>,
        _function_declaration: &mut Program,
    ) -> Result<(), CompilerError> {
        panic!("Should not be called")
    }

    fn visit_declaration(
        &mut self,
        _line_number: &Rc<Position>,
        declaration: &mut Declaration,
    ) -> Result<(), CompilerError> {
        match declaration {
            Declaration::VariableDeclaration(v) => {
                let (identifier, expression) = (&v.kind.name, &mut v.kind.init);
                let pseudoregister = Rc::new(Pseudoregister::new(
                    (*self.name).clone(),
                    self.body.variable_count,
                ));
                self.body.variable_to_pseudoregister.insert(
                    (*Rc::clone(&identifier)).clone(),
                    Rc::clone(&pseudoregister),
                );
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
                self.body.add_instruction(FunctionInstruction {
                    name: Rc::clone(&func.kind.name),
                });
                self.body.add_instruction(AllocateStackInstruction);
                func.kind.body.accept(self)
            }
        }
    }

    fn visit_assignment(
        &mut self,
        line_number: &Rc<Position>,
        left: &mut Box<ASTNode<Expression>>,
        right: &mut Box<ASTNode<Expression>>,
    ) -> Result<(), CompilerError> {
        left.accept(self)?;
        let dest = Rc::clone(&self.result);
        right.accept(self)?;
        let src = Rc::clone(&self.result);
        match dest.as_ref() {
            Operand::Register(variable) => {
                let dest_registry: Rc<Pseudoregister> = Rc::clone(variable);
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
        self.body.add_instruction(UnaryOpInstruction {
            dest: Rc::clone(&dest),
            op: *op,
            operand: src,
        });
        self.result = Rc::from(Operand::Register(Rc::clone(&dest)));
        Ok(())
    }

    fn visit_binary(
        &mut self,
        _line_number: &Rc<Position>,
        op: &mut BinaryOperator,
        left: &mut Box<ASTNode<Expression>>,
        right: &mut Box<ASTNode<Expression>>,
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

                let dest = Rc::new(Pseudoregister::new(
                    (*self.name).clone(),
                    self.body.variable_count,
                ));
                self.body.add_instruction(StoreValueInstruction {
                    dest: Rc::clone(&dest),
                    src: Rc::new(Operand::Immediate(1)),
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
                    src: Rc::new(Operand::Immediate(0)),
                });

                // end
                self.body.add_instruction(Label {
                    label: Rc::clone(&end_label),
                });
                self.result = Rc::from(Operand::Register(dest));
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

                let dest = Rc::new(Pseudoregister::new(
                    (*self.name).clone(),
                    self.body.variable_count,
                ));
                self.body.add_instruction(StoreValueInstruction {
                    dest: Rc::clone(&dest),
                    src: Rc::new(Operand::Immediate(0)),
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
                    src: Rc::new(Operand::Immediate(1)),
                });
                //end
                self.body.add_instruction(Label {
                    label: Rc::clone(&end_label),
                });
                self.body.variable_count += 1;
                self.result = Rc::from(Operand::Register(dest));
                Ok(())
            }
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
                self.body.add_instruction(BinaryOpInstruction {
                    dest: Rc::clone(&dest),
                    op: *op,
                    left,
                    right,
                });
                self.result = Rc::from(Operand::Register(dest));
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
    ) -> Result<(), CompilerError> {
        condition.accept(self)?;
        let else_label: Rc<String> = Rc::from(format!(".{}{}_else", self.name, self.label_count));
        self.label_count += 1;
        let end_label: Rc<String> = Rc::from(format!(".{}{}_end", self.name, self.label_count));
        self.label_count += 1;
        let dest = Rc::new(Pseudoregister::new(
            (*self.name).clone(),
            self.body.variable_count,
        ));
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
        self.result = Rc::from(Operand::Register(dest));
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
        value: &mut Number,
    ) -> Result<(), CompilerError> {
        self.result = Rc::from(Operand::Immediate(*value));
        Ok(())
    }

    fn visit_variable(
        &mut self,
        _line_number: &Rc<Position>,
        identifier: &mut Rc<String>,
    ) -> Result<(), CompilerError> {
        let pseudoregister = self
            .body
            .variable_to_pseudoregister
            .get(&(*Rc::clone(&identifier)).clone())
            .unwrap();
        self.result = Rc::from(Operand::Register(pseudoregister.clone()));
        Ok(())
    }

    fn visit_function_call(
        &mut self,
        line_number: &Rc<Position>,
        identifier: &mut Rc<Identifier>,
        arguments: &mut Box<Vec<ASTNode<Expression>>>,
    ) -> Result<(), CompilerError> {
        todo!()
    }

    fn visit_prefix(
        &mut self,
        line_number: &Rc<Position>,
        variable: &mut Box<ASTNode<Expression>>,
        operator: &mut UnaryOperator,
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
                    dest: Rc::clone(&pseudoregister),
                    op: binary_operator,
                    left: Rc::clone(&self.result),
                    right: Rc::from(Operand::Immediate(1)),
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
    ) -> Result<(), CompilerError> {
        let binary_operator = if *operator == UnaryOperator::Increment {
            BinaryOperator::Addition
        } else {
            BinaryOperator::Subtraction
        };
        variable.accept(self)?;
        let dest = match &*self.result {
            Operand::Register(pseudoregister) => Rc::clone(&pseudoregister),
            _ => {
                return Err(SemanticError(format!(
                    "Expected lvalue at {:?}",
                    line_number
                )));
            }
        };
        let temp1 = Rc::new(Pseudoregister::new(
            (*self.name).clone(),
            self.body.variable_count,
        ));
        self.body.variable_count += 1;
        self.body.add_instruction(StoreValueInstruction {
            dest: Rc::clone(&temp1),
            src: Rc::clone(&self.result),
        });
        self.body.add_instruction(BinaryOpInstruction {
            dest: Rc::clone(&dest),
            op: binary_operator,
            left: Rc::clone(&self.result),
            right: Rc::from(Operand::Immediate(1)),
        });
        self.result = Rc::from(Operand::Register(temp1));
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
}
