#include "parser.hpp"
#include <deque>
#include "ast_nodes.hpp"
#include "tac.hpp"
#include "exceptions.hpp"

class Parser::Impl {
public:
	struct GetTokenAndAdvance {
		std::deque<Token>* tokens;

		template <typename T>
		Token operator()(const T&) const {
			auto t = std::get<T>(tokens->front());
			tokens->pop_front();
			return t;
		}
	};

	int loopLabelCount;
	std::deque<Token> tokens;
	Position lineNumber;
	GetTokenAndAdvance getTokenAndAdvanceVisitor;

	std::unique_ptr<ASTNode> parseProgram();
	std::unique_ptr<ASTNode> parseFunctionDeclaration();
	std::unique_ptr<ASTNode> parseStatement();
	std::unique_ptr<ASTNode> parsePrimary();
	std::unique_ptr<ASTNode> parseUnaryOrPrimary();
	std::unique_ptr<ASTNode> parseBinaryOp(int minPrecedence);
	std::unique_ptr<ASTNode> parseExpression();
	std::unique_ptr<ASTNode> parseDeclaration();
	std::unique_ptr<ASTNode> parseIncrementDecrement(std::unique_ptr<ASTNode>& expression, Symbol symbol);
	std::unique_ptr<ASTNode> parseCondition();
	std::unique_ptr<ASTNode> parseBlockItem();

	Token getTokenAndAdvance();

	template <typename T>
	T getTokenAndAdvance();

	template <typename T>
	T getTokenAndAdvance(T expected);

	template <typename T, typename... Args>
	std::unique_ptr<T> make_node(Args&&... args);

	Token peekToken();
	void endLine() {
		getTokenAndAdvance(Symbol::SEMICOLON);
		lineNumber.first++;
	}

	Impl(const std::vector<Token>& tokens) : tokens(tokens.begin(), tokens.end()), lineNumber({1, ""}) {
		getTokenAndAdvanceVisitor.tokens = &this->tokens;
	}
};

template <typename T>
T Parser::Impl::getTokenAndAdvance() {
	if (tokens.empty()) {
		throw syntax_error("Unexpected EOF");
	}
	if (!std::holds_alternative<T>(tokens.front())) {
		throw syntax_error(std::format("Unexpected token {} at {}", tokens.front(), lineNumber));
	}
	auto t = std::get<T>(getTokenAndAdvance());
	return t;
}

template <typename T>
T Parser::Impl::getTokenAndAdvance(T expected) {
	if (!std::holds_alternative<T>(peekToken())) {
		throw syntax_error(std::format("Expected {} but got {} at {}", tokenPrinter(expected), peekToken(),
		                               lineNumber));
	}
	auto t = std::get<T>(getTokenAndAdvance());
	if (t != expected) {
		throw syntax_error(std::format("Expected {} but got {} at {}", tokenPrinter(expected), tokenPrinter(t),
		                               lineNumber));
	}
	return t;
}

template <typename T, typename... Args>
std::unique_ptr<T> Parser::Impl::make_node(Args&&... args) {
	auto node = std::make_unique<T>(forward::forward<Args>(args)...);
	node->lineNumber = lineNumber;
	return node;
}

Token Parser::Impl::peekToken() {
	if (tokens.empty()) {
		throw syntax_error("Unexpected EOF");
	}
	return tokens.front();
}

Parser::Parser(const std::vector<Token>& tokens) : impl(std::make_unique<Impl>(tokens)) {
}

Token Parser::Impl::getTokenAndAdvance() {
	if (tokens.empty()) {
		throw syntax_error("Unexpected EOF");
	}
	return std::visit(getTokenAndAdvanceVisitor, tokens.front());
}

std::unique_ptr<ASTNode> Parser::Impl::parseProgram() {
	auto program = make_node<ProgramNode>();
	program->function_declaration = parseFunctionDeclaration();
	return program;
}

std::unique_ptr<ASTNode> Parser::Impl::parseFunctionDeclaration() {
	getTokenAndAdvance(Keyword::INT);
	std::string functionName = getTokenAndAdvance<std::string>();
	auto function_declaration = make_node<FunctionDefinitionNode>(functionName, make_node<BlockNode>());
	auto& body = function_declaration->body;
	lineNumber = {1, function_declaration->identifier}; // reset line number at new function
	getTokenAndAdvance(Symbol::OPEN_PAREN);
	getTokenAndAdvance(Symbol::CLOSED_PAREN);
	getTokenAndAdvance(Symbol::OPEN_BRACE);

	for (Token nextToken = peekToken(); nextToken != Symbol::CLOSED_BRACE; nextToken = peekToken()) {
		if (std::unique_ptr<ASTNode> blockItem = parseBlockItem()) {
			body->block_items.emplace_back(std::move(blockItem));
		}
	}
	getTokenAndAdvance(Symbol::CLOSED_BRACE);
	if (!tokens.empty()) {
		throw syntax_error(std::format("Unexpected token {} at {}", tokens.front(), lineNumber));
	}
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
	Token token = peekToken();
	std::unique_ptr<ASTNode> blockItem = nullptr;
	if (std::holds_alternative<Keyword>(token)) {
		switch (std::get<Keyword>(token)) {
		case Keyword::INT:
			getTokenAndAdvance();
			blockItem = parseDeclaration();
			endLine();
			break;
		default:
			blockItem = parseStatement();
		}
	} else {
		blockItem = parseStatement();
	}
	return blockItem;
}

