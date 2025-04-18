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
