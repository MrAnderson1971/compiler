#pragma once

#include "lexer.hpp"
#include "parser.hpp"

#ifdef _DEBUG
constexpr bool debug = true;
#else
constexpr bool debug = false;
#endif

inline void compile(std::string& source, std::ostream& os) {
	Lexer lexer(source);
	lexer.lex();
	if constexpr (debug) {
		std::cout << lexer << std::endl;
	}
	Parser parser(std::move(lexer.tokens));
	std::unique_ptr<ASTNode> programNode = parser.parse();
	if constexpr (debug) {
		std::cout << *programNode << std::endl;
	}
	CodeContext context{ os };
	programNode->generate(context);
}
