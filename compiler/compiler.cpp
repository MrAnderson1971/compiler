#include <iostream>
#include "compiler.hpp"
#include "parser.hpp"

Compiler::Compiler(std::string& source) : source(source), lexer(Lexer(source)) {}

void Compiler::compile() {
	lexer.lex();
	Parser parser(std::move(lexer.tokens));
	parser.parse();
	std::cout << lexer;
}
