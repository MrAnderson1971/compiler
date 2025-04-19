use crate::common::{Operand, Position, Pseudoregister};
use crate::lexer::BinaryOperator::{BitwiseShiftLeft, BitwiseShiftRight};
use crate::lexer::{BinaryOperator, UnaryOperator};
use crate::tac::TACInstructionType::ReturnInstruction;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub(crate)enum TACInstructionType {
    FunctionInstruction {
        name: Rc<String>,
    },
    UnaryOpInstruction {
        dest: Rc<Pseudoregister>,
        op: UnaryOperator,
        operand: Rc<Operand>,
    },
    BinaryOpInstruction {
        dest: Rc<Pseudoregister>,
        op: BinaryOperator,
        left: Rc<Operand>,
        right: Rc<Operand>,
    },
    JumpIfZero {
        label: Rc<String>,
        operand: Rc<Operand>,
    },
    JumpIfNotZero {
        label: Rc<String>,
        operand: Rc<Operand>,
    },
    Jump {
        label: Rc<String>,
    },
    Label {
        label: Rc<String>,
    },
    StoreValueInstruction {
        dest: Rc<Pseudoregister>,
        src: Rc<Operand>,
    },
    ReturnInstruction {
        val: Rc<Operand>,
    },
    AllocateStackInstruction,
}

#[derive(Debug)]
pub(crate)struct FunctionBody {
    pub(crate) variable_count: i32,
    pub(crate) instructions: Vec<TACInstruction>,
    pub(crate) variable_to_pseudoregister: HashMap<String, Rc<Pseudoregister>>,
}

impl FunctionBody {
    pub(crate)fn new() -> Self {
        FunctionBody {
            variable_count: 0,
            instructions: vec![],
            variable_to_pseudoregister: HashMap::new(),
        }
    }

    pub(crate)fn add_instruction(&mut self, line_number: &Rc<Position>, instruction: TACInstructionType) {
        self.instructions
            .push(TACInstruction::new(Rc::clone(&line_number), instruction));
    }

    pub(crate)fn add_default_return_to_main(&mut self, line_number: &Rc<Position>) {
        match &self.instructions.last().unwrap().kind {
            ReturnInstruction { .. } => {}
            _ => {
                self.add_instruction(
                    line_number,
                    ReturnInstruction {
                        val: Rc::from(Operand::Immediate(0)),
                    },
                );
            }
        }
    }
}

#[derive(Debug)]
pub(crate)struct TACInstruction {
    line_number: Rc<Position>,
    kind: TACInstructionType,
}

impl TACInstruction {
    fn new(line_number: Rc<Position>, kind: TACInstructionType) -> Self {
        Self { line_number, kind }
    }

    pub(crate)fn make_assembly(&self, out: &mut String, function_body: &FunctionBody) {
        match &self.kind {
            TACInstructionType::FunctionInstruction { name } => {
                *out += &format!(
                    ".global {}\n\
                {}:\n\
                pushq %rbp\n\
                movq %rsp, %rbp\n",
                    name, name
                );
            }
            TACInstructionType::UnaryOpInstruction { dest, op, operand } => {
                *out += &format!("movl {}, %r10\n", operand);
                *out += &format!("movl %r10, {}\n", dest);
                match op {
                    UnaryOperator::LogicalNot => {
                        *out += &format!("cmpl $0, {}\n", dest);
                        *out += &format!("sete {}\n", dest);
                    }
                    UnaryOperator::BitwiseNot => {
                        *out += &format!("notl {}\n", dest);
                    }
                    UnaryOperator::Negate => {
                        *out += &format!("negl {}\n", dest);
                    }
                    _ => {}
                }
            }
            TACInstructionType::BinaryOpInstruction {
                dest,
                op,
                left,
                right,
            } => make_binary_op_instruction(out, dest, op, left, right),
            TACInstructionType::JumpIfZero { label, operand } => {
                *out += &format!("movl {}, %edx\n", operand);
                *out += "cmpl $0, %edx\n";
                *out += &format!("je {}\n", label);
            }
            TACInstructionType::JumpIfNotZero { label, operand } => {
                *out += &format!("movl {}, %edx\n", operand);
                *out += "cmpl $0, %edx\n";
                *out += &format!("jne {}\n", label);
            }
            TACInstructionType::Jump { label } => *out += &format!("jmp {}\n", label),
            TACInstructionType::Label { label } => *out += &format!("{}:\n", label),
            TACInstructionType::StoreValueInstruction { dest, src } => {
                *out += &format!("movl {}, %r10\n", src);
                *out += &format!("movl %r10, {}\n", dest);
            }
            TACInstructionType::ReturnInstruction { val } => {
                *out += &format!("movl {}, %eax\n", val);
                *out += "movq %rbp, %rsp\n\
popq %rbp\n\
ret\n";
            }
            TACInstructionType::AllocateStackInstruction => {
                *out += &format!("subq ${}, %rsp\n", function_body.variable_count * 4)
            }
        }
    }
}

