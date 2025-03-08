#include "compiler.hpp"
#include "simulator.hpp"

TEST_F(CompilerTest, TestBitwise) {
	std::string source = R"(
int main() {
	return ~12;
})";
	compile(source, ss);
	simulator.loadProgram(ss.str());
	EXPECT_EQ(~12, simulator.execute());
}
