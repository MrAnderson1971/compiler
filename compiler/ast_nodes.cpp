#include "ast_nodes.hpp"
#include <iostream>
#include <sstream>
#include "exceptions.hpp"
#include "tac.hpp"

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

private:
    std::ostream& os;
    int indent;

    void increaseIndent();
    void decreaseIndent();
    std::string getIndent() const;
};

// TAC generation visitor
class TacVisitor : public FullVisitor {
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
	void visitPostfix(PostfixNode* const node) override;

    Operand getResult() const;

private:
    FunctionBody& body;
    Operand result;
};

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

private:
    int counter;
    std::unordered_map<std::string, std::string> variableMap;

    std::string makeTemporary(const std::string& name) {
        return std::format("{}.{}", name, counter++);
    }
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

TacVisitor::TacVisitor(FunctionBody& body)
    : body(body), result(nullptr) {
}

Operand TacVisitor::getResult() const {
    return result;
}

void TacVisitor::visitProgram(ProgramNode* const node) {
    if (node->function_declaration) {
        node->function_declaration->accept(*this);
    }
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
        body.emplaceInstruction<StoreValueInstruction>(node->lineNumber, result);
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
        body.emplaceInstruction<StoreValueInstruction>(node->lineNumber, variable, src);
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
	if (!dynamic_cast<VariableNode*>(node->variable.get())) {
		throw semantic_error(std::format("Invalid lvalue at {}", node->lineNumber));
	}
	node->variable->accept(*this); // get variable
	PseudoRegister variable = std::get<PseudoRegister>(result); // save variable
	PseudoRegister temp1 = body.emplaceInstruction<StoreValueInstruction>(node->lineNumber, result); // temp1 = a
    body.variableCount++;
	PseudoRegister temp2 = body.emplaceInstruction<BinaryOpInstruction>(node->lineNumber, node->op, 
        variable, static_cast<unsigned int>(1)); // t2 = a + 1
	++body.variableCount;
    body.emplaceInstruction<StoreValueInstruction>(node->lineNumber, variable, temp2); // a = t2
	result = temp1;
}

void VariableResolutionVisitor::visitProgram(ProgramNode* const node) {
    if (node->function_declaration) {
        node->function_declaration->accept(*this);
    }
}

void VariableResolutionVisitor::visitFunctionDefinition(FunctionDefinitionNode* const node) {
    for (const auto& statement : node->block_items) {
        statement->accept(*this);
    }
}

/*
 *resolve_declaration(Declaration(name, init), variable_map):
 1 if name is in variable_map:
 fail("Duplicate variable declaration!")
 unique_name = make_temporary()
 2 variable_map.add(name, unique_name)
 3 if init is not null:
 init = resolve_exp(init, variable_map)
 4 return Declaration(unique_name, init)
 */
void VariableResolutionVisitor::visitDeclaration(DeclarationNode* const node) {
    if (variableMap.contains(node->identifier)) {
        throw semantic_error(std::format("Duplicate variable declaration {} at {}", node->identifier, node->lineNumber));
    }
    std::string newName = makeTemporary(node->identifier);
    variableMap[node->identifier] = newName;
    node->identifier = newName;

    if (node->expression) {
        node->expression->accept(*this);
    }
}

/*
 *resolve_exp(e, variable_map):
 match e with
 | Assignment(left, right) ->
 if left is not a Var node:
 fail("Invalid lvalue!")
 return Assignment(1 resolve_exp(left, variable_map), 2 resolve_exp(right, variable_map))
 | Var(v) ->
 if v is in variable_map:
 return Var(3 variable_map.get(v))
 else:
 fail("Undeclared variable!")
 | --snip--
 */
void VariableResolutionVisitor::visitAssignment(AssignmentNode* const node) {
    if (!dynamic_cast<VariableNode*>(node->left.get())) {
        throw semantic_error(std::format("Invalid lvalue at {}", node->lineNumber));
    }
    node->left->accept(*this);
    node->right->accept(*this);
}

void VariableResolutionVisitor::visitReturn(ReturnNode* const node) {
    if (node->expression) {
        node->expression->accept(*this);
    }
}

void VariableResolutionVisitor::visitUnary(UnaryNode* const node) {
    node->expression->accept(*this);
}

void VariableResolutionVisitor::visitBinary(BinaryNode* const node) {
    node->left->accept(*this);
    node->right->accept(*this);
}

void VariableResolutionVisitor::visitVariable(VariableNode* const node) {
    if (!variableMap.contains(node->identifier)) {
        throw semantic_error(std::format("Undeclared variable {} at {}", node->identifier, node->lineNumber));
    }
    node->identifier = variableMap[node->identifier];
}

void VariableResolutionVisitor::visitPostfix(PostfixNode* const node) {
	node->variable->accept(*this);
}
