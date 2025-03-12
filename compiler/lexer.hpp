#pragma once

#include <string>
#include <variant>
#include <vector>
#include "type.hpp"

enum class Symbol {
	// unary or binary op
	MINUS,

	_UNARY_BEGIN = 99,
	// unary op
	TILDE,
	EXCLAMATION_MARK,
	DOUBLE_MINUS,
	DOUBLE_PLUS,

	_BINARY_BEGIN = 199,
	// binary op
	PLUS,
	ASTERISK,
	FORWARD_SLASH,
	PERCENTAGE,
	CARET,
	AMPERSAND,
	PIPE,
	DOUBLE_LESS_THAN,
	DOUBLE_GREATER_THAN,
	LESS_THAN,
	GREATER_THAN,

	_MISC_BEGIN = 299,
	// misc
	OPEN_BRACE,
	CLOSED_BRACE,
	OPEN_PAREN,
	CLOSED_PAREN,
	SEMICOLON,
};

inline bool isUnaryOp(Symbol s) {
	return s == Symbol::MINUS ||
	static_cast<int>(s) > static_cast<int>(Symbol::_UNARY_BEGIN) &&
		static_cast<int>(s) < static_cast<int>(Symbol::_BINARY_BEGIN);
}

inline bool isBinaryOp(Symbol s) {
	return s == Symbol::MINUS ||
	static_cast<int>(s) > static_cast<int>(Symbol::_BINARY_BEGIN) &&
		static_cast<int>(s) < static_cast<int>(Symbol::_MISC_BEGIN);
}

enum class Keyword {
	RETURN,
	INT
};

struct UnknownToken {
	const int position;
};

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