std::unique_ptr<ASTNode> Parser::Impl::parseStatement() {
	std::unique_ptr<ASTNode> statement = nullptr;
	Token token = peekToken();
	if (std::holds_alternative<Keyword>(token)) {
		switch (getTokenAndAdvance<Keyword>()) {
		case Keyword::RETURN:
			{
				auto returnNode = make_node<ReturnNode>();
				returnNode->expression = parseExpression();
				statement = std::move(returnNode);
				endLine();
				break;
			}
		case Keyword::IF:
			{
				getTokenAndAdvance(Symbol::OPEN_PAREN);
				auto expression = parseExpression();
				getTokenAndAdvance(Symbol::CLOSED_PAREN);
				auto body = parseStatement();
				if (peekToken() == Keyword::ELSE) {
					getTokenAndAdvance();
					auto elseBody = parseStatement();
					statement = make_node<ConditionNode>(expression, body, elseBody, false);
				}
				else {
					statement = make_node<ConditionNode>(expression, body, nullptr, false);
				}
				break;
			}
		case Keyword::ELSE: // else without if
			throw syntax_error(std::format("Unexpected else at {}", lineNumber));
		case Keyword::WHILE:
		{
			getTokenAndAdvance(Symbol::OPEN_PAREN);
			auto expression = parseExpression();
			getTokenAndAdvance(Symbol::CLOSED_PAREN);
			auto body = parseStatement();
			statement = make_node<WhileNode>(expression, body, std::to_string(loopLabelCount++), false);
			break;
		}
		case Keyword::BREAK:
			statement = make_node<BreakNode>();
			endLine();
			break;
		case Keyword::CONTINUE:
			statement = make_node<ContinueNode>();
			endLine();
			break;
		case Keyword::DO:
		{
			auto body = parseStatement();
			getTokenAndAdvance(Keyword::WHILE);
			getTokenAndAdvance(Symbol::OPEN_PAREN);
			auto expression = parseExpression();
			getTokenAndAdvance(Symbol::CLOSED_PAREN);
			statement = make_node<WhileNode>(expression, body, std::to_string(loopLabelCount++), true);
			endLine();
			break;
		}
		case Keyword::FOR:
		{
			getTokenAndAdvance(Symbol::OPEN_PAREN);
			auto init = parseBlockItem();
			auto condition = parseStatement();
			std::unique_ptr<ASTNode> increment = nullptr;
			if (peekToken() != Symbol::CLOSED_PAREN) { // may not exist
				increment = parseExpression();
			}
			getTokenAndAdvance(Symbol::CLOSED_PAREN);
			auto body = parseStatement();
			statement = make_node<ForNode>(init, condition, increment, body, std::to_string(loopLabelCount++));
			break;
		}
		default:
			throw syntax_error(std::format("Unexpected keyword {} at {}", token, lineNumber));
		}
	} else if (token == Symbol::OPEN_BRACE) {
		auto block = make_node<BlockNode>();
		getTokenAndAdvance();
		for (Token nextToken = peekToken(); nextToken != Symbol::CLOSED_BRACE; nextToken = peekToken()) {
			if (std::unique_ptr<ASTNode> blockItem = parseBlockItem()) {
				block->block_items.emplace_back(std::move(blockItem));
			}
		}
		getTokenAndAdvance(Symbol::CLOSED_BRACE);
		statement = std::move(block);
	} else if (token == Symbol::SEMICOLON) {
		endLine();
	} else {
		statement = parseExpression();
		endLine();
	}
	return statement;
}

