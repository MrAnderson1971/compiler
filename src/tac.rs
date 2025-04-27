use crate::common::{Const, Identifier};
use crate::lexer::{BinaryOperator, Type, UnaryOperator};
use std::collections::HashMap;
use std::fmt::Display;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub(crate) enum Pseudoregister {
    Pseudoregister(i32, Type),
    Register(String),
    Data(Rc<String>, Type),
}

impl Pseudoregister {
    pub(crate) fn new(offset: i32, t: &Type) -> Self {
        Pseudoregister::Pseudoregister(offset, *t)
    }

    fn size(&self) -> i32 {
        match self {
            Pseudoregister::Pseudoregister(_, t) => t.size(),
            Pseudoregister::Register(reg) => {
                if reg.starts_with("e") || reg.ends_with("d") {
                    4
                } else {
                    8
                }
            }
            Pseudoregister::Data(_, t) => t.size(),
        }
    }
}

#[derive(Debug)]
pub(crate) enum Operand {
    Register(Pseudoregister),
    Immediate(Const),
    MemoryReference(usize, String, Type),
    None,
}

impl Operand {
    fn size(&self) -> i32 {
        match self {
            Operand::Register(reg) => reg.size(),
            Operand::Immediate(c) => c.size(),
            Operand::MemoryReference(_, _, t) => t.size(),
            Operand::None => 0,
        }
    }

    fn is_immediate(&self) -> bool {
        match self {
            Operand::Immediate(_) => true,
            _ => false,
        }
    }
}

impl Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operand::Immediate(i) => write!(f, "${}", i),
            Operand::None => write!(f, ""),
            Operand::Register(r) => r.fmt(f),
            Operand::MemoryReference(offset, reg, _) => write!(f, "{}(%{})", offset, reg),
        }
    }
}

