#pragma once
#include <memory>
#include <string>
#include "type.hpp"

struct ProgramNode;

// Visitor base class
class Visitor {
public:
    virtual ~Visitor() = default;
    virtual void visitProgram(ProgramNode* const node) = 0;
};

// CodeContext definition
struct CodeContext {
    std::ostream& out;
    std::string methodName = "";
    unsigned int variableCounter = 0;
};

// Base node class
struct ASTNode {
    Position lineNumber;
    virtual ~ASTNode() = default;

    // Single visitor pattern method
    virtual void accept(Visitor& visitor) = 0;
    friend std::ostream& operator<<(std::ostream& os, ASTNode& node);
};

// Program node (root of AST)
struct ProgramNode : public ASTNode {
    std::unique_ptr<ASTNode> function_declaration;

    void accept(Visitor& visitor) override {
        visitor.visitProgram(this);
    }

    void generate(const CodeContext& context) const;
};
