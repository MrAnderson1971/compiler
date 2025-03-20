/*
 Separate header file for AST nodes for faster compilation times.
 */

#pragma once
#include "ast.hpp"
#include "tac.hpp"

struct FunctionDefinitionNode;
struct DeclarationNode;
struct AssignmentNode;
struct ReturnNode;
struct UnaryNode;
struct BinaryNode;
struct ConstNode;
struct VariableNode;

class FullVisitor : public Visitor {
public:
    virtual void visitFunctionDefinition(FunctionDefinitionNode* const node) = 0;
    virtual void visitDeclaration(DeclarationNode* const node) = 0;
    virtual void visitAssignment(AssignmentNode* const node) = 0;
    virtual void visitReturn(ReturnNode* const node) = 0;
    virtual void visitUnary(UnaryNode* const node) = 0;
    virtual void visitBinary(BinaryNode* const node) = 0;
    virtual void visitConst(ConstNode* const node) = 0;
    virtual void visitVariable(VariableNode* const node) = 0;
};

// Function definition node
struct FunctionDefinitionNode : public ASTNode {
    std::string identifier;
    std::vector<std::unique_ptr<ASTNode>> block_items;

    void accept(Visitor& visitor) override {
		static_cast<FullVisitor&>(visitor).visitFunctionDefinition(this); // static_cast failure is impossible
    }

    void generate(const CodeContext& context);
};

// Variable declaration node
struct DeclarationNode : public ASTNode {
    std::string identifier;
    std::unique_ptr<ASTNode> expression;

    void accept(Visitor& visitor) override {
        static_cast<FullVisitor&>(visitor).visitDeclaration(this);
    }
};

// Assignment node
struct AssignmentNode : public ASTNode {
    std::unique_ptr<ASTNode> left;
    std::unique_ptr<ASTNode> right;

    AssignmentNode(std::unique_ptr<ASTNode>& left, std::unique_ptr<ASTNode>& right)
        : left(std::move(left)), right(std::move(right)) {
    }

    void accept(Visitor& visitor) override {
        static_cast<FullVisitor&>(visitor).visitAssignment(this);
    }
};

// Return statement node
struct ReturnNode : public ASTNode {
    std::unique_ptr<ASTNode> expression;

    void accept(Visitor& visitor) override {
        static_cast<FullVisitor&>(visitor).visitReturn(this);
    }
};

// Unary operation node
struct UnaryNode : public ASTNode {
    UnaryOperator op;
    std::unique_ptr<ASTNode> expression;

    UnaryNode(UnaryOperator op, std::unique_ptr<ASTNode>& expression) : op(op), expression(std::move(expression)) {}

    void accept(Visitor& visitor) override {
        static_cast<FullVisitor&>(visitor).visitUnary(this);
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

    void accept(Visitor& visitor) override {
        static_cast<FullVisitor&>(visitor).visitBinary(this);
    }
};

// Constant value node
struct ConstNode : public ASTNode {
    Number value;

    explicit ConstNode(Number value) : value(value) {}

    void accept(Visitor& visitor) override {
        static_cast<FullVisitor&>(visitor).visitConst(this);
    }
};

// Variable reference node
struct VariableNode : public ASTNode {
    std::string identifier;

    explicit VariableNode(const std::string& identifier) : identifier(identifier) {}

    void accept(Visitor& visitor) override {
        static_cast<FullVisitor&>(visitor).visitVariable(this);
    }
};
