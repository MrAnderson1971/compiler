#include <functional>
#include "parser.hpp"

Token Parser::peekToken() {
	if (tokens.empty()) {
		throw compiler_error("Unexpected EOF");
	}
	return tokens.front();
}

Parser::Parser(std::vector<Token>&& tokens) : tokens(tokens.begin(), tokens.end()) {}

Token Parser::getTokenAndAdvance() {
	if (tokens.empty()) {
		throw compiler_error("Unexpected EOF");
	}
	return std::visit(GetTokenAndAdvance{ tokens }, tokens.front());
}

std::unique_ptr<ASTNode> Parser::parseProgram() {
	auto program = std::make_unique<ProgramNode>();
	program->function_declaration = parseFunctionDeclaration();
	return program;
}

std::unique_ptr<ASTNode> Parser::parseFunctionDeclaration() {
	getTokenAndAdvance(Keyword::INT);
	auto function_declaration = std::make_unique<FunctionDeclarationNode>();
	function_declaration->identifier = getTokenAndAdvance<std::string>();
	getTokenAndAdvance(Symbol::OPEN_PAREN);
	getTokenAndAdvance(Symbol::CLOSED_PAREN);
	getTokenAndAdvance(Symbol::OPEN_BRACE);
	function_declaration->statement = parseReturn();
	getTokenAndAdvance(Symbol::SEMICOLON);
	getTokenAndAdvance(Symbol::CLOSED_BRACE);
	return function_declaration;
}

std::unique_ptr<ASTNode> Parser::parseReturn() {
	getTokenAndAdvance(Keyword::RETURN);
	auto returnNode = std::make_unique<ReturnNode>();
	returnNode->expression = parseExpression();
	return returnNode;
}

std::unique_ptr<ASTNode> Parser::parseConst(Number value) {
	auto constNode = std::make_unique<ConstNode>(value);
	return constNode;
}

int getPrecedence(Symbol op) {
	switch (op) {
	case DOUBLE_GREATER_THAN: case DOUBLE_LESS_THAN:
		return 6;
	case AMPERSAND:
		return 5;
	case CARET:
		return 4;
	case PIPE:
		return 3;
	case ASTERISK: case FORWARD_SLASH: case PERCENTAGE:
		return 2;
	case PLUS: case MINUS:
		return 1;
	default:
		return -1;
	}
}

std::unique_ptr<ASTNode> Parser::parsePrimary() {
	Token token = peekToken();
	if (std::holds_alternative<Number>(token)) {
		return parseConst(getTokenAndAdvance<Number>());
	}
	else if (std::holds_alternative<Symbol>(token)) {
		getTokenAndAdvance(OPEN_PAREN);
		auto expression = parseExpression();
		getTokenAndAdvance(CLOSED_PAREN);
		return expression;
	}
	throw std::runtime_error("Unexpected token");
}

std::unique_ptr<ASTNode> Parser::parseUnaryOrPrimary() {
	Token token = peekToken();
	if (std::holds_alternative<Symbol>(token) && isOneOf(std::get<Symbol>(token), TILDE, EXCLAMATION_MARK, MINUS)) {
		UnaryOperator op;
		switch (getTokenAndAdvance<Symbol>()) {
		case TILDE:
			op = BITWISE_NOT;
			break;
		case EXCLAMATION_MARK:
			op = LOGICAL_NOT;
			break;
		case MINUS:
			op = NEGATION;
			break;
		}
		auto expression = parseUnaryOrPrimary();
		auto unaryNode = std::make_unique<UnaryNode>(op, expression);
		return unaryNode;
	} else {
		return parsePrimary();
	}
}

std::unique_ptr<ASTNode> Parser::parseBinaryOp(int minPrecedence) {
	auto left = parseUnaryOrPrimary();
	Symbol token = std::get<Symbol>(peekToken());
	while (isOneOf(token, PLUS, MINUS, ASTERISK, FORWARD_SLASH, PERCENTAGE, CARET, AMPERSAND, PIPE, DOUBLE_GREATER_THAN, DOUBLE_LESS_THAN) &&
		getPrecedence(token) >= minPrecedence) {
		Symbol symbol = getTokenAndAdvance<Symbol>();
		auto right = parseBinaryOp(getPrecedence(symbol) + 1);
		BinaryOperator op;
		switch (symbol) {
		case PLUS:
			op = ADD;
			break;
		case MINUS:
			op = SUBTRACT;
			break;
		case ASTERISK:
			op = MULTIPLY;
			break;
		case FORWARD_SLASH:
			op = DIVIDE;
			break;
		case PERCENTAGE:
			op = MODULO;
			break;
		case CARET:
			op = XOR;
			break;
		case AMPERSAND:
			op = AND;
			break;
		case PIPE:
			op = OR;
			break;
		case DOUBLE_LESS_THAN:
			op = SHIFT_LEFT;
			break;
		case DOUBLE_GREATER_THAN:
			op = SHIFT_RIGHT;
			break;
		}
		auto binaryNode = std::make_unique<BinaryNode>(op, left, right);
		left = std::move(binaryNode);
		token = std::get<Symbol>(peekToken());
	}
	return left;
}

std::unique_ptr<ASTNode> Parser::parseExpression() {
	return parseBinaryOp(0);
}


std::unique_ptr<ASTNode> Parser::parse() {
	return parseProgram();
}
