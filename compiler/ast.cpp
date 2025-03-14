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
		function_declaration->generate(context);
	}
}

std::ostream& ReturnNode::print(std::ostream& os, int indent) const {
	os << std::string(indent, ' ') << "RETURN NODE\n";
	expression->print(os, indent + 1);
	return os;
}

void ReturnNode::generate(CodeContext& context) const {
	if (expression) {
		expression->generate(context);
	}
	else {
		context.out << "    xor %eax, %eax\n";
	}

	context.out << "    leave\n";
	context.out << "    ret\n";
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

void ConstNode::generate(CodeContext& context) const {
	// Load the constant value into %eax
	context.out << "    mov $" << value << ", %eax\n";
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

void UnaryNode::generate(CodeContext& context) const {
	expression->generate(context);
	switch (op) {
	case UnaryOperator::NEGATION:
		context.out << "    neg %eax\n";
		break;
	case UnaryOperator::BITWISE_NOT:
		context.out << "    not %eax\n";
		break;
	case UnaryOperator::LOGICAL_NOT:
		context.out << "    cmp $0, %eax\n";
		context.out << "    sete %al\n";
		context.out << "    movzx %al, %eax\n";
		break;
	}
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

void BinaryNode::generate(CodeContext& context) const {
	left->generate(context);
	context.out << "    push %eax\n";
	right->generate(context);
	context.out << "    pop %ecx\n";
	switch (op) {
	case BinaryOperator::ADD:
		context.out << "    add %ecx, %eax\n";
		break;
	case BinaryOperator::SUBTRACT:
		context.out << "    sub %ecx, %eax\n";
		break;
	case BinaryOperator::MULTIPLY:
		context.out << "    imul %ecx, %eax\n";
		break;
	case BinaryOperator::DIVIDE:
		context.out << "    cdq\n";
		context.out << "    idiv %ecx\n";
		break;
	}
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

std::ostream& FunctionDeclarationNode::print(std::ostream& os, int indent) const {
	os << std::string(indent, ' ') << "FUNCTION DECLARATION NODE: " << identifier << '\n';
	if (statement) {
		statement->print(os, indent + 1);
	}
	return os;
}

Operand FunctionDeclarationNode::makeTac(FunctionBody& body) const {
	body.emplaceInstruction<FunctionInstruction>(body.name);
	body.emplaceInstruction<AllocateStackInstruction>();
	return nullptr;
}

void FunctionDeclarationNode::generate(CodeContext& context) const {
	FunctionBody body(identifier);
	makeTac(body);
	if (statement) {
		statement->makeTac(body);
		
		std::stringstream ss;
		for (const auto& instruction : body.instructions) {
			instruction->makeAssembly(ss, body);
		}
		context.out << ss.str();
		std::cout << body << std::endl;
	}
}
