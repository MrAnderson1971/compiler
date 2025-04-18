use crate::ast::{ASTNode, Visitor};
use crate::common::{Operand, Position, Pseudoregister};
use crate::errors::CompilerError;
use crate::errors::CompilerError::SemanticError;
use crate::lexer::BinaryOperator::Addition;
use crate::lexer::{BinaryOperator, Number, UnaryOperator};
use crate::tac::FunctionBody;
use crate::tac::TACInstructionType::{
    BinaryOpInstruction, FunctionInstruction, Jump, JumpIfNotZero, JumpIfZero, Label,
    ReturnInstruction, StoreValueInstruction, UnaryOpInstruction,
};
use std::rc::Rc;

pub struct TacVisitor<'a> {
    name: Rc<String>,
    body: &'a mut FunctionBody,
    result: Rc<Operand>,
    label_count: i32,
}

impl<'a> TacVisitor<'a> {
    pub fn new(name: Rc<String>, body: &'a mut FunctionBody) -> Self {
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
                name: Rc::clone(&identifier),
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
        self.body.variable_to_pseudoregister.insert(
            (*Rc::clone(&identifier)).clone(),
            Rc::clone(&pseudoregister),
        );
        if let Some(expression) = expression {
            expression.accept(self)?;
            self.body.add_instruction(
                Rc::clone(&line_number),
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
                    Rc::clone(&line_number),
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
                Rc::clone(&line_number),
                ReturnInstruction {
                    val: Rc::clone(&self.result),
                },
            );
        } else {
            self.body.add_instruction(
                Rc::clone(&line_number),
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
            Rc::clone(&line_number),
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
                    Rc::from(format!("{}{}_false", self.name, self.label_count));
                self.label_count += 1;
                let end_label: Rc<String> =
                    Rc::from(format!("{}{}_end", self.name, self.label_count));
                self.label_count += 1;

                // Short circuiting
                left.accept(self)?;
                let left_operand = Rc::clone(&self.result);
                self.body.add_instruction(
                    Rc::clone(&line_number),
                    JumpIfZero {
                        label: Rc::clone(&false_label),
                        operand: left_operand,
                    },
                ); // goto false

                right.accept(self)?;
                let right_operand = Rc::clone(&self.result);
                self.body.add_instruction(
                    Rc::clone(&line_number),
                    JumpIfZero {
                        label: Rc::clone(&false_label),
                        operand: right_operand,
                    },
                ); // goto false

                let dest = Rc::new(Pseudoregister::new(
                    (*self.name).clone(),
                    self.body.variable_count,
                ));
                self.body.add_instruction(
                    Rc::clone(&line_number),
                    StoreValueInstruction {
                        dest: Rc::clone(&dest),
                        src: Rc::new(Operand::Immediate(1)),
                    },
                );
                self.body.add_instruction(
                    Rc::clone(&line_number),
                    Jump {
                        label: Rc::clone(&end_label),
                    },
                ); // goto end

                // false label
                self.body.add_instruction(
                    Rc::clone(&line_number),
                    Label {
                        label: Rc::clone(&false_label),
                    },
                );
                self.body.add_instruction(
                    Rc::clone(&line_number),
                    StoreValueInstruction {
                        dest: Rc::clone(&dest),
                        src: Rc::new(Operand::Immediate(0)),
                    },
                );

                // end
                self.body.add_instruction(
                    Rc::clone(&line_number),
                    Label {
                        label: Rc::clone(&end_label),
                    },
                );
                self.result = Rc::from(Operand::Register(dest));
                Ok(())
            }
            BinaryOperator::LogicalOr => {
                let true_label: Rc<String> =
                    Rc::from(format!(".{}{}_true", self.name, self.body.variable_count));
                self.body.variable_count += 1;
                let end_label: Rc<String> =
                    Rc::from(format!(".{}{}_end", self.name, self.body.variable_count));
                self.body.variable_count += 1;

                left.accept(self)?;
                let left_operand = Rc::clone(&self.result);
                self.body.add_instruction(
                    Rc::clone(&line_number),
                    JumpIfNotZero {
                        // goto true
                        label: Rc::clone(&true_label),
                        operand: left_operand,
                    },
                );

                right.accept(self)?;
                let right_operand = Rc::clone(&self.result);
                self.body.add_instruction(
                    Rc::clone(&line_number),
                    JumpIfNotZero {
                        label: Rc::clone(&true_label),
                        operand: right_operand,
                    },
                ); // goto true

                let dest = Rc::new(Pseudoregister::new(
                    (*self.name).clone(),
                    self.body.variable_count,
                ));
                self.body.add_instruction(
                    Rc::clone(line_number),
                    StoreValueInstruction {
                        dest: Rc::clone(&dest),
                        src: Rc::new(Operand::Immediate(0)),
                    },
                );
                self.body.add_instruction(
                    // goto end
                    Rc::clone(&line_number),
                    Jump {
                        label: Rc::clone(&end_label),
                    },
                );

                self.body.add_instruction(
                    Rc::clone(&line_number),
                    Label {
                        label: Rc::clone(&true_label),
                    },
                ); // true
                self.body.add_instruction(
                    Rc::clone(&line_number),
                    StoreValueInstruction {
                        dest: Rc::clone(&dest),
                        src: Rc::new(Operand::Immediate(1)),
                    },
                );
                //end
                self.body.add_instruction(
                    Rc::clone(&line_number),
                    Label {
                        label: Rc::clone(&end_label),
                    },
                );
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
                self.body.add_instruction(
                    Rc::clone(&line_number),
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
        is_ternary: &mut bool,
    ) -> Result<(), CompilerError> {
        if *is_ternary {
            condition.accept(self)?;
            let else_label: Rc<String> =
                Rc::from(format!(".{}{}_else", self.name, self.label_count));
            self.label_count += 1;
            let end_label: Rc<String> = Rc::from(format!(".{}{}_end", self.name, self.label_count));
            self.label_count += 1;
            let dest = Rc::new(Pseudoregister::new(
                (*self.name).clone(),
                self.body.variable_count,
            ));
            self.body.add_instruction(
                Rc::clone(&line_number),
                JumpIfZero {
                    // if false goto else
                    label: Rc::clone(&else_label),
                    operand: Rc::clone(&self.result),
                },
            );
            if let Some(if_true) = if_true {
                if_true.accept(self)?;
            }
            self.body.add_instruction(
                Rc::clone(&line_number),
                StoreValueInstruction {
                    dest: Rc::clone(&dest),
                    src: Rc::clone(&self.result),
                },
            );
            self.body.add_instruction(
                Rc::clone(&line_number),
                Jump {
                    label: Rc::clone(&end_label),
                },
            ); // goto end
            self.body.add_instruction(
                Rc::clone(&line_number),
                Label {
                    label: Rc::clone(&else_label),
                },
            ); // else
            if let Some(if_false) = if_false {
                if_false.accept(self)?;
            }
            self.body.add_instruction(
                Rc::clone(&line_number),
                StoreValueInstruction {
                    dest: Rc::clone(&dest),
                    src: Rc::clone(&self.result),
                },
            );
            self.body.add_instruction(
                Rc::clone(&line_number),
                Label {
                    label: Rc::clone(&end_label),
                },
            );
            self.result = Rc::from(Operand::Register(dest));
            return Ok(());
        }
        match if_false {
            None => {
                condition.accept(self)?;
                let end_label: Rc<String> =
                    Rc::from(format!(".{}{}_end", self.name, self.label_count));
                self.label_count += 1;
                self.body.add_instruction(
                    Rc::clone(&line_number),
                    JumpIfZero {
                        // if false goto end
                        label: Rc::clone(&end_label),
                        operand: Rc::clone(&self.result),
                    },
                );
                if let Some(if_true) = if_true {
                    if_true.accept(self)?;
                }
                self.body.add_instruction(
                    Rc::clone(&line_number),
                    Label {
                        label: Rc::clone(&end_label),
                    },
                );
            }
            Some(if_false) => {
                condition.accept(self)?;
                let else_label: Rc<String> =
                    Rc::from(format!(".{}{}_else", self.name, self.label_count));
                self.label_count += 1;
                let end_label: Rc<String> =
                    Rc::from(format!(".{}{}_end", self.name, self.label_count));
                self.label_count += 1;
                self.body.add_instruction(
                    Rc::clone(&line_number),
                    JumpIfZero {
                        // if false goto else
                        label: Rc::clone(&else_label),
                        operand: Rc::clone(&self.result),
                    },
                );
                if let Some(if_true) = if_true {
                    if_true.accept(self)?;
                }

                self.body.add_instruction(
                    Rc::clone(&line_number),
                    Jump {
                        label: Rc::clone(&end_label),
                    },
                ); // goto end
                self.body.add_instruction(
                    Rc::clone(&line_number),
                    Label {
                        label: Rc::clone(&else_label),
                    },
                ); // else
                if_false.accept(self)?;
                self.body.add_instruction(
                    Rc::clone(&line_number),
                    Label {
                        label: Rc::clone(&end_label),
                    },
                );
            }
        };
        self.result = Rc::from(Operand::None);
        Ok(())
    }

    fn visit_while(
        &mut self,
        line_number: &Rc<Position>,
        condition: &mut Box<ASTNode>,
        body: &mut Option<Box<ASTNode>>,
        label: &mut Rc<String>,
        is_do_while: &mut bool,
    ) -> Result<(), CompilerError> {
        if !*is_do_while {
            let start_label: Rc<String> = Rc::from(format!(".{}{}_start.loop", self.name, label));
            let end_label: Rc<String> = Rc::from(format!(".{}{}_end.loop", self.name, label));
            self.body.add_instruction(
                // start
                Rc::clone(&line_number),
                Label {
                    label: Rc::clone(&start_label),
                },
            );
            condition.accept(self)?;
            self.body.add_instruction(
                Rc::clone(&line_number),
                JumpIfZero {
                    // if false goto end
                    label: Rc::clone(&end_label),
                    operand: Rc::clone(&self.result),
                },
            );
            if let Some(body) = body {
                body.accept(self)?;
            }
            self.body.add_instruction(
                Rc::clone(&line_number),
                Jump {
                    label: Rc::clone(&start_label),
                },
            ); // goto start
            self.body.add_instruction(
                Rc::clone(&line_number),
                Label {
                    label: Rc::clone(&end_label),
                },
            ); // end
            self.result = Rc::from(Operand::None);
            Ok(())
        } else {
            todo!()
        }
    }

    fn visit_break(
        &mut self,
        line_number: &Rc<Position>,
        label: &mut Rc<String>,
    ) -> Result<(), CompilerError> {
        self.body.add_instruction(
            Rc::clone(&line_number),
            Jump {
                label: format!(".{}{}_end.loop", self.name, label).into(),
            },
        );
        self.result = Rc::from(Operand::None);
        Ok(())
    }

    fn visit_continue(
        &mut self,
        line_number: &Rc<Position>,
        label: &mut Rc<String>,
        is_for: &mut bool,
    ) -> Result<(), CompilerError> {
        if *is_for {
            self.body.add_instruction(
                Rc::clone(&line_number),
                Jump {
                    label: format!(".{}{}_increment.loop", self.name, label).into(),
                },
            );
        } else {
            self.body.add_instruction(
                Rc::clone(&line_number),
                Jump {
                    label: format!(".{}{}_start.loop", self.name, label).into(),
                },
            );
        }
        self.result = Rc::from(Operand::None);
        Ok(())
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
        let start_label: Rc<String> = Rc::from(format!(".{}{}_start.loop", self.name, label));
        let end_label: Rc<String> = Rc::from(format!(".{}{}_end.loop", self.name, label));
        let increment_label: Rc<String> =
            Rc::from(format!(".{}{}_increment.loop", self.name, label));
        if let Some(init) = init {
            init.accept(self)?;
        }
        self.body.add_instruction(
            // start
            Rc::clone(&line_number),
            Label {
                label: Rc::clone(&start_label),
            },
        );
        if let Some(condition) = condition {
            condition.accept(self)?;
            self.body.add_instruction(
                Rc::clone(&line_number),
                JumpIfZero {
                    // if false goto end
                    label: Rc::clone(&end_label),
                    operand: Rc::clone(&self.result),
                },
            );
        }
        if let Some(body) = body {
            body.accept(self)?;
        }
        self.body.add_instruction(
            Rc::clone(&line_number),
            Label {
                label: Rc::clone(&increment_label),
            },
        ); // increment
        if let Some(increment) = increment {
            increment.accept(self)?;
        }
        self.body.add_instruction(
            Rc::clone(&line_number),
            Jump {
                label: Rc::clone(&start_label),
            },
        ); // goto start
        self.body.add_instruction(
            Rc::clone(&line_number),
            Label {
                label: Rc::clone(&end_label),
            },
        ); // end
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
        line_number: &Rc<Position>,
        identifier: &mut Rc<String>,
    ) -> Result<(), CompilerError> {
        match self
            .body
            .variable_to_pseudoregister
            .get(&(*Rc::clone(&identifier)).clone())
        {
            None => Err(SemanticError(format!(
                "Variable {:?} at {} not found",
                line_number, identifier
            ))
            .into()),
            Some(pseudoregister) => {
                self.result = Rc::from(Operand::Register(pseudoregister.clone()));
                Ok(())
            }
        }
    }

    fn visit_prefix(
        &mut self,
        line_number: &Rc<Position>,
        variable: &mut Box<ASTNode>,
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
                self.body.add_instruction(
                    Rc::clone(&line_number),
                    BinaryOpInstruction {
                        dest: Rc::clone(&pseudoregister),
                        op: binary_operator,
                        left: Rc::clone(&self.result),
                        right: Rc::from(Operand::Immediate(1)),
                    },
                );
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
        variable: &mut Box<ASTNode>,
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
        self.body.add_instruction(
            Rc::clone(&line_number),
            BinaryOpInstruction {
                dest: Rc::clone(&dest),
                op: binary_operator,
                left: Rc::clone(&self.result),
                right: Rc::from(Operand::Immediate(1)),
            },
        );
        let temp2 = Rc::new(Pseudoregister::new(
            (*self.name).clone(),
            self.body.variable_count,
        ));
        self.body.variable_count += 1;
        self.body.add_instruction(
            Rc::clone(&line_number),
            StoreValueInstruction {
                dest,
                src: Rc::from(Operand::Register(temp2)),
            },
        );
        self.result = Rc::from(Operand::Register(temp1));
        Ok(())
    }
}
