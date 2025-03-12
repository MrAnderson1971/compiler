#include "lexer.hpp"
#include <regex>
#include <cctype>

Lexer::Lexer(std::string& source) : source(source) {}

void TokenPrinter::operator()(Symbol s) const {
	switch (s) {
	case Symbol::OPEN_BRACE:
		os << "{";
		break;
	case Symbol::CLOSED_BRACE:
		os << "}";
		break;
	case Symbol::OPEN_PAREN:
		os << "(";
		break;
	case Symbol::CLOSED_PAREN:
		os << ")";
		break;
	case Symbol::SEMICOLON:
		os << ";";
		break;
	case Symbol::MINUS:
		os << "-";
		break;
	case Symbol::TILDE:
		os << "~";
		break;
	case Symbol::EXCLAMATION_MARK:
		os << "!";
		break;
	case Symbol::PLUS:
		os << "+";
		break;
	case Symbol::ASTERISK:
		os << "*";
		break;
	case Symbol::FORWARD_SLASH:
		os << "/";
		break;
	case Symbol::DOUBLE_MINUS:
		os << "--";
		break;
	case Symbol::PERCENTAGE:
		os << "%";
		break;
	case Symbol::PIPE:
		os << "|";
		break;
	case Symbol::AMPERSAND:
		os << "&";
		break;
	case Symbol::CARET:
		os << "^";
		break;
	case Symbol::DOUBLE_LESS_THAN:
		os << "<<";
		break;
	case Symbol::DOUBLE_GREATER_THAN:
		os << ">>";
		break;
	default:
		os << "UNKNOWN SYMBOL";
	}
}

void TokenPrinter::operator()(Keyword k) const {
	switch (k) {
	case Keyword::RETURN:
		os << "RETURN";
		break;
	case Keyword::INT:
		os << "INT";
		break;
	default:
		os << "UNKNOWN KEYWORD";
	}
}

void TokenPrinter::operator()(UnknownToken) const {
	os << "UNKNOWN";
}

void TokenPrinter::operator()(Number i) const {
	os << i;
}

void TokenPrinter::operator()(const std::string& s) const {
	os << s;
}

void Lexer::lex() {
	for (int i = 0; i < source.size(); i++) {
		switch (source[i]) {
		case '{':
			tokens.push_back(Symbol::OPEN_BRACE);
			break;
		case '}':
			tokens.push_back(Symbol::CLOSED_BRACE);
			break;
		case '(':
			tokens.push_back(Symbol::OPEN_PAREN);
			break;
		case ')':
			tokens.push_back(Symbol::CLOSED_PAREN);
			break;
		case ';':
			tokens.push_back(Symbol::SEMICOLON);
			break;
		case '-':
			if (i + 1 < source.size() && source[i + 1] == '-') {
				tokens.push_back(Symbol::DOUBLE_MINUS);
				i++;
			}
			else {
				tokens.push_back(Symbol::MINUS);
			}
			break;
		case '~':
			tokens.push_back(Symbol::TILDE);
			break;
		case '!':
			tokens.push_back(Symbol::EXCLAMATION_MARK);
			break;
		case '+':
			if (i + 1 < source.size() && source[i + 1] == '+') {
				tokens.push_back(Symbol::DOUBLE_PLUS);
				i++;
			}
			else {
				tokens.push_back(Symbol::PLUS);
			}
			break;
		case '*':
			tokens.push_back(Symbol::ASTERISK);
			break;
		case '/':
			tokens.push_back(Symbol::FORWARD_SLASH);
			break;
		case '%':
			tokens.push_back(Symbol::PERCENTAGE);
			break;
		case '|':
			tokens.push_back(Symbol::PIPE);
			break;
		case '&':
			tokens.push_back(Symbol::AMPERSAND);
			break;
		case '^':
			tokens.push_back(Symbol::CARET);
			break;
		case '<':
			if (i + 1 < source.size() && source[i + 1] == '<') {
				tokens.push_back(Symbol::DOUBLE_LESS_THAN);
				i++;
			} else {
				tokens.push_back(Symbol::LESS_THAN);
			}
			break;
		case '>':
			if (i + 1 < source.size() && source[i + 1] == '>') {
				tokens.push_back(Symbol::DOUBLE_GREATER_THAN);
				i++;
			} else {
				tokens.push_back(Symbol::GREATER_THAN);
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
				Number intToken = 0;
				while (i < source.size() && std::isdigit(source[i])) {
					intToken = intToken * 10 + (source[i++] - '0');
				}
				i--;
				tokens.push_back(intToken);
			}
			else {
				tokens.push_back(UnknownToken{i}); // unknown
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
