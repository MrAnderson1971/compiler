#pragma once

#include <variant>
#include <string>
#include <ostream>
#include <format>

#ifdef _DEBUG
constexpr bool DEBUG = true;
#else
constexpr bool DEBUG = false;
#endif

template<typename T>
bool isOneOf(T) {
	return false;
}

template<typename T, typename... Args>
bool isOneOf(T first, T second, Args... rest) {
	return first == second || isOneOf(first, rest...);
}

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

using Position = std::pair<int, std::string>; // function name and line number, for error messages
template<>
struct std::formatter<Position> : std::formatter<std::string> {
	auto format(const Position& pos, std::format_context& ctx) const {
		return std::formatter<std::string>::format(std::format("line {} in {}", pos.first, pos.second), ctx);
	}
};
