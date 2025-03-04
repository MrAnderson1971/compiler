#include "compiler.hpp"
#include <iostream>

Compiler::Compiler(std::string& source) : source(source), lexer(Lexer(source)) {}

void Compiler::compile() {
	lexer.lex();
	std::cout << lexer;
}
