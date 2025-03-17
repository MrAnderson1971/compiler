#pragma once
#include <memory>
#include <string>
#include "tac.hpp"
#include "type.hpp"

struct CodeContext {
	std::ostream& out;

	std::string methodName = "";
	unsigned int variableCounter = 0;
};

struct ASTNode {
	virtual ~ASTNode() = default;
	virtual std::ostream& print(std::ostream& os, int) const = 0;
	virtual Operand makeTac(FunctionBody& body) const {
		return nullptr;
	}
};

inline std::ostream& operator<<(std::ostream& os, const ASTNode& node) {
	return node.print(os, 0);
}

struct ProgramNode : public ASTNode {
	std::unique_ptr<ASTNode> function_declaration;
	std::ostream& print(std::ostream&, int) const override;

	void generate(CodeContext& context) const;
};

struct FunctionDefinitionNode : public ASTNode {
	std::string identifier;
	std::vector<std::unique_ptr<ASTNode>> block_items;

	std::ostream& print(std::ostream& os, int indent) const override;
	Operand makeTac(FunctionBody& body) const override;
	void generate(CodeContext& context) const;
};

struct DeclarationNode : public ASTNode {
	std::string identifier;
	std::unique_ptr<ASTNode> expression;

	std::ostream& print(std::ostream& os, int indent) const override;
	Operand makeTac(FunctionBody& body) const override { return nullptr; } // TODO
};

struct AssignmentNode : public ASTNode {
	std::unique_ptr<ASTNode> left;
	std::unique_ptr<ASTNode> right;

	AssignmentNode(std::unique_ptr<ASTNode>& left, std::unique_ptr<ASTNode>& right) : left(std::move(left)), right(std::move(right)) {}

	std::ostream& print(std::ostream& os, int indent) const override;
	Operand makeTac(FunctionBody& body) const override { return nullptr; } // TODO
};

struct ReturnNode : public ASTNode {
	std::unique_ptr<ASTNode> expression;

	std::ostream& print(std::ostream&, int) const override;
	Operand makeTac(FunctionBody& body) const override;
};

struct UnaryNode : public ASTNode {
	UnaryOperator op;
	std::unique_ptr<ASTNode> expression;

	UnaryNode(UnaryOperator op, std::unique_ptr<ASTNode>& expression) : op(op), expression(std::move(expression)) {}

	std::ostream& print(std::ostream&, int) const override;
	Operand makeTac(FunctionBody& body) const override;
};

struct BinaryNode : public ASTNode {
	BinaryOperator op;
	std::unique_ptr<ASTNode> left;
	std::unique_ptr<ASTNode> right;

	BinaryNode(BinaryOperator op, std::unique_ptr<ASTNode>& left, std::unique_ptr<ASTNode>& right) : op(op), left(std::move(left)), right(std::move(right)) {}

	std::ostream& print(std::ostream& os, int indent) const override;
	Operand makeTac(FunctionBody& body) const override;
};

struct ConstNode : public ASTNode {
	Number value;

	ConstNode(Number value) : value(value) {}
	std::ostream& print(std::ostream&, int) const override;
	Operand makeTac(FunctionBody& body) const override;
};

struct VariableNode : public ASTNode {
	std::string identifier;

	VariableNode(std::string identifier) : identifier(std::move(identifier)) {}

	std::ostream& print(std::ostream&os, int indent) const override;
	Operand makeTac(FunctionBody& body) const override { return nullptr; }; // TODO
};
