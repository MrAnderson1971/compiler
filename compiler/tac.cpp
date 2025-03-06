#include "tac.hpp"
#include "ast.hpp"

std::ostream& operator<<(std::ostream& os, const FunctionBody& instruction) {
	for (const auto& i : instruction.instructions) {
		os << i->print() << "\n";
	}
	return os;
}

void OperandToAsm::operator()(const Number n) const {
	ss << "$" << n;
}

void OperandToAsm::operator()(const PseudoRegister& reg) const {
	ss << -4 * reg.position << "(%rbp)";
}

void OperandToAsm::operator()(const std::nullptr_t) const {}

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

void UnaryOpInstruction::makeAssembly(std::stringstream& ss) const {
	ss << "movl ";
	std::visit(OperandToAsm{ ss }, arg);
	ss << ", %r10d\n"
		<< "movl %r10d, ";
	OperandToAsm{ ss }(dest);
	ss << "\n";
	switch (op) {
	case NEGATION:
		ss << "negl ";
		break;
	case BITWISE_NOT:
		ss << "notl ";
		break;
	case LOGICAL_NOT:
		ss << "cmp $0, ";
		std::visit(OperandToAsm{ ss }, arg);
		ss << "\n    sete %al\n    movzx %al, " << dest;
		ss << "\n";
		return;
	}
	OperandToAsm{ ss }(dest);
	ss << "\n";
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

void ReturnInstruction::makeAssembly(std::stringstream& ss) const {
	/* movq %rbp, %rsp
popq %rbp
ret*/
	ss << "movl ";
	std::visit(OperandToAsm{ ss }, val);
	ss << ", %eax\n";
	ss << "movq %rbp, %rsp\n"
		<< "popq %rbp\n"
		<< "ret\n";
}

std::string FunctionInstruction::print() const {
	return "function";
}

void FunctionInstruction::makeAssembly(std::stringstream& ss) const {
	ss << ".global " << name << "\n" 
		<< name << ":\n"
	<< "pushq %rbp\n"
		<< "movq %rsp, %rbp\n";
}
