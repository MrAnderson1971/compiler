use crate::asm_ast::AsmAst::{
    Binary, Call, Cdq, Cmp, Function, Idiv, Jmp, JmpCC, Label, Mov, MovAl, Movsx, Push, Ret, SetCC,
    Static, Testl, Unary,
};
use crate::asm_ast::{AsmAst, CondCode};
use crate::common::Const;
use crate::common::Const::ConstLong;
use crate::lexer::{BinaryOperator, Type, UnaryOperator};
use crate::tac::Pseudoregister::Register;
use std::collections::{HashMap, VecDeque};
use std::fmt::Display;
use std::rc::Rc;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) enum Reg {
    BP,
    SP,
    AX,
    DX,
    DI,
    SI,
    CX,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
}

#[derive(Debug, Clone)]
pub(crate) enum Pseudoregister {
    Pseudoregister(i32, Type),
    Register(Reg, Type),
    Data(Rc<String>, Type),
}

impl Pseudoregister {
    pub(crate) fn new(offset: i32, t: &Type) -> Self {
        Pseudoregister::Pseudoregister(offset, *t)
    }

    fn size(&self) -> i32 {
        match self {
            Pseudoregister::Pseudoregister(_, t) => t.size(),
            Register(_, t) => t.size(),
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

    pub(crate) fn is_immediate(&self) -> bool {
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
            Register(r, t) => {
                let reg_name = format!("{:?}", r).to_lowercase();

                // Handle special cases for traditional registers
                if matches!(
                    r,
                    Reg::AX | Reg::DX | Reg::CX | Reg::BP | Reg::SP | Reg::DI | Reg::SI
                ) {
                    if t.size() == 4 {
                        // 32-bit registers - e prefix
                        write!(f, "%e{}", reg_name)
                    } else {
                        // 64-bit registers - r prefix
                        write!(f, "%r{}", reg_name)
                    }
                } else {
                    // For R8-R15, the format is different
                    if t.size() == 4 {
                        // 32-bit versions of extended registers get a 'd' suffix
                        write!(f, "%{}d", reg_name)
                    } else {
                        // 64-bit versions of extended registers have no suffix
                        write!(f, "%{}", reg_name)
                    }
                }
            }
            Pseudoregister::Data(d, _) => write!(f, "{}(%rip)", d),
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
    FunctionCall(Rc<String>),
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
    ZeroExtend {
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
    pub(crate) fn make_assembly(&self, out: &mut VecDeque<AsmAst>, function_body: &FunctionBody) {
        match &self {
            TACInstruction::FunctionInstruction { name, global } => out.push_back(Function {
                name: Rc::clone(name),
                global: *global,
            }),
            TACInstruction::UnaryOpInstruction { dest, op, operand } => {
                out.push_back(Mov {
                    size: dest.size(),
                    dest: Rc::clone(dest),
                    src: Rc::clone(operand),
                });
                out.push_back(Unary {
                    operator: *op,
                    size: dest.size(),
                    dest: Rc::clone(dest),
                });
            }
            TACInstruction::BinaryOpInstruction {
                dest,
                op,
                left,
                right,
            } => make_binary_op_instruction(out, dest, op, left, right),
            TACInstruction::JumpIfZero { label, operand } => {
                out.push_back(Mov {
                    size: 4,
                    src: Rc::clone(operand),
                    dest: Rc::from(Register(Reg::DX, Type::Int)),
                });
                out.push_back(Testl(Rc::from(Register(Reg::DX, Type::Int))));
                out.push_back(JmpCC {
                    condition: CondCode::Equal,
                    label: Rc::clone(&label),
                });
            }
            TACInstruction::JumpIfNotZero { label, operand } => {
                out.push_back(Mov {
                    size: 4,
                    src: Rc::clone(operand),
                    dest: Rc::from(Register(Reg::DX, Type::Int)),
                });
                out.push_back(Testl(Rc::from(Register(Reg::DX, Type::Int))));
                out.push_back(JmpCC {
                    condition: CondCode::NotEqual,
                    label: Rc::clone(&label),
                });
            }
            TACInstruction::Jump { label } => out.push_back(Jmp(Rc::clone(label))),
            TACInstruction::Label { label } => out.push_back(Label(Rc::clone(label))),
            TACInstruction::StoreValueInstruction { dest, src } => out.push_back(Mov {
                size: dest.size(),
                src: Rc::clone(src),
                dest: Rc::clone(dest),
            }),
            TACInstruction::ReturnInstruction { val } => {
                let t = if val.size() == 4 {
                    Type::Int
                } else {
                    Type::Long
                };
                out.push_back(Mov {
                    size: val.size(),
                    src: Rc::clone(val),
                    dest: Rc::from(Register(Reg::AX, t)),
                });
                out.push_back(Ret);
            }
            TACInstruction::AllocateStackInstruction => {
                let allocate = (function_body.current_offset + 15) & !15;
                out.push_back(Binary {
                    operator: BinaryOperator::Subtraction,
                    size: 8,
                    src: Rc::from(Operand::Immediate(ConstLong(allocate as i64))),
                    dest: Rc::from(Register(Reg::SP, Type::Long)),
                });
            }
            TACInstruction::FunctionCall(name) => out.push_back(Call(Rc::clone(name))),
            TACInstruction::PushArgument(value) => {
                out.push_back(Mov {
                    size: 4,
                    src: Rc::clone(value),
                    dest: Rc::from(Register(Reg::R10, Type::Int)),
                });
                out.push_back(Push(Rc::from(Operand::Register(Register(
                    Reg::R10,
                    Type::Long,
                )))));
            }
            TACInstruction::AdjustStack(size) => {
                out.push_back(Binary {
                    size: 8,
                    operator: BinaryOperator::Addition,
                    src: Rc::from(Operand::Immediate(ConstLong(*size as i64))),
                    dest: Rc::from(Register(Reg::SP, Type::Long)),
                });
            }
            TACInstruction::StaticVariable { name, global, init } => {
                out.push_back(Static {
                    size: init.size(),
                    name: Rc::clone(name),
                    global: *global,
                    init: init.clone(),
                });
            }
            TACInstruction::SignExtend { dest, src } => {
                let r10 = Rc::from(Register(Reg::R10, Type::Int));
                out.push_back(Mov {
                    size: 4,
                    src: Rc::clone(src),
                    dest: Rc::clone(&r10),
                });
                out.push_back(Movsx {
                    src: Rc::from(Operand::Register(r10.as_ref().clone())),
                    dest: Rc::clone(&dest),
                });
            }
            TACInstruction::Truncate { dest, src } => out.push_back(Mov {
                size: 4,
                src: Rc::clone(src),
                dest: Rc::clone(dest),
            }),
            TACInstruction::ZeroExtend { dest, src } => {
                out.push_back(Mov {
                    size: 4,
                    src: Rc::clone(src),
                    dest: Rc::from(Register(Reg::AX, Type::Int)),
                });
                out.push_back(Mov {
                    size: 8,
                    src: Rc::from(Operand::Register(Register(Reg::AX, Type::Long))),
                    dest: Rc::clone(dest),
                });
            }
        }
    }
}

fn make_binary_op_instruction(
    out: &mut VecDeque<AsmAst>,
    dest: &Rc<Pseudoregister>,
    op: &BinaryOperator,
    left: &Rc<Operand>,
    right: &Rc<Operand>,
) {
    match op {
        BinaryOperator::BitwiseShiftLeft | BinaryOperator::BitwiseShiftRight => {
            // First, move the left operand (value to be shifted) to the destination
            out.push_back(Mov {
                size: dest.size(),
                src: Rc::clone(left),
                dest: Rc::clone(dest),
            });

            // For shift operations in x86, the shift count must be either an immediate or in CL register
            if right.is_immediate() {
                // If the shift count is an immediate, we can use it directly with the shift operation
                out.push_back(Binary {
                    operator: *op,
                    size: dest.size(),
                    src: Rc::clone(right),
                    dest: Rc::clone(dest),
                });
            } else {
                // If the shift count is not an immediate, we need to move it to CL register first
                // Move right operand (shift count) to CX/ECX
                out.push_back(Mov {
                    size: right.size(),
                    src: Rc::clone(right),
                    dest: Rc::from(Register(
                        Reg::CX,
                        if right.size() == 4 {
                            Type::Int
                        } else {
                            Type::Long
                        },
                    )),
                });

                // Now perform the shift operation using CL as the shift count
                out.push_back(Binary {
                    operator: *op,
                    size: dest.size(),
                    src: Rc::from(Operand::Register(Register(Reg::CX, Type::Int))),
                    dest: Rc::clone(dest),
                });
            }
        }
        BinaryOperator::Addition
        | BinaryOperator::Subtraction
        | BinaryOperator::BitwiseAnd
        | BinaryOperator::BitwiseOr
        | BinaryOperator::BitwiseXor => {
            // Move left operand to destination
            out.push_back(Mov {
                size: dest.size(),
                src: Rc::clone(left),
                dest: Rc::clone(dest),
            });
            // Apply binary operation with right operand
            out.push_back(Binary {
                operator: *op,
                size: dest.size(),
                src: Rc::clone(right),
                dest: Rc::clone(dest),
            });
        }
        BinaryOperator::Multiply => {
            // Multiply
            // Move left operand to AX register
            out.push_back(Mov {
                size: left.size(),
                src: Rc::clone(left),
                dest: Rc::from(Register(
                    Reg::AX,
                    if left.size() == 4 {
                        Type::Int
                    } else {
                        Type::Long
                    },
                )),
            });
            // Multiply AX by right operand
            out.push_back(Binary {
                operator: BinaryOperator::Multiply,
                size: right.size(),
                src: Rc::clone(right),
                dest: Rc::from(Register(
                    Reg::AX,
                    if right.size() == 4 {
                        Type::Int
                    } else {
                        Type::Long
                    },
                )),
            });
            // Move result from AX to destination
            out.push_back(Mov {
                size: dest.size(),
                src: Rc::from(Operand::Register(Register(
                    Reg::AX,
                    if dest.size() == 4 {
                        Type::Int
                    } else {
                        Type::Long
                    },
                ))),
                dest: Rc::clone(dest),
            });
        }
        BinaryOperator::Divide | BinaryOperator::Modulo => {
            // Divide/Modulo
            // Move left operand to AX register
            out.push_back(Mov {
                size: left.size(),
                src: Rc::clone(left),
                dest: Rc::from(Register(
                    Reg::AX,
                    if left.size() == 4 {
                        Type::Int
                    } else {
                        Type::Long
                    },
                )),
            });
            // Sign-extend AX to DX:AX
            out.push_back(Cdq { size: left.size() });
            // Move right operand to CX register
            out.push_back(Mov {
                size: right.size(),
                src: Rc::clone(right),
                dest: Rc::from(Register(
                    Reg::CX,
                    if right.size() == 4 {
                        Type::Int
                    } else {
                        Type::Long
                    },
                )),
            });
            // Divide DX:AX by CX, result in AX (quotient) and DX (remainder)
            out.push_back(Idiv {
                size: right.size(),
                operand: Rc::from(Register(
                    Reg::CX,
                    if right.size() == 4 {
                        Type::Int
                    } else {
                        Type::Long
                    },
                )),
            });
            // Move quotient (AX) or remainder (DX) to destination
            if *op == BinaryOperator::Divide {
                out.push_back(Mov {
                    size: dest.size(),
                    src: Rc::from(Operand::Register(Register(
                        Reg::AX,
                        if dest.size() == 4 {
                            Type::Int
                        } else {
                            Type::Long
                        },
                    ))),
                    dest: Rc::clone(dest),
                });
            } else {
                // Modulo
                out.push_back(Mov {
                    size: dest.size(),
                    src: Rc::from(Operand::Register(Register(
                        Reg::DX,
                        if dest.size() == 4 {
                            Type::Int
                        } else {
                            Type::Long
                        },
                    ))),
                    dest: Rc::clone(dest),
                });
            }
        }
        BinaryOperator::Equals
        | BinaryOperator::NotEquals
        | BinaryOperator::GreaterThan
        | BinaryOperator::GreaterThanOrEquals
        | BinaryOperator::LessThan
        | BinaryOperator::LessThanOrEquals => {
            // Move left operand to DX register
            out.push_back(Mov {
                size: left.size(),
                src: Rc::clone(left),
                dest: Rc::from(Register(
                    Reg::DX,
                    if left.size() == 4 {
                        Type::Int
                    } else {
                        Type::Long
                    },
                )),
            });

            // Handle comparison
            if right.size() == 8 && right.is_immediate() {
                out.push_back(Mov {
                    size: right.size(),
                    src: Rc::clone(right),
                    dest: Rc::from(Register(Reg::R11, Type::Long)),
                });
                out.push_back(Cmp {
                    size: 8,
                    left: Rc::from(Operand::Register(Register(Reg::R11, Type::Long))),
                    right: Rc::from(Operand::Register(Register(Reg::DX, Type::Long))),
                });
            } else {
                out.push_back(Cmp {
                    size: left.size(),
                    left: Rc::clone(right),
                    right: Rc::from(Operand::Register(Register(
                        Reg::DX,
                        if left.size() == 4 {
                            Type::Int
                        } else {
                            Type::Long
                        },
                    ))),
                });
            }

            // Initialize destination with 0
            out.push_back(Mov {
                size: dest.size(),
                src: Rc::from(Operand::Immediate(Const::ConstInt(0))),
                dest: Rc::clone(dest),
            });

            // Set AL based on comparison
            let condition = match op {
                BinaryOperator::Equals => CondCode::Equal,
                BinaryOperator::NotEquals => CondCode::NotEqual,
                BinaryOperator::LessThan => CondCode::LessThan,
                BinaryOperator::GreaterThan => CondCode::GreaterThan,
                BinaryOperator::LessThanOrEquals => CondCode::LessEqual,
                BinaryOperator::GreaterThanOrEquals => CondCode::GreaterEqual,
                _ => unreachable!(),
            };

            // We'll hardcode to use AL register in the SetCC implementation
            out.push_back(SetCC(condition));

            // Zero-extend AL to R10D
            out.push_back(MovAl(Rc::from(Register(Reg::R10, Type::Int))));

            // Move the result to destination
            out.push_back(Mov {
                size: 4,
                src: Rc::from(Operand::Register(Register(Reg::R10, Type::Int))),
                dest: Rc::clone(dest),
            })
        }
        _ => unreachable!(),
    }
}
