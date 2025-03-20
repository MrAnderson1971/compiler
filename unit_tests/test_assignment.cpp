#include "simulator.hpp"
#include "compiler.hpp"
#include "exceptions.hpp"

TEST_F(CompilerTest, TestDeclaration) {
	compile("int main() { int a = 5; return a; }", ss);
	simulator.loadProgram(ss.str());
	EXPECT_EQ(simulator.execute(), 5);
}

TEST_F(CompilerTest, TestDeclareThenAssign) {
	compile("int main() { int a; a = 5; return a; }", ss);
	simulator.loadProgram(ss.str());
	EXPECT_EQ(simulator.execute(), 5);
}

TEST_F(CompilerTest, TestNonShortCircuit) {
	std::string source = R"(
int main() {
	int a = 0;
		0 || (a = 1);
		return a;
		}
		)";
	compile(source, ss);
	simulator.loadProgram(ss.str());
	EXPECT_EQ(simulator.execute(), 1);
}

TEST_F(CompilerTest, TestShortCircuit) {
	std::string source = R"(
int main() {
		int a = 42;
		1 || (a = 1);
		return a;
})";
	compile(source, ss);
	simulator.loadProgram(ss.str());
	EXPECT_EQ(simulator.execute(), 42);
}

TEST_F(CompilerTest, TestShortCircuit2) {
	std::string source = R"(
int main() {
		int a = 42;
		0 && (a = 1);
		return a;
})";
	compile(source, ss);
	simulator.loadProgram(ss.str());
	EXPECT_EQ(simulator.execute(), 42);
}

TEST_F(CompilerTest, TestNonShortCircuit2) {
	std::string source = R"(
int main() {
		int a = 0;
		1 && (a = 1);
		return a;
})";
	compile(source, ss);
	simulator.loadProgram(ss.str());
	EXPECT_EQ(simulator.execute(), 1);
}

TEST_F(CompilerTest, TestAssignmentPrecedence) {
	std::string source = R"(
int main() {
		int a = 0;
		a = 1 + 2;
		return a;
		})";
	compile(source, ss);
	simulator.loadProgram(ss.str());
	EXPECT_EQ(simulator.execute(), 3);
}

TEST_F(CompilerTest, TestVariablePartOfDeclaration) {
	std::string source = R"(
int main() {
int a = 0 && a;
	return a;
})";
	compile(source, ss);
	simulator.loadProgram(ss.str());
	EXPECT_EQ(simulator.execute(), 0);
}

TEST_F(CompilerTest, TestMixedPrecedence) {
	std::string source = R"(
int main() {
    int a = 1;
    int b = 0;
    a = 3 * (b = a);
    return a + b;
})";
	compile(source, ss);
	simulator.loadProgram(ss.str());
	EXPECT_EQ(simulator.execute(), 4);
}

TEST_F(CompilerTest, TestExpressionThenDeclaration) {
	std::string source = R"(
int main() {
	int a = 999;
		a = a % 2;
int b = -a;
	return b;
})";
	compile(source, ss);
	simulator.loadProgram(ss.str());
	EXPECT_EQ(simulator.execute(), -1);
}

TEST_F(CompilerTest, TestAssignToReturn) {
	std::string source = "int main() { int return = 5; return return;}";
	EXPECT_THROW(compile(source, ss), syntax_error);
}

TEST_F(CompilerTest, TestDeclarationInReturn) {
	std::string source = "int main() { return int a = 5; }";
	EXPECT_THROW(compile(source, ss), syntax_error);
}

TEST_F(CompilerTest, TestBadType) {
	std::string source = "int main() { ints a = 0; return a; }";
	EXPECT_THROW(compile(source, ss), syntax_error);
}

TEST_F(CompilerTest, TestBadPrecedence) {
	std::string source = "int main() { int a = 0; a = 3 * a + 1; return a; }";
	compile(source, ss);
	simulator.loadProgram(ss.str());
	EXPECT_EQ(simulator.execute(), 1);
}

TEST_F(CompilerTest, TestUndefined) {
	std::string source = "int main() { return a; }";
	EXPECT_THROW(compile(source, ss), semantic_error);
}

TEST_F(CompilerTest, TestAssignBeforeDeclare) {
	std::string source = "int main() { a = 5; int a; return a; }";
	EXPECT_THROW(compile(source, ss), semantic_error);
}

TEST_F(CompilerTest, TestDuplicateDeclaration) {
	std::string source = "int main() { int a = 1; int a = 2; return a; }";
	EXPECT_THROW(compile(source, ss), semantic_error);
}

TEST_F(CompilerTest, TestPrefixIncrement) {
	std::string source = "int main() { int a = 0; return ++a; }";
	compile(source, ss);
	simulator.loadProgram(ss.str());
	EXPECT_EQ(simulator.execute(), 1);
}

TEST_F(CompilerTest, TestPrefixDecrement) {
	std::string source = "int main() { int a = 0; return --a; }";
	compile(source, ss);
	simulator.loadProgram(ss.str());
	EXPECT_EQ(simulator.execute(), -1);
}

TEST_F(CompilerTest, TestAssignmentInReturn) {
	std::string source = "int main() { int a = 0; return a = (a + 5); }";
	compile(source, ss);
	simulator.loadProgram(ss.str());
	EXPECT_EQ(simulator.execute(), 5);
}

TEST_F(CompilerTest, TestComplexPrefixIncrementDecrementAndAssigns) {
	std::string source = "int main() { int a = 0; return a = ++a + a + a + --a; }";
	compile(source, ss);
	simulator.loadProgram(ss.str());
	EXPECT_EQ(simulator.execute(), 3);
}

TEST_F(CompilerTest, TestInvalidPrefixIncrement) {
	std::string source = "int main() { int a = 0; return ++0; }";
	EXPECT_THROW(compile(source, ss), syntax_error);
}

TEST_F(CompilerTest, TestInvalidPrefixDecrement) {
	std::string source = "int main() { int a = 0; return --0; }";
	EXPECT_THROW(compile(source, ss), syntax_error);
}