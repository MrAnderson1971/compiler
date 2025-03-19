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
			throw syntax_error(std::format("Unexpected token {} at line {}", tokens.front(), lineNumber));
		}
		auto t = std::get<T>(tokens.front());
		tokens.pop_front();
		return t;
	}

	template<typename T>
	T getTokenAndAdvance(T expected) {
		if (!std::holds_alternative<T>(peekToken())) {
			throw syntax_error(std::format("Expected {} but got {} at line {}", tokenPrinter(expected), peekToken(), lineNumber));
		}
		auto t = std::get<T>(getTokenAndAdvance());
		if (t != expected) {
			throw syntax_error(std::format("Expected {} but got {} at line {}", tokenPrinter(expected), tokenPrinter(t), lineNumber));
		}
		return t;
	}

	Token peekToken();

public:
	Parser(const std::vector<Token>& tokens);
	std::unique_ptr<ASTNode> parse();
	int lineNumber;
};