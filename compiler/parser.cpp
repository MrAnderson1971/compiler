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

static std::unique_ptr<ASTNode> parseConst(Number value) {
	auto constNode = std::make_unique<ConstNode>(value);
	return constNode;
}

static int getPrecedence(Symbol op) {
	switch (op) {
	case Symbol::DOUBLE_GREATER_THAN: case Symbol::DOUBLE_LESS_THAN:
		return 6;
	case Symbol::AMPERSAND:
		return 5;
	case Symbol::CARET:
		return 4;
	case Symbol::PIPE:
		return 3;
	case Symbol::ASTERISK: case Symbol::FORWARD_SLASH: case Symbol::PERCENTAGE:
		return 2;
	case Symbol::PLUS: case Symbol::MINUS:
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
		getTokenAndAdvance(Symbol::OPEN_PAREN);
		auto expression = parseExpression();
		getTokenAndAdvance(Symbol::CLOSED_PAREN);
		return expression;
	}
	throw std::runtime_error("Unexpected token");
}

std::unique_ptr<ASTNode> Parser::parseUnaryOrPrimary() {
	Token token = peekToken();
	if (std::holds_alternative<Symbol>(token) && isUnaryOp(std::get<Symbol>(token))) {
		auto op = static_cast<UnaryOperator>(getTokenAndAdvance<Symbol>());
		auto expression = parseUnaryOrPrimary();
		auto unaryNode = std::make_unique<UnaryNode>(op, expression);
		return unaryNode;
	}
	return parsePrimary();
}

std::unique_ptr<ASTNode> Parser::parseBinaryOp(int minPrecedence) {
	auto left = parseUnaryOrPrimary();
	Symbol token = std::get<Symbol>(peekToken());
	while (isBinaryOp(token) && getPrecedence(token) >= minPrecedence) {
		Symbol symbol = getTokenAndAdvance<Symbol>();
		auto right = parseBinaryOp(getPrecedence(symbol) + 1);
		BinaryOperator op = static_cast<BinaryOperator>(symbol);
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
