use crate::common::{Operand, Position, Pseudoregister};
use crate::lexer::{BinaryOperator, UnaryOperator};
use std::collections::HashMap;
use std::rc::Rc;
use crate::tac::TACInstructionType::ReturnInstruction;

#[derive(Debug)]
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

#[derive(Debug)]
pub struct FunctionBody {
    pub(crate) variable_count: i32,
    pub(crate) instructions: Vec<TACInstruction>,
    pub(crate) variable_to_pseudoregister: HashMap<String, Rc<Pseudoregister>>,
}

impl FunctionBody {
    pub fn new() -> Self {
        FunctionBody {
            variable_count: 0,
            instructions: vec![],
            variable_to_pseudoregister: HashMap::new(),
        }
    }

    pub fn add_instruction(&mut self, line_number: &Rc<Position>, instruction: TACInstructionType) {
        self.instructions
            .push(TACInstruction::new(Rc::clone(&line_number), instruction));
    }

    pub fn add_default_return_to_main(&mut self, line_number: &Rc<Position>) {
        match &self.instructions.last().unwrap().kind {
            ReturnInstruction {  .. } => {},
            _ => {
                self.add_instruction(line_number, ReturnInstruction {val: Rc::from(Operand::Immediate(0))});
            }
        }
    }
}

#[derive(Debug)]
struct TACInstruction {
    line_number: Rc<Position>,
    kind: TACInstructionType,
}

impl TACInstruction {
    fn new(line_number: Rc<Position>, kind: TACInstructionType) -> Self {
        Self { line_number, kind }
    }
}
