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
