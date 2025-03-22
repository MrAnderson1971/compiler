#include "lexer.hpp"
#include <regex>
#include <cctype>

TokenPrinter tokenPrinter;

Lexer::Lexer(std::string source) : source(std::move(source)) {}

std::string TokenPrinter::operator()(const Symbol s) const {
	switch (s) {
	case Symbol::OPEN_BRACE:
		return "{";
	case Symbol::CLOSED_BRACE:
		return "}";
	case Symbol::OPEN_PAREN:
		return "(";
	case Symbol::CLOSED_PAREN:
		return ")";
	case Symbol::SEMICOLON:
		return ";";
	case Symbol::MINUS:
		return "-";
	case Symbol::TILDE:
		return "~";
	case Symbol::EXCLAMATION_MARK:
		return "!";
	case Symbol::PLUS:
		return "+";
	case Symbol::ASTERISK:
		return "*";
	case Symbol::FORWARD_SLASH:
		return "/";
	case Symbol::PERCENTAGE:
		return "%";
	case Symbol::PIPE:
		return "|";
	case Symbol::AMPERSAND:
		return "&";
	case Symbol::CARET:
		return "^";
	case Symbol::DOUBLE_LESS_THAN:
		return "<<";
	case Symbol::DOUBLE_GREATER_THAN:
		return ">>";
	case Symbol::DOUBLE_AMPERSAND:
		return "&&";
	case Symbol::DOUBLE_PIPE:
		return "||";
	case Symbol::DOUBLE_EQUALS:
		return "==";
	case Symbol::NOT_EQUALS:
		return "!=";
	case Symbol::LESS_THAN_OR_EQUAL:
		return "<=";
	case Symbol::GREATER_THAN_OR_EQUAL:
		return ">=";
	case Symbol::LESS_THAN:
		return "<";
	case Symbol::GREATER_THAN:
		return ">";
	case Symbol::EQUALS:
		return "=";
	case Symbol::DOUBLE_PLUS:
		return "++";
	case Symbol::DOUBLE_MINUS:
		return "--";
	default:
		return "UNKNOWN SYMBOL";
	}
}

std::string TokenPrinter::operator()(const Keyword k) const {
	switch (k) {
	case Keyword::RETURN:
		return "RETURN";
	case Keyword::INT:
		return "INT";
	default:
		return "UNKNOWN KEYWORD";
	}
}

std::string TokenPrinter::operator()(const std::nullptr_t) const {
	return "UNKNOWN";
}

std::string TokenPrinter::operator()(const Number i) const {
	return std::to_string(i);
}

std::string TokenPrinter::operator()(const std::string& s) const {
	return s;
}

void Lexer::lex() {
	for (size_t i = 0; i < source.size(); i++) {
		switch (source[i]) {
		case '{':
			tokens.emplace_back(Symbol::OPEN_BRACE);
			break;
		case '}':
			tokens.emplace_back(Symbol::CLOSED_BRACE);
			break;
		case '(':
			tokens.emplace_back(Symbol::OPEN_PAREN);
			break;
		case ')':
			tokens.emplace_back(Symbol::CLOSED_PAREN);
			break;
		case ';':
			tokens.emplace_back(Symbol::SEMICOLON);
			break;
		case '-':
			if (i + 1 < source.size() && source[i + 1] == '-') {
				tokens.emplace_back(Symbol::DOUBLE_MINUS);
				i++;
			}
			else {
				tokens.emplace_back(Symbol::MINUS);
			}
			break;
		case '~':
			tokens.emplace_back(Symbol::TILDE);
			break;
		case '!':
			if (i + 1 < source.size() && source[i + 1] == '=') {
				tokens.emplace_back(Symbol::NOT_EQUALS);
				i++;
			} else {
				tokens.emplace_back(Symbol::EXCLAMATION_MARK);
			}
			break;
		case '+':
			if (i + 1 < source.size() && source[i + 1] == '+') {
				tokens.emplace_back(Symbol::DOUBLE_PLUS);
				i++;
			}
			else {
				tokens.emplace_back(Symbol::PLUS);
			}
			break;
		case '*':
			tokens.emplace_back(Symbol::ASTERISK);
			break;
		case '/':
			tokens.emplace_back(Symbol::FORWARD_SLASH);
			break;
		case '%':
			tokens.emplace_back(Symbol::PERCENTAGE);
			break;
		case '|':
			if (i + 1 < source.size() && source[i + 1] == '|') {
				tokens.emplace_back(Symbol::DOUBLE_PIPE);
				i++;
			} else {
				tokens.emplace_back(Symbol::PIPE);
			}
			break;
		case '&':
			if (i + 1 < source.size() && source[i + 1] == '&') {
				tokens.emplace_back(Symbol::DOUBLE_AMPERSAND);
				i++;
			} else {
				tokens.emplace_back(Symbol::AMPERSAND);
			}
			break;
		case '^':
			tokens.emplace_back(Symbol::CARET);
			break;
		case '<':
			if (i + 1 < source.size() && source[i + 1] == '<') {
				tokens.emplace_back(Symbol::DOUBLE_LESS_THAN);
				i++;
			} else if (i + 1 < source.size() && source[i + 1] == '=') {
				tokens.emplace_back(Symbol::LESS_THAN_OR_EQUAL);
				i++;
			} else {
				tokens.emplace_back(Symbol::LESS_THAN);
			}
			break;
		case '>':
			if (i + 1 < source.size() && source[i + 1] == '>') {
				tokens.emplace_back(Symbol::DOUBLE_GREATER_THAN);
				i++;
			} else if (i + 1 < source.size() && source[i + 1] == '=') {
				tokens.emplace_back(Symbol::GREATER_THAN_OR_EQUAL);
				i++;
			} else {
				tokens.emplace_back(Symbol::GREATER_THAN);
			}
			break;
		case '=':
			if (i + 1 < source.size() && source[i + 1] == '=') {
				tokens.emplace_back(Symbol::DOUBLE_EQUALS);
				i++;
			} else {
				tokens.emplace_back(Symbol::EQUALS);
			}
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
					tokens.emplace_back(Keyword::RETURN);
				}
				else if (identifier == "int") {
					tokens.emplace_back(Keyword::INT);
				}
				else {
					tokens.emplace_back(identifier);
				}
			}
			else if (std::isdigit(source[i])) { // int literal
				Number intToken = 0;
				while (i < source.size() && std::isdigit(source[i])) {
					intToken = intToken * 10 + (source[i++] - '0');
				}
				i--;
				tokens.emplace_back(intToken);
			}
			else {
				tokens.emplace_back(nullptr); // unknown
			}
		}
	}
}

std::ostream& operator<<(std::ostream& os, const Lexer& lexer) {
	os << "[";
	for (size_t i = 0; i < lexer.tokens.size(); i++) {
		os << std::visit(tokenPrinter, lexer.tokens[i]);
		if (i + 1 < lexer.tokens.size()) {
			os << ", ";
		}
	}
	os << "]\n";
	return os;
}
