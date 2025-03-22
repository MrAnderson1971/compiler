#include "variable_resolution.hpp"

#include "exceptions.hpp"

void VariableResolutionVisitor::visitProgram(ProgramNode* const node) {
	throw std::logic_error("ProgramNode should not be visited by VariableResolutionVisitor");
}

void VariableResolutionVisitor::visitFunctionDefinition(FunctionDefinitionNode* const node) {
    for (const auto& statement : node->block_items) {
        statement->accept(*this);
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
    if (variableMap.contains(node->identifier)) {
        throw semantic_error(std::format("Duplicate variable declaration {} at {}", node->identifier, node->lineNumber));
    }
    std::string newName = makeTemporary(node->identifier);
    variableMap[node->identifier] = newName;
    node->identifier = newName;

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
    node->identifier = variableMap[node->identifier];
}

void VariableResolutionVisitor::visitPostfix(PostfixNode* const node) {
    node->variable->accept(*this);
}

void VariableResolutionVisitor::visitPrefix(PrefixNode* const node) {
    node->variable->accept(*this);
}
