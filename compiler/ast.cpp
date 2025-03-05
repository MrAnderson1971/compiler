#include "ast.hpp"

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

std::ostream& ConstNode::print(std::ostream& os, int indent) const {
	return os << std::string(indent, ' ') << "CONST NODE: " << value << '\n';
}

void ConstNode::generate(CodeContext& context) const {
	// Load the constant value into %eax
	context.out << "    mov $" << value << ", %eax\n";
}

std::ostream& UnaryNode::print(std::ostream& os, int indent) const {
	os << std::string(indent, ' ') << "UNARY NODE: ";
	switch (op) {
	case UnaryOperator::MINUS:
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
	case UnaryOperator::MINUS:
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
