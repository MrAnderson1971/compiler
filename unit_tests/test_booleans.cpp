#include "compiler.hpp"
#include "simulator.hpp"

TEST_F(CompilerTest, TestEquality) {
	std::string source = R"(
int main() {
		return 0 == 0;
		})";
	compile(source, ss);
	simulator.loadProgram(ss.str());
	EXPECT_EQ(1, simulator.execute());
}

TEST_F(CompilerTest, TestInequality) {
	std::string source = R"(
int main() {
		return 0 != 0;
		})";
	compile(source, ss);
	simulator.loadProgram(ss.str());
	EXPECT_EQ(0, simulator.execute());
}

TEST_F(CompilerTest, TestLessThan) {
	std::string source = R"(
int main() {
		return 0 < 1;
		})";
	compile(source, ss);
	simulator.loadProgram(ss.str());
	EXPECT_EQ(1, simulator.execute());
}

TEST_F(CompilerTest, TestBooleanAndArithmetic) {
	std::string source = R"(
int main() {
		return 100 == 36 + 64;
		})";
	compile(source, ss);
	simulator.loadProgram(ss.str());
	EXPECT_EQ(1, simulator.execute());
}

TEST_F(CompilerTest, TestLogicalAnd) {
	std::string source = R"(
int main() {
		return 1 && 1;
		})";
	compile(source, ss);
	simulator.loadProgram(ss.str());
	EXPECT_EQ(1, simulator.execute());
}

TEST_F(CompilerTest, TestFalseLogicalAnd1) {
	std::string source = R"(
int main() {
		return 1 && 0;
		})";
	compile(source, ss);
	simulator.loadProgram(ss.str());
	EXPECT_EQ(0, simulator.execute());
}

TEST_F(CompilerTest, TestFalseLogicalAnd2) {
	std::string source = R"(
int main() {
		return 0 && 1;
		})";
	compile(source, ss);
	simulator.loadProgram(ss.str());
	EXPECT_EQ(0, simulator.execute());
}

TEST_F(CompilerTest, TestFalseLogicalAndBoth) {
	std::string source = R"(
int main() {
		return 0 && 0;
		})";
	compile(source, ss);
	simulator.loadProgram(ss.str());
	EXPECT_EQ(0, simulator.execute());
}