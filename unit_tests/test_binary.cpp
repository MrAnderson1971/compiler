#include "simulator.hpp"
#include "compiler.hpp"

TEST_F(CompilerTest, TestAddition) {
	std::string source = "int main() { return 1 + 2; }";
	compile(source, ss);
	std::cout << ss.str() << std::endl;
	simulator.loadProgram(ss.str());
	EXPECT_EQ(simulator.execute(), 3);
}

TEST_F(CompilerTest, TestSubtraction)
{
	std::string source = "int main() { return 3 - 2; }";
	compile(source, ss);
	std::cout << ss.str() << std::endl;
	simulator.loadProgram(ss.str());
	EXPECT_EQ(simulator.execute(), 1);
}

TEST_F(CompilerTest, TestMultiplication)
{
	std::string source = "int main() { return 2 * 3; }";
	compile(source, ss);
	std::cout << ss.str() << std::endl;
	simulator.loadProgram(ss.str());
	EXPECT_EQ(simulator.execute(), 6);
}

TEST_F(CompilerTest, TestDivision)
{
	std::string source = "int main() { return 6 / 2; }";
	compile(source, ss);
	std::cout << ss.str() << std::endl;
	simulator.loadProgram(ss.str());
	EXPECT_EQ(simulator.execute(), 3);
}

TEST_F(CompilerTest, TestModulus)
{
	std::string source = "int main() { return 7 % 3; }";
	compile(source, ss);
	std::cout << ss.str() << std::endl;
	simulator.loadProgram(ss.str());
	EXPECT_EQ(simulator.execute(), 1);
}

TEST_F(CompilerTest, TestPrecedence)
{
	std::string source = "int main() { return 1 + 2 * 3; }";
	compile(source, ss);
	std::cout << ss.str() << std::endl;
	simulator.loadProgram(ss.str());
	EXPECT_EQ(simulator.execute(), 7);
}

TEST_F(CompilerTest, TestParentheses)
{
	std::string source = "int main() { return (1 + 2) * 3; }";
	compile(source, ss);
	std::cout << ss.str() << std::endl;
	simulator.loadProgram(ss.str());
	EXPECT_EQ(simulator.execute(), 9);
}
