#pragma once
#include <memory>
#include <string>
#include <unordered_map>
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

    virtual void visitProgram(ProgramNode* const node) = 0;
    virtual void visitFunctionDefinition(FunctionDefinitionNode* const node) = 0;
    virtual void visitDeclaration(DeclarationNode* const node) = 0;
    virtual void visitAssignment(AssignmentNode* const node) = 0;
    virtual void visitReturn(ReturnNode* const node) = 0;
    virtual void visitUnary(UnaryNode* const node) = 0;
    virtual void visitBinary(BinaryNode* const node) = 0;
    virtual void visitConst(ConstNode* const node) = 0;
    virtual void visitVariable(VariableNode* const node) = 0;
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
    virtual void accept(Visitor& visitor) = 0;
    friend std::ostream& operator<<(std::ostream& os, const ASTNode& node);
};

// Program node (root of AST)
struct ProgramNode : public ASTNode {
    std::unique_ptr<ASTNode> function_declaration;

    void accept(Visitor& visitor) override {
        visitor.visitProgram(this);
    }

    void generate(const CodeContext& context) const;
};

// Function definition node
struct FunctionDefinitionNode : public ASTNode {
    std::string identifier;
    std::vector<std::unique_ptr<ASTNode>> block_items;

    void accept(Visitor& visitor) override {
        visitor.visitFunctionDefinition(this);
    }

    void generate(const CodeContext& context);
};

// Variable declaration node
struct DeclarationNode : public ASTNode {
    std::string identifier;
    std::unique_ptr<ASTNode> expression;

    void accept(Visitor& visitor) override {
        visitor.visitDeclaration(this);
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
        visitor.visitAssignment(this);
    }
};

// Return statement node
struct ReturnNode : public ASTNode {
    std::unique_ptr<ASTNode> expression;

    void accept(Visitor& visitor) override {
        visitor.visitReturn(this);
    }
};

// Unary operation node
struct UnaryNode : public ASTNode {
    UnaryOperator op;
    std::unique_ptr<ASTNode> expression;

    UnaryNode(UnaryOperator op, std::unique_ptr<ASTNode>& expression) : op(op), expression(std::move(expression)) {}

    void accept(Visitor& visitor) override {
        visitor.visitUnary(this);
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
        visitor.visitBinary(this);
    }
};

// Constant value node
struct ConstNode : public ASTNode {
    Number value;

    explicit ConstNode(Number value) : value(value) {}

    void accept(Visitor& visitor) override {
        visitor.visitConst(this);
    }
};

// Variable reference node
struct VariableNode : public ASTNode {
    std::string identifier;

	explicit VariableNode(const std::string& identifier) : identifier(identifier) {}

    void accept(Visitor& visitor) override {
        visitor.visitVariable(this);
    }
};

// Non-modifying visitor for printing
class PrintVisitor : public Visitor {
public:
    PrintVisitor(std::ostream& os, int indent = 0);

    void visitProgram(ProgramNode* const node) override;
    void visitFunctionDefinition(FunctionDefinitionNode* const node) override;
    void visitDeclaration(DeclarationNode* const node) override;
    void visitAssignment(AssignmentNode* const node) override;
    void visitReturn(ReturnNode* const node) override;
    void visitUnary(UnaryNode* const node) override;
    void visitBinary(BinaryNode* const node) override;
    void visitConst(ConstNode* const node) override;
    void visitVariable(VariableNode* const node) override;

private:
    std::ostream& os;
    int indent;

    void increaseIndent();
    void decreaseIndent();
    std::string getIndent() const;
};

// TAC generation visitor
class TacVisitor : public Visitor {
public:
    TacVisitor(FunctionBody& body);

    void visitProgram(ProgramNode* const node) override;
    void visitFunctionDefinition(FunctionDefinitionNode* const node) override;
    void visitDeclaration(DeclarationNode* const node) override;
    void visitAssignment(AssignmentNode* const node) override;
    void visitReturn(ReturnNode* const node) override;
    void visitUnary(UnaryNode* const node) override;
    void visitBinary(BinaryNode* const node) override;
    void visitConst(ConstNode* const node) override;
    void visitVariable(VariableNode* const node) override;

    Operand getResult() const;

private:
    FunctionBody& body;
    Operand result;
};

// Variable resolution visitor
class VariableResolutionVisitor : public Visitor {
public:
    VariableResolutionVisitor() : counter(0) {}

    void visitProgram(ProgramNode* const node) override;
    void visitFunctionDefinition(FunctionDefinitionNode* const node) override;
    void visitDeclaration(DeclarationNode* const node) override;
    void visitAssignment(AssignmentNode* const node) override;
    void visitReturn(ReturnNode* const node) override;
    void visitUnary(UnaryNode* const node) override;
    void visitBinary(BinaryNode* const node) override;
    void visitConst(ConstNode* const node) override {}
    void visitVariable(VariableNode* const node) override;

private:
    int counter;
    std::unordered_map<std::string, std::string> variableMap;

    std::string makeTemporary(const std::string& name) {
        return std::format("{}.{}", name, counter++);
    }
};

inline std::ostream& operator<<(std::ostream& os, ASTNode& node) {
    PrintVisitor p(os);
    node.accept(p);
    return os;
}