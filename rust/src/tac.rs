use std::collections::HashMap;
use crate::common::{Operand, Position, Pseudoregister};
use crate::lexer::{BinaryOperator, UnaryOperator};
use std::rc::Rc;

pub enum TACInstructionType {
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

pub struct FunctionBody {
    pub(crate) variable_count: i32,
    instructions: Vec<TACInstruction>,
    pub(crate) variable_to_pseudoregister: HashMap<String, Rc<Pseudoregister>>,
}

impl FunctionBody {
    pub fn add_instruction(&mut self, line_number: Rc<Position>, instruction: TACInstructionType) {
        self.instructions.push(TACInstruction::new(line_number, instruction));
    }
}

struct TACInstruction {
    line_number: Rc<Position>,
    kind: TACInstructionType,
}

impl TACInstruction {
    fn new(line_number: Rc<Position>, kind: TACInstructionType) -> Self {
        Self { line_number, kind }
    }
}