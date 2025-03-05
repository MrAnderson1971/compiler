#include <functional>
#include "parser.hpp"

Token Parser::peekToken() {
	if (tokens.empty()) {
		throw std::runtime_error("Unexpected EOF");
	}
	return tokens.front();
}

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
	auto constNode = std::make_unique<ConstNode>();
	constNode->value = value;
	return constNode;
}

std::unique_ptr<ASTNode> Parser::parseTerm() {
	auto factor = parseFactor();
	while (true) {
		Token next = peekToken();
		if (!std::holds_alternative<Symbol>(next) || !(std::get<Symbol>(next) == ASTERISK || std::get<Symbol>(next) == FORWARD_SLASH)) {
			break;
		}
		BinaryOperator op = std::get<Symbol>(getTokenAndAdvance()) == ASTERISK ? MULTIPLY : DIVIDE;
		auto nextFactor = parseFactor();
		auto binaryNode = std::make_unique<BinaryNode>(op, factor, nextFactor);
		factor = std::move(binaryNode);
	}
	return factor;
}

std::unique_ptr<ASTNode> Parser::parseExpression() {
/*def parse_expression(toks):
    term = parse_term(toks) //pops off some tokens
    next = toks.peek() //check the next token, but don't pop it off the list yet
    while next == PLUS or next == MINUS: //there's another term!
        op = convert_to_op(toks.next())
        next_term = parse_term(toks) //pops off some more tokens
        term = BinOp(op, term, next_term)
        next = toks.peek()

    return t1*/
	auto term = parseTerm();
	while (true) {
		Token next = peekToken();
		if (!std::holds_alternative<Symbol>(next) || !(std::get<Symbol>(next) == Symbol::PLUS || std::get<Symbol>(next) == Symbol::MINUS)) {
			break;
		}

		BinaryOperator op = std::get<Symbol>(getTokenAndAdvance()) == Symbol::PLUS ? BinaryOperator::ADD : BinaryOperator::SUBTRACT;
		auto nextTerm = parseTerm();
		auto binaryNode = std::make_unique<BinaryNode>(op, term, nextTerm);
		term = std::move(binaryNode);
	
	}
	return term;
}

std::unique_ptr<ASTNode> Parser::parseFactor() {
	/*def parse_factor(toks)
    next = toks.next()
    if next == OPEN_PAREN:
        //<factor> ::= "(" <exp> ")"
        exp = parse_exp(toks) //parse expression inside parens
        if toks.next() != CLOSE_PAREN: //make sure parens are balanced
            fail()
        return exp
    else if is_unop(next)
        //<factor> ::= <unary_op> <factor>
        op = convert_to_op(next)
        factor = parse_factor(toks)
        return UnOp(op, factor)
    else if next.type == "INT":
        //<factor> ::= <int>
        return Const(convert_to_int(next))
    else:
        fail()*/
	Token next = getTokenAndAdvance();
	if (std::holds_alternative<Symbol>(next) && std::get<Symbol>(next) == Symbol::OPEN_PAREN) {
		auto expression = parseExpression();
		getTokenAndAdvance(Symbol::CLOSED_PAREN);
		return expression;
	}
	else if (std::holds_alternative<Symbol>(next) && (std::get<Symbol>(next) == MINUS || std::get<Symbol>(next) == EXCLAMATION_MARK || std::get<Symbol>(next) == TILDE)) {
		UnaryOperator op;
		switch (std::get<Symbol>(next)) {
		case MINUS:
			op = NEGATION;
			break;
		case EXCLAMATION_MARK:
			op = LOGICAL_NOT;
			break;
		case TILDE:
			op = BITWISE_NOT;
			break;
		default:
			std::stringstream ss;
			ss << "Unexpected unary operator ";
			std::visit(TokenPrinter{ ss }, next);
			throw std::runtime_error(ss.str());
		}
		auto factor = parseFactor();
		return std::make_unique<UnaryNode>(op, factor);
	}
	else if (std::holds_alternative<Number>(next)) {
		return parseConst(std::get<Number>(next));
	}
	else {
		std::stringstream ss;
		ss << "Unexpected token ";
		std::visit(TokenPrinter{ ss }, next);
		throw std::runtime_error(ss.str());
	}

}

std::unique_ptr<ASTNode> Parser::parse() {
	return parseProgram();
}
