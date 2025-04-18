#include "tac.hpp"
#include "ast.hpp"
#include <format>
#include <sstream>

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

std::string OperandToAsm::operator()(const std::shared_ptr<PseudoRegister>& reg) const {
	return std::format("-{}(%rbp)", 4 * reg->position);
}

std::string OperandToAsm::operator()(const std::nullptr_t) const {
	return "";
}

std::string UnaryOpInstruction::print() const {
	std::stringstream ss;
	ss << dest << " = ";
	switch (op) {
	case UnaryOperator::NEGATION:
		ss << "-";
		break;
	case UnaryOperator::UNARY_ADD:
		ss << "+";
		break;
	case UnaryOperator::BITWISE_NOT:
		ss << "~";
		break;
	case UnaryOperator::LOGICAL_NOT:
		ss << "!";
		break;
	}
	std::visit(OperandPrinter{ ss }, *arg);
	return ss.str();
}

void UnaryOpInstruction::makeAssembly(std::stringstream& ss, FunctionBody& body) const {
	if constexpr (DEBUG) {
		ss << std::format("; {}\n", print());
	}
	ss << std::format("movl {}, %r10d\n", std::visit(operandToAsm, *arg));
	ss << std::format("movl %r10d, {}\n", dest);
	switch (op) {
	case UnaryOperator::NEGATION:
		ss << std::format("negl {}\n", dest);
		break;
	case UnaryOperator::BITWISE_NOT:
		ss << std::format("notl {}\n", dest);
		break;
	case UnaryOperator::LOGICAL_NOT:
		ss << std::format("cmpl $0, {}\n", dest);
		ss << std::format("sete {}\n", dest);
		break;
	}
	if constexpr (DEBUG) {
		ss << "\n";
	}
}

std::string BinaryOpInstruction::print() const {
	std::stringstream ss;
	ss << dest << " = ";
	std::visit(OperandPrinter{ ss }, *left);
	ss << " ";
	ss << tokenPrinter(static_cast<Symbol>(op));
	ss << " ";
	std::visit(OperandPrinter{ ss }, *right);
	return ss.str();
}

void BinaryOpInstruction::makeAssembly(std::stringstream& ss, FunctionBody& body) const {
	if constexpr (DEBUG) {
		ss << std::format("; {}\n", print());
	}
	std::string src1 = std::visit(operandToAsm, *left);
	std::string src2 = std::visit(operandToAsm, *right);
	std::string d = operandToAsm(dest);

	bool src2IsImmediate = src2.find('$') != std::string::npos;

	if (isOneOf(op, BinaryOperator::ADD, BinaryOperator::SUBTRACT, BinaryOperator::BITWISE_AND,
		BinaryOperator::BITWISE_OR, BinaryOperator::BITWISE_XOR, BinaryOperator::SHIFT_LEFT,
		BinaryOperator::SHIFT_RIGHT)) {
		// Load the first operand into a register
		ss << std::format("movl {}, %r10d\n", src1);

		// Special handling for shift operations
		if (isOneOf(op, BinaryOperator::SHIFT_LEFT, BinaryOperator::SHIFT_RIGHT)) {
			if (src2IsImmediate) {
				// Immediate shift count
				std::string shiftOp = (op == BinaryOperator::SHIFT_LEFT) ? "shll" : "shrl";
				ss << std::format("{} {}, %r10d\n", shiftOp, src2);
			} else {
				// Variable shift count (must use CL)
				ss << std::format("movl {}, %ecx\n", src2);
				std::string shiftOp = (op == BinaryOperator::SHIFT_LEFT) ? "shll" : "shrl";
				ss << std::format("{} %cl, %r10d\n", shiftOp);
			}
		} else {
			// Handle other operations
			std::string opcode;
			switch (op) {
			case BinaryOperator::ADD:
				opcode = "addl";
				break;
			case BinaryOperator::SUBTRACT:
				opcode = "subl";
				break;
			case BinaryOperator::BITWISE_AND:
				opcode = "andl";  // Added 'l' suffix
				break;
			case BinaryOperator::BITWISE_OR:
				opcode = "orl";   // Added 'l' suffix
				break;
			case BinaryOperator::BITWISE_XOR:
				opcode = "xorl";  // Added 'l' suffix
				break;
			}

			// Apply operation
			if (src2IsImmediate) {
				ss << std::format("{} {}, %r10d\n", opcode, src2);
			} else {
				ss << std::format("movl {}, %r11d\n", src2);
				ss << std::format("{} %r11d, %r10d\n", opcode, src2);
			}
		}

		// Store result
		ss << std::format("movl %r10d, {}\n", d);
	} else if (op == BinaryOperator::MULTIPLY) {
		ss << std::format("movl {}, %r11d\n", src1);

		// Handle second operand
		if (src2IsImmediate) {
			ss << std::format("imull {}, %r11d\n", src2);
		} else {
			ss << std::format("movl {}, %r10d\n", src2);
			ss << "imull %r10d, %r11d\n";
		}

		// Store result to destination
		ss << std::format("movl %r11d, {}\n", d);
	} else if (isOneOf(op, BinaryOperator::DIVIDE, BinaryOperator::MODULO)) {
		// Load dividend into eax (required for division)
		ss << std::format("movl {}, %eax\n", src1);
		ss << "cdq\n";  // Sign-extend eax into edx:eax pair

		// Load divisor into ecx
		ss << std::format("movl {}, %ecx\n", src2);
		ss << "idiv %ecx\n";

		// Store appropriate result (quotient or remainder)
		if (op == BinaryOperator::DIVIDE) {
			ss << std::format("movl %eax, {}\n", d);  // Division result in eax
		} else {
			ss << std::format("movl %edx, {}\n", d);  // Remainder in edx
		}
	} else if (isOneOf(op, BinaryOperator::EQUALS, BinaryOperator::NOT_EQUALS, BinaryOperator::LESS_THAN, BinaryOperator::GREATER_THAN, BinaryOperator::LESS_THAN_OR_EQUAL, BinaryOperator::GREATER_THAN_OR_EQUAL)) {
		ss << std::format("movl {}, %edx\n", src1);
		ss << std::format("cmpl {}, %edx\n", src2);
		ss << std::format("movl $0, {}\n", d);
		switch (op) {
		case BinaryOperator::EQUALS:
			ss << std::format("sete {}\n", d);
			break;
		case BinaryOperator::NOT_EQUALS:
			ss << std::format("setne {}\n", d);
			break;
		case BinaryOperator::LESS_THAN:
			ss << std::format("setl {}\n", d);
			break;
		case BinaryOperator::GREATER_THAN:
			ss << std::format("setg {}\n", d);
			break;
		case BinaryOperator::LESS_THAN_OR_EQUAL:
			ss << std::format("setle {}\n", d);
			break;
		case BinaryOperator::GREATER_THAN_OR_EQUAL:
			ss << std::format("setge {}\n", d);
			break;
		}
	}
	if constexpr (DEBUG) {
		ss << "\n";
	}
}

