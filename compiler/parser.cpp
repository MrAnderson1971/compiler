#include "parser.hpp"
#include <deque>
#include "ast_nodes.hpp"
#include "tac.hpp"
#include "exceptions.hpp"

class Parser::Impl {
public:
	struct GetTokenAndAdvance {
		std::deque<Token>& tokens;

		template<typename T>
		Token operator()(const T&) const {
			auto t = std::get<T>(tokens.front());
			tokens.pop_front();
			return t;
		}

		Token operator()(UnknownToken unknown) const {
			throw syntax_error("Unknown token at position " + std::to_string(unknown.position));
		}
	};
	std::deque<Token> tokens;
	Position lineNumber;

	std::unique_ptr<ASTNode> parseProgram();
	std::unique_ptr<ASTNode> parseFunctionDeclaration();
	std::unique_ptr<ASTNode> parseBlockItem();
	std::unique_ptr<ASTNode> parsePrimary();
	std::unique_ptr<ASTNode> parseUnaryOrPrimary();
	std::unique_ptr<ASTNode> parseBinaryOp(int minPrecedence);
	std::unique_ptr<ASTNode> parseExpression();
	std::unique_ptr<ASTNode> parseDeclaration();

	Token getTokenAndAdvance();

	template<typename T>
	T getTokenAndAdvance();

	template<typename T>
	T getTokenAndAdvance(T expected);

	template<typename T, typename... Args>
	std::unique_ptr<T> make_node(Args&&... args);

	Token peekToken();
	Impl(const std::vector<Token>& tokens) : tokens(tokens.begin(), tokens.end()), lineNumber({ 1, "" }) {}
};

template <typename T>
T Parser::Impl::getTokenAndAdvance() {
	if (tokens.empty()) {
		throw syntax_error("Unexpected EOF");
	}
	if (!std::holds_alternative<T>(tokens.front())) {
		throw syntax_error(std::format("Unexpected token {} at {}", tokens.front(), lineNumber));
	}
	auto t = std::get<T>(tokens.front());
	tokens.pop_front();
	return t;
}

template <typename T>
T Parser::Impl::getTokenAndAdvance(T expected) {
	if (!std::holds_alternative<T>(peekToken())) {
		throw syntax_error(std::format("Expected {} but got {} at {}", tokenPrinter(expected), peekToken(), lineNumber));
	}
	auto t = std::get<T>(getTokenAndAdvance());
	if (t != expected) {
		throw syntax_error(std::format("Expected {} but got {} at {}", tokenPrinter(expected), tokenPrinter(t), lineNumber));
	}
	return t;
}

template <typename T, typename ... Args>
std::unique_ptr<T> Parser::Impl::make_node(Args&&... args) {
	auto node = std::make_unique<T>(std::forward<Args>(args)...);
	node->lineNumber = lineNumber;
	return node;
}

Token Parser::Impl::peekToken() {
	if (tokens.empty()) {
		throw syntax_error("Unexpected EOF");
	}
	return tokens.front();
}

Parser::Parser(const std::vector<Token>& tokens) : impl(std::make_unique<Impl>(tokens)) {}

Token Parser::Impl::getTokenAndAdvance() {
	if (tokens.empty()) {
		throw syntax_error("Unexpected EOF");
	}
	return std::visit(GetTokenAndAdvance{ tokens }, tokens.front());
}

std::unique_ptr<ASTNode> Parser::Impl::parseProgram() {
	auto program = make_node<ProgramNode>();
	program->function_declaration = parseFunctionDeclaration();
	return program;
}

std::unique_ptr<ASTNode> Parser::Impl::parseFunctionDeclaration() {
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

std::unique_ptr<ASTNode> Parser::Impl::parseDeclaration() {
	auto declarationNode = make_node<DeclarationNode>();
	declarationNode->identifier = getTokenAndAdvance<std::string>();
	if (peekToken() == Symbol::EQUALS) {
		getTokenAndAdvance(Symbol::EQUALS);
		declarationNode->expression = parseExpression();
	}
	return declarationNode;
}

std::unique_ptr<ASTNode> Parser::Impl::parseBlockItem() {
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

std::unique_ptr<ASTNode> Parser::Impl::parsePrimary() {
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

std::unique_ptr<ASTNode> Parser::Impl::parseUnaryOrPrimary() {
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
std::unique_ptr<ASTNode> Parser::Impl::parseBinaryOp(int minPrecedence) {
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

std::unique_ptr<ASTNode> Parser::Impl::parseExpression() {
	return parseBinaryOp(0);
}


std::unique_ptr<ASTNode> Parser::parse() const {
	return impl->parseProgram();
}

Parser::~Parser() = default;
