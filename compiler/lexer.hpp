#pragma once

#include <string>
#include <variant>
#include <vector>
#include "type.hpp"

enum class Symbol {
	// unary or binary op
	MINUS,
	PLUS, // unary plus exists

	_UNARY_BEGIN = 99,
	// unary op
	TILDE,
	EXCLAMATION_MARK,
	DOUBLE_MINUS,
	DOUBLE_PLUS,

	_BINARY_BEGIN = 199,
	// binary op
	ASTERISK,
	FORWARD_SLASH,
	PERCENTAGE,
	CARET,
	AMPERSAND,
	PIPE,
	DOUBLE_LESS_THAN,
	DOUBLE_GREATER_THAN,

	// bools (still binary)
	DOUBLE_AMPERSAND,
	DOUBLE_PIPE,
	DOUBLE_EQUALS,
	NOT_EQUALS,
	LESS_THAN_OR_EQUAL,
	GREATER_THAN_OR_EQUAL,
	LESS_THAN,
	GREATER_THAN,
	EQUALS,

	_MISC_BEGIN = 299,
	// misc
	OPEN_BRACE,
	CLOSED_BRACE,
	OPEN_PAREN,
	CLOSED_PAREN,
	SEMICOLON,
};

inline bool isUnaryOp(Symbol s) {
	return static_cast<int>(s) < static_cast<int>(Symbol::_UNARY_BEGIN) ||
	static_cast<int>(s) > static_cast<int>(Symbol::_UNARY_BEGIN) &&
		static_cast<int>(s) < static_cast<int>(Symbol::_BINARY_BEGIN);
}

inline bool isBinaryOp(Symbol s) {
	return static_cast<int>(s) < static_cast<int>(Symbol::_UNARY_BEGIN) ||
	static_cast<int>(s) > static_cast<int>(Symbol::_BINARY_BEGIN) &&
		static_cast<int>(s) < static_cast<int>(Symbol::_MISC_BEGIN);
}

enum class Keyword {
	RETURN,
	INT
};

using Token = std::variant<Symbol, // tokens
	Keyword,
	Number,  // int literal
	std::string, // identifiers
	std::nullptr_t // unknown
>;

template<typename T>
bool operator==(const Token& t, const T& s) {
	return std::holds_alternative<T>(t) && std::get<T>(t) == s;
}

template<typename T>
bool operator==(const T& s, const Token& t) {
	return t == s;
}

struct TokenPrinter {
	std::string operator()(const Symbol s) const;
	std::string operator()(const Keyword k) const;
	std::string operator()(const Number i) const;
	std::string operator()(const std::string& s) const;
	std::string operator()(const std::nullptr_t) const;
};

extern TokenPrinter tokenPrinter;

template<>
struct std::formatter<Token> : std::formatter<std::string> {
	auto format(const Token& token, std::format_context& ctx) const {
		return std::formatter<std::string>::format(std::visit(tokenPrinter, token), ctx);
	}
};

class Lexer {
	std::string source;

public:
	std::vector<Token> tokens;
	Lexer(std::string source);
	void lex();

	friend std::ostream& operator<<(std::ostream&, const Lexer&);
};

std::ostream& operator<<(std::ostream&, const Lexer&);
