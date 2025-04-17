use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum CompilerError {
    SyntaxError(String),
    SemanticError(String),
}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CompilerError::SyntaxError(what) => write!(f, "Syntax Error: {}", what),
            CompilerError::SemanticError(what) => write!(f, "Semantic Error: {}", what),
        }
    }
}

impl Error for CompilerError {}
