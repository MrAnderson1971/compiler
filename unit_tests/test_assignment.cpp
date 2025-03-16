#include "simulator.hpp"
#include "compiler.hpp"

TEST_F(CompilerTest, TestAssign) {
	compile("int main() { int a = 5; return a; }", ss);
	simulator.loadProgram(ss.str());
	EXPECT_EQ(simulator.execute(), 5);
}