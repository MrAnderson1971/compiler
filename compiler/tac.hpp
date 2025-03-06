#pragma once

#include <vector>
#include <memory>
#include "type.hpp"

// three address code
struct TACInstruction {
	PseudoRegister dest;
	TACInstruction(PseudoRegister dest) : dest(dest) {}
	virtual ~TACInstruction() = default;
	virtual std::string print() const = 0;
};

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

	UnaryOpInstruction(PseudoRegister dest, UnaryOperator op, Operand arg) : TACInstruction(dest), op(op), arg(arg) {}
	std::string print() const override;
};

struct BinaryOpInstruction : public TACInstruction {
	BinaryOperator op;
	Operand left;
	Operand right;

	BinaryOpInstruction(PseudoRegister dest, BinaryOperator op, Operand left, Operand right) : TACInstruction(dest), op(op), left(left), right(right) {}
	std::string print() const override;
};

struct ReturnInstruction : public TACInstruction {
	Operand val;

	ReturnInstruction(PseudoRegister dest, Operand val) : TACInstruction(dest), val(val) {}
	std::string print() const override;
};

struct FunctionBody {
	std::string name;
	unsigned int variableCount = 0;
	std::vector<std::unique_ptr<TACInstruction>> instructions;

	template<typename Instruction, typename... Args>
	PseudoRegister emplaceInstruction(Args... args) {
		PseudoRegister destination{ name, variableCount++ };
		instructions.push_back(std::make_unique<Instruction>(destination, std::forward<Args>(args)...));
		return destination;
	}
};

std::ostream& operator<<(std::ostream& os, const FunctionBody& instruction);
