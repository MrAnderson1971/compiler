#pragma once

#include "lexer.hpp"
#include "parser.hpp"

inline void compile(std::string& source) {
	Lexer lexer(source);
	lexer.lex();
	std::cout << lexer << std::endl;
	Parser parser(std::move(lexer.tokens));
	std::unique_ptr<ASTNode> programNode = parser.parse();
	std::cout << *programNode << std::endl;
	CodeContext context{ std::cout };
	programNode->generate(context);
}
