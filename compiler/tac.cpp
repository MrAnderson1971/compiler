#include "tac.hpp"
#include "ast.hpp"
#include <format>

OperandToAsm operandToAsm;

std::ostream& operator<<(std::ostream& os, const FunctionBody& instruction) {
	for (const auto& i : instruction.instructions) {
		os << i->print() << "\n";
	}
	return os;
}

std::string OperandToAsm::operator()(const Number n) const {
	return std::format("${}", n);
}

std::string OperandToAsm::operator()(const PseudoRegister& reg) const {
	return std::format("-{}(%rbp)", 4 * reg.position);
}

std::string OperandToAsm::operator()(const std::nullptr_t) const {
	throw compiler_error("nullptr operand");
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

void UnaryOpInstruction::makeAssembly(std::stringstream& ss) const {
	ss << std::format("movl {}, %r10d\n", std::visit(operandToAsm, arg));
	ss << std::format("movl %r10d, {}\n", dest);
	switch (op) {
	case NEGATION:
		ss << "negl ";
		break;
	case BITWISE_NOT:
		ss << "notl ";
		break;
	}
	ss << operandToAsm(dest) << "\n";
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

void BinaryOpInstruction::makeAssembly(std::stringstream& ss) const {
	std::string src1 = std::visit(operandToAsm, left);
	std::string src2 = std::visit(operandToAsm, right);
	std::string d = operandToAsm(dest);
	switch (op) {
	case ADD: case SUBTRACT: case MULTIPLY:
		ss << std::format("movl {}, {}\n", src1, d);
		switch (op) {
		case ADD:
			ss << std::format("addl {}, {}\n", src2, d);
			break;
		case SUBTRACT:
			ss << std::format("subl {}, {}\n", src2, d);
			break;
		case MULTIPLY:
			ss << std::format("movl {}, %r11\n", d);
			ss << std::format("imull {}, %r11\n", src2);
			ss << std::format("movl %r11, {}\n", d);
			break;
		}
		break;
	case DIVIDE: case MODULO:
		ss << std::format("movl {}, %eax\n", src1);
		ss << "cdq\n";
		ss << std::format("idivl {}\n", src2);
		if (op == DIVIDE) {
			ss << std::format("movl %eax, {}\n", dest);
		}
		else {
			ss << std::format("movl %edx, {}\n", dest);
		}
		break;
	}
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
	ss << std::format("movl {}, %eax\n", val);
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
