#pragma once

#include <vector>
#include <memory>
#include "type.hpp"
#include "lexer.hpp"

struct FunctionBody;


enum class UnaryOperator {
	NEGATION,

	_BEGIN = static_cast<int>(Symbol::_UNARY_BEGIN),
	BITWISE_NOT,
	LOGICAL_NOT
};

enum class BinaryOperator {
	SUBTRACT,

	_BEGIN = static_cast<int>(Symbol::_BINARY_BEGIN),
	ADD,
	MULTIPLY,
	DIVIDE,
	MODULO,
	BITWISE_XOR,
	BITWISE_AND,
	BITWISE_OR,
	SHIFT_LEFT,
	SHIFT_RIGHT,

	// bools
	LOGICAL_AND,
	LOGICAL_OR,
	EQUALS,
	NOT_EQUALS,
	LESS_THAN_OR_EQUAL,
	GREATER_THAN_OR_EQUAL,
	LESS_THAN,
	GREATER_THAN
};

// three address code
struct TACInstruction {
	virtual ~TACInstruction() = default;
	virtual std::string print() const = 0;
	virtual void makeAssembly(std::stringstream& ss, FunctionBody& body) const {};
};

struct has_dest : public TACInstruction {
	PseudoRegister dest;
	has_dest(const PseudoRegister& dest) : dest(dest) {}
};

struct OperandPrinter {
	std::ostream& os;

	template<typename Any>
	void operator()(const Any s) const {
		os << s;
	}
};

struct OperandToAsm {
	std::string operator()(const Number n) const;
	std::string operator()(const PseudoRegister& reg) const;
	std::string operator()(const std::nullptr_t) const;
};

extern OperandToAsm operandToAsm;

template<>
struct std::formatter<Operand> : std::formatter<std::string> {
	auto format(const Operand& op, std::format_context& ctx) const {
		return std::formatter<std::string>::format(std::visit(operandToAsm, op), ctx);
	}
};

struct FunctionInstruction : public TACInstruction {
	std::string name;
	std::string print() const override;
	void makeAssembly(std::stringstream& ss, FunctionBody& body) const override;
	FunctionInstruction(std::string name) : name(std::move(name)) {}
};

struct UnaryOpInstruction : public has_dest {
	UnaryOperator op;
	Operand arg;

	UnaryOpInstruction(const PseudoRegister& dest, UnaryOperator op, Operand arg) : has_dest(dest), op(op), arg(std::move(arg)) {}
	std::string print() const override;
	void makeAssembly(std::stringstream& ss, FunctionBody& body) const override;
};

struct BinaryOpInstruction : public has_dest {
	BinaryOperator op;
	Operand left;
	Operand right;

	BinaryOpInstruction(PseudoRegister dest, BinaryOperator op, Operand left, Operand right) :
	has_dest(dest), op(op), left(std::move(left)), right(std::move(right)) {}
	std::string print() const override;
	void makeAssembly(std::stringstream& ss, FunctionBody& body) const override;
};

struct JumpIfZero : public TACInstruction {
	std::string label;
	Operand op;

	JumpIfZero(Operand operand, std::string label) : op(std::move(operand)), label(std::move(label)) {}
	std::string print() const override;
	void makeAssembly(std::stringstream& ss, FunctionBody& body) const override;
};

struct JumpIfNotZero : public TACInstruction {
	std::string label;
	Operand op;
	JumpIfNotZero(Operand operand, std::string label) : label(std::move(label)), op(std::move(operand)) {}
	std::string print() const override;
	void makeAssembly(std::stringstream& ss, FunctionBody& body) const override;
};

struct Jump : public TACInstruction {
	std::string label;
	Jump(std::string label) : label(std::move(label)) {}
	std::string print() const override;
	void makeAssembly(std::stringstream& ss, FunctionBody& body) const override;
};

struct Label : public TACInstruction {
	std::string label;
	Label(std::string label) : label(std::move(label)) {}
	std::string print() const override;
	void makeAssembly(std::stringstream& ss, FunctionBody& body) const override;
};

struct StoreValueInstruction : public has_dest {
	Operand val;
	StoreValueInstruction(PseudoRegister dest, Operand val) : has_dest(dest), val(std::move(val)) {}
	std::string print() const override;

	void makeAssembly(std::stringstream& ss, FunctionBody& body) const override;;
};

struct ReturnInstruction : public TACInstruction {
	Operand val;

	ReturnInstruction(Operand val) : val(std::move(val)) {}
	std::string print() const override;
	void makeAssembly(std::stringstream& ss, FunctionBody& body) const override;
};

struct AllocateStackInstruction : public TACInstruction {
	std::string print() const override;
	void makeAssembly(std::stringstream& ss, FunctionBody& body) const override;
};

struct FunctionBody {
	std::string name;
	int variableCount = 1;
	int labelCount = 0;
	std::vector<std::unique_ptr<TACInstruction>> instructions;

	template<typename Instruction, typename... Args>
	PseudoRegister emplaceInstruction(Args... args) 
		requires std::is_base_of_v<has_dest, Instruction>
	{
		PseudoRegister destination{ name, variableCount };
		instructions.push_back(std::make_unique<Instruction>(destination, std::forward<Args>(args)...));
		return destination;
	}

	template<typename Instruction, typename... Args>
		requires (!std::is_base_of_v<has_dest, Instruction>)
	void emplaceInstruction(Args... args) {
		instructions.push_back(std::make_unique<Instruction>(std::forward<Args>(args)...));
	}
};

std::ostream& operator<<(std::ostream& os, const FunctionBody& instruction);
