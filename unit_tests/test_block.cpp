#include "simulator.hpp"
#include "compiler.hpp"
#include "exceptions.hpp"

namespace tests {
	TEST_F(CompilerTest, TestBlock) {
		std::string source = R"(
int main() {
			int a = 2;
			{
			 int a = 3;
			 int b = a + 2;
			}
			return a;
		})";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 2);
	}

	TEST_F(CompilerTest, TestDuplicateInSameBlock) {
		std::string source = R"(
int main() {
			int a = 2;
			{
			 int a = 3;
			 int a = 4;
			 int b = a + 2;
			}
			return a;
		})";
		EXPECT_THROW(compile(source, ss), semantic_error);
	}

	TEST_F(CompilerTest, TestIfBlockScoping) {
		std::string source = R"(
int main() {
	int a = 2;
	if (1) {
		int a = 3;
		int b = a + 2;
	}
	return a;
})";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 2);
	}

	TEST_F(CompilerTest, TestIfElseBlockScoping) {
		std::string source = R"(
int main() {
	int a = 2;
	int result = 0;
	if (0) {
		int a = 3;
		result = a;
	} else {
		int a = 4;
		result = a;
	}
	return result + a;
})";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 6);  // result=4 from else block, a=2 from outer scope
	}

	TEST_F(CompilerTest, TestIfElseIfElseScoping) {
		std::string source = R"(
int main() {
	int a = 5;
	int b = 0;
	
	if (0) {
		int a = 10;
		b = a;
	} else if (1) {
		int a = 20;
		b = a;
	} else {
		int a = 30;
		b = a;
	}
	
	return a + b;
})";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 25);  // a=5 from outer scope, b=20 from else-if block
	}

	TEST_F(CompilerTest, TestDuplicateInIfBlock) {
		std::string source = R"(
int main() {
	int a = 5;
	
	if (1) {
		int b = 10;
		int b = 20;  // Duplicate variable in same block
	}
	
	return a;
})";
		EXPECT_THROW(compile(source, ss), semantic_error);
	}

	TEST_F(CompilerTest, TestDuplicateInElseBlock) {
		std::string source = R"(
int main() {
	int a = 5;
	
	if (0) {
		int b = 10;
	} else {
		int c = 15;
		int c = 25;  // Duplicate variable in same block
	}
	
	return a;
})";
		EXPECT_THROW(compile(source, ss), semantic_error);
	}

	TEST_F(CompilerTest, TestNestedBlocksInIf) {
		std::string source = R"(
int main() {
	int x = 1;
	
	if (1) {
		int x = 2;
		{
			int x = 3;
			{
				int x = 4;
			}
		}
	}
	
	return x;
})";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 1);  // x=1 from outer scope
	}

	TEST_F(CompilerTest, TestComplexNestedScopes) {
		std::string source = R"(
int main() {
	int a = 1;
	int result = 0;
	
	if (1) {
		int a = 2;
		{
			int a = 3;
			if (1) {
				int a = 4;
				result = a;
			} else {
				result = a;
			}
		}
	} else if (0) {
		int a = 5;
		result = a;
	} else {
		result = a;
	}
	
	return result;
})";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 4);  // result=4 from innermost if block
	}

	TEST_F(CompilerTest, TestVariableAccessAcrossBlocks) {
		std::string source = R"(
int main() {
	int a = 10;
	int b = 20;
	
	if (1) {
		int c = a + b;  // Access outer variables
		if (1) {
			int d = c + a;  // Access variables from multiple outer scopes
			return d;
		}
	}
	
	return 0;
})";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 40);  // d = c + a = (a + b) + a = 10 + 20 + 10 = 40
	}

	TEST_F(CompilerTest, TestMultipleVariablesInSameScope) {
		std::string source = R"(
int main() {
	int a = 5;
	int b = 10;
	int c = 15;
	
	if (1) {
		int a = 1;
		int b = 2;
		int c = 3;
		return a + b + c;
	}
	
	return a + b + c;
})";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 6);  // a=1, b=2, c=3 from if block
	}
}