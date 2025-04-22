use crate::lexer::Number;
use std::fmt::Display;

pub(crate) type Position = (i32, String);

#[derive(Debug, Clone)]
pub(crate) enum Pseudoregister {
    //name: String,
    Pseudoregister(i32),
    Register(String),
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
    None,
}

impl Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operand::Immediate(i) => write!(f, "${}", i),
            Operand::None => write!(f, ""),
            Operand::Register(r) => r.fmt(f),
        }
    }
}

impl Display for Pseudoregister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Pseudoregister::Pseudoregister(size) => write!(f, "-{}(%rbp)", 4 * size),
            Pseudoregister::Register(s) => write!(f, "{}", s),
        }
    }
}

pub(crate) type Identifier = String;
