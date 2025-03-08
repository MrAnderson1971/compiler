#pragma once

#include <variant>
#include <string>
#include <ostream>
#include <format>

class compiler_error : public std::exception {
	const std::string message;

public:
	explicit compiler_error(const std::string& message) : message(message) {}
	const char* what() const noexcept override {
		return message.c_str();
	}
};

struct PseudoRegister {
	std::string name;
	int position;
};

template <>
struct std::formatter<PseudoRegister> : std::formatter<std::string> {
	auto format(const PseudoRegister& reg, std::format_context& ctx) const {
		return std::formatter<std::string>::format(std::to_string(-4 * reg.position) + "(%rbp)", ctx);
	}
};

inline std::ostream& operator<<(std::ostream& os, const PseudoRegister& reg) {
	return os << reg.name << "$" << reg.position;
}

using Number = unsigned int;
using Operand = std::variant<PseudoRegister, 
	Number, // number literal
	std::nullptr_t
>;

enum class Types {
	INT
};

enum UnaryOperator {
	NEGATION,
	BITWISE_NOT,
	LOGICAL_NOT
};

enum BinaryOperator {
	ADD,
	SUBTRACT,
	MULTIPLY,
	DIVIDE,
	MODULO
};
