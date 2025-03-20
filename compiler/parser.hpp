#pragma once

#include "lexer.hpp"
#include "ast.hpp"

class Parser {
	class Impl;
	std::unique_ptr<Impl> impl;

public:
	~Parser();
	Parser(const Parser& other) = delete;
	Parser(Parser&& other) = delete;
	Parser& operator=(const Parser& other) = delete;
	Parser& operator=(Parser&& other) = delete;
	Parser(const std::vector<Token>& tokens);
	std::unique_ptr<ASTNode> parse() const;
};