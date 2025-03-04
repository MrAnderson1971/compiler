#pragma once
#include <memory>
#include <string>
#include <ostream>

enum class Types {
	INT
};

struct ASTNode {
	virtual ~ASTNode() = default;
	virtual std::ostream& print(std::ostream& os, int) const = 0;
};

inline std::ostream& operator<<(std::ostream& os, const ASTNode& node) {
	return node.print(os, 0);
}

struct ProgramNode : public ASTNode {
	std::unique_ptr<ASTNode> function_declaration;
	std::ostream& print(std::ostream&, int) const override;
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
};

struct ReturnNode : public ASTNode {
	std::unique_ptr<ASTNode> expression;
	std::ostream& print(std::ostream&, int) const override;
};

struct ConstNode : public ASTNode {
	unsigned int value;
	std::ostream& print(std::ostream&, int) const override;
};
