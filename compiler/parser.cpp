#include "parser.hpp"

Token Parser::peekToken() {
	if (tokens.empty()) {
		throw syntax_error("Unexpected EOF");
	}
	return tokens.front();
}

Parser::Parser(const std::vector<Token>& tokens) : tokens(tokens.begin(), tokens.end()) {}

Token Parser::getTokenAndAdvance() {
	if (tokens.empty()) {
		throw syntax_error("Unexpected EOF");
	}
	return std::visit(GetTokenAndAdvance{ tokens }, tokens.front());
}

std::unique_ptr<ASTNode> Parser::parseProgram() {
	auto program = make_node<ProgramNode>();
	program->function_declaration = parseFunctionDeclaration();
	return program;
}

std::unique_ptr<ASTNode> Parser::parseFunctionDeclaration() {
	getTokenAndAdvance(Keyword::INT);
	auto function_declaration = make_node<FunctionDefinitionNode>();
	function_declaration->identifier = getTokenAndAdvance<std::string>();
	lineNumber = {1, function_declaration->identifier}; // reset line number at new function
	getTokenAndAdvance(Symbol::OPEN_PAREN);
	getTokenAndAdvance(Symbol::CLOSED_PAREN);
	getTokenAndAdvance(Symbol::OPEN_BRACE);

	for (Token nextToken = peekToken(); nextToken != Symbol::CLOSED_BRACE; nextToken = peekToken()) {
		if (std::unique_ptr<ASTNode> blockItem = parseBlockItem()) {
			function_declaration->block_items.emplace_back(std::move(blockItem));
		}
	}
	getTokenAndAdvance(Symbol::CLOSED_BRACE);
	return function_declaration;
}

std::unique_ptr<ASTNode> Parser::parseDeclaration() {
	auto declarationNode = make_node<DeclarationNode>();
	declarationNode->identifier = getTokenAndAdvance<std::string>();
	if (peekToken() == Symbol::EQUALS) {
		getTokenAndAdvance(Symbol::EQUALS);
		declarationNode->expression = parseExpression();
	}
	return declarationNode;
}

std::unique_ptr<ASTNode> Parser::parseBlockItem() {
	std::unique_ptr<ASTNode> blockItem = nullptr;
	Token token = peekToken();
	if (std::holds_alternative<Keyword>(token)) {
		switch (getTokenAndAdvance<Keyword>()) {
			case Keyword::RETURN: {
				auto returnNode = make_node<ReturnNode>();
				returnNode->expression = parseExpression();
				blockItem = std::move(returnNode);
				break;
			}
			case Keyword::INT: {
				blockItem = parseDeclaration();
				break;
			}
		}
	} else {
		blockItem = parseExpression();
	}
	getTokenAndAdvance(Symbol::SEMICOLON);
	lineNumber.first++;
	return blockItem;
}

static int getPrecedence(Symbol op) {
	switch (op) {
	case Symbol::ASTERISK: case Symbol::FORWARD_SLASH: case Symbol::PERCENTAGE:
		return 50;
	case Symbol::PLUS: case Symbol::MINUS:
		return 45;
	case Symbol::DOUBLE_GREATER_THAN: case Symbol::DOUBLE_LESS_THAN:
		return 40;
	case Symbol::LESS_THAN: case Symbol::LESS_THAN_OR_EQUAL:
	case Symbol::GREATER_THAN: case Symbol::GREATER_THAN_OR_EQUAL:
		return 35;
	case Symbol::DOUBLE_EQUALS: case Symbol::NOT_EQUALS:
		return 30;
	case Symbol::AMPERSAND:
		return 25;
	case Symbol::CARET:
		return 20;
	case Symbol::PIPE:
		return 15;
	case Symbol::DOUBLE_AMPERSAND:
		return 10;
	case Symbol::DOUBLE_PIPE:
		return 5;
	case Symbol::EQUALS:
		return 1;
	default:
		return -1;
	}
}

std::unique_ptr<ASTNode> Parser::parsePrimary() {
	Token token = peekToken();
	if (std::holds_alternative<Number>(token)) {
		return make_node<ConstNode>(getTokenAndAdvance<Number>());
	}
	if (std::holds_alternative<Symbol>(token)) {
		getTokenAndAdvance(Symbol::OPEN_PAREN);
		auto expression = parseExpression();
		getTokenAndAdvance(Symbol::CLOSED_PAREN);
		return expression;
	}
	if (std::holds_alternative<std::string>(token)) { // variable
		return make_node<VariableNode>(getTokenAndAdvance<std::string>());
	}
	throw syntax_error(std::format("Unexpected token {} at {}", peekToken(), lineNumber));
}

std::unique_ptr<ASTNode> Parser::parseUnaryOrPrimary() {
	Token token = peekToken();
	if (std::holds_alternative<Symbol>(token) && isUnaryOp(std::get<Symbol>(token))) {
		auto op = static_cast<UnaryOperator>(getTokenAndAdvance<Symbol>());
		auto expression = parseUnaryOrPrimary();
		auto unaryNode = make_node<UnaryNode>(op, expression);
		return unaryNode;
	}
	return parsePrimary();
}

/*
 *parse_exp(tokens, min_prec):
 left = parse_factor(tokens)
 next_token = peek(tokens)
 while next_token is a binary operator and precedence(next_token) >= min_prec:
 if next_token is "=":
 take_token(tokens) // remove "=" from list of tokens
 right = parse_exp(tokens, precedence(next_token))
 left = Assignment(left, right)
 else:
 operator = parse_binop(tokens)
 right = parse_exp(tokens, precedence(next_token) + 1)
 left = Binary(operator, left, right)
 next_token = peek(tokens)
 return left
 */
std::unique_ptr<ASTNode> Parser::parseBinaryOp(int minPrecedence) {
	auto left = parseUnaryOrPrimary();
	try {
		for (Symbol token = std::get<Symbol>(peekToken()); isBinaryOp(token) && getPrecedence(token) >= minPrecedence;
			token = std::get<Symbol>(peekToken())) {
			Symbol symbol = getTokenAndAdvance<Symbol>();
			if (symbol == Symbol::EQUALS) {
				auto right = parseBinaryOp(getPrecedence(symbol));
				left = make_node<AssignmentNode>(left, right);
			} else {
				auto right = parseBinaryOp(getPrecedence(symbol) + 1);
				left = make_node<BinaryNode>(static_cast<BinaryOperator>(symbol), left, right);
			}
		}
		return left;
	} catch (std::bad_variant_access&) {
		throw syntax_error(std::format("Unexpected token {} at {}", peekToken(), lineNumber));
	}
}

std::unique_ptr<ASTNode> Parser::parseExpression() {
	return parseBinaryOp(0);
}


std::unique_ptr<ASTNode> Parser::parse() {
	return parseProgram();
}
