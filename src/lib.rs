// src/lib.rs

pub(crate) mod ast;
pub(crate) mod common;
pub(crate) mod lexer;
pub(crate) mod parser;
pub(crate) mod tac;
pub(crate) mod tac_generator;
pub(crate) mod variable_resolution;
pub(crate) mod type_check;

// Make these public externally
pub mod compiler;
pub mod errors;

// ... re-exports ...
pub use compiler::compile;
pub use errors::CompilerError;
