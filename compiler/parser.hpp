#pragma once

#include <deque>
#include <sstream>
#include <optional>
#include "lexer.hpp"
#include "ast.hpp"

class Parser {
	struct GetTokenAndAdvance {
		std::deque<Token>& tokens;
		
		template<typename T>
		Token operator()(T value) const {
			auto t = std::get<T>(tokens.front());
			tokens.pop_front();
			return t;
		}

		Token operator()(UnknownToken unknown) const {
			throw compiler_error("Unknown token at position " + std::to_string(unknown.position));
		}
	};
	std::deque<Token> tokens;

	std::unique_ptr<ASTNode> parseProgram();
	std::unique_ptr<ASTNode> parseFunctionDeclaration();
	std::unique_ptr<ASTNode> parseReturn();
	std::unique_ptr<ASTNode> parsePrimary();
	std::unique_ptr<ASTNode> parseUnaryOrPrimary();
	std::unique_ptr<ASTNode> parseBinaryOp(int minPrecedence);
	std::unique_ptr<ASTNode> parseExpression();
	std::unique_ptr<ASTNode> parseConst(Number value);

	Token getTokenAndAdvance();

	template<typename T>
	T getTokenAndAdvance() {
		if (tokens.empty()) {
			throw compiler_error("Unexpected EOF");
		}
		if (!std::holds_alternative<T>(tokens.front())) {
			std::stringstream ss;
			ss << "Unexpected token ";
			std::visit(TokenPrinter{ ss }, tokens.front());
			throw compiler_error(ss.str());
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
			throw compiler_error(ss.str());
		}
		auto t = std::get<T>(getTokenAndAdvance());
		if (t != expected) {
			std::stringstream ss;
			ss << "Expected ";
			TokenPrinter{ ss }(expected);
			ss<< " but got ";
			TokenPrinter{ ss }(t);
			throw compiler_error(ss.str());
		}
		return t;
	}

	Token peekToken();

public:
	Parser(std::vector<Token>&& tokens);
	std::unique_ptr<ASTNode> parse();
};