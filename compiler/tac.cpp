#include "tac.hpp"
#include "ast.hpp"

std::ostream& operator<<(std::ostream& os, const FunctionBody& instruction) {
	for (const auto& i : instruction.instructions) {
		os << i->print() << "\n";
	}
	return os;
}

std::string UnaryOpInstruction::print() const {
	std::stringstream ss;
	ss << dest << " = ";
	switch (op) {
	case NEGATION:
		ss << "-";
		break;
	case BITWISE_NOT:
		ss << "~";
		break;
	case LOGICAL_NOT:
		ss << "!";
		break;
	}
	std::visit(OperandPrinter{ ss }, arg);
	return ss.str();
}

std::string BinaryOpInstruction::print() const {
	std::stringstream ss;
	ss << dest << " = ";
	std::visit(OperandPrinter{ ss }, left);
	switch (op) {
	case ADD:
		ss << " + ";
		break;
	case SUBTRACT:
		ss << " - ";
		break;
	case MULTIPLY:
		ss << " * ";
		break;
	case DIVIDE:
		ss << " / ";
		break;
	}
	std::visit(OperandPrinter{ ss }, right);
	return ss.str();
}

std::string ReturnInstruction::print() const {
	std::stringstream ss;
	ss << "return ";
	std::visit(OperandPrinter{ ss }, val);
	return ss.str();
}
