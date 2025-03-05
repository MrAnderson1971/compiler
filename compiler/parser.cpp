#include "parser.hpp"

Parser::Parser(std::vector<Token>&& tokens) : tokens(tokens.begin(), tokens.end()) {}

Token Parser::getTokenAndAdvance() {
	if (tokens.empty()) {
		throw std::runtime_error("Unexpected EOF");
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
	auto function_declaration = std::make_unique<FunctionDeclarationNode<int>>();
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
	auto constNode = std::make_unique<ConstNode>();
	constNode->value = value;
	return constNode;
}

std::unique_ptr<ASTNode> Parser::parseExpression() {
	Token token = getTokenAndAdvance();
	return std::visit([this](const auto& t) -> std::unique_ptr<ASTNode> {
		using T = std::decay_t<decltype(t)>;

		if constexpr (std::is_same_v<T, Number>) {
			return parseConst(t);
		}
		else if constexpr (std::is_same_v<T, Symbol>) {
			Symbol unaryOp = t;
			auto unaryNode = std::make_unique<UnaryNode>();
			switch (unaryOp) {
			case Symbol::MINUS:
				unaryNode->op = UnaryOperator::MINUS;
				break;
			case Symbol::EXCLAMATION_MARK:
				unaryNode->op = UnaryOperator::LOGICAL_NOT;
				break;
			case Symbol::BITWISE_NOT:
				unaryNode->op = UnaryOperator::BITWISE_NOT;
				break;
			default:
				throw std::runtime_error("Unexpected unary operator");
			}
			unaryNode->expression = parseExpression();
			return unaryNode;
		}
		throw std::runtime_error("Unexpected token");
	}, token);
}

std::unique_ptr<ASTNode> Parser::parse() {
	return parseProgram();
}
