#include "lexer.hpp"
#include <regex>
#include <cctype>

Lexer::Lexer(std::string& source) : source(source) {}

void TokenPrinter::operator()(Symbol s) const {
	switch (s) {
	case OPEN_BRACE:
		os << "{";
		break;
	case CLOSED_BRACE:
		os << "}";
		break;
	case OPEN_PAREN:
		os << "(";
		break;
	case CLOSED_PAREN:
		os << ")";
		break;
	case SEMICOLON:
		os << ";";
		break;
	case MINUS:
		os << "-";
		break;
	case TILDE:
		os << "~";
		break;
	case EXCLAMATION_MARK:
		os << "!";
		break;
	case PLUS:
		os << "+";
		break;
	case ASTERISK:
		os << "*";
		break;
	case FORWARD_SLASH:
		os << "/";
		break;
	case DOUBLE_MINUS:
		os << "--";
		break;
	case PERCENTAGE:
		os << "%";
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
		case '-':
			if (i + 1 < source.size() && source[i + 1] == '-') {
				tokens.push_back(DOUBLE_MINUS);
				i++;
			}
			else {
				tokens.push_back(MINUS);
			}
			break;
		case '~':
			tokens.push_back(TILDE);
			break;
		case '!':
			tokens.push_back(EXCLAMATION_MARK);
			break;
		case '+':
			if (i + 1 < source.size() && source[i + 1] == '+') {
				tokens.push_back(DOUBLE_PLUS);
				i++;
			}
			else {
				tokens.push_back(PLUS);
			}
			break;
		case '*':
			tokens.push_back(ASTERISK);
			break;
		case '/':
			tokens.push_back(FORWARD_SLASH);
			break;
		case '%':
			tokens.push_back(PERCENTAGE);
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
