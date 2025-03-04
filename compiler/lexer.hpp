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

enum class Keyword {
	RETURN,
	INT
};

using Token = std::variant<Symbol, // tokens
	Keyword,
	unsigned int,  // int literal
	std::string, // identifiers
	std::nullptr_t // unknown
>;

struct TokenPrinter {
	std::ostream& os;
	void operator()(Symbol s) const {
		switch (s) {
		case OPEN_BRACE:
			os << "OPEN_BRACE";
			break;
		case CLOSED_BRACE:
			os << "CLOSED_BRACE";
			break;
		case OPEN_PAREN:
			os << "OPEN_PAREN";
			break;
		case CLOSED_PAREN:
			os << "CLOSED_PAREN";
			break;
		case SEMICOLON:
			os << "SEMICOLON";
			break;
		}
	}
	void operator()(Keyword k) const {
		switch (k) {
		case Keyword::RETURN:
			os << "RETURN";
			break;
		case Keyword::INT:
			os << "INT";
			break;
		}
	}
	void operator()(unsigned int i) const {
		os << i;
	}
	void operator()(const std::string& s) const {
		os << s;
	}
	void operator()(std::nullptr_t) const {
		os << "UNKNOWN";
	}
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