impl Display for Pseudoregister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Pseudoregister::Pseudoregister(offset, _) => write!(f, "-{}(%rbp)", offset),
            Pseudoregister::Register(s) => write!(f, "%{}", s),
            Pseudoregister::Data(d, _) => write!(f, "{}(%rip)", d),
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub(crate) enum TACInstruction {
    FunctionInstruction {
        name: Rc<String>,
        global: bool,
    },
    StaticVariable {
        name: Rc<String>,
        global: bool,
        init: Const,
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
    SignExtend {
        dest: Rc<Pseudoregister>,
        src: Rc<Operand>,
    },
    Truncate {
        dest: Rc<Pseudoregister>,
        src: Rc<Operand>,
    },
}

#[derive(Debug)]
pub(crate) struct FunctionBody {
    pub(crate) current_offset: i32,
    pub(crate) instructions: Vec<TACInstruction>,
    pub(crate) variable_to_pseudoregister: HashMap<String, Rc<Pseudoregister>>,
}

impl FunctionBody {
    pub(crate) fn new() -> Self {
        FunctionBody {
            current_offset: 8,
            instructions: vec![],
            variable_to_pseudoregister: HashMap::new(),
        }
    }

    pub(crate) fn add_instruction(&mut self, instruction: TACInstruction) {
        self.instructions.push(instruction);
    }

    pub(crate) fn add_default_return(&mut self) {
        match &self.instructions.last() {
            Some(TACInstruction::ReturnInstruction { .. }) | None => {}
            _ => {
                self.add_instruction(TACInstruction::ReturnInstruction {
                    val: Rc::from(Operand::Immediate(0u32.into())),
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
                let mov = if dest.size() == 4 { "movl" } else { "movq" };
                if operand.is_immediate() && operand.size() == 8 {
                    *out += &format!(
                        r#"movabsq {}, %r10
movq %r10, {}
"#,
                        operand, dest
                    );
                } else if operand.is_immediate() {
                    *out += &format!("{} {}, {}\n", mov, operand, dest);
                } else {
                    let r10 = if dest.size() == 4 { "r10d" } else { "r10" };
                    *out += &format!("{} {}, %{}\n", mov, operand, r10);
                    *out += &format!("{} %{}, {}\n", mov, r10, dest);
                }
                match op {
                    UnaryOperator::LogicalNot => {
                        *out += &format!("xor $1, {}", dest);
                    }
                    UnaryOperator::BitwiseNot => {
                        let not = if dest.size() == 4 { "notl" } else { "notq" };
                        *out += &format!("{} {}\n", not, dest);
                    }
                    UnaryOperator::Negate => {
                        let neg = if dest.size() == 4 { "negl" } else { "negq" };
                        *out += &format!("{} {}\n", neg, dest);
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
                if format!("{}", dest) == format!("{}", src) {
                    return;
                }
                if src.is_immediate() && src.size() == 8 {
                    *out += &format!(
                        r#"movabsq {}, %r10
movq %r10, {}
"#,
                        src, dest
                    );
                    return;
                }
                let mov = if src.size() == 4 { "movl" } else { "movq" };
                let r10 = if src.size() == 4 { "r10d" } else { "r10" };
                if src.is_immediate() {
                    *out += &format!("{} {}, {}\n", mov, src, dest);
                } else {
                    *out += &format!("{} {}, %{}\n", mov, src, r10);
                    *out += &format!("{} %{}, {}\n", mov, r10, dest);
                }
            }
            TACInstruction::ReturnInstruction { val } => {
                if val.size() == 4 {
                    *out += &format!("movl {}, %eax\n", val);
                } else if val.is_immediate() && val.size() == 8 {
                    *out += &format!("movabsq {}, %rax\n", val);
                } else {
                    *out += &format!("movq {}, %rax\n", val);
                }
                *out += "movq %rbp, %rsp\n\
popq %rbp\n\
ret\n";
            }
            TACInstruction::AllocateStackInstruction => {
                let allocate = (function_body.current_offset + 15) & !15;
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
                if matches!(*init, Const::ConstInt(0) | Const::ConstLong(0)) {
                    *out += &format!(
                        r#".bss
.align {}
.zero {}
{}:
"#,
                        init.size(),
                        init.size(),
                        name
                    );
                } else {
                    let which = if init.size() == 4 { "long" } else { "quad" };
                    *out += &format!(
                        r#".data
.align {}
{}:
.{} {}
"#,
                        init.size(),
                        name,
                        which,
                        init
                    );
                }
            }
            TACInstruction::SignExtend { dest, src } => {
                *out += &format!(
                    r#"movl {}, %r10d
movslq %r10d, %r10
movq %r10, {}
"#,
                    src, dest
                );
            }
            TACInstruction::Truncate { dest, src } => {
                if src.is_immediate() {
                    *out += &format!("movl {}, {}\n", src, dest);
                } else {
                    *out += &format!(
                        r#"movl {}, %r10d
movl %r10d, {}
"#,
                        src, dest
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
    let src2_is_immediate = right.is_immediate();
    let r10 = if left.size() == 4 { "r10d" } else { "r10" };
    let r11 = if left.size() == 4 { "r11d" } else { "r11" };
    let mov = if left.size() == 4 { "movl" } else { "movq" };
    let cx = if dest.size() == 4 { "ecx" } else { "rcx" };

    match op {
        BinaryOperator::BitwiseShiftLeft | BinaryOperator::BitwiseShiftRight => {
            let opcode = if let BinaryOperator::BitwiseShiftLeft = op {
                if right.size() == 4 { "shll" } else { "shlq" }
            } else {
                if right.size() == 4 { "shrl" } else { "shrq" }
            };
            if left.is_immediate() && left.size() == 8 {
                *out += &format!("movabsq {}, %r10\n", src1);
            } else {
                *out += &format!("{} {}, %{}\n", mov, src1, r10);
            }
            if right.is_immediate() {
                *out += &format!("{} {}, %{}\n", opcode, src2, r10);
            } else {
                *out += &format!(
                    r#"{} {}, %{}
{} %cl, %{}
"#,
                    mov, src2, cx, opcode, r10
                );
            }
            *out += &format!("{} %{}, {}\n", mov, r10, d);
        }
        BinaryOperator::Addition
        | BinaryOperator::Subtraction
        | BinaryOperator::BitwiseAnd
        | BinaryOperator::BitwiseOr
        | BinaryOperator::BitwiseXor => {
            let opcode = match op {
                BinaryOperator::Addition => {
                    if right.size() == 4 {
                        "addl"
                    } else {
                        "addq"
                    }
                }
                BinaryOperator::Subtraction => {
                    if right.size() == 4 {
                        "subl"
                    } else {
                        "subq"
                    }
                }
                BinaryOperator::BitwiseAnd => {
                    if right.size() == 4 {
                        "andl"
                    } else {
                        "andq"
                    }
                }
                BinaryOperator::BitwiseOr => {
                    if right.size() == 4 {
                        "orl"
                    } else {
                        "orq"
                    }
                }
                BinaryOperator::BitwiseXor => {
                    if right.size() == 4 {
                        "xorl"
                    } else {
                        "xorq"
                    }
                }
                _ => unreachable!(),
            };
            if right.is_immediate() && left.is_immediate() && left.size() == 4 {
                *out += &format!(r#"movl {}, {}
{} {}, {}
"#, src1, dest, opcode, src2, dest);
                return;
            }
            if left.is_immediate() && left.size() == 8 {
                *out += &format!("movabsq {}, %r10\n", src1);
            } else {
                *out += &format!("{} {}, %{}\n", mov, src1, r10);
            }
            if src2_is_immediate {
                *out += &format!("{} {}, %{}\n", opcode, src2, r10);
            } else {
                *out += &format!("{} {}, %{}\n", mov, src2, r11);
                *out += &format!("{} %{}, %{}\n", opcode, r11, r10);
            }
            *out += &format!("{} %{}, {}\n", mov, r10, d);
        }
        BinaryOperator::Multiply => {
            let mull = if dest.size() == 4 { "imull" } else { "imulq" };
            if left.is_immediate() && left.size() == 8 {
                *out += &format!("movabsq {}, %r11\n", src1);
            } else {
                *out += &format!("{} {}, %{}\n", mov, src1, r11);
            }
            if src2_is_immediate && right.size() == 8 {
                *out += &format!("movabsq {}, %r10\n", src2);
                *out += &format!("{} %r10, %r11\n", mull);
            } else if src2_is_immediate {
                *out += &format!("{} {}, %{}\n", mull, src2, r11);
            } else {
                *out += &format!("{} {}, %{}\n", mov, src2, r10);
                *out += &format!("{} %{}, %{}\n", mull, r10, r11);
            }
            *out += &format!("{} %{}, {}\n", mov, r11, d);
        }
        BinaryOperator::Divide | BinaryOperator::Modulo => {
            let ax = if dest.size() == 4 { "eax" } else { "rax" };
            *out += &format!("{} {}, %{}\n", mov, src1, ax);
            *out += if dest.size() == 4 { "cdq\n" } else { "cqo\n" };
            if src2_is_immediate && right.size() == 8 {
                *out += &format!("movabsq {}, %{}\n", src2, cx);
            } else {
                *out += &format!("{} {}, %{}\n", mov, src2, cx);
            }
            *out += &format!("idiv %{}\n", cx);
            if *op == BinaryOperator::Divide {
                *out += &format!("{} %{}, {}\n", mov, ax, d);
            } else {
                let dx = if dest.size() == 4 { "edx" } else { "rdx" };
                *out += &format!("{} %{}, {}\n", mov, dx, d);
            }
        }
        BinaryOperator::Equals
        | BinaryOperator::NotEquals
        | BinaryOperator::GreaterThan
        | BinaryOperator::GreaterThanOrEquals
        | BinaryOperator::LessThan
        | BinaryOperator::LessThanOrEquals => {
            let cmp = if left.size() == 4 { "cmpl" } else { "cmpq" };
            let dest_reg = if left.size() == 4 { "edx" } else { "rdx" };
            *out += &format!("{} {}, %{}\n", mov, src1, dest_reg);
            if right.size() == 8 && src2_is_immediate {
                *out += &format!("movabsq {}, %r11\n", src2);
                *out += &format!("cmpq %r11, %{}\n", dest_reg);
            } else {
                *out += &format!("{} {}, %{}\n", cmp, src2, dest_reg);
            }
            *out += &format!("{} $0, {}\n", mov, d);
            let opcode = match op {
                BinaryOperator::Equals => "sete",
                BinaryOperator::NotEquals => "setne",
                BinaryOperator::LessThan => "setl",
                BinaryOperator::GreaterThan => "setg",
                BinaryOperator::LessThanOrEquals => "setle",
                BinaryOperator::GreaterThanOrEquals => "setge",
                _ => unreachable!(),
            };
            *out += &format!("{} %al\n", opcode);
            *out += &"movzbl %al, %r10d\n".to_string();
            *out += &format!("movl %r10d, {}\n", d);
        }
        _ => unreachable!(),
    }
}
