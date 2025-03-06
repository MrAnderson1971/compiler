#pragma once

#include "ast.hpp"

// three address code
struct TACInstruction {
	std::string dest;
	TACInstruction(std::string dest) : dest(dest) {}
	virtual ~TACInstruction() = default;
};

using Operand = std::variant<std::string, Number>;

struct OperandPrinter {
	std::ostream& os;

	template<typename Any>
	void operator()(const Any s) const {
		os << s;
	}
};

struct UnaryOpInstruction : public TACInstruction {
	UnaryOperator op;
	Operand arg;

	UnaryOpInstruction(std::string dest, UnaryOperator op, Operand arg) : TACInstruction(dest), op(op), arg(arg) {}
};

struct BinaryOpInstruction : public TACInstruction {
	BinaryOperator op;
	Operand left;
	Operand right;

	BinaryOpInstruction(std::string dest, BinaryOperator op, Operand left, Operand right) : TACInstruction(dest), op(op), left(left), right(right) {}
};

struct ReturnInstruction : public TACInstruction {
	Operand val;

	ReturnInstruction(std::string dest, Operand val) : TACInstruction(dest), val(val) {}
};

struct FunctionBody {
	std::string name;
	unsigned int variableCount = 0;
	std::vector<std::unique_ptr<TACInstruction>> instructions;

	template<typename Instruction, typename... Args>
	std::string emplaceInstruction(Args... args) {
		std::string destination = name + "$" + std::to_string(variableCount++);
		instructions.push_back(std::make_unique<Instruction>(destination, std::forward<Args>(args)...));
		return destination;
	}
};

Operand makeTac(const std::unique_ptr<ASTNode>& node, FunctionBody& body);
std::ostream& operator<<(std::ostream& os, const FunctionBody& instruction);
