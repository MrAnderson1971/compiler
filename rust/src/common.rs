use std::fmt::Display;
use std::rc::Rc;
use crate::lexer::Number;

pub type Position = (i32, String);

#[derive(Debug)]
pub struct Pseudoregister {
    name: String,
    size: i32,
}

impl Pseudoregister {
    pub fn new(name: String, size: i32) -> Self {
        Self { name, size }
    }
}

#[derive(Debug)]
pub enum Operand {
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
