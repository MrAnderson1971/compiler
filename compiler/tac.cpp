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

void UnaryOpInstruction::makeAssembly(std::stringstream& ss, FunctionBody& body) const {
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
	case MODULO:
		ss << " % ";
		break;
	case XOR:
		ss << " ^ ";
		break;
	case AND:
		ss << " & ";
		break;
	case OR:
		ss << " | ";
		break;
	case SHIFT_LEFT:
		ss << " << ";
		break;
	case SHIFT_RIGHT:
		ss << " >> ";
		break;
	}
	std::visit(OperandPrinter{ ss }, right);
	return ss.str();
}

void BinaryOpInstruction::makeAssembly(std::stringstream& ss, FunctionBody& body) const {
	std::string src1 = std::visit(operandToAsm, left);
	std::string src2 = std::visit(operandToAsm, right);
	std::string d = operandToAsm(dest);

	bool src2IsImmediate = src2.find('$') != std::string::npos;

	if (isOneOf(op, ADD, SUBTRACT, AND, OR, XOR, SHIFT_LEFT, SHIFT_RIGHT)) {
		ss << std::format("movl {}, %r10d\n", src1);
		ss << std::format("movl %r10d, {}\n", d);

		// For add/subtract, we need to handle memory-to-memory operations
		std::string opcode;
		switch (op) {
		case ADD:
			opcode = "addl";
			break;
		case SUBTRACT:
			opcode = "subl";
			break;
		case AND:
			opcode = "and";
			break;
		case OR:
			opcode = "or";
			break;
		case XOR:
			opcode = "xor";
			break;
		case SHIFT_LEFT:
			opcode = "shl";
			break;
		case SHIFT_RIGHT:
			opcode = "shr";
			break;
		}
		if (src2IsImmediate) {
			ss << std::format("{} {}, {}\n", opcode, src2, d);
		}
		else {
			ss << std::format("movl {}, %r10d\n", src2);
			ss << std::format("{} %r10d, {}\n", opcode, d);
		}
	}
	else if (op == MULTIPLY) {
		ss << std::format("movl {}, %r10d\n", src1);
		ss << std::format("movl %r10d, {}\n", d);

		ss << std::format("movl {}, %r11d\n", d);

		if (src2IsImmediate) {
			ss << std::format("imull {}, %r11d\n", src2);
		}
		else {
			ss << std::format("movl {}, %r10d\n", src2);
			ss << std::format("imull %r10d, %r11d\n");
		}

		ss << std::format("movl %r11d, {}\n", d);
	}
	else if (isOneOf(op, DIVIDE, MODULO)) {
		ss << std::format("movl {}, %eax\n", src1);
		ss << "cdq\n";

		if (src2IsImmediate) {
			std::string immValue = src2.substr(1);
			ss << std::format("movl {}, %ecx\n", src2);
			ss << "idiv %ecx\n";  // Use idiv without suffix
		}
		else {
			ss << std::format("movl {}, %ecx\n", src2);
			ss << "idiv %ecx\n";
		}

		if (op == DIVIDE) {
			ss << std::format("movl %eax, {}\n", d);
		}
		else {
			ss << std::format("movl %edx, {}\n", d);
		}
	}
}

std::string ReturnInstruction::print() const {
	std::stringstream ss;
	ss << "return ";
	std::visit(OperandPrinter{ ss }, val);
	return ss.str();
}

void ReturnInstruction::makeAssembly(std::stringstream& ss, FunctionBody& body) const {
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

void FunctionInstruction::makeAssembly(std::stringstream& ss, FunctionBody& body) const {
	ss << ".global " << name << "\n" 
		<< name << ":\n"
	<< "pushq %rbp\n"
		<< "movq %rsp, %rbp\n";
}

std::string AllocateStackInstruction::print() const {
	return std::format("allocate_stack");
}

void AllocateStackInstruction::makeAssembly(std::stringstream& ss, FunctionBody& body) const {
	ss << std::format("subq ${}, %rsp\n", body.variableCount * 4);
}
