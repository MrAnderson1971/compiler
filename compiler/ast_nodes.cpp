#include "ast_nodes.hpp"
#include <iostream>
#include <sstream>
#include "tac.hpp"
#include "tac_visitor.hpp"
#include "variable_resolution.hpp"

// Non-modifying visitor for printing
class PrintVisitor : public FullVisitor {
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
	void visitPostfix(PostfixNode* const node) override;
	void visitPrefix(PrefixNode* const node) override;

private:
    std::ostream& os;
    int indent;

    void increaseIndent();
    void decreaseIndent();
    std::string getIndent() const;
};


std::ostream& operator<<(std::ostream& os, ASTNode& node) {
    PrintVisitor p(os);
    node.accept(p);
    return os;
}

void ProgramNode::generate(const CodeContext& context) const {
    if (function_declaration) {
        if (auto* funcDef = dynamic_cast<FunctionDefinitionNode*>(function_declaration.get())) {
            funcDef->generate(context);
        }
    }
}

void FunctionDefinitionNode::generate(const CodeContext& context) {
    VariableResolutionVisitor resolver;
    accept(resolver);

    FunctionBody body{ identifier };

    TacVisitor visitor(body);
    accept(visitor);
    if (!dynamic_cast<ReturnInstruction*>(body.instructions.back().get()) && body.name == "main") { // Default return statement in main method
        body.emplaceInstruction<ReturnInstruction>(lineNumber, static_cast<unsigned int>(0));
    }

    std::stringstream ss;
    for (const auto& instruction : body.instructions) {
        instruction->makeAssembly(ss, body);
    }
    context.out << ss.str();
    if constexpr (DEBUG) {
        std::cout << body << std::endl;
    }
}

PrintVisitor::PrintVisitor(std::ostream& os, int indent)
    : os(os), indent(indent) {
}

void PrintVisitor::increaseIndent() {
    indent++;
}

void PrintVisitor::decreaseIndent() {
    if (indent > 0) {
        indent--;
    }
}

std::string PrintVisitor::getIndent() const {
    return std::string(indent, ' ');
}

void PrintVisitor::visitProgram(ProgramNode* const node) {
    os << "PROGRAM NODE\n";
    increaseIndent();
    if (node->function_declaration) {
        node->function_declaration->accept(*this);
    }
    decreaseIndent();
}

void PrintVisitor::visitFunctionDefinition(FunctionDefinitionNode* const node) {
    os << getIndent() << "FUNCTION DECLARATION NODE: " << node->identifier << '\n';
    increaseIndent();
    for (const auto& statement : node->block_items) {
        statement->accept(*this);
    }
    decreaseIndent();
}

void PrintVisitor::visitDeclaration(DeclarationNode* const node) {
    os << getIndent() << "DECLARATION NODE: " << node->identifier << '\n';
    if (node->expression) {
        increaseIndent();
        node->expression->accept(*this);
        decreaseIndent();
    }
}

void PrintVisitor::visitAssignment(AssignmentNode* const node) {
    os << getIndent() << "ASSIGNMENT NODE:\n";
    increaseIndent();
    node->left->accept(*this);
    node->right->accept(*this);
    decreaseIndent();
}

void PrintVisitor::visitReturn(ReturnNode* const node) {
    os << getIndent() << "RETURN NODE\n";
    if (node->expression) {
        increaseIndent();
        node->expression->accept(*this);
        decreaseIndent();
    }
}

void PrintVisitor::visitUnary(UnaryNode* const node) {
    os << getIndent() << "UNARY NODE: ";
    switch (node->op) {
    case UnaryOperator::NEGATION:
        os << "MINUS\n";
        break;
    case UnaryOperator::UNARY_ADD:
        os << "PLUS\n";
    	break;
    case UnaryOperator::BITWISE_NOT:
        os << "BITWISE NOT\n";
        break;
    case UnaryOperator::LOGICAL_NOT:
        os << "LOGICAL NOT\n";
        break;
    }
    increaseIndent();
    node->expression->accept(*this);
    decreaseIndent();
}

void PrintVisitor::visitBinary(BinaryNode* const node) {
    os << getIndent();
    switch (node->op) {
    case BinaryOperator::ADD:
        os << "ADD\n";
        break;
    case BinaryOperator::SUBTRACT:
        os << "SUBTRACT\n";
        break;
    case BinaryOperator::MULTIPLY:
        os << "MULTIPLY\n";
        break;
    case BinaryOperator::DIVIDE:
        os << "DIVIDE\n";
        break;
        // Add other operators as needed
    }
    increaseIndent();
    node->left->accept(*this);
    node->right->accept(*this);
    decreaseIndent();
}

void PrintVisitor::visitConst(ConstNode* const node) {
    os << getIndent() << "CONST NODE: " << node->value << '\n';
}

void PrintVisitor::visitVariable(VariableNode* const node) {
    os << getIndent() << "VARIABLE NODE: " << node->identifier << '\n';
}

void PrintVisitor::visitPostfix(PostfixNode* const node) {
	os << getIndent() << "POSTFIX NODE";
	os << (node->op == BinaryOperator::ADD ? "++" : "--") << "\n";
	increaseIndent();
	node->variable->accept(*this);
	decreaseIndent();
}

void PrintVisitor::visitPrefix(PrefixNode* const node) {
	os << getIndent() << "PREFIX NODE";
	os << (node->op == BinaryOperator::ADD ? "++" : "--") << "\n";
	increaseIndent();
	node->variable->accept(*this);
	decreaseIndent();
}
