#include "ast.hpp"

std::ostream& ProgramNode::print(std::ostream& os, int indent) const {
	os << "PROGRAM NODE\n";
	function_declaration->print(os, 1);
	return os;
}

std::ostream& ReturnNode::print(std::ostream& os, int indent) const {
	os << std::string(indent, ' ') << "RETURN NODE\n";
	expression->print(os, indent + 1);
	return os;
}

std::ostream& ConstNode::print(std::ostream& os, int indent) const {
	return os << std::string(indent, ' ') << "CONST NODE: " << value << '\n';
}

void ProgramNode::generate(std::stringstream& ss) const {
	function_declaration->generate(ss);
}

std::string ProgramNode::evaluate() const {
	std::stringstream ss;
	generate(ss);
	return ss.str();
}

void ReturnNode::generate(std::stringstream& ss) const {
	ss << "movl $" << expression->evaluate() << ", %eax\nret";
}

void ConstNode::generate(std::stringstream& ss) const {
	 ss << value;
}

std::string ConstNode::evaluate() const {
	return std::to_string(value);
}
