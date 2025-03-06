#pragma once

#include <variant>
#include <string>

using Number = unsigned int;
using Operand = std::variant<std::string, Number>;

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
	DIVIDE
};