static int getPrecedence(Symbol op) {
	switch (op) {
	case Symbol::ASTERISK:
	case Symbol::FORWARD_SLASH:
	case Symbol::PERCENTAGE:
		return 50;
	case Symbol::PLUS:
	case Symbol::MINUS:
		return 45;
	case Symbol::DOUBLE_GREATER_THAN:
	case Symbol::DOUBLE_LESS_THAN:
		return 40;
	case Symbol::LESS_THAN:
	case Symbol::LESS_THAN_OR_EQUAL:
	case Symbol::GREATER_THAN:
	case Symbol::GREATER_THAN_OR_EQUAL:
		return 35;
	case Symbol::DOUBLE_EQUALS:
	case Symbol::NOT_EQUALS:
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
	case Symbol::QUESTION_MARK:
		return 3;
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
	if (std::holds_alternative<std::string>(token)) {
		// variable
		return make_node<VariableNode>(getTokenAndAdvance<std::string>());
	}
	throw syntax_error(std::format("Unexpected token {} at {}", peekToken(), lineNumber));
}

std::unique_ptr<ASTNode> Parser::Impl::parseIncrementDecrement(std::unique_ptr<ASTNode>& expression, Symbol symbol) {
	if (dynamic_cast<LvalueNode*>(expression.get())) {
		return make_node<PrefixNode>(std::unique_ptr<LvalueNode>(static_cast<LvalueNode*>(expression.release())),
		                             symbol == Symbol::DOUBLE_PLUS ? BinaryOperator::ADD : BinaryOperator::SUBTRACT);
	}
	throw semantic_error(std::format("Expected lvalue at {}", expression->lineNumber));
}

std::unique_ptr<ASTNode> Parser::Impl::parseUnaryOrPrimary() {
	Token token = peekToken();
	if (std::holds_alternative<Symbol>(token)) {
		const Symbol symbol = std::get<Symbol>(token);

		// prefix increment/decrement
		if (symbol == Symbol::DOUBLE_PLUS || symbol == Symbol::DOUBLE_MINUS) {
			getTokenAndAdvance();
			auto expression = parsePrimary();

			return parseIncrementDecrement(expression, symbol);
		}
		if (isUnaryOp(symbol)) {
			auto op = static_cast<UnaryOperator>(getTokenAndAdvance<Symbol>());
			auto expression = parseUnaryOrPrimary();
			auto unaryNode = make_node<UnaryNode>(op, expression);
			return unaryNode;
		}
	}
	auto primary = parsePrimary();

	// try postfix increment/decrement
	token = peekToken();
	if (token == Symbol::DOUBLE_PLUS || token == Symbol::DOUBLE_MINUS) {
		getTokenAndAdvance();
		if (dynamic_cast<LvalueNode*>(primary.get())) {
			return make_node<PostfixNode>(std::unique_ptr<LvalueNode>(static_cast<LvalueNode*>(primary.release())),
			                              token == Symbol::DOUBLE_PLUS
				                              ? BinaryOperator::ADD
				                              : BinaryOperator::SUBTRACT);
		}
		throw semantic_error(std::format("Expected lvalue at {}", primary->lineNumber));
	}
	return primary;
}

/*
 Parse the middle term of a ternary statement, keeps going until it hits a colon
 */
std::unique_ptr<ASTNode> Parser::Impl::parseCondition() {
	auto middle = parseBinaryOp(0);
	getTokenAndAdvance(Symbol::COLON);
	return middle;
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
	 else if next_token is "?":
		 middle = parse_conditional_middle(tokens)
		 right = parse_exp(tokens, precedence(next_token))
		 left = Conditional(left, middle, right)
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
				// regular assignment
				if (dynamic_cast<LvalueNode*>(left.get())) {
					auto right = parseBinaryOp(getPrecedence(symbol));
					left = make_node<AssignmentNode>(
						std::unique_ptr<LvalueNode>(static_cast<LvalueNode*>(left.release())),
						right);
				}
				else {
					throw semantic_error(std::format("Expected lvalue at {}", left->lineNumber));
				}
			}
			else if (peekToken() == Symbol::EQUALS) {
				// compound assignment
				if (auto* var = dynamic_cast<LvalueNode*>(left.get())) {
					/*
					 Turn x ?= rhs into x = (x ? rhs)
					 */
					getTokenAndAdvance(); // remove the = operator
					auto right = parseBinaryOp(getPrecedence(Symbol::EQUALS));
					left = make_node<AssignmentNode>(var->clone(),
					                                 make_node<BinaryNode>(
						                                 static_cast<BinaryOperator>(symbol), left, right));
				}
				else {
					throw semantic_error(std::format("Expected lvalue at {}", left->lineNumber));
				}
			}
			else if (symbol == Symbol::QUESTION_MARK) {
				// ternary
				auto middle = parseCondition();
				auto right = parseBinaryOp(getPrecedence(symbol));
				left = make_node<ConditionNode>(left, middle, right, true);
			}
			else {
				auto right = parseBinaryOp(getPrecedence(symbol) + 1);
				left = make_node<BinaryNode>(static_cast<BinaryOperator>(symbol), left, right);
			}
		}
		return left;
	}
	catch (std::bad_variant_access&) {
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
