#include <iostream>
#include <sstream>
#include "ast.hpp"
#include "tac.hpp"

std::ostream& ProgramNode::print(std::ostream& os, int indent) const {
	os << "PROGRAM NODE\n";
	function_declaration->print(os, 1);
	return os;
}

void ProgramNode::generate(CodeContext& context) const {
	// Assembly prologue
	//context.out << ".text\n";

	// Generate code for the function (only main for now)
	if (function_declaration) {
		dynamic_cast<FunctionDefinitionNode*>(function_declaration.get())->generate(context);
	}
}

std::ostream& AssignmentNode::print(std::ostream& os, int indent) const {
	os << std::string(indent, ' ') << "ASSIGNMENT NODE:\n";
	left->print(os, indent + 1);
	right->print(os, indent + 1);
	return os;
}

std::ostream& ReturnNode::print(std::ostream& os, int indent) const {
	os << std::string(indent, ' ') << "RETURN NODE\n";
	expression->print(os, indent + 1);
	return os;
}

Operand ReturnNode::makeTac(FunctionBody& body) const {
	Operand dest = nullptr;
	if (expression) {
		dest = expression->makeTac(body);
	}
	body.emplaceInstruction<ReturnInstruction>(dest);
	return nullptr;
}

std::ostream& ConstNode::print(std::ostream& os, int indent) const {
	return os << std::string(indent, ' ') << "CONST NODE: " << value << '\n';
}

Operand ConstNode::makeTac(FunctionBody& body) const {
	return value;
}

std::ostream& UnaryNode::print(std::ostream& os, int indent) const {
	os << std::string(indent, ' ') << "UNARY NODE: ";
	switch (op) {
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
	expression->print(os, indent + 1);
	return os;
}

Operand UnaryNode::makeTac(FunctionBody& body) const {
	Operand src = expression->makeTac(body);
	PseudoRegister dest = body.emplaceInstruction<UnaryOpInstruction>(op, src);
	body.variableCount++;
	return dest;
}

std::ostream& BinaryNode::print(std::ostream& os, int indent) const {
	os << std::string(indent, ' ');
	switch (op) {
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
	}
	left->print(os, indent + 1);
	right->print(os, indent + 1);
	return os;
}

Operand BinaryNode::makeTac(FunctionBody& body) const {
	if (op == BinaryOperator::LOGICAL_AND) {
		std::string falseLabel = std::format(".{}{}_false", body.name, ++body.labelCount);
		std::string endLabel = std::format(".{}{}_end", body.name, ++body.labelCount);
		// Short-circuiting
		Operand leftOperand = left->makeTac(body);
		body.emplaceInstruction<JumpIfZero>(leftOperand, falseLabel); // goto false label
		Operand rightOperand = right->makeTac(body);
		body.emplaceInstruction<JumpIfZero>(rightOperand, falseLabel);
		PseudoRegister dest = body.emplaceInstruction<StoreValueInstruction>(static_cast<Number>(1));
		body.emplaceInstruction<Jump>(endLabel); // goto end
		body.emplaceInstruction<Label>(falseLabel); // false label
		dest = body.emplaceInstruction<StoreValueInstruction>(static_cast<Number>(0));
		body.emplaceInstruction<Label>(endLabel); // end
		body.variableCount++;
		return dest;
	}
	if (op == BinaryOperator::LOGICAL_OR) {
		std::string trueLabel = std::format(".{}{}_true", body.name, ++body.labelCount);
		std::string endLabel = std::format(".{}{}_end", body.name, ++body.labelCount);
		// Short-circuiting
		Operand leftOperand = left->makeTac(body);
		body.emplaceInstruction<JumpIfNotZero>(leftOperand, trueLabel); // goto true label
		Operand rightOperand = right->makeTac(body);
		body.emplaceInstruction<JumpIfNotZero>(rightOperand, trueLabel);
		PseudoRegister dest = body.emplaceInstruction<StoreValueInstruction>(static_cast<Number>(0));
		body.emplaceInstruction<Jump>(endLabel); // goto end
		body.emplaceInstruction<Label>(trueLabel); // true label
		dest = body.emplaceInstruction<StoreValueInstruction>(static_cast<Number>(1));
		body.emplaceInstruction<Label>(endLabel); // end
		body.variableCount++;
		return dest;
	}
	Operand leftOperand = left->makeTac(body);
	Operand rightOperand = right->makeTac(body);

	PseudoRegister dest = body.emplaceInstruction<BinaryOpInstruction>(op, leftOperand, rightOperand);
	body.variableCount++;
	return dest;
}

std::ostream& FunctionDefinitionNode::print(std::ostream& os, int indent) const {
	os << std::string(indent, ' ') << "FUNCTION DECLARATION NODE: " << identifier << '\n';
	if (!block_items.empty()) {
		for (const auto& statement : block_items) {
			statement->print(os, indent + 1);
		}
	}
	return os;
}

Operand FunctionDefinitionNode::makeTac(FunctionBody& body) const {
	body.emplaceInstruction<FunctionInstruction>(body.name);
	body.emplaceInstruction<AllocateStackInstruction>();
	return nullptr;
}

void FunctionDefinitionNode::generate(CodeContext& context) const {
	FunctionBody body(identifier);
	makeTac(body);
	if (!block_items.empty()) {
		for (const auto& statement : block_items) {
			statement->makeTac(body);
		}
		
		std::stringstream ss;
		for (const auto& instruction : body.instructions) {
			instruction->makeAssembly(ss, body);
		}
		context.out << ss.str();
		std::cout << body << std::endl;
	}
}

std::ostream& DeclarationNode::print(std::ostream& os, int indent) const {
	os << std::string(indent, ' ') << "DECLARATION NODE: " << identifier << '\n';
	if (expression) {
		expression->print(os, indent + 1);
	}
	return os;
}

std::ostream& VariableNode::print(std::ostream& os, int indent) const {
	return os << std::string(indent, ' ') << "VARIABLE NODE: " << identifier << '\n';
}
