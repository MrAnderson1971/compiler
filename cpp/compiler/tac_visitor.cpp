#include "tac_visitor.hpp"
#include "exceptions.hpp"

TacVisitor::TacVisitor(FunctionBody& body) : body(body), result(nullptr) {
}

void TacVisitor::visitProgram(ProgramNode* const node) {
	throw std::logic_error("ProgramNode should not be visited by TacVisitor");
}

void TacVisitor::visitFunctionDefinition(FunctionDefinitionNode* const node) {
    body.emplaceInstruction<FunctionInstruction>(node->lineNumber, body.name);
    body.emplaceInstruction<AllocateStackInstruction>(node->lineNumber);

    if (node->body) {
    	node->body->accept(*this);
    }
}

void TacVisitor::visitDeclaration(DeclarationNode* const node) {
    auto pseudoRegister = std::make_shared<PseudoRegister>(PseudoRegister{ *body.name, body.variableCount });
    body.variableToPseudoregister[*node->identifier] = pseudoRegister;
    if (node->expression) {
        node->expression->accept(*this);
        body.emplaceInstructionWithDestination<StoreValueInstruction>(node->lineNumber, pseudoRegister, result);
    }
    body.variableCount++;
}

void TacVisitor::visitAssignment(AssignmentNode* const node) {
    node->right->accept(*this);
    std::shared_ptr<Operand> src = result;
    try {
        node->left->accept(*this);
        auto variable = std::get<std::shared_ptr<PseudoRegister>>(*result);
        //node->right->accept(*this);
        //Operand src = result;
        body.emplaceInstructionWithDestination<StoreValueInstruction>(node->lineNumber, variable, src);
    } catch (std::bad_variant_access&) {
        throw semantic_error(std::format("Invalid lvalue {} at {}", result, node->lineNumber));
    }
}

void TacVisitor::visitReturn(ReturnNode* const node) {
    std::shared_ptr<Operand> dest = nullptr;
    if (node->expression) {
        node->expression->accept(*this);
        dest = result;
    }
    body.emplaceInstruction<ReturnInstruction>(node->lineNumber, dest);
    result = nullptr;
}

void TacVisitor::visitUnary(UnaryNode* const node) {
    node->expression->accept(*this);
    if (node->op == UnaryOperator::UNARY_ADD) { // if unary add do nothing
        return;
    }
    std::shared_ptr<Operand> src = result;
    auto dest = body.emplaceInstruction<UnaryOpInstruction>(node->lineNumber, node->op, src);
    body.variableCount++;
    result = std::make_shared<Operand>(dest);
}

void TacVisitor::visitBinary(BinaryNode* const node) {
    if (node->op == BinaryOperator::LOGICAL_AND) {
        auto falseLabel = std::make_shared<std::string>(std::format(".{}{}_false", body.name, ++body.labelCount));
        auto endLabel = std::make_shared<std::string>(std::format(".{}{}_end", body.name, ++body.labelCount));

        // Short-circuiting
        node->left->accept(*this);
        auto leftOperand = result;
        body.emplaceInstruction<JumpIfZero>(node->lineNumber, leftOperand, falseLabel); // goto false label

        node->right->accept(*this);
        auto rightOperand = result;
        body.emplaceInstruction<JumpIfZero>(node->lineNumber, rightOperand, falseLabel);

        auto dest = body.emplaceInstruction<StoreValueInstruction>(node->lineNumber, std::make_shared<Operand>(static_cast<Number>(1)));
        body.emplaceInstruction<Jump>(node->lineNumber, endLabel); // goto end

        body.emplaceInstruction<Label>(node->lineNumber, falseLabel); // false label
        dest = body.emplaceInstruction<StoreValueInstruction>(node->lineNumber, std::make_shared<Operand>(static_cast<Number>(0)));

        body.emplaceInstruction<Label>(node->lineNumber, endLabel); // end
        body.variableCount++;
        result = std::make_shared<Operand>(dest);
        return;
    }

    if (node->op == BinaryOperator::LOGICAL_OR) {
        auto trueLabel = std::make_shared<std::string>(std::format(".{}{}_true", body.name, ++body.labelCount));
        auto endLabel = std::make_shared<std::string>(std::format(".{}{}_end", body.name, ++body.labelCount));

        // Short-circuiting
        node->left->accept(*this);
        auto leftOperand = result;
        body.emplaceInstruction<JumpIfNotZero>(node->lineNumber, leftOperand, trueLabel); // goto true label

        node->right->accept(*this);
        auto rightOperand = result;
        body.emplaceInstruction<JumpIfNotZero>(node->lineNumber, rightOperand, trueLabel);

        auto dest = body.emplaceInstruction<StoreValueInstruction>(node->lineNumber, std::make_shared<Operand>(static_cast<Number>(0)));
        body.emplaceInstruction<Jump>(node->lineNumber, endLabel); // goto end

        body.emplaceInstruction<Label>(node->lineNumber, trueLabel); // true label
        dest = body.emplaceInstruction<StoreValueInstruction>(node->lineNumber, std::make_shared<Operand>(static_cast<Number>(1)));

        body.emplaceInstruction<Label>(node->lineNumber, endLabel); // end
        body.variableCount++;
        result = std::make_shared<Operand>(dest);
        return;
    }

    node->left->accept(*this);
    auto leftOperand = result;

    node->right->accept(*this);
    auto rightOperand = result;

    auto dest = body.emplaceInstruction<BinaryOpInstruction>(node->lineNumber, node->op, leftOperand, rightOperand);
    body.variableCount++;
    result = std::make_shared<Operand>(dest);
}

