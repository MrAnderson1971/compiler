#pragma once

#include "lexer.hpp"

class Compiler {
	Lexer lexer;
	std::string source;

public:
	Compiler(std::string& source);
	void compile();
};
