#pragma once
#include <memory>
#include <string>
#include <ostream>
#include <sstream>
#include "lexer.hpp"

enum class Types {
	INT
};

struct ASTNode {
	virtual ~ASTNode() = default;
	virtual std::ostream& print(std::ostream& os, int) const = 0;
	virtual void generate(std::stringstream&) const = 0;
	virtual std::string evaluate() const {
		throw std::runtime_error("Not implemented");
	}
};

inline std::ostream& operator<<(std::ostream& os, const ASTNode& node) {
	return node.print(os, 0);
}

struct ProgramNode : public ASTNode {
	std::unique_ptr<ASTNode> function_declaration;
	std::ostream& print(std::ostream&, int) const override;
	void generate(std::stringstream&) const override;
	std::string evaluate() const override;
};

template<typename ReturnType>
struct FunctionDeclarationNode : public ASTNode {
	using return_type = ReturnType;
	std::string identifier;
	std::unique_ptr<ASTNode> statement;
	std::ostream& print(std::ostream& os, int indent) const override {
		os << std::string(indent, ' ') << "FUNCTION DECLARATION NODE: " << identifier << '\n';
		statement->print(os, indent + 1);
		return os;
	}
	void generate(std::stringstream& ss) const override {
		ss << ".global _" << identifier << "\n";
		ss << "_" << identifier << ":\n";
		statement->generate(ss);
	}
};

struct ReturnNode : public ASTNode {
	std::unique_ptr<ASTNode> expression;
	std::ostream& print(std::ostream&, int) const override;
	void generate(std::stringstream&) const override;
};

struct ConstNode : public ASTNode {
	Number value;
	std::ostream& print(std::ostream&, int) const override;
	void generate(std::stringstream&) const override;
	std::string evaluate() const override;
};