fn make_binary_op_instruction(
    out: &mut String,
    dest: &Pseudoregister,
    op: &BinaryOperator,
    left: &Operand,
    right: &Operand,
) {
    let src1 = format!("{}", left);
    let src2 = format!("{}", right);
    let d = format!("{}", dest);
    let src2_is_immediate = src2.find("$").is_some();

    match op {
        BinaryOperator::Addition
        | BinaryOperator::Subtraction
        | BinaryOperator::BitwiseShiftLeft
        | BinaryOperator::BitwiseShiftRight
        | BinaryOperator::BitwiseAnd
        | BinaryOperator::BitwiseOr
        | BinaryOperator::BitwiseXor => {
            *out += &format!("movl {}, %r10\n", src1);
            if *op == BitwiseShiftLeft || *op == BitwiseShiftRight {
                let shift_op = if *op == BitwiseShiftLeft {
                    "shll"
                } else {
                    "shrl"
                };
                if src2_is_immediate {
                    *out += &format!("{} {}, %r10d\n", shift_op, src2);
                } else {
                    *out += &format!("movl {}, %ecx\n", src2);
                    *out += &format!("{} %cl, %r10d\n", shift_op);
                }
            } else {
                let opcode = match op {
                    BinaryOperator::Addition => "addl",
                    BinaryOperator::Subtraction => "subl",
                    BinaryOperator::BitwiseAnd => "andl",
                    BinaryOperator::BitwiseOr => "orl",
                    BinaryOperator::BitwiseXor => "xorl",
                    _ => unreachable!(),
                };
                if src2_is_immediate {
                    *out += &format!("{} {}, %r10d\n", opcode, src2);
                } else {
                    *out += &format!("movl {}, %r11d\n", src2);
                    *out += &format!("{} %r11d, %r10d\n", opcode);
                }
            }
            *out += &format!("movl %r10d, {}\n", d);
        }
        BinaryOperator::Multiply => {
            *out += &format!("movl {}, %r11d\n", src1);
            if src2_is_immediate {
                *out += &format!("imull {}, %r11d\n", src2);
            } else {
                *out += &format!("movl {}, %r10d\n", src2);
                *out += &"imull %r10d, %r11d\n".to_string();
            }
            *out += &format!("movl %r11d, {}\n", d);
        }
        BinaryOperator::Divide | BinaryOperator::Modulo => {
            *out += &format!("movl {}, %eax\n", src1);
            *out += "cdq\n";
            *out += &format!("movl {}, %ecx\n", src2);
            *out += "idiv %ecx\n";
            if *op == BinaryOperator::Divide {
                *out += &format!("movl %eax, {}\n", d);
            } else {
                *out += &format!("movl %edx, {}\n", d);
            }
        }
        BinaryOperator::Equals
        | BinaryOperator::NotEquals
        | BinaryOperator::GreaterThan
        | BinaryOperator::GreaterThanOrEquals
        | BinaryOperator::LessThan
        | BinaryOperator::LessThanOrEquals => {
            *out += &format!("movl {}, %edx", src1);
            *out += &format!("cmpl {}, %edx", src2);
            *out += &format!("movl $0, {}\n", d);
            let opcode = match op {
                BinaryOperator::Equals => "sete",
                BinaryOperator::NotEquals => "setne",
                BinaryOperator::LessThan => "setl",
                BinaryOperator::GreaterThan => "setg",
                BinaryOperator::LessThanOrEquals => "setle",
                BinaryOperator::GreaterThanOrEquals => "setge",
                _ => unreachable!(),
            };
            *out += &format!("{} {}\n", opcode, d);
        }
        _ => unreachable!(),
    }
}
