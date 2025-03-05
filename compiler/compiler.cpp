#include <iostream>
#include "compiler.hpp"
#include "parser.hpp"

Compiler::Compiler(std::string& source) : source(source), lexer(Lexer(source)) {}

void Compiler::compile() {
	lexer.lex();
	std::cout << lexer << std::endl;
	Parser parser(std::move(lexer.tokens));
	std::unique_ptr<ASTNode> programNode = parser.parse();
	std::cout << *programNode << std::endl;
	//std::cout << programNode->evaluate() << std::endl;
}
