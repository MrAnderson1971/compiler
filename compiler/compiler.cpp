#include "compiler.hpp"
#include <iostream>

#ifdef _DEBUG
constexpr bool debug = true;
#else
constexpr bool debug = false;
#endif

void compile(const std::string& source, std::ostream& os) {
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
