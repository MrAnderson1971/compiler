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
		std::cout << lexer << "\n";
	}
	Parser parser(lexer.tokens);
	std::unique_ptr<ASTNode> programNode = parser.parse();
	if constexpr (debug) {
		std::cout << *programNode << "\n";
	}
	CodeContext context{ os };
	dynamic_cast<ProgramNode*>(programNode.get())->generate(context);
}
