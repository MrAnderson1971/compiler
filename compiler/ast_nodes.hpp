/*
 Separate header file for AST nodes for faster compilation times.
 */

#pragma once
#include "ast.hpp"
#include "tac.hpp"

struct ForNode;
struct ContinueNode;
struct BreakNode;
struct WhileNode;
struct ConditionNode;
struct PrefixNode;
struct PostfixNode;
struct FunctionDefinitionNode;
struct DeclarationNode;
struct AssignmentNode;
struct ReturnNode;
struct UnaryNode;
struct BinaryNode;
struct ConstNode;
struct VariableNode;
struct BlockNode;

class FullVisitor : public Visitor {
public:
    virtual void visitFunctionDefinition(FunctionDefinitionNode* const node) = 0;
    virtual void visitDeclaration(DeclarationNode* const node) = 0;
    virtual void visitAssignment(AssignmentNode* const node) = 0;
    virtual void visitReturn(ReturnNode* const node) = 0;
    virtual void visitUnary(UnaryNode* const node) = 0;
    virtual void visitBinary(BinaryNode* const node) = 0;
    virtual void visitConst(ConstNode* const node) = 0;
    virtual void visitVariable(VariableNode* const node) = 0;
	virtual void visitPostfix(PostfixNode* const node) = 0;
	virtual void visitPrefix(PrefixNode* const node) = 0;
	virtual void visitCondition(ConditionNode* const node) = 0;
	virtual void visitBlock(BlockNode* const node) = 0;
	virtual void visitWhile(WhileNode* const node) = 0;
	virtual void visitBreak(BreakNode* const node) = 0;
	virtual void visitContinue(ContinueNode* const node) = 0;
	virtual void visitFor(ForNode* const node) = 0;
};

// Function definition node
struct FunctionDefinitionNode : public ASTNode {
    std::string identifier;
    std::unique_ptr<BlockNode> body;

	FunctionDefinitionNode(const std::string& identifier, std::unique_ptr<BlockNode> body)
		: identifier(identifier), body(std::move(body)) {
	}

    void accept(Visitor& visitor) override {
		static_cast<FullVisitor&>(visitor).visitFunctionDefinition(this); // static_cast failure is impossible
    }

    void generate(const CodeContext& context);
};

struct BlockNode : public ASTNode {
	std::vector<std::unique_ptr<ASTNode>> block_items;
	void accept(Visitor& visitor) override {
		static_cast<FullVisitor&>(visitor).visitBlock(this);
	}
};

// Variable declaration node
struct DeclarationNode : public ASTNode {
    std::string identifier;
    std::unique_ptr<ASTNode> expression;

    void accept(Visitor& visitor) override {
        static_cast<FullVisitor&>(visitor).visitDeclaration(this);
    }
};

// Return statement node
struct ReturnNode : public ASTNode {
    std::unique_ptr<ASTNode> expression;

    void accept(Visitor& visitor) override {
        static_cast<FullVisitor&>(visitor).visitReturn(this);
    }
};

// Unary operation node
struct UnaryNode : public ASTNode {
    UnaryOperator op;
    std::unique_ptr<ASTNode> expression;

    UnaryNode(UnaryOperator op, std::unique_ptr<ASTNode> expression) : op(op), expression(std::move(expression)) {}

    void accept(Visitor& visitor) override {
        static_cast<FullVisitor&>(visitor).visitUnary(this);
    }
};

// Binary operation node
struct BinaryNode : public ASTNode {
    BinaryOperator op;
    std::unique_ptr<ASTNode> left;
    std::unique_ptr<ASTNode> right;

    BinaryNode(BinaryOperator op, std::unique_ptr<ASTNode> left, std::unique_ptr<ASTNode> right)
        : op(op), left(std::move(left)), right(std::move(right)) {
    }

    void accept(Visitor& visitor) override {
        static_cast<FullVisitor&>(visitor).visitBinary(this);
    }
};

// Constant value node
struct ConstNode : public ASTNode {
    Number value;

    explicit ConstNode(Number value) : value(value) {}

    void accept(Visitor& visitor) override {
        static_cast<FullVisitor&>(visitor).visitConst(this);
    }
};

// permanent location in memory
struct LvalueNode : public ASTNode {
	virtual std::unique_ptr<LvalueNode> clone() const = 0;
};

