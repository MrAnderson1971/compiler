use crate::lexer::lex;

mod ast;
mod common;
mod errors;
mod lexer;
mod parser;

fn main() {
    let tokens = lex("int main() {\
    return 0; // entry point\n\
    }"
    .parse()
    .unwrap());
    println!("{:?}", tokens);
    let mut parser = parser::Parser::new(tokens);

    let root = parser.parse();
    match root {
        Ok(ast) => println!("{:?}", ast),
        Err(err) => println!("{}", err),
    }
}
