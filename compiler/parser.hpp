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
			throw std::runtime_error("Unknown token at position " + std::to_string(unknown.position));
		}
	};
	std::deque<Token> tokens;

	std::unique_ptr<ASTNode> parseProgram();
	std::unique_ptr<ASTNode> parseFunctionDeclaration();
	std::unique_ptr<ASTNode> parseReturn();
	std::unique_ptr<ASTNode> parseExpression();

	Token getTokenAndAdvance();

	template<typename T>
	T getTokenAndAdvance() {
		auto t = std::get<T>(tokens.front());
		tokens.pop_front();
		return t;
	}

	template<typename T>
	T getTokenAndAdvance(T expected) {
		auto t = std::get<T>(getTokenAndAdvance());
		if (t != expected) {
			std::stringstream ss;
			ss << "Expected ";
			TokenPrinter{ ss }(expected);
			ss<< " but got ";
			TokenPrinter{ ss }(t);
			throw std::runtime_error(ss.str());
		}
		return t;
	}

public:
	Parser(std::vector<Token>&& tokens);
	std::unique_ptr<ASTNode> parse();
};