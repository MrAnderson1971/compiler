#include "compiler.hpp"
#include "simulator.hpp"
#include "exceptions.hpp"

namespace tests {
	TEST_F(CompilerTest, TestBitwise) {
		std::string source = R"(
int main() {
	return ~12;
})";
		compile(source, ss);
		std::cout << ss.str() << std::endl;
		simulator.loadProgram(ss.str());
		EXPECT_EQ(~12, static_cast<int>(simulator.execute()));
	}

	TEST_F(CompilerTest, TestBitwise0) {
		std::string source = R"(
int main() {
	return ~0;
})";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(~0, static_cast<int>(simulator.execute()));
	}

	TEST_F(CompilerTest, TestMissingConst) {
		std::string source = R"(
int main() {
	return ~;
})";
		EXPECT_THROW(compile(source, ss), syntax_error);
	}

	TEST_F(CompilerTest, TestMissingSemicolon2) {
		std::string source = R"(
int main() {
	return ~12
})";
		EXPECT_THROW(compile(source, ss), syntax_error);
	}

	TEST_F(CompilerTest, TestWrongOrder) {
		std::string source = R"(
int main() {
	return 12~;
})";
		EXPECT_THROW(compile(source, ss), syntax_error);
	}
}
