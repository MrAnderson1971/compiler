#include "variable_resolution.hpp"
#include "exceptions.hpp"
#include <ranges>

void VariableResolutionVisitor::visitProgram(ProgramNode* const node) {
	throw std::logic_error("ProgramNode should not be visited by VariableResolutionVisitor");
}

void VariableResolutionVisitor::visitFunctionDefinition(FunctionDefinitionNode* const node) {
    if (node->body) {
		node->body->accept(*this);
    }
}

/*
 *resolve_declaration(Declaration(name, init), variable_map):
 1 if name is in variable_map:
 fail("Duplicate variable declaration!")
 unique_name = make_temporary()
 2 variable_map.add(name, unique_name)
 3 if init is not null:
 init = resolve_exp(init, variable_map)
 4 return Declaration(unique_name, init)
 */
void VariableResolutionVisitor::visitDeclaration(DeclarationNode* const node) {
    if (!variableMap.contains(node->identifier)) {
		std::stack<Variable> stack;
        stack.emplace(function, node->identifier, layer);
		variableMap[node->identifier] = std::move(stack);
	} else {
		auto& stack = variableMap[node->identifier];

		if (!stack.empty() && stack.top().layer == layer) {
			throw semantic_error(std::format("Duplicate variable declaration {} at {}", node->identifier, node->lineNumber));
		}

		stack.emplace(function, node->identifier, layer);
	}
	node->identifier = function + "::" + node->identifier + "::" + std::to_string(layer);
	if (node->expression) {
		node->expression->accept(*this);
	}
}

/*
 *resolve_exp(e, variable_map):
 match e with
 | Assignment(left, right) ->
 if left is not a Var node:
 fail("Invalid lvalue!")
 return Assignment(1 resolve_exp(left, variable_map), 2 resolve_exp(right, variable_map))
 | Var(v) ->
 if v is in variable_map:
 return Var(3 variable_map.get(v))
 else:
 fail("Undeclared variable!")
 | --snip--
 */
void VariableResolutionVisitor::visitAssignment(AssignmentNode* const node) {
    node->left->accept(*this);
    node->right->accept(*this);
}

void VariableResolutionVisitor::visitReturn(ReturnNode* const node) {
    if (node->expression) {
        node->expression->accept(*this);
    }
}

void VariableResolutionVisitor::visitUnary(UnaryNode* const node) {
    node->expression->accept(*this);
}

void VariableResolutionVisitor::visitBinary(BinaryNode* const node) {
    node->left->accept(*this);
    node->right->accept(*this);
}

void VariableResolutionVisitor::visitVariable(VariableNode* const node) {
    if (!variableMap.contains(node->identifier)) {
        throw semantic_error(std::format("Undeclared variable {} at {}", node->identifier, node->lineNumber));
    }
	if (variableMap[node->identifier].empty()) {
		throw semantic_error(std::format("Variable {} at {} out of scope", node->identifier, node->lineNumber));
	}
	auto& variable = variableMap[node->identifier].top();
	node->identifier = variable.function + "::" + variable.name + "::" + std::to_string(variable.layer);
}

void VariableResolutionVisitor::visitPostfix(PostfixNode* const node) {
    node->variable->accept(*this);
}

void VariableResolutionVisitor::visitPrefix(PrefixNode* const node) {
    node->variable->accept(*this);
}

void VariableResolutionVisitor::visitCondition(ConditionNode* const node) {
	node->condition->accept(*this);
	node->ifTrue->accept(*this);
	if (node->ifFalse) {
		node->ifFalse->accept(*this);
	}
}

void VariableResolutionVisitor::visitBlock(BlockNode* const node) {
    layer++;
	for (auto& statement : node->block_items) {
		statement->accept(*this);
	}

	for (auto& stack : variableMap | std::views::values) {
		if (!stack.empty() && stack.top().layer == layer) {
			stack.pop();
		}
	}
	layer--;
}

void VariableResolutionVisitor::visitWhile(WhileNode* const node) {
	loopLabels.emplace(node->label, false);
	node->condition->accept(*this);
	if (node->body) {
		node->body->accept(*this);
	}
	loopLabels.pop();
}

void VariableResolutionVisitor::visitBreak(BreakNode* const node) {
	if (loopLabels.empty()) {
		throw semantic_error(std::format("Break statement at {} outside of loop", node->lineNumber));
	}
	node->label = loopLabels.top().first;
}

void VariableResolutionVisitor::visitContinue(ContinueNode* const node) {
	if (loopLabels.empty()) {
		throw semantic_error(std::format("Continue statement at {} outside of loop", node->lineNumber));
	}
	node->label = loopLabels.top().first;
	node->isFor = loopLabels.top().second;
}

void VariableResolutionVisitor::visitFor(ForNode* const node) {
	if (node->init) { // the init adds a scope
		layer++;
	}
	loopLabels.emplace(node->label, true);
	if (node->init) {
		node->init->accept(*this);
	}
	if (node->condition) {
		node->condition->accept(*this);
	}
	if (node->increment) {
		node->increment->accept(*this);
	}
	if (node->body) {
		node->body->accept(*this);
	}
	loopLabels.pop();
	if (node->init) {
		for (auto& stack : variableMap | std::views::values) {
			if (!stack.empty() && stack.top().layer == layer) {
				stack.pop();
			}
		}
		layer--;
	}
}
