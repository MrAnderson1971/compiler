#include "compiler.hpp"
#include <iostream>

void compile(const std::string& source, std::ostream& os) {
	Lexer lexer(source);
	lexer.lex();
	if constexpr (DEBUG) {
		std::cout << lexer << "\n";
	}
	Parser parser(lexer.tokens);
	std::unique_ptr<ASTNode> programNode = parser.parse();
	if constexpr (DEBUG) {
		std::cout << *programNode << "\n";
	}
	CodeContext context{ os };
	dynamic_cast<ProgramNode*>(programNode.get())->generate(context);
}
