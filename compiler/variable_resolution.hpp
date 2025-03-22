#pragma once
#include "ast_nodes.hpp"

// Variable resolution visitor
class VariableResolutionVisitor : public FullVisitor {
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
    void visitPostfix(PostfixNode* const node) override;
    void visitPrefix(PrefixNode* const node) override;

private:
    int counter;
    std::unordered_map<std::string, std::string> variableMap;

    std::string makeTemporary(const std::string& name) {
        return std::format("{}.{}", name, counter++);
    }
};
