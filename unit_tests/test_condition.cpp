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

	TEST_F(CompilerTest, TestSingleIfTrue) {
		std::string source = R"(
int main() {
if (2 + 3 == 5) return 6;
			})";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 6);
	}

	TEST_F(CompilerTest, TestSingleIfFalse) {
		std::string source = R"(
int main() {
			if (2 + 3 == 6) return 6;
			})";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 0);
	}

	TEST_F(CompilerTest, TestIfElseIntoIf) {
		std::string source = R"(
int main() {
	int a = 1;
	if (a) return 2;
	else return 3;
}
)";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 2);
	}

	TEST_F(CompilerTest, TestIfElseIntoElse) {
		std::string source = R"(
int main() {
	int a = 0;
	if (a) return 2;
	else return 3;
}
)";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 3);
	}

	TEST_F(CompilerTest, TestElseWithoutIf) {
		std::string source = R"(
int main() {
	else return 3;
)";
		EXPECT_THROW(compile(source, ss), syntax_error);
	}

	TEST_F(CompilerTest, TestTernaryWithoutCondition) {
		std::string source = R"(
int main() {
			return ? 1 : 2;
			})";
		EXPECT_THROW(compile(source, ss), syntax_error);
	}

	TEST_F(CompilerTest, TestTernaryWithoutFirstCase) {
		std::string source = R"(
int main() {
			return ? : 2;
			})";
		EXPECT_THROW(compile(source, ss), syntax_error);
	}

	TEST_F(CompilerTest, TestTernaryWithoutSecondCase) {
		std::string source = R"(
int main() {
			return 0 ? 1 :;
			})";
		EXPECT_THROW(compile(source, ss), syntax_error);
	}

	TEST_F(CompilerTest, TestTernaryWithoutQuestion) {
		std::string source = R"(
int main() {
			return 1 : 2;
			})";
		EXPECT_THROW(compile(source, ss), syntax_error);
	}

	TEST_F(CompilerTest, TestTernaryWithoutColon) {
		std::string source = R"(
int main() {
			return 0 ? 1  2;
			})";
		EXPECT_THROW(compile(source, ss), syntax_error);
	}

	TEST_F(CompilerTest, TestIfEmptyCondition) {
		std::string source = R"(
int main() {
			if () return 1;
			})";
		EXPECT_THROW(compile(source, ss), syntax_error);
	}

	TEST_F(CompilerTest, TestIfEmptyBody) {
		std::string source = R"(
int main() {
			if (1)
			})";
		EXPECT_THROW(compile(source, ss), syntax_error);
	}

	TEST_F(CompilerTest, TestIfElseIfElse) {
		std::string source = R"(
int main() {
			if (0) return 1;
			else if (1) return 2;
			else return 3;
			})";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 2);
	}
}