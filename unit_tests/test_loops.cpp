#include "simulator.hpp"
#include "compiler.hpp"
#include "exceptions.hpp"

namespace tests {
	TEST_F(CompilerTest, TestWhile) {
		std::string code = R"(
			int main() {
				int i = 0;
				while (i < 10) {
					i = i + 1;
				}
				return i;
			}
		)";
		compile(code, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 10);
	}

	TEST_F(CompilerTest, TestBreak) {
		std::string code = R"(
			int main() {
				int i = 0;
				while (i < 10) {
if (i >= 5) {
break;
}
					i = i + 1;
				}
				return i;
			}
		)";
		compile(code, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 5);
	}

	TEST_F(CompilerTest, TestContinue) {
		std::string code = R"(
			int main() {
				int i = 0;
int result = 0;
				while (i < 10) {
					i = i + 1;
if (i % 2 == 1) {
continue;
}
result += i;
				}
				return result;
			}
		)";
		compile(code, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 30);
	}

	TEST_F(CompilerTest, TestBreakOutsideLoop) {
		std::string code = R"(
			int main() {
				break;
				return 0;
			}
		)";
		EXPECT_THROW(compile(code, ss), semantic_error);
	}

	TEST_F(CompilerTest, TestContinueOutsideLoop) {
		std::string code = R"(
			int main() {
				continue;
				return 0;
			}
		)";
		EXPECT_THROW(compile(code, ss), semantic_error);
	}

	TEST_F(CompilerTest, TestWhileWithoutBody) {
		std::string source = R"(
			int main() {
				int i = 0;
				while (++i && i < 10);
				return i;
			}
		)";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 10);
	}
}
