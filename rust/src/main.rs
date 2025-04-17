use crate::lexer::lex;

mod lexer;
mod ast;
mod parser;
mod errors;
mod common;

fn main() {
    let tokens = lex("int main() {\
    return 0; // entry point\
    }".parse().unwrap());
    println!("{:?}", tokens);
}
