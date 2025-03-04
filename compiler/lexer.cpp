#include "lexer.hpp"
#include <regex>
#include <cctype>

Lexer::Lexer(std::string& source) : source(source) {}

void Lexer::lex() {
	for (int i = 0; i < source.size(); i++) {
		switch (source[i]) {
		case '{':
			tokens.push_back(OPEN_BRACE);
			break;
		case '}':
			tokens.push_back(CLOSED_BRACE);
			break;
		case '(':
			tokens.push_back(OPEN_PAREN);
			break;
		case ')':
			tokens.push_back(CLOSED_PAREN);
			break;
		case ';':
			tokens.push_back(SEMICOLON);
			break;
		case ' ': // whitespace, do nothing
		case '\n':
		case '\r':
		case '\t':
			break;
		default:
			if (std::isalpha(source[i]) || source[i] == '_') { // identifiers 
				std::string identifier = "";
				while (i < source.size() && (std::isalnum(source[i]) || source[i] == '_')) {
					identifier += source[i++];
				}
				i--;
				if (identifier == "return") {
					tokens.push_back(Keyword::RETURN);
				}
				else if (identifier == "int") {
					tokens.push_back(Keyword::INT);
				}
				else {
					tokens.push_back(identifier);
				}
			}
			else if (std::isdigit(source[i])) { // int literal
				unsigned int intToken = 0;
				while (i < source.size() && std::isdigit(source[i])) {
					intToken = intToken * 10 + (source[i++] - '0');
				}
				i--;
				tokens.push_back(intToken);
			}
			else {
				tokens.push_back(nullptr); // unknown
			}
		}
	}
}

std::ostream& operator<<(std::ostream& os, Lexer lexer) {
	os << "[";
	for (const auto& token : lexer.tokens) {
		std::visit(TokenPrinter{ os }, token);
		os << ", ";
	}
	os << "]" << std::endl;
	return os;
}
