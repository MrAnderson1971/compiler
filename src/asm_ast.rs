use crate::common::Const;
use crate::lexer::{BinaryOperator, Type, UnaryOperator};
use crate::tac::Pseudoregister::Register;
use crate::tac::{Operand, Pseudoregister, Reg};
use std::collections::VecDeque;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

fn should_split(src: &Rc<Operand>, dest: &Rc<Pseudoregister>) -> bool {
    matches!(
        src.as_ref(),
        Operand::Register(Pseudoregister::Pseudoregister(_, _) | Pseudoregister::Data(_, _))
            | Operand::MemoryReference(_, _, _)
    ) && matches!(
        dest.as_ref(),
        Pseudoregister::Pseudoregister(_, _) | Pseudoregister::Data(_, _)
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum CondCode {
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterEqual,
    LessEqual,
    Above,
    AboveOrEqual,
    Below,
    BelowOrEqual,
}

impl Display for CondCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CondCode::Equal => write!(f, "e"),
            CondCode::NotEqual => write!(f, "ne"),
            CondCode::GreaterThan => write!(f, "g"),
            CondCode::LessThan => write!(f, "l"),
            CondCode::GreaterEqual => write!(f, "ge"),
            CondCode::LessEqual => write!(f, "le"),
            CondCode::Above => write!(f, "a"),
            CondCode::AboveOrEqual => write!(f, "ae"),
            CondCode::Below => write!(f, "b"),
            CondCode::BelowOrEqual => write!(f, "be"),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum AsmAst {
    Function {
        name: Rc<String>,
        global: bool,
    },
    Static {
        size: i32,
        name: Rc<String>,
        global: bool,
        init: Const,
    },
    StaticConstant {
        name: Rc<String>,
        alignment: i32,
        init: Const,
    },
    Mov {
        size: i32,
        src: Rc<Operand>,
        dest: Rc<Pseudoregister>,
    },
    Movsx {
        src: Rc<Operand>,
        dest: Rc<Pseudoregister>,
    },
    MovZeroExtend {
        src: Rc<Operand>,
        dest: Rc<Pseudoregister>,
    },
    MovAl(Rc<Pseudoregister>),
    Unary {
        operator: UnaryOperator,
        size: i32,
        dest: Rc<Pseudoregister>,
    },
    Binary {
        operator: BinaryOperator,
        size: i32,
        src: Rc<Operand>,
        dest: Rc<Pseudoregister>,
    },
    Cmp {
        size: i32,
        left: Rc<Operand>,
        right: Rc<Operand>,
    },
    Idiv {
        size: i32,
        operand: Rc<Pseudoregister>,
    },
    Div {
        size: i32,
        operand: Rc<Operand>,
    },
    Cdq {
        size: i32,
    },
    Jmp(Rc<String>),
    JmpCC {
        condition: CondCode,
        label: Rc<String>,
    },
    SetCC(CondCode),
    Label(Rc<String>),
    Push(Rc<Operand>),
    Call(Rc<String>),
    Ret,
    Testl(Rc<Pseudoregister>),
    Cvttsd2si {
        dst_size: i32,
        src: Rc<Operand>,
        dst: Rc<Pseudoregister>,
    },
    Cvtsi2sd {
        src_size: i32,
        src: Rc<Operand>,
        dst: Rc<Pseudoregister>,
    },
}

pub(crate) fn assembly_fix(mut instructions: VecDeque<AsmAst>) -> VecDeque<AsmAst> {
    let mut out = VecDeque::new();
    while let Some(instruction) = instructions.pop_front() {
        instruction.fix_intermediate(&mut out);
    }
    out
}

impl AsmAst {
    fn fix_intermediate(&self, out: &mut VecDeque<AsmAst>) {
        match self {
            AsmAst::Binary {
                operator,
                size,
                src,
                dest,
            } => {
                // Handle floating-point binary operations
                if *size == 8 && matches!(dest.as_ref(), Pseudoregister::Register(_, Type::Double) |
                                                      Pseudoregister::Pseudoregister(_, Type::Double) |
                                                      Pseudoregister::Data(_, Type::Double)) {
                    // SSE instruction constraints
                    if should_split(src, dest) {
                        let xmm14 = std::rc::Rc::from(Register(Reg::XMM14, Type::Double));
                        out.push_back(Self::Mov {
                            size: *size,
                            src: src.clone(),
                            dest: xmm14.clone(),
                        });
                        out.push_back(AsmAst::Binary {
                            operator: *operator,
                            size: *size,
                            src: std::rc::Rc::from(Operand::Register(xmm14.as_ref().clone())),
                            dest: dest.clone(),
                        });
                    } else {
                        out.push_back(self.clone());
                    }
                } else if should_split(src, dest) {
                    // Integer binary operations
                    let r10 = std::rc::Rc::from(Register(
                        Reg::R10,
                        if *size == 4 { Type::Int } else { Type::Long },
                    ));
                    out.push_back(Self::Mov {
                        size: *size,
                        src: src.clone(),
                        dest: r10.clone(),
                    });
                    out.push_back(AsmAst::Binary {
                        operator: *operator,
                        size: *size,
                        src: std::rc::Rc::from(Operand::Register(r10.as_ref().clone())),
                        dest: dest.clone(),
                    });
                } else {
                    out.push_back(self.clone());
                }
            }
            AsmAst::Mov { size, src, dest } => {
                // Handle floating-point mov operations
                if *size == 8 && matches!(dest.as_ref(), Pseudoregister::Register(_, Type::Double) |
                                                      Pseudoregister::Pseudoregister(_, Type::Double) |
                                                      Pseudoregister::Data(_, Type::Double)) {
                    if should_split(src, dest) {
                        let xmm14 = std::rc::Rc::from(Register(Reg::XMM14, Type::Double));
                        out.push_back(AsmAst::Mov {
                            size: *size,
                            src: src.clone(),
                            dest: xmm14.clone(),
                        });
                        out.push_back(AsmAst::Mov {
                            size: *size,
                            src: std::rc::Rc::from(Operand::Register(xmm14.as_ref().clone())),
                            dest: dest.clone(),
                        });
                    } else {
                        out.push_back(self.clone());
                    }
                } else if should_split(src, dest) {
                    // Integer mov operations
                    let r10 = std::rc::Rc::from(Register(
                        Reg::R10,
                        if *size == 4 { Type::Int } else { Type::Long },
                    ));
                    out.push_back(AsmAst::Mov {
                        size: *size,
                        src: src.clone(),
                        dest: r10.clone(),
                    });
                    out.push_back(AsmAst::Mov {
                        size: *size,
                        src: std::rc::Rc::from(Operand::Register(r10.as_ref().clone())),
                        dest: dest.clone(),
                    });
                } else {
                    out.push_back(self.clone());
                }
            }
            AsmAst::MovZeroExtend { src, dest } => {
                out.push_back(AsmAst::Mov {
                    size: 4,
                    src: src.clone(),
                    dest: Rc::from(Register(Reg::R10, Type::Int)),
                });
                out.push_back(AsmAst::Mov {
                    size: 8,
                    src: std::rc::Rc::from(Operand::Register(Register(Reg::R10, Type::Long))),
                    dest: dest.clone(),
                });
            }
            // Fix SSE conversion instructions
            AsmAst::Cvttsd2si { dst_size, src, dst } => {
                // Destination must be a register
                if !matches!(dst.as_ref(), Pseudoregister::Register(_, _)) {
                    let r11 = Rc::from(Register(Reg::R11, if *dst_size == 4 { Type::Int } else { Type::Long }));
                    out.push_back(AsmAst::Cvttsd2si {
                        dst_size: *dst_size,
                        src: src.clone(),
                        dst: r11.clone(),
                    });
                    out.push_back(AsmAst::Mov {
                        size: *dst_size,
                        src: Rc::new(Operand::Register(r11.as_ref().clone())),
                        dest: dst.clone(),
                    });
                } else {
                    out.push_back(self.clone());
                }
            }
            AsmAst::Cvtsi2sd { src_size, src, dst } => {
                let mut need_src_fix = false;
                let mut need_dst_fix = false;

                // Source can't be immediate
                if matches!(src.as_ref(), Operand::Immediate(_)) {
                    need_src_fix = true;
                }

                // Destination must be register
                if !matches!(dst.as_ref(), Pseudoregister::Register(_, _)) {
                    need_dst_fix = true;
                }

                if need_src_fix || need_dst_fix {
                    let temp_src = if need_src_fix {
                        let r10 = Rc::from(Register(Reg::R10, if *src_size == 4 { Type::Int } else { Type::Long }));
                        out.push_back(AsmAst::Mov {
                            size: *src_size,
                            src: src.clone(),
                            dest: r10.clone(),
                        });
                        Rc::new(Operand::Register(r10.as_ref().clone()))
                    } else {
                        src.clone()
                    };

                    let temp_dst = if need_dst_fix {
                        Rc::from(Register(Reg::XMM15, Type::Double))
                    } else {
                        dst.clone()
                    };

                    out.push_back(AsmAst::Cvtsi2sd {
                        src_size: *src_size,
                        src: temp_src,
                        dst: temp_dst.clone(),
                    });

                    if need_dst_fix {
                        out.push_back(AsmAst::Mov {
                            size: 8,
                            src: Rc::new(Operand::Register(temp_dst.as_ref().clone())),
                            dest: dst.clone(),
                        });
                    }
                } else {
                    out.push_back(self.clone());
                }
            }
            AsmAst::Cmp { size, left, right } => {
                // Handle floating-point comparisons
                if *size == 8 && (matches!(left.as_ref(), Operand::Register(Pseudoregister::Register(_, Type::Double)) |
                                                        Operand::Register(Pseudoregister::Pseudoregister(_, Type::Double)) |
                                                        Operand::Register(Pseudoregister::Data(_, Type::Double))) ||
                    matches!(right.as_ref(), Operand::Register(Pseudoregister::Register(_, Type::Double)) |
                                                         Operand::Register(Pseudoregister::Pseudoregister(_, Type::Double)) |
                                                         Operand::Register(Pseudoregister::Data(_, Type::Double)))) {
                    // For comisd, the right operand (destination position) must be a register
                    if !matches!(right.as_ref(), Operand::Register(Pseudoregister::Register(_, _))) {
                        let xmm15 = Rc::from(Register(Reg::XMM15, Type::Double));
                        out.push_back(AsmAst::Mov {
                            size: 8,
                            src: right.clone(),
                            dest: xmm15.clone(),
                        });
                        out.push_back(AsmAst::Cmp {
                            size: *size,
                            left: left.clone(),
                            right: Rc::new(Operand::Register(xmm15.as_ref().clone())),
                        });
                    } else {
                        out.push_back(self.clone());
                    }
                } else {
                    out.push_back(self.clone());
                }
            }
            _ => out.push_back(self.clone()),
        }
    }

    pub(crate) fn make_assembly(&self, out: &mut String) {
        match &self {
            AsmAst::Function { name, global } => {
                if *global {
                    *out += &format!(".global {}\n", name);
                }
                *out += &format!(
                    r#".text
{}:
pushq %rbp
movq %rsp, %rbp
"#,
                    name
                );
            }
            AsmAst::StaticConstant { name, alignment, init } => {
                // Emit floating-point constants in read-only section
                #[cfg(target_os = "macos")]
                {
                    if *alignment == 16 {
                        *out += ".literal16\n";
                        *out += ".balign 16\n";
                        *out += &format!("{}:\n", name);
                        match init {
                            Const::ConstDouble(val) => {
                                *out += &format!(".double {}\n", val);
                                *out += ".quad 0\n"; // Pad to 16 bytes for macOS
                            }
                            _ => *out += &format!(".quad {}\n", init),
                        }
                    } else {
                        *out += ".literal8\n";
                        *out += ".balign 8\n";
                        *out += &format!("{}:\n", name);
                        match init {
                            Const::ConstDouble(val) => *out += &format!(".double {}\n", val),
                            _ => *out += &format!(".quad {}\n", init),
                        }
                    }
                }
                #[cfg(not(target_os = "macos"))]
                {
                    *out += ".section .rodata\n";
                    *out += &format!(".balign {}\n", alignment);
                    *out += &format!("{}:\n", name);
                    match init {
                        Const::ConstDouble(val) => *out += &format!(".double {}\n", val),
                        _ => *out += &format!(".quad {}\n", init),
                    }
                }
            }
            AsmAst::Mov { size, src, dest } => {
                // Handle floating-point moves
                if *size == 8 && matches!(dest.as_ref(), Pseudoregister::Register(_, Type::Double) |
                                                      Pseudoregister::Pseudoregister(_, Type::Double) |
                                                      Pseudoregister::Data(_, Type::Double)) {
                    *out += &format!("movsd {}, {}\n", src, dest);
                } else if *size == 8 && src.is_immediate() {
                    *out += &format!(
                        r#"movabsq {}, %r10
movq %r10, {}
"#,
                        src, dest
                    );
                } else if *size == 8 {
                    *out += &format!("movq {}, {}\n", src, dest);
                } else {
                    *out += &format!("movl {}, {}\n", src, dest);
                }
            }
            AsmAst::Movsx { src, dest } => *out += &format!("movslq {}, {}\n", src, dest),
            AsmAst::MovZeroExtend { src, dest } => *out += &format!("movzbl {}, {}\n", src, dest),
            AsmAst::Unary {
                size,
                operator,
                dest,
            } => {
                let suffix = if *size == 4 { 'l' } else { 'q' };
                let opcode = match operator {
                    UnaryOperator::Increment => format!("inc{}", suffix),
                    UnaryOperator::Decrement => format!("dec{}", suffix),
                    UnaryOperator::LogicalNot => {
                        *out += &format!("xorl $1, {}", dest);
                        return;
                    }
                    UnaryOperator::BitwiseNot => format!("not{}", suffix),
                    UnaryOperator::Negate => format!("neg{}", suffix),
                    UnaryOperator::UnaryAdd => return,
                };
                *out += &format!("{} {}\n", opcode, dest);
            }
            AsmAst::Binary {
                operator,
                size,
                src,
                dest,
            } => {
                // Handle floating-point operations
                if *size == 8 && matches!(dest.as_ref(), Pseudoregister::Register(_, Type::Double) |
                                                      Pseudoregister::Pseudoregister(_, Type::Double) |
                                                      Pseudoregister::Data(_, Type::Double)) {
                    let opcode = match operator {
                        BinaryOperator::Addition => "addsd",
                        BinaryOperator::Subtraction => "subsd",
                        BinaryOperator::Multiply => "mulsd",
                        BinaryOperator::Divide => "divsd",
                        BinaryOperator::BitwiseXor => "xorpd",
                        _ => unreachable!("Invalid floating-point operation"),
                    };
                    *out += &format!("{} {}, {}\n", opcode, src, dest);
                } else {
                    // Integer operations
                    let suffix = if *size == 4 { 'l' } else { 'q' };
                    let opcode = match operator {
                        BinaryOperator::Addition => format!("add{}", suffix),
                        BinaryOperator::Subtraction => format!("sub{}", suffix),
                        BinaryOperator::BitwiseXor => format!("xor{}", suffix),
                        BinaryOperator::BitwiseAnd => format!("and{}", suffix),
                        BinaryOperator::BitwiseOr => format!("or{}", suffix),
                        BinaryOperator::Multiply => format!("imul{}", suffix),
                        BinaryOperator::BitwiseShiftLeft => format!("shl{}", suffix),
                        BinaryOperator::BitwiseShiftRight => format!("shr{}", suffix),
                        _ => unreachable!(),
                    };
                    if src.is_immediate() && *size == 8 {
                        *out += &format!(
                            r#"movabsq {}, %r10
{} %r10, {}
"#,
                            src, opcode, dest
                        );
                    } else {
                        *out += &format!("{} {}, {}\n", opcode, src, dest);
                    }
                }
            }
            AsmAst::Cmp { size, left, right } => {
                // Handle floating-point comparisons
                if *size == 8 && (matches!(left.as_ref(), Operand::Register(Pseudoregister::Register(_, Type::Double)) |
                                                        Operand::Register(Pseudoregister::Pseudoregister(_, Type::Double)) |
                                                        Operand::Register(Pseudoregister::Data(_, Type::Double))) ||
                    matches!(right.as_ref(), Operand::Register(Pseudoregister::Register(_, Type::Double)) |
                                                         Operand::Register(Pseudoregister::Pseudoregister(_, Type::Double)) |
                                                         Operand::Register(Pseudoregister::Data(_, Type::Double)))) {
                    *out += &format!("comisd {}, {}\n", left, right);
                } else {
                    let suffix = if *size == 4 { 'l' } else { 'q' };
                    *out += &format!("cmp{} {}, {}\n", suffix, left, right);
                }
            }
            AsmAst::Idiv { size, operand } => {
                let suffix = if *size == 4 { 'l' } else { 'q' };
                *out += &format!("idiv{} {}\n", suffix, operand);
            }
            AsmAst::Div { size, operand } => {
                let suffix = if *size == 4 { 'l' } else { 'q' };
                *out += &format!("div{} {}\n", suffix, operand);
            }
            AsmAst::Cdq { size } => *out += if *size == 4 { "cdq\n" } else { "cqo\n" },
            AsmAst::Jmp(label) => *out += &format!("jmp {}\n", label),
            AsmAst::JmpCC { condition, label } => *out += &format!("j{} {}\n", condition, label),
            AsmAst::SetCC(condition) => *out += &format!("set{} %al\n", condition),
            AsmAst::Label(label) => *out += &format!("{}:\n", label),
            AsmAst::Push(operand) => *out += &format!("pushq {}\n", operand),
            AsmAst::Call(name) => *out += &format!("call {}\n", name),
            AsmAst::Ret => {
                *out += r#"movq %rbp, %rsp
popq %rbp
ret
"#
            }
            AsmAst::Static {
                size,
                name,
                global,
                init,
            } => {
                let (initial, bss_data) = if matches!(
                    init,
                    Const::ConstLong(0)
                        | Const::ConstULong(0)
                        | Const::ConstInt(0)
                        | Const::ConstUInt(0)
                        | Const::ConstDouble(0.0)
                ) {
                    (&format!(".zero {}\n", size), ".bss")
                } else {
                    let which = if *size == 4 { "long" } else if matches!(init, Const::ConstDouble(_)) { "double" } else { "quad" };
                    (&format!(".{} {}\n", which, init), ".data")
                };
                let align = &format!(".align {}\n", size);
                if *global {
                    *out += &format!(".global {}\n", name);
                }
                *out += &format!(
                    r#"{}
{}
{}:
{}"#,
                    bss_data, align, name, initial
                );
            }
            AsmAst::Testl(reg) => *out += &format!("testl {}, {}\n", reg, reg),
            AsmAst::MovAl(dest) => *out += &format!("movzbl %al, {}\n", dest),
            AsmAst::Cvttsd2si { dst_size, src, dst } => {
                let suffix = if *dst_size == 4 { "l" } else { "q" };
                *out += &format!("cvttsd2si{} {}, {}\n", suffix, src, dst);
            }
            AsmAst::Cvtsi2sd { src_size, src, dst } => {
                let suffix = if *src_size == 4 { "l" } else { "q" };
                *out += &format!("cvtsi2sd{} {}, {}\n", suffix, src, dst);
            }
        }
    }
}