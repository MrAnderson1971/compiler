#pragma once
#include "ast_nodes.hpp"
#include <stack>

// Variable resolution visitor
class VariableResolutionVisitor : public FullVisitor {
public:
    VariableResolutionVisitor(const std::shared_ptr<std::string>& function) : counter(0), layer(0), function(function) {}

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
	void visitCondition(ConditionNode* const node) override;
	void visitBlock(BlockNode* const node) override;
	void visitWhile(WhileNode* const node) override;
    void visitBreak(BreakNode* const node) override;
	void visitContinue(ContinueNode* const node) override;
    void visitFor(ForNode* const node) override;

private:
    int counter;
    int layer;
    std::shared_ptr<std::string> function;
    std::unordered_map<std::string, std::stack<int>> variableMap;
	std::stack<std::pair<std::shared_ptr<std::string>, bool>> loopLabels;
};
