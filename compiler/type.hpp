#pragma once

#include <variant>
#include <string>
#include <ostream>
#include <format>
#include <functional>

#ifdef _DEBUG
constexpr bool DEBUG = true;
#else
constexpr bool DEBUG = false;
#endif

template<typename T>
bool isOneOf(const T&) {
	return false;
}

template<typename T, typename U, typename... Args>
bool isOneOf(const T& first, const U& second, const Args&... rest) {
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


/*
 Turns all unique_ptr to rvalues. Everything else, does perfect forwarding.
 */
namespace forward {
	template<typename T>
	struct is_unique_ptr : std::false_type {};

	template<typename T, typename D>
	struct is_unique_ptr<std::unique_ptr<T, D>> : std::true_type {};

	template<typename T>
	inline constexpr bool is_unique_ptr_v = is_unique_ptr<std::remove_cvref_t<T>>::value;

	// Separate overloads for lvalues and rvalues
	template<typename T>
	decltype(auto) forward(std::remove_reference_t<T>& arg) {
		if constexpr (is_unique_ptr_v<T>) {
			return std::move(arg); // Always move unique_ptrs
		} else {
			return std::forward<T>(arg); // Standard forwarding
		}
	}

	// For rvalues
	template<typename T>
	decltype(auto) forward(std::remove_reference_t<T>&& arg) {
		static_assert(!std::is_lvalue_reference_v<T>, "Cannot forward an rvalue as an lvalue");
		return std::move(arg);
	}
}