std::string JumpIfZero::print() const {
	std::stringstream ss;
	ss << "if ";
	std::visit(OperandPrinter{ ss }, *op);
	ss << " == 0 goto " << *label;
	return ss.str();
}

void JumpIfZero::makeAssembly(std::stringstream& ss, FunctionBody& body) const {
	if constexpr (DEBUG) {
		ss << std::format("; {}\n", print());
	}
	std::string src = std::visit(operandToAsm, *op);
	ss << std::format("movl {}, %edx\n", src);
	ss << "cmpl $0, %edx\n";
	ss << std::format("je {}\n", label);
	if constexpr (DEBUG) {
		ss << "\n";
	}
}

std::string JumpIfNotZero::print() const {
	std::stringstream ss;
	ss << "if ";
	std::visit(OperandPrinter{ ss }, *op);
	ss << " != 0 goto " << *label;
	return ss.str();
}

void JumpIfNotZero::makeAssembly(std::stringstream& ss, FunctionBody& body) const {
	if constexpr (DEBUG) {
		ss << std::format("; {}\n", print());
	}
	std::string src = std::visit(operandToAsm, *op);
	ss << std::format("movl {}, %edx\n", src);
	ss << "cmpl $0, %edx\n";
	ss << std::format("jne {}\n", label);
	if constexpr (DEBUG) {
		ss << "\n";
	}
}

std::string Jump::print() const {
	return "goto " + *label;
}

void Jump::makeAssembly(std::stringstream& ss, FunctionBody& body) const {
	ss << "jmp " << *label << "\n";
}

std::string Label::print() const {
	return *label + ":";
}

void Label::makeAssembly(std::stringstream& ss, FunctionBody& body) const {
	ss << *label << ":\n";
}

std::string StoreValueInstruction::print() const {
	std::stringstream ss;
	ss << dest << " = ";
	std::visit(OperandPrinter{ ss }, *val);
	return ss.str();
}

void StoreValueInstruction::makeAssembly(std::stringstream& ss, FunctionBody& body) const {
	if constexpr (DEBUG) {
		ss << std::format("; {}\n", print());
	}
	if (std::holds_alternative<std::shared_ptr<PseudoRegister>>(*val)) {
		ss << std::format("movl {}, %r10d\n", val);
		ss << std::format("movl %r10d, {}\n", dest);
	} else {
		ss << std::format("movl {}, {}\n", val, dest);
	}
	if constexpr (DEBUG) {
		ss << "\n";
	}
}

std::string ReturnInstruction::print() const {
	std::stringstream ss;
	ss << "return ";
	std::visit(OperandPrinter{ ss }, *val);
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
	ss << ".global " << *name << "\n" 
		<< *name << ":\n"
	<< "pushq %rbp\n"
		<< "movq %rsp, %rbp\n";
}

std::string AllocateStackInstruction::print() const {
	return std::format("allocate_stack");
}

void AllocateStackInstruction::makeAssembly(std::stringstream& ss, FunctionBody& body) const {
	ss << std::format("subq ${}, %rsp\n", body.variableCount * 4);
}
