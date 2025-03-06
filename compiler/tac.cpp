#include "tac.hpp"
#include "ast.hpp"

Operand makeTac(const std::unique_ptr<ASTNode>& node, FunctionBody& body) {
	if (auto* constNode = dynamic_cast<ConstNode*>(node.get())) {
		return constNode->value;
	}
	else if (auto* unaryNode = dynamic_cast<UnaryNode*>(node.get())) {
		Operand src = makeTac(unaryNode->expression, body);
		std::string dest = body.emplaceInstruction<UnaryOpInstruction>(unaryNode->op, src);
		return dest;
	}
	//else if (auto* returnNode = dynamic_cast<ReturnNode*>(node.get())) {
	auto* returnNode = dynamic_cast<ReturnNode*>(node.get());
	Operand dest = "";
	if (returnNode->expression) {
		dest = makeTac(returnNode->expression, body);
	}
	body.emplaceInstruction<ReturnInstruction>(dest);
	return "";
	//}
}

std::ostream& operator<<(std::ostream& os, const FunctionBody& instruction) {
	for (const auto& i : instruction.instructions) {
		if (UnaryOpInstruction* unary = dynamic_cast<UnaryOpInstruction*>(i.get())) {
			os << unary->dest << " = ";
			switch (unary->op) {
			case NEGATION:
				os << "-";
				break;
			case BITWISE_NOT:
				os << "~";
				break;
			case LOGICAL_NOT:
				os << "!";
				break;
			}
			std::visit(OperandPrinter{ os }, unary->arg);
			os << "\n";
		}
		else if (auto* returnNode = dynamic_cast<ReturnInstruction*>(i.get())) {
			os << "return ";
			std::visit(OperandPrinter{ os }, returnNode->val);
			os << "\n";
		}
	}
	return os;
}
