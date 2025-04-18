use crate::lexer::lex;

mod ast;
mod common;
mod errors;
mod lexer;
mod parser;
mod variable_resolution;

fn main() {
    let tokens = lex("int main() {\
    int a = 1;\n\
    return a; // entry point\n\
    }"
    .parse()
    .unwrap());
    println!("{:?}", tokens);
    let mut parser = parser::Parser::new(tokens);

    let root = parser.parse_program();
    match root {
        Ok(mut ast) => {
            println!("{:?}", ast);
            let mut out = String::with_capacity(1024);
            match ast.generate(&mut out) {
                Ok(_) => {
                    println!("{}", out);
                    println!("{:?}", ast);
                }
                Err(err) => println!("{}", err),
            }
        }
        Err(err) => println!("{}", err),
    }
}
