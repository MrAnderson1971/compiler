#include "semantics.hpp"
#include <unordered_map>

namespace {
	std::string makeTemporary(const std::string& name) {
		static int counter = 0;
		return std::format("name{}", counter++);
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
	void resolveExp(const std::unique_ptr<ASTNode>& node, std::unordered_map<std::string, std::string>& variableMap) {
		if (auto* declaration = dynamic_cast<AssignmentNode*>(node.get())) {
			if (!dynamic_cast<VariableNode*>(declaration->left.get())) {
				throw semantic_error("Invalid lvalue!");
			}
			resolveExp(declaration->left, variableMap);
			resolveExp(declaration->right, variableMap);
		} else if (auto* variable = dynamic_cast<VariableNode*>(node.get())) {
			if (!variableMap.contains(variable->identifier)) {
				throw semantic_error(std::format("Undeclared variable {}", variable->identifier));
			}
			variable->identifier = variableMap[variable->identifier];
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
	void resolveDeclaration(DeclarationNode* const node, std::unordered_map<std::string, std::string>& variableMap) {
		if (variableMap.contains(node->identifier)) {
			throw semantic_error(std::format("Duplicate variable declaration {}", node->identifier));
		}
		std::string newName = makeTemporary(node->identifier);
		variableMap[node->identifier] = newName;
		node->identifier = newName;
		if (node->expression) {
			resolveExp(node->expression, variableMap);
		}
	}
}

void traverse(ASTNode* const node) {
	static std::unordered_map<std::string, std::string> variableMap;
	if (auto* declaration = dynamic_cast<DeclarationNode*>(node)) {
		resolveDeclaration(declaration, variableMap);
	} else {
		
	}
}
