#include "simulator.hpp"
#include "compiler.hpp"

TEST_F(CompilerTest, TestAddition) {
	std::string source = "int main() { return 1 + 2; }";
	compile(source, ss);
	simulator.loadProgram(ss.str());
	EXPECT_EQ(simulator.execute(), 3);
}

TEST_F(CompilerTest, TestSubtraction)
{
	std::string source = "int main() { return 3 - 2; }";
	compile(source, ss);
	simulator.loadProgram(ss.str());
	EXPECT_EQ(simulator.execute(), 1);
}

TEST_F(CompilerTest, TestMultiplication)
{
	std::string source = "int main() { return 2 * 3; }";
	compile(source, ss);
	simulator.loadProgram(ss.str());
	EXPECT_EQ(simulator.execute(), 6);
}

TEST_F(CompilerTest, TestDivision)
{
	std::string source = "int main() { return 6 / 2; }";
	compile(source, ss);
	simulator.loadProgram(ss.str());
	EXPECT_EQ(simulator.execute(), 3);
}

TEST_F(CompilerTest, TestModulus)
{
	std::string source = "int main() { return 7 % 3; }";
	compile(source, ss);
	simulator.loadProgram(ss.str());
	EXPECT_EQ(simulator.execute(), 1);
}

TEST_F(CompilerTest, TestPrecedence)
{
	std::string source = "int main() { return 1 + 2 * 3; }";
	compile(source, ss);
	simulator.loadProgram(ss.str());
	EXPECT_EQ(simulator.execute(), 7);
}

TEST_F(CompilerTest, TestParentheses)
{
	std::string source = "int main() { return (1 + 2) * 3; }";
	compile(source, ss);
	simulator.loadProgram(ss.str());
	EXPECT_EQ(simulator.execute(), 9);
}

TEST_F(CompilerTest, TestAssociativity) {
	std::string source = "int main() { return 1 - 2 - 3; }";
	compile(source, ss);
	simulator.loadProgram(ss.str());
	EXPECT_EQ(simulator.execute(), -4);
}

TEST_F(CompilerTest, TestAssociativityAndPrecedence) {
	std::string source = R"(int main() {
    return 5 * 4 / 2 -
        3 % (2 + 1);
})";
	compile(source, ss);
	simulator.loadProgram(ss.str());
	EXPECT_EQ(simulator.execute(), 10);
}

TEST_F(CompilerTest, TestDivideNegative) {
	std::string source = R"(int main() {
    return (-12) / 5;
})";
	compile(source, ss);
	simulator.loadProgram(ss.str());
	EXPECT_EQ(simulator.execute(), -2);
}

TEST_F(CompilerTest, TestUnaryAndBinary) {
	std::string source = R"(int main(void) {
	return ~(1+1);
})";
	compile(source, ss);
	simulator.loadProgram(ss.str());
	EXPECT_EQ(simulator.execute(), -2);
}
