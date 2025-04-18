#pragma once

#include <vector>
#include <memory>
#include "type.hpp"
#include "lexer.hpp"
#include <unordered_map>

struct FunctionBody;

enum class UnaryOperator {
	NEGATION,
	UNARY_ADD,

	_BEGIN = static_cast<int>(Symbol::_UNARY_BEGIN),
	BITWISE_NOT,
	LOGICAL_NOT
};

enum class BinaryOperator {
	SUBTRACT,
	ADD,

	_BEGIN = static_cast<int>(Symbol::_BINARY_BEGIN),
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
	std::shared_ptr<Position> lineNumber;
	virtual ~TACInstruction() = default;
	virtual std::string print() const = 0;
	virtual void makeAssembly(std::stringstream& ss, FunctionBody& body) const = 0;
};

struct has_dest : public TACInstruction {
	std::shared_ptr<PseudoRegister> dest;
	has_dest(const std::shared_ptr<PseudoRegister>& dest) : dest(dest) {}
};

struct OperandPrinter {
	std::ostream& os;

	template<typename Any>
	void operator()(const Any& s) const {
		os << s;
	}
};

struct OperandToAsm {
	std::string operator()(const Number n) const;
	std::string operator()(const std::shared_ptr<PseudoRegister>& reg) const;
	std::string operator()(const std::nullptr_t) const;
};

extern OperandToAsm operandToAsm;

template<>
struct std::formatter<std::shared_ptr<Operand>> : std::formatter<std::string> {
	auto format(const std::shared_ptr<Operand>& op, std::format_context& ctx) const {
		return std::formatter<std::string>::format(std::visit(operandToAsm, *op), ctx);
	}
};

struct FunctionInstruction : public TACInstruction {
	std::shared_ptr<std::string> name;
	std::string print() const override;
	void makeAssembly(std::stringstream& ss, FunctionBody& body) const override;
	FunctionInstruction(const std::shared_ptr<std::string>& name) : name(name) {}
};

struct UnaryOpInstruction : public has_dest {
	UnaryOperator op;
	std::shared_ptr<Operand> arg;

	UnaryOpInstruction(const std::shared_ptr<PseudoRegister>& dest, UnaryOperator op, const std::shared_ptr<Operand>& arg) : has_dest(dest), op(op), arg(arg) {}
	std::string print() const override;
	void makeAssembly(std::stringstream& ss, FunctionBody& body) const override;
};

struct BinaryOpInstruction : public has_dest {
	BinaryOperator op;
	std::shared_ptr<Operand> left;
	std::shared_ptr<Operand> right;

	BinaryOpInstruction(const std::shared_ptr<PseudoRegister>& dest, BinaryOperator op, const std::shared_ptr<Operand>& left, const std::shared_ptr<Operand>& right) :
	has_dest(dest), op(op), left(left), right(right) {}
	std::string print() const override;
	void makeAssembly(std::stringstream& ss, FunctionBody& body) const override;
};

struct JumpIfZero : public TACInstruction {
	std::shared_ptr<std::string> label;
	std::shared_ptr<Operand> op;

	JumpIfZero(const std::shared_ptr<Operand>& operand, const std::shared_ptr<std::string>& label) : label(label), op(operand) {}
	std::string print() const override;
	void makeAssembly(std::stringstream& ss, FunctionBody& body) const override;
};

struct JumpIfNotZero : public TACInstruction {
	std::shared_ptr<std::string> label;
	std::shared_ptr<Operand> op;
	JumpIfNotZero(const std::shared_ptr<Operand>& operand, const std::shared_ptr<std::string>& label) : label(label), op(operand) {}
	std::string print() const override;
	void makeAssembly(std::stringstream& ss, FunctionBody& body) const override;
};

struct Jump : public TACInstruction {
	std::shared_ptr<std::string> label;
	Jump(const std::shared_ptr<std::string>& label) : label(label) {}
	std::string print() const override;
	void makeAssembly(std::stringstream& ss, FunctionBody& body) const override;
};

struct Label : public TACInstruction {
	std::shared_ptr<std::string> label;
	Label(const std::shared_ptr<std::string>& label) : label(label) {}
	std::string print() const override;
	void makeAssembly(std::stringstream& ss, FunctionBody& body) const override;
};

struct StoreValueInstruction : public has_dest {
	std::shared_ptr<Operand> val;
	StoreValueInstruction(const std::shared_ptr<PseudoRegister>& dest, const std::shared_ptr<Operand>& val) : has_dest(dest), val(val) {}
	std::string print() const override;

	void makeAssembly(std::stringstream& ss, FunctionBody& body) const override;
};

struct ReturnInstruction : public TACInstruction {
	std::shared_ptr<Operand> val;

	ReturnInstruction(const std::shared_ptr<Operand>& val) : val(val) {}
	std::string print() const override;
	void makeAssembly(std::stringstream& ss, FunctionBody& body) const override;
};

struct AllocateStackInstruction : public TACInstruction {
	std::string print() const override;
	void makeAssembly(std::stringstream& ss, FunctionBody& body) const override;
};

struct FunctionBody {
	std::shared_ptr<std::string> name;
	int variableCount = 1;
	int labelCount = 0;
	std::vector<std::unique_ptr<TACInstruction>> instructions;
	std::unordered_map<std::string, std::shared_ptr<PseudoRegister>> variableToPseudoregister;

	// auto-generated destination
	template<typename Instruction, typename... Args>
		requires std::is_base_of_v<has_dest, Instruction>
	std::shared_ptr<PseudoRegister> emplaceInstruction(const std::shared_ptr<Position>& lineNumber, Args&&... args) {
		std::shared_ptr<PseudoRegister> destination = std::make_shared<PseudoRegister>(PseudoRegister{ *name, variableCount });
		instructions.push_back(std::make_unique<Instruction>(destination, std::forward<Args>(args)...));
		instructions.back()->lineNumber = lineNumber;
		return destination;
	}

	// custom destination
	template <typename Instruction, typename... Args>
		requires std::is_base_of_v<has_dest, Instruction>
	std::shared_ptr<PseudoRegister> emplaceInstructionWithDestination(const std::shared_ptr<Position>& lineNumber, const std::shared_ptr<PseudoRegister>& customDest, Args&&... args) {
		instructions.push_back(std::make_unique<Instruction>(customDest, std::forward<Args>(args)...));
		instructions.back()->lineNumber = lineNumber;
		return customDest;
	}

	template<typename Instruction, typename... Args>
		requires (!std::is_base_of_v<has_dest, Instruction>)
	void emplaceInstruction(const std::shared_ptr<Position>& lineNumber, Args&&... args) {
		instructions.push_back(std::make_unique<Instruction>(forward::forward<Args>(args)...));
		instructions.back()->lineNumber = lineNumber;
	}
};

std::ostream& operator<<(std::ostream& os, const FunctionBody& instruction);
