#include "ast.hpp"
#include <iostream>
#include <sstream>

void ProgramNode::generate(CodeContext& context) const {
	if (function_declaration) {
		dynamic_cast<FunctionDefinitionNode*>(function_declaration.get())->generate(context);
	}
}

void FunctionDefinitionNode::generate(CodeContext& context) const {
    FunctionBody body(identifier);
	TacVisitor visitor(body);
	accept(visitor);
    if (!block_items.empty()) {
        for (const auto& statement : block_items) {
            statement->accept(visitor);
        }

        std::stringstream ss;
        for (const auto& instruction : body.instructions) {
            instruction->makeAssembly(ss, body);
        }
        context.out << ss.str();
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
    if (indent > 0) indent--;
}

std::string PrintVisitor::getIndent() const {
    return std::string(indent, ' ');
}

void PrintVisitor::visitProgram(const ProgramNode& node) {
    os << "PROGRAM NODE\n";
    increaseIndent();
    if (node.function_declaration) {
        node.function_declaration->accept(*this);
    }
    decreaseIndent();
}

void PrintVisitor::visitFunctionDefinition(const FunctionDefinitionNode& node) {
    os << getIndent() << "FUNCTION DECLARATION NODE: " << node.identifier << '\n';
    increaseIndent();
    for (const auto& statement : node.block_items) {
        statement->accept(*this);
    }
    decreaseIndent();
}

void PrintVisitor::visitDeclaration(const DeclarationNode& node) {
    os << getIndent() << "DECLARATION NODE: " << node.identifier << '\n';
    if (node.expression) {
        increaseIndent();
        node.expression->accept(*this);
        decreaseIndent();
    }
}

void PrintVisitor::visitAssignment(const AssignmentNode& node) {
    os << getIndent() << "ASSIGNMENT NODE:\n";
    increaseIndent();
    node.left->accept(*this);
    node.right->accept(*this);
    decreaseIndent();
}

void PrintVisitor::visitReturn(const ReturnNode& node) {
    os << getIndent() << "RETURN NODE\n";
    if (node.expression) {
        increaseIndent();
        node.expression->accept(*this);
        decreaseIndent();
    }
}

void PrintVisitor::visitUnary(const UnaryNode& node) {
    os << getIndent() << "UNARY NODE: ";
    switch (node.op) {
    case UnaryOperator::NEGATION:
        os << "MINUS\n";
        break;
    case UnaryOperator::BITWISE_NOT:
        os << "BITWISE NOT\n";
        break;
    case UnaryOperator::LOGICAL_NOT:
        os << "LOGICAL NOT\n";
        break;
    }
    increaseIndent();
    node.expression->accept(*this);
    decreaseIndent();
}

void PrintVisitor::visitBinary(const BinaryNode& node) {
    os << getIndent();
    switch (node.op) {
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
    node.left->accept(*this);
    node.right->accept(*this);
    decreaseIndent();
}

void PrintVisitor::visitConst(const ConstNode& node) {
    os << getIndent() << "CONST NODE: " << node.value << '\n';
}

void PrintVisitor::visitVariable(const VariableNode& node) {
    os << getIndent() << "VARIABLE NODE: " << node.identifier << '\n';
}

TacVisitor::TacVisitor(FunctionBody& body)
    : body(body), result(nullptr) {
}

Operand TacVisitor::getResult() const {
    return result;
}

void TacVisitor::visitProgram(const ProgramNode& node) {
    if (node.function_declaration) {
        node.function_declaration->accept(*this);
    }
}

void TacVisitor::visitFunctionDefinition(const FunctionDefinitionNode& node) {
    body.emplaceInstruction<FunctionInstruction>(body.name);
    body.emplaceInstruction<AllocateStackInstruction>();

    for (const auto& statement : node.block_items) {
        statement->accept(*this);
    }
}

void TacVisitor::visitDeclaration(const DeclarationNode& node) {
    // TODO: Implement declaration TAC generation
    result = nullptr;
}

void TacVisitor::visitAssignment(const AssignmentNode& node) {
    // TODO: Implement assignment TAC generation
    result = nullptr;
}

void TacVisitor::visitReturn(const ReturnNode& node) {
    Operand dest = nullptr;
    if (node.expression) {
        node.expression->accept(*this);
        dest = result;
    }
    body.emplaceInstruction<ReturnInstruction>(dest);
    result = nullptr;
}

void TacVisitor::visitUnary(const UnaryNode& node) {
    node.expression->accept(*this);
    Operand src = result;

    PseudoRegister dest = body.emplaceInstruction<UnaryOpInstruction>(node.op, src);
    body.variableCount++;
    result = dest;
}

void TacVisitor::visitBinary(const BinaryNode& node) {
    if (node.op == BinaryOperator::LOGICAL_AND) {
        std::string falseLabel = std::format(".{}{}_false", body.name, ++body.labelCount);
        std::string endLabel = std::format(".{}{}_end", body.name, ++body.labelCount);

        // Short-circuiting
        node.left->accept(*this);
        Operand leftOperand = result;
        body.emplaceInstruction<JumpIfZero>(leftOperand, falseLabel); // goto false label

        node.right->accept(*this);
        Operand rightOperand = result;
        body.emplaceInstruction<JumpIfZero>(rightOperand, falseLabel);

        PseudoRegister dest = body.emplaceInstruction<StoreValueInstruction>(static_cast<Number>(1));
        body.emplaceInstruction<Jump>(endLabel); // goto end

        body.emplaceInstruction<Label>(falseLabel); // false label
        dest = body.emplaceInstruction<StoreValueInstruction>(static_cast<Number>(0));

        body.emplaceInstruction<Label>(endLabel); // end
        body.variableCount++;
        result = dest;
        return;
    }

    if (node.op == BinaryOperator::LOGICAL_OR) {
        std::string trueLabel = std::format(".{}{}_true", body.name, ++body.labelCount);
        std::string endLabel = std::format(".{}{}_end", body.name, ++body.labelCount);

        // Short-circuiting
        node.left->accept(*this);
        Operand leftOperand = result;
        body.emplaceInstruction<JumpIfNotZero>(leftOperand, trueLabel); // goto true label

        node.right->accept(*this);
        Operand rightOperand = result;
        body.emplaceInstruction<JumpIfNotZero>(rightOperand, trueLabel);

        PseudoRegister dest = body.emplaceInstruction<StoreValueInstruction>(static_cast<Number>(0));
        body.emplaceInstruction<Jump>(endLabel); // goto end

        body.emplaceInstruction<Label>(trueLabel); // true label
        dest = body.emplaceInstruction<StoreValueInstruction>(static_cast<Number>(1));

        body.emplaceInstruction<Label>(endLabel); // end
        body.variableCount++;
        result = dest;
        return;
    }

    node.left->accept(*this);
    Operand leftOperand = result;

    node.right->accept(*this);
    Operand rightOperand = result;

    PseudoRegister dest = body.emplaceInstruction<BinaryOpInstruction>(node.op, leftOperand, rightOperand);
    body.variableCount++;
    result = dest;
}

void TacVisitor::visitConst(const ConstNode& node) {
    result = node.value;
}

void TacVisitor::visitVariable(const VariableNode& node) {
    // TODO: Implement variable reference TAC generation
    result = nullptr;
}
