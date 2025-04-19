use crate::lexer::lex;
use crate::parser::Parser;
use crate::errors::CompilerError;

pub fn compile(source: String) -> Result<String, CompilerError> {
    let mut out = String::with_capacity(1024);
    let tokens = lex(source);
    let mut parser = Parser::new(tokens);
    let mut program_node = parser.parse_program()?;
    program_node.generate(&mut out)?;
    Ok(out)
}
