#pragma once

#include <string>
#include <variant>
#include <vector>
#include <ostream>
#include <cstddef>

enum Symbol {
	OPEN_BRACE,
	CLOSED_BRACE,
	OPEN_PAREN,
	CLOSED_PAREN,
	SEMICOLON,
};

enum Keyword {
	RETURN,
	INT
};

using Token = std::variant<Symbol, // tokens
	Keyword,
	unsigned int,  // int literal
	std::string, // identifiers
	std::nullptr_t // unknown
>;

class Lexer {
	std::string source;
	std::vector<Token> tokens;

public:
	Lexer(std::string& source);
	void lex();

	friend std::ostream& operator<<(std::ostream&, Lexer);
};

std::ostream& operator<<(std::ostream&, Lexer);
