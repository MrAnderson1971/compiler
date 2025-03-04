#pragma once
#include <memory>
#include <string>

enum class Types {
	INT
};

struct ASTNode {
	virtual ~ASTNode() = default;
};

struct ProgramNode : ASTNode {
	std::unique_ptr<ASTNode> function_declaration;
};

template<typename ReturnType>
struct FunctionDeclarationNode : ASTNode {
	using return_type = ReturnType;
	std::string identifier;
	std::unique_ptr<ASTNode> statement;
};

struct ReturnNode : ASTNode {
	std::unique_ptr<ASTNode> expression;
};

struct ConstNode : ASTNode {
	unsigned int value;
};
