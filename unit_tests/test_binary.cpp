#include "simulator.hpp"
#include "compiler.hpp"

TEST_F(CompilerTest, TestAddition) {
	std::string source = "int main() { return 1 + 2; }";
	compile(source, ss);
	std::cout << ss.str() << std::endl;
	simulator.loadProgram(ss.str());
	EXPECT_EQ(simulator.execute(), 3);
}
