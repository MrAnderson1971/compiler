#pragma once

#include <deque>
#include <sstream>
#include "lexer.hpp"
#include "ast.hpp"

class Parser {
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
	T getTokenAndAdvance() {
		if (tokens.empty()) {
			throw syntax_error("Unexpected EOF");
		}
		if (!std::holds_alternative<T>(tokens.front())) {
			std::stringstream ss;
			ss << "Unexpected token ";
			std::visit(TokenPrinter{ ss }, tokens.front());
			throw syntax_error(ss.str());
		}
		auto t = std::get<T>(tokens.front());
		tokens.pop_front();
		return t;
	}

	template<typename T>
	T getTokenAndAdvance(T expected) {
		if (!std::holds_alternative<T>(peekToken())) {
			std::stringstream ss;
			ss << "Expected ";
			TokenPrinter{ ss }(expected);
			ss << " but got ";
			std::visit(TokenPrinter{ ss }, peekToken());
			throw syntax_error(ss.str());
		}
		auto t = std::get<T>(getTokenAndAdvance());
		if (t != expected) {
			std::stringstream ss;
			ss << "Expected ";
			TokenPrinter{ ss }(expected);
			ss<< " but got ";
			TokenPrinter{ ss }(t);
			throw syntax_error(ss.str());
		}
		return t;
	}

	Token peekToken();

public:
	Parser(const std::vector<Token>& tokens);
	std::unique_ptr<ASTNode> parse();
};