#include <iostream>
#include "ast.hpp"
#include "tac.hpp"

std::ostream& ProgramNode::print(std::ostream& os, int indent) const {
	os << "PROGRAM NODE\n";
	function_declaration->print(os, 1);
	return os;
}

void ProgramNode::generate(CodeContext& context) const {
	// Assembly prologue
	context.out << ".text\n";
	context.out << ".global main\n\n";

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
	case NEGATION:
		os << "MINUS\n";
		break;
	case BITWISE_NOT:
		os << "BITWISE NOT\n";
		break;
	case LOGICAL_NOT:
		os << "LOGICAL NOT\n";
		break;
	}
	expression->print(os, indent + 1);
	return os;
}

void UnaryNode::generate(CodeContext& context) const {
	expression->generate(context);
	switch (op) {
	case NEGATION:
		context.out << "    neg %eax\n";
		break;
	case BITWISE_NOT:
		context.out << "    not %eax\n";
		break;
	case LOGICAL_NOT:
		context.out << "    cmp $0, %eax\n";
		context.out << "    sete %al\n";
		context.out << "    movzx %al, %eax\n";
		break;
	}
}

Operand UnaryNode::makeTac(FunctionBody& body) const {
	Operand src = expression->makeTac(body);
	PseudoRegister dest = body.emplaceInstruction<UnaryOpInstruction>(op, src);
	return dest;
}

std::ostream& BinaryNode::print(std::ostream& os, int indent) const {
	os << std::string(indent, ' ');
	switch (op) {
	case ADD:
		os << "ADD\n";
		break;
	case SUBTRACT:
		os << "SUBTRACT\n";
		break;
	case MULTIPLY:
		os << "MULTIPLY\n";
		break;
	case DIVIDE:
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
	case ADD:
		context.out << "    add %ecx, %eax\n";
		break;
	case SUBTRACT:
		context.out << "    sub %ecx, %eax\n";
		break;
	case MULTIPLY:
		context.out << "    imul %ecx, %eax\n";
		break;
	case DIVIDE:
		context.out << "    cdq\n";
		context.out << "    idiv %ecx\n";
		break;
	}
}

std::ostream& FunctionDeclarationNode::print(std::ostream& os, int indent) const {
	os << std::string(indent, ' ') << "FUNCTION DECLARATION NODE: " << identifier << '\n';
	if (statement) {
		statement->print(os, indent + 1);
	}
	return os;
}

void FunctionDeclarationNode::generate(CodeContext& context) const {
	if (statement) {
		FunctionBody body(identifier);
		statement->makeTac(body);
		std::cout << body;
		
		std::stringstream ss;
		for (const auto& instruction : body.instructions) {
			instruction->makeAssembly(ss);
		}
		std::cout << ss.str();
	}
}
