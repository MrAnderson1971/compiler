#include "simulator.hpp"
#include "exceptions.hpp"
#include "compiler.hpp"

namespace tests {
	TEST_F(CompilerTest, TestTernary) {
		std::string source = R"(
int main() {
	int a = 1;
	int b = 2;
return a > b ? a : b;
})";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 2);
	}

	TEST_F(CompilerTest, TestTernaryOtherSide) {
		std::string source = R"(
int main() {
	int a = 1;
	int b = 2;
return a < b ? a : b;
})";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 1);
	}
}