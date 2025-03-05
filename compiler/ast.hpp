#pragma once
#include <memory>
#include <string>
#include <ostream>
#include <sstream>
#include "lexer.hpp"

enum class Types {
	INT
};

enum UnaryOperator {
	NEGATION,
	BITWISE_NOT,
	LOGICAL_NOT
};

enum BinaryOperator {
	ADD,
	SUBTRACT,
	MULTIPLY,
	DIVIDE
};

struct CodeContext {
	std::ostream& out;
};

struct ASTNode {
	virtual ~ASTNode() = default;
	virtual std::ostream& print(std::ostream& os, int) const = 0;
	virtual void generate(CodeContext&) const = 0;
};

inline std::ostream& operator<<(std::ostream& os, const ASTNode& node) {
	return node.print(os, 0);
}

struct ProgramNode : public ASTNode {
	std::unique_ptr<ASTNode> function_declaration;
	std::ostream& print(std::ostream&, int) const override;

	void generate(CodeContext& context) const override;
};

template<typename ReturnType>
struct FunctionDeclarationNode : public ASTNode {
	using return_type = ReturnType;
	std::string identifier;
	std::unique_ptr<ASTNode> statement;

	std::ostream& print(std::ostream& os, int indent) const override {
		os << std::string(indent, ' ') << "FUNCTION DECLARATION NODE: " << identifier << '\n';
		statement->print(os, indent + 1);
		return os;
	}

	void generate(CodeContext& context) const override {
		if (statement) {
			statement->generate(context);
		}
	}
};

struct ReturnNode : public ASTNode {
	std::unique_ptr<ASTNode> expression;

	std::ostream& print(std::ostream&, int) const override;
	void generate(CodeContext& context) const override;
};

struct UnaryNode : public ASTNode {
	UnaryOperator op;
	std::unique_ptr<ASTNode> expression;

	UnaryNode(UnaryOperator op, std::unique_ptr<ASTNode>& expression) : op(op), expression(std::move(expression)) {}

	std::ostream& print(std::ostream&, int) const override;
	void generate(CodeContext& context) const override;
};

struct BinaryNode : public ASTNode {
	BinaryOperator op;
	std::unique_ptr<ASTNode> left;
	std::unique_ptr<ASTNode> right;

	BinaryNode(BinaryOperator op, std::unique_ptr<ASTNode>& left, std::unique_ptr<ASTNode>& right) : op(op), left(std::move(left)), right(std::move(right)) {}

	std::ostream& print(std::ostream& os, int indent) const override;
	void generate(CodeContext& context) const override {}
};

struct ConstNode : public ASTNode {
	Number value;

	std::ostream& print(std::ostream&, int) const override;
	void generate(CodeContext& context) const override;
};
