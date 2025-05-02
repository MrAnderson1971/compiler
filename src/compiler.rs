use std::collections::VecDeque;
use crate::asm_ast::assembly_fix;
use crate::lexer::lex;
use crate::parser::Parser;
use crate::errors::CompilerError;

pub fn compile(source: String) -> Result<String, CompilerError> {
    let mut out = String::with_capacity(1024);
    let tokens = lex(source);
    let mut parser = Parser::new(tokens);
    let mut program_node = parser.parse_program()?;
    let mut asm = VecDeque::new();
    program_node.generate(&mut asm)?;
    let asm = assembly_fix(asm);
    for instruction in asm.iter() {
        out += "\n";
        instruction.make_assembly(&mut out);
    }
    Ok(out)
}
