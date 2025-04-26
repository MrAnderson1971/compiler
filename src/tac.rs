use crate::common::Identifier;
use crate::lexer::{BinaryOperator, Number, UnaryOperator};
use std::collections::HashMap;
use std::fmt::Display;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub(crate) enum Pseudoregister {
    //name: String,
    Pseudoregister(i32),
    Register(String),
    Data(Rc<String>),
}

impl Pseudoregister {
    pub(crate) fn new(_name: String, size: i32) -> Self {
        Pseudoregister::Pseudoregister(size)
    }
}

#[derive(Debug)]
pub(crate) enum Operand {
    Register(Pseudoregister),
    Immediate(Number),
    MemoryReference(usize, String),
    None,
}

impl Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operand::Immediate(i) => write!(f, "${}", i),
            Operand::None => write!(f, ""),
            Operand::Register(r) => r.fmt(f),
            Operand::MemoryReference(offset, reg) => write!(f, "{}(%{})", offset, reg),
        }
    }
}

impl Display for Pseudoregister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Pseudoregister::Pseudoregister(size) => write!(f, "-{}(%rbp)", 4 * size),
            Pseudoregister::Register(s) => write!(f, "%{}", s),
            Pseudoregister::Data(d) => write!(f, "{}(%rip)", d),
        }
    }
}

#[derive(Debug)]
pub(crate) enum TACInstruction {
    FunctionInstruction {
        name: Rc<String>,
        global: bool,
    },
    StaticVariable {
        name: Rc<String>,
        global: bool,
        init: Number,
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
    FunctionCall(Rc<Identifier>),
    PushArgument(Rc<Operand>),
    AdjustStack(usize),
}

#[derive(Debug)]
pub(crate) struct FunctionBody {
    pub(crate) variable_count: i32,
    pub(crate) instructions: Vec<TACInstruction>,
    pub(crate) variable_to_pseudoregister: HashMap<String, Rc<Pseudoregister>>,
}

impl FunctionBody {
    pub(crate) fn new() -> Self {
        FunctionBody {
            variable_count: 1,
            instructions: vec![],
            variable_to_pseudoregister: HashMap::new(),
        }
    }

    pub(crate) fn add_instruction(&mut self, instruction: TACInstruction) {
        self.instructions.push(instruction);
    }

    pub(crate) fn add_default_return_to_main(&mut self) {
        match &self.instructions.last().unwrap() {
            TACInstruction::ReturnInstruction { .. } => {}
            _ => {
                self.add_instruction(TACInstruction::ReturnInstruction {
                    val: Rc::from(Operand::Immediate(0)),
                });
            }
        }
    }
}

impl TACInstruction {
    pub(crate) fn make_assembly(&self, out: &mut String, function_body: &FunctionBody) {
        match &self {
            TACInstruction::FunctionInstruction { name, global } => {
                if *global {
                    *out += &format!(".global {}\n", name);
                }
                *out += &format!(
                    ".text\n\
                    {}:\n\
                pushq %rbp\n\
                movq %rsp, %rbp\n",
                    name
                );
            }
            TACInstruction::UnaryOpInstruction { dest, op, operand } => {
                *out += &format!("movl {}, %r10d\n", operand);
                *out += &format!("movl %r10d, {}\n", dest);
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
            TACInstruction::BinaryOpInstruction {
                dest,
                op,
                left,
                right,
            } => make_binary_op_instruction(out, dest, op, left, right),
            TACInstruction::JumpIfZero { label, operand } => {
                *out += &format!("movl {}, %edx\n", operand);
                *out += "cmpl $0, %edx\n";
                *out += &format!("je {}\n", label);
            }
            TACInstruction::JumpIfNotZero { label, operand } => {
                *out += &format!("movl {}, %edx\n", operand);
                *out += "cmpl $0, %edx\n";
                *out += &format!("jne {}\n", label);
            }
            TACInstruction::Jump { label } => *out += &format!("jmp {}\n", label),
            TACInstruction::Label { label } => *out += &format!("{}:\n", label),
            TACInstruction::StoreValueInstruction { dest, src } => {
                *out += &format!("movl {}, %r10d\n", src);
                *out += &format!("movl %r10d, {}\n", dest);
            }
            TACInstruction::ReturnInstruction { val } => {
                *out += &format!("movl {}, %eax\n", val);
                *out += "movq %rbp, %rsp\n\
popq %rbp\n\
ret\n";
            }
            TACInstruction::AllocateStackInstruction => {
                let allocate = ((function_body.variable_count * 4) + 15) & !15;
                *out += &format!("subq ${}, %rsp\n", allocate)
            }
            TACInstruction::FunctionCall(name) => {
                *out += &format!("call {}\n", name);
            }
            TACInstruction::PushArgument(value) => {
                *out += &format!("movl {}, %r10d\n", value);
                *out += "pushq %r10\n";
            }
            TACInstruction::AdjustStack(size) => {
                *out += &format!("addq ${}, %rsp\n", size);
            }
            TACInstruction::StaticVariable { name, global, init } => {
                if *global {
                    *out += &format!(".global {}\n", name);
                }
                if *init == 0 {
                    *out += &format!(
                        r#".bss
.align 4
.zero 4
{}:
                    "#,
                        name
                    );
                } else {
                    *out += &format!(
                        r#".data
.align 4
{}:
.long {}
                    "#,
                        name, init
                    );
                }
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
            *out += &format!("movl {}, %r10d\n", src1);
            if *op == BinaryOperator::BitwiseShiftLeft || *op == BinaryOperator::BitwiseShiftRight {
                let shift_op = if *op == BinaryOperator::BitwiseShiftLeft {
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
            *out += &format!("movl {}, %edx\n", src1);
            *out += &format!("cmpl {}, %edx\n", src2);
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
