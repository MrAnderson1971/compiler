#pragma once
#include <memory>
#include <string>
#include <vector>
#include "tac.hpp"
#include "type.hpp"

// Forward declarations
struct ProgramNode;
struct FunctionDefinitionNode;
struct DeclarationNode;
struct AssignmentNode;
struct ReturnNode;
struct UnaryNode;
struct BinaryNode;
struct ConstNode;
struct VariableNode;

// Visitor base class
class Visitor {
public:
    virtual ~Visitor() = default;

    virtual void visitProgram(const ProgramNode& node) = 0;
    virtual void visitFunctionDefinition(const FunctionDefinitionNode& node) = 0;
    virtual void visitDeclaration(const DeclarationNode& node) = 0;
    virtual void visitAssignment(const AssignmentNode& node) = 0;
    virtual void visitReturn(const ReturnNode& node) = 0;
    virtual void visitUnary(const UnaryNode& node) = 0;
    virtual void visitBinary(const BinaryNode& node) = 0;
    virtual void visitConst(const ConstNode& node) = 0;
    virtual void visitVariable(const VariableNode& node) = 0;
};

// CodeContext definition
struct CodeContext {
    std::ostream& out;
    std::string methodName = "";
    unsigned int variableCounter = 0;
};

// Base node class
struct ASTNode {
    virtual ~ASTNode() = default;

    // Single visitor pattern method
    virtual void accept(Visitor& visitor) const = 0;
	friend std::ostream& operator<<(std::ostream& os, const ASTNode& node);
};

// Program node (root of AST)
struct ProgramNode : public ASTNode {
    std::unique_ptr<ASTNode> function_declaration;

    void accept(Visitor& visitor) const override {
        visitor.visitProgram(*this);
    }

	void generate(CodeContext& context) const;
};

// Function definition node
struct FunctionDefinitionNode : public ASTNode {
    std::string identifier;
    std::vector<std::unique_ptr<ASTNode>> block_items;

    void accept(Visitor& visitor) const override {
        visitor.visitFunctionDefinition(*this);
    }

	void generate(CodeContext& context) const;
};

// Variable declaration node
struct DeclarationNode : public ASTNode {
    std::string identifier;
    std::unique_ptr<ASTNode> expression;

    void accept(Visitor& visitor) const override {
        visitor.visitDeclaration(*this);
    }
};

// Assignment node
struct AssignmentNode : public ASTNode {
    std::unique_ptr<ASTNode> left;
    std::unique_ptr<ASTNode> right;

	AssignmentNode(std::unique_ptr<ASTNode>& left, std::unique_ptr<ASTNode>& right)
		: left(std::move(left)), right(std::move(right)) {
	}

    void accept(Visitor& visitor) const override {
        visitor.visitAssignment(*this);
    }
};

// Return statement node
struct ReturnNode : public ASTNode {
    std::unique_ptr<ASTNode> expression;

    void accept(Visitor& visitor) const override {
        visitor.visitReturn(*this);
    }
};

// Unary operation node
struct UnaryNode : public ASTNode {
    UnaryOperator op;
    std::unique_ptr<ASTNode> expression;

	UnaryNode(UnaryOperator op, std::unique_ptr<ASTNode>& expression) : op(op), expression(std::move(expression)) {}

    void accept(Visitor& visitor) const override {
        visitor.visitUnary(*this);
    }
};

// Binary operation node
struct BinaryNode : public ASTNode {
    BinaryOperator op;
    std::unique_ptr<ASTNode> left;
    std::unique_ptr<ASTNode> right;

	BinaryNode(BinaryOperator op, std::unique_ptr<ASTNode>& left, std::unique_ptr<ASTNode>& right)
		: op(op), left(std::move(left)), right(std::move(right)) {
	}

    void accept(Visitor& visitor) const override {
        visitor.visitBinary(*this);
    }
};

// Constant value node
struct ConstNode : public ASTNode {
    Number value;

	explicit ConstNode(Number value) : value(value) {}

    void accept(Visitor& visitor) const override {
        visitor.visitConst(*this);
    }
};

// Variable reference node
struct VariableNode : public ASTNode {
    std::string identifier;

    void accept(Visitor& visitor) const override {
        visitor.visitVariable(*this);
    }
};

class PrintVisitor : public Visitor {
public:
    PrintVisitor(std::ostream& os, int indent = 0);

    void visitProgram(const ProgramNode& node) override;
    void visitFunctionDefinition(const FunctionDefinitionNode& node) override;
    void visitDeclaration(const DeclarationNode& node) override;
    void visitAssignment(const AssignmentNode& node) override;
    void visitReturn(const ReturnNode& node) override;
    void visitUnary(const UnaryNode& node) override;
    void visitBinary(const BinaryNode& node) override;
    void visitConst(const ConstNode& node) override;
    void visitVariable(const VariableNode& node) override;

private:
    std::ostream& os;
    int indent;

    void increaseIndent();
    void decreaseIndent();
    std::string getIndent() const;
};

class TacVisitor : public Visitor {
public:
    TacVisitor(FunctionBody& body);

    void visitProgram(const ProgramNode& node) override;
    void visitFunctionDefinition(const FunctionDefinitionNode& node) override;
    void visitDeclaration(const DeclarationNode& node) override;
    void visitAssignment(const AssignmentNode& node) override;
    void visitReturn(const ReturnNode& node) override;
    void visitUnary(const UnaryNode& node) override;
    void visitBinary(const BinaryNode& node) override;
    void visitConst(const ConstNode& node) override;
    void visitVariable(const VariableNode& node) override;

    Operand getResult() const;

private:
    FunctionBody& body;
    Operand result;
};

inline std::ostream& operator<<(std::ostream& os, const ASTNode& node) {
    PrintVisitor p(os);
    node.accept(p);
    return os;
}
