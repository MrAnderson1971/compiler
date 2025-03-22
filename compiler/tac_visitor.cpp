#include "tac_visitor.hpp"

#include "exceptions.hpp"

TacVisitor::TacVisitor(FunctionBody& body)
    : body(body), result(nullptr) {
}

Operand TacVisitor::getResult() const {
    return result;
}

void TacVisitor::visitProgram(ProgramNode* const node) {
	throw std::logic_error("ProgramNode should not be visited by TacVisitor");
}

void TacVisitor::visitFunctionDefinition(FunctionDefinitionNode* const node) {
    body.emplaceInstruction<FunctionInstruction>(node->lineNumber, body.name);
    body.emplaceInstruction<AllocateStackInstruction>(node->lineNumber);

    for (const auto& statement : node->block_items) {
        statement->accept(*this);
    }
}

void TacVisitor::visitDeclaration(DeclarationNode* const node) {
    PseudoRegister pseudoRegister{ body.name, body.variableCount };
    body.variableToPseudoregister[node->identifier] = pseudoRegister;
    if (node->expression) {
        node->expression->accept(*this);
        body.emplaceInstructionWithDestination<StoreValueInstruction>(node->lineNumber, pseudoRegister, result);
    }
    body.variableCount++;
}

void TacVisitor::visitAssignment(AssignmentNode* const node) {
    node->right->accept(*this);
    Operand src = result;
    try {
        node->left->accept(*this);
        PseudoRegister variable = std::get<PseudoRegister>(result);
        //node->right->accept(*this);
        //Operand src = result;
        body.emplaceInstructionWithDestination<StoreValueInstruction>(node->lineNumber, variable, src);
    } catch (std::bad_variant_access&) {
        throw semantic_error(std::format("Invalid lvalue {} at {}", result, node->lineNumber));
    }
}

void TacVisitor::visitReturn(ReturnNode* const node) {
    Operand dest = nullptr;
    if (node->expression) {
        node->expression->accept(*this);
        dest = result;
    }
    body.emplaceInstruction<ReturnInstruction>(node->lineNumber, dest);
    result = nullptr;
}

void TacVisitor::visitUnary(UnaryNode* const node) {
    node->expression->accept(*this);
    if (node->op == UnaryOperator::UNARY_ADD) { // if unary add do nothing
        return;
    }
    Operand src = result;
    PseudoRegister dest = body.emplaceInstruction<UnaryOpInstruction>(node->lineNumber, node->op, src);
    body.variableCount++;
    result = dest;
}

void TacVisitor::visitBinary(BinaryNode* const node) {
    if (node->op == BinaryOperator::LOGICAL_AND) {
        std::string falseLabel = std::format(".{}{}_false", body.name, ++body.labelCount);
        std::string endLabel = std::format(".{}{}_end", body.name, ++body.labelCount);

        // Short-circuiting
        node->left->accept(*this);
        Operand leftOperand = result;
        body.emplaceInstruction<JumpIfZero>(node->lineNumber, leftOperand, falseLabel); // goto false label

        node->right->accept(*this);
        Operand rightOperand = result;
        body.emplaceInstruction<JumpIfZero>(node->lineNumber, rightOperand, falseLabel);

        PseudoRegister dest = body.emplaceInstruction<StoreValueInstruction>(node->lineNumber, static_cast<Number>(1));
        body.emplaceInstruction<Jump>(node->lineNumber, endLabel); // goto end

        body.emplaceInstruction<Label>(node->lineNumber, falseLabel); // false label
        dest = body.emplaceInstruction<StoreValueInstruction>(node->lineNumber, static_cast<Number>(0));

        body.emplaceInstruction<Label>(node->lineNumber, endLabel); // end
        body.variableCount++;
        result = dest;
        return;
    }

    if (node->op == BinaryOperator::LOGICAL_OR) {
        std::string trueLabel = std::format(".{}{}_true", body.name, ++body.labelCount);
        std::string endLabel = std::format(".{}{}_end", body.name, ++body.labelCount);

        // Short-circuiting
        node->left->accept(*this);
        Operand leftOperand = result;
        body.emplaceInstruction<JumpIfNotZero>(node->lineNumber, leftOperand, trueLabel); // goto true label

        node->right->accept(*this);
        Operand rightOperand = result;
        body.emplaceInstruction<JumpIfNotZero>(node->lineNumber, rightOperand, trueLabel);

        PseudoRegister dest = body.emplaceInstruction<StoreValueInstruction>(node->lineNumber, static_cast<Number>(0));
        body.emplaceInstruction<Jump>(node->lineNumber, endLabel); // goto end

        body.emplaceInstruction<Label>(node->lineNumber, trueLabel); // true label
        dest = body.emplaceInstruction<StoreValueInstruction>(node->lineNumber, static_cast<Number>(1));

        body.emplaceInstruction<Label>(node->lineNumber, endLabel); // end
        body.variableCount++;
        result = dest;
        return;
    }

    node->left->accept(*this);
    Operand leftOperand = result;

    node->right->accept(*this);
    Operand rightOperand = result;

    PseudoRegister dest = body.emplaceInstruction<BinaryOpInstruction>(node->lineNumber, node->op, leftOperand, rightOperand);
    body.variableCount++;
    result = dest;
}

void TacVisitor::visitConst(ConstNode* const node) {
    result = node->value;
}

void TacVisitor::visitVariable(VariableNode* const node) {
    if (!body.variableToPseudoregister.contains(node->identifier)) {
        throw semantic_error(std::format("Undeclared variable {} at {}", node->identifier, node->lineNumber));
    }
    result = body.variableToPseudoregister[node->identifier];
}

void TacVisitor::visitPostfix(PostfixNode* const node) {
    node->variable->accept(*this); // get variable
    PseudoRegister variable = std::get<PseudoRegister>(result); // save variable
    PseudoRegister temp1 = body.emplaceInstruction<StoreValueInstruction>(node->lineNumber, result); // temp1 = a
    body.variableCount++;
    PseudoRegister temp2 = body.emplaceInstruction<BinaryOpInstruction>(node->lineNumber, node->op,
        variable, static_cast<unsigned int>(1)); // t2 = a + 1
    ++body.variableCount;
    body.emplaceInstructionWithDestination<StoreValueInstruction>(node->lineNumber, variable, temp2); // a = t2
    result = temp1;
}

void TacVisitor::visitPrefix(PrefixNode* const node) {
    node->variable->accept(*this);
    PseudoRegister variable = std::get<PseudoRegister>(result);
    body.emplaceInstructionWithDestination<BinaryOpInstruction>(node->lineNumber, variable, node->op, variable, static_cast<unsigned int>(1));
    body.variableCount++;
}
