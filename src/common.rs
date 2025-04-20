use crate::lexer::Number;
use std::fmt::Display;
use std::rc::Rc;

pub(crate) type Position = (i32, String);

#[derive(Debug)]
pub(crate) struct Pseudoregister {
    //name: String,
    size: i32,
}

impl Pseudoregister {
    pub(crate) fn new(_name: String, size: i32) -> Self {
        Self { size }
    }
}

#[derive(Debug)]
pub(crate) enum Operand {
    Register(Rc<Pseudoregister>),
    Immediate(Number),
    None,
}

impl Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operand::Register(r) => r.fmt(f),
            Operand::Immediate(i) => write!(f, "${}", i),
            Operand::None => write!(f, ""),
        }
    }
}

impl Display for Pseudoregister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "-{}(%rbp)", 4 * self.size)
    }
}

pub(crate) type Identifier = String;
