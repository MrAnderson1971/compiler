use crate::lexer::lex;

mod lexer;

fn main() {
    let tokens = lex("int main() {\
    return 0; // entry point\
    }".parse().unwrap());
    println!("{:?}", tokens);
}