// Variable reference node
struct VariableNode : public LvalueNode {
    std::string identifier;

    explicit VariableNode(const std::string& identifier) : identifier(identifier) {}

    void accept(Visitor& visitor) override {
        static_cast<FullVisitor&>(visitor).visitVariable(this);
    }

	std::unique_ptr<LvalueNode> clone() const override {
		return std::make_unique<VariableNode>(identifier);
	}
};

struct PrefixNode : public LvalueNode {
	std::unique_ptr<LvalueNode> variable;
	BinaryOperator op;
	PrefixNode(std::unique_ptr<LvalueNode> expression, BinaryOperator op) : variable(std::move(expression)), op(op) {}
	void accept(Visitor& visitor) override {
		static_cast<FullVisitor&>(visitor).visitPrefix(this);
	}

	std::unique_ptr<LvalueNode> clone() const override {
		return std::make_unique<PrefixNode>(variable->clone(), op);
	}
};

// postfix inc / dec (harder than prefix)
struct PostfixNode : public ASTNode {
	std::unique_ptr<LvalueNode> variable;
	BinaryOperator op;
	PostfixNode(std::unique_ptr<LvalueNode> expression, BinaryOperator op) : variable(std::move(expression)), op(op) {}
	void accept(Visitor& visitor) override {
		static_cast<FullVisitor&>(visitor).visitPostfix(this);
	}
};

// Assignment node
struct AssignmentNode : public ASTNode {
    std::unique_ptr<LvalueNode> left;
    std::unique_ptr<ASTNode> right;

    AssignmentNode(std::unique_ptr<LvalueNode> left, std::unique_ptr<ASTNode> right)
        : left(std::move(left)), right(std::move(right)) {
    }

    void accept(Visitor& visitor) override {
        static_cast<FullVisitor&>(visitor).visitAssignment(this);
    }
};

struct ConditionNode : public ASTNode {
	std::unique_ptr<ASTNode> condition;
	std::unique_ptr<ASTNode> ifTrue;
	std::unique_ptr<ASTNode> ifFalse;
	bool isTernary;
	ConditionNode(std::unique_ptr<ASTNode> condition, std::unique_ptr<ASTNode> ifTrue, std::unique_ptr<ASTNode> ifFalse, bool isTernary)
		: condition(std::move(condition)), ifTrue(std::move(ifTrue)), ifFalse(std::move(ifFalse)), isTernary(isTernary) {
	}
	void accept(Visitor& visitor) override {
		static_cast<FullVisitor&>(visitor).visitCondition(this);
	}
};

struct WhileNode : public ASTNode {
	std::unique_ptr<ASTNode> condition;
	std::unique_ptr<ASTNode> body;
    std::string label;
	bool isDoWhile;
	WhileNode(std::unique_ptr<ASTNode> condition, std::unique_ptr<ASTNode> block, const std::string& label, bool isDoWhile)
		: condition(std::move(condition)), body(std::move(block)), label(label), isDoWhile(isDoWhile) {
	}
	void accept(Visitor& visitor) override {
		static_cast<FullVisitor&>(visitor).visitWhile(this);
	}
};

struct BreakNode : public ASTNode {
	std::string label;
	BreakNode() = default;
	void accept(Visitor& visitor) override {
		static_cast<FullVisitor&>(visitor).visitBreak(this);
	}
};

struct ContinueNode : public ASTNode {
	std::string label;
	bool isFor;
	ContinueNode() = default;
	void accept(Visitor& visitor) override {
		static_cast<FullVisitor&>(visitor).visitContinue(this);
	}
};

struct ForNode : public ASTNode {
	std::unique_ptr<ASTNode> init;
	std::unique_ptr<ASTNode> condition;
	std::unique_ptr<ASTNode> increment;
	std::unique_ptr<ASTNode> body;
	std::string label;
	ForNode(std::unique_ptr<ASTNode> init, std::unique_ptr<ASTNode> condition, std::unique_ptr<ASTNode> increment, std::unique_ptr<ASTNode> block, const std::string& label)
		: init(std::move(init)), condition(std::move(condition)), increment(std::move(increment)), body(std::move(block)), label(label) {
	}
	void accept(Visitor& visitor) override {
		static_cast<FullVisitor&>(visitor).visitFor(this);
	}
};