void TacVisitor::visitConst(ConstNode* const node) {
    result = std::make_shared<Operand>(node->value);
}

void TacVisitor::visitVariable(VariableNode* const node) {
    if (!body.variableToPseudoregister.contains(*node->identifier)) {
        throw semantic_error(std::format("Undeclared variable {} at {}", node->identifier, node->lineNumber));
    }
    result = std::make_shared<Operand>(body.variableToPseudoregister[*node->identifier]);
}

void TacVisitor::visitPostfix(PostfixNode* const node) {
    node->variable->accept(*this); // get variable
    auto variable = result; // save variable
    auto temp1 = body.emplaceInstruction<StoreValueInstruction>(node->lineNumber, result); // temp1 = a
    body.variableCount++;
    auto temp2 = body.emplaceInstruction<BinaryOpInstruction>(node->lineNumber, node->op,
        variable, std::make_shared<Operand>(static_cast<Number>(1))); // t2 = a + 1
    ++body.variableCount;
    body.emplaceInstructionWithDestination<StoreValueInstruction>(node->lineNumber, std::get<std::shared_ptr<PseudoRegister>>(*variable), std::make_shared<Operand>(temp2)); // a = t2
    result = std::make_shared<Operand>(temp1);
}

void TacVisitor::visitPrefix(PrefixNode* const node) {
    node->variable->accept(*this);
    auto variable = result;
    body.emplaceInstructionWithDestination<BinaryOpInstruction>(node->lineNumber, std::get<std::shared_ptr<PseudoRegister>>(*variable), node->op, variable, std::make_shared<Operand>(static_cast<Number>(1)));
    body.variableCount++;
}

