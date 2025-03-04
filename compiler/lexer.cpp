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
				if (identifier == "return") {
					tokens.push_back(RETURN);
				}
				else if (identifier == "int") {
					tokens.push_back(INT);
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
		std::visit([&os](const auto& t) {
			using T = std::decay_t<decltype(t)>;

			if constexpr (std::is_same_v<Keyword, T>) {
				os << "Keyword: " << static_cast<Keyword>(t) << ", ";
			}
			else if constexpr (std::is_same_v<Symbol, T>) {
				os << "Symbol: " << static_cast<Symbol>(t) << ", ";
			}
			else if constexpr (std::is_same_v<unsigned int, T>) {
				os << static_cast<unsigned int>(t) << ", ";
			}
			else if constexpr (std::is_same_v<std::string, T>) {
				os << static_cast<std::string>(t) << ", ";
			}
			else if constexpr (std::is_same_v<std::nullptr_t, T>) {
				os << "Unknown, ";
			}
			}, token);
	}
	os << "]" << std::endl;
	return os;
}
