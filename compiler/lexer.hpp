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
	MINUS,
	BITWISE_NOT,
	EXCLAMATION_MARK
};

enum class Keyword {
	RETURN,
	INT
};

struct UnknownToken {
	const int position;
};

using Number = unsigned int;

using Token = std::variant<Symbol, // tokens
	Keyword,
	Number,  // int literal
	std::string, // identifiers
	UnknownToken // unknown
>;

struct TokenPrinter {
	std::ostream& os;
	void operator()(Symbol s) const;
	void operator()(Keyword k) const;
	void operator()(Number i) const;
	void operator()(const std::string& s) const;
	void operator()(UnknownToken) const;
};

class Lexer {
	std::string source;

public:
	std::vector<Token> tokens;
	Lexer(std::string& source);
	void lex();

	friend std::ostream& operator<<(std::ostream&, Lexer);
};

std::ostream& operator<<(std::ostream&, Lexer);