/*
 <instructions for condition>
c = <result of condition>
JumpIfZero(c, end)
<instructions for statement>
Label(end)
*/
void TacVisitor::visitCondition(ConditionNode* const node) {
    if (node->isTernary) {
        node->condition->accept(*this);
        auto condition = result;
        auto elseLabel = std::make_shared<std::string>(std::format(".{}{}_else", body.name, ++body.labelCount));
        auto endLabel = std::make_shared<std::string>(std::format(".{}{}_end", body.name, ++body.labelCount));
        auto dest = std::make_shared<PseudoRegister>(PseudoRegister{ *body.name, body.variableCount++ });
        body.emplaceInstruction<JumpIfZero>(node->lineNumber, condition, elseLabel); // if false goto else
        node->ifTrue->accept(*this);
        body.emplaceInstructionWithDestination<StoreValueInstruction>(node->lineNumber, dest, result);
        body.emplaceInstruction<Jump>(node->lineNumber, endLabel); // goto end
        body.emplaceInstruction<Label>(node->lineNumber, elseLabel); // else
        node->ifFalse->accept(*this);
        body.emplaceInstructionWithDestination<StoreValueInstruction>(node->lineNumber, dest, result);
        body.emplaceInstruction<Label>(node->lineNumber, endLabel); // end
        result = std::make_shared<Operand>(dest);
    } else if (node->ifFalse == nullptr) {
		node->condition->accept(*this);
		auto condition = result;
		auto endLabel = std::make_shared<std::string>(std::format(".{}{}_end", body.name, ++body.labelCount));
		body.emplaceInstruction<JumpIfZero>(node->lineNumber, condition, endLabel); // if false goto end
		node->ifTrue->accept(*this);
		body.emplaceInstruction<Label>(node->lineNumber, endLabel); // end
        result = nullptr;
    } else {
		node->condition->accept(*this);
		auto condition = result;
		auto elseLabel = std::make_shared<std::string>(std::format(".{}{}_else", body.name, ++body.labelCount));
		auto endLabel = std::make_shared<std::string>(std::format(".{}{}_end", body.name, ++body.labelCount));
		body.emplaceInstruction<JumpIfZero>(node->lineNumber, condition, elseLabel); // if false goto else
		node->ifTrue->accept(*this);
		body.emplaceInstruction<Jump>(node->lineNumber, endLabel); // goto end
		body.emplaceInstruction<Label>(node->lineNumber, elseLabel); // else
		node->ifFalse->accept(*this);
		body.emplaceInstruction<Label>(node->lineNumber, endLabel); // end
        result = nullptr;
    }
}

void TacVisitor::visitBlock(BlockNode* const node) {
	for (auto& item : node->block_items) {
		item->accept(*this);
	}
}

/*
Label(start)
<instructions for condition>
v = <result of condition>
JumpIfZero(v, end)
<instructions for body>
1 Jump(start)
Label(end)
 */
void TacVisitor::visitWhile(WhileNode* const node) {
	auto startLabel = std::make_shared<std::string>(std::format(".{}{}_start.loop", body.name, node->label));
	auto endLabel = std::make_shared<std::string>(std::format(".{}{}_end.loop", body.name, node->label));
	body.emplaceInstruction<Label>(node->lineNumber, startLabel); // start
	node->condition->accept(*this);
	auto condition = result;
	body.emplaceInstruction<JumpIfZero>(node->lineNumber, condition, endLabel); // if false goto end
    if (node->body) {
        node->body->accept(*this);
    }
	body.emplaceInstruction<Jump>(node->lineNumber, startLabel); // goto start
	body.emplaceInstruction<Label>(node->lineNumber, endLabel); // end
	result = nullptr;
}

void TacVisitor::visitBreak(BreakNode* const node) {
	body.emplaceInstruction<Jump>(node->lineNumber, std::make_shared<std::string>(std::format(".{}{}_end.loop", body.name, node->label)));
}

void TacVisitor::visitContinue(ContinueNode* const node) {
	if (node->isFor) {
		body.emplaceInstruction<Jump>(node->lineNumber, std::make_shared<std::string>(std::format(".{}{}_increment.loop", body.name, node->label)));
	} else {
		body.emplaceInstruction<Jump>(node->lineNumber, std::make_shared<std::string>(std::format(".{}{}_start.loop", body.name, node->label)));
	}
}

void TacVisitor::visitFor(ForNode* const node) {
	auto startLabel = std::make_shared<std::string>(std::format(".{}{}_start.loop", body.name, node->label));
	auto endLabel = std::make_shared<std::string>(std::format(".{}{}_end.loop", body.name, node->label));
	auto incrementLabel = std::make_shared<std::string>(std::format(".{}{}_increment.loop", body.name, node->label));
	if (node->init) {
		node->init->accept(*this);
	}
	body.emplaceInstruction<Label>(node->lineNumber, startLabel); // start
	if (node->condition) {
		node->condition->accept(*this);
		auto condition = result;
		body.emplaceInstruction<JumpIfZero>(node->lineNumber, condition, endLabel); // if false goto end
	}
	if (node->body) {
		node->body->accept(*this);
	}
    body.emplaceInstruction<Label>(node->lineNumber, incrementLabel); // increment
	if (node->increment) {
		node->increment->accept(*this);
	}
	body.emplaceInstruction<Jump>(node->lineNumber, startLabel); // goto start
	body.emplaceInstruction<Label>(node->lineNumber, endLabel); // end
	result = nullptr;
}
