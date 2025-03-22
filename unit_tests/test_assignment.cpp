#include <format>
#include "simulator.hpp"
#include "compiler.hpp"
#include "exceptions.hpp"
#include <limits>

namespace tests {
	TEST_F(CompilerTest, TestDeclaration) {
		compile("int main() { int a = 5; return a; }", ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 5);
	}

	TEST_F(CompilerTest, TestDeclareThenAssign) {
		compile("int main() { int a; a = 5; return a; }", ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 5);
	}

	TEST_F(CompilerTest, TestNonShortCircuit) {
		std::string source = R"(
int main() {
	int a = 0;
		0 || (a = 1);
		return a;
		}
		)";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 1);
	}

	TEST_F(CompilerTest, TestShortCircuit) {
		std::string source = R"(
int main() {
		int a = 42;
		1 || (a = 1);
		return a;
})";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 42);
	}

	TEST_F(CompilerTest, TestShortCircuit2) {
		std::string source = R"(
int main() {
		int a = 42;
		0 && (a = 1);
		return a;
})";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 42);
	}

	TEST_F(CompilerTest, TestNonShortCircuit2) {
		std::string source = R"(
int main() {
		int a = 0;
		1 && (a = 1);
		return a;
})";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 1);
	}

	TEST_F(CompilerTest, TestAssignmentPrecedence) {
		std::string source = R"(
int main() {
		int a = 0;
		a = 1 + 2;
		return a;
		})";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 3);
	}

	TEST_F(CompilerTest, TestVariablePartOfDeclaration) {
		std::string source = R"(
int main() {
int a = 0 && a;
	return a;
})";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 0);
	}

	TEST_F(CompilerTest, TestMixedPrecedence) {
		std::string source = R"(
int main() {
    int a = 1;
    int b = 0;
    a = 3 * (b = a);
    return a + b;
})";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 4);
	}

	TEST_F(CompilerTest, TestExpressionThenDeclaration) {
		std::string source = R"(
int main() {
	int a = 999;
		a = a % 2;
int b = -a;
	return b;
})";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), -1);
	}

	TEST_F(CompilerTest, TestAssignToReturn) {
		std::string source = "int main() { int return = 5; return return;}";
		EXPECT_THROW(compile(source, ss), syntax_error);
	}

	TEST_F(CompilerTest, TestDeclarationInReturn) {
		std::string source = "int main() { return int a = 5; }";
		EXPECT_THROW(compile(source, ss), syntax_error);
	}

	TEST_F(CompilerTest, TestBadType) {
		std::string source = "int main() { ints a = 0; return a; }";
		EXPECT_THROW(compile(source, ss), syntax_error);
	}

	TEST_F(CompilerTest, TestBadPrecedence) {
		std::string source = "int main() { int a = 0; a = 3 * a + 1; return a; }";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 1);
	}

	TEST_F(CompilerTest, TestUndefined) {
		std::string source = "int main() { return a; }";
		EXPECT_THROW(compile(source, ss), semantic_error);
	}

	TEST_F(CompilerTest, TestAssignBeforeDeclare) {
		std::string source = "int main() { a = 5; int a; return a; }";
		EXPECT_THROW(compile(source, ss), semantic_error);
	}

	TEST_F(CompilerTest, TestDuplicateDeclaration) {
		std::string source = "int main() { int a = 1; int a = 2; return a; }";
		EXPECT_THROW(compile(source, ss), semantic_error);
	}

	TEST_F(CompilerTest, TestPrefixIncrement) {
		std::string source = "int main() { int a = 0; return ++a; }";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 1);
	}

	TEST_F(CompilerTest, TestPrefixDecrement) {
		std::string source = "int main() { int a = 0; return --a; }";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), -1);
	}

	TEST_F(CompilerTest, TestAssignmentInReturn) {
		std::string source = "int main() { int a = 0; return a = (a + 5); }";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 5);
	}

	TEST_F(CompilerTest, TestComplexPrefixIncrementDecrementAndAssigns) {
		std::string source = "int main() { int a = 0; return a = ++a + a + a + --a; }";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 3);
	}

	TEST_F(CompilerTest, TestInvalidPrefixIncrement) {
		std::string source = "int main() { int a = 0; return ++0; }";
		EXPECT_THROW(compile(source, ss), semantic_error);
	}

	TEST_F(CompilerTest, TestInvalidPrefixDecrement) {
		std::string source = "int main() { int a = 0; return --0; }";
		EXPECT_THROW(compile(source, ss), semantic_error);
	}

	TEST_F(CompilerTest, TestPostfixIncrement) {
		std::string source = "int main() { int a = 0; return a++; }";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 0);
	}

	TEST_F(CompilerTest, TestGetValueOfPostfixIncrement) {
		std::string source = "int main() { int a = 0; a++; return a; }";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 1);
	}

	TEST_F(CompilerTest, TestPostfixDecrement) {
		std::string source = "int main() { int a = 0; return a--; }";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 0);
	}

	TEST_F(CompilerTest, TestGetValueOfPostfixDecrement) {
		std::string source = "int main() { int a = 0; a--; return a; }";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), -1);
	}

	TEST_F(CompilerTest, TestInvalidPostfixIncrement) {
		std::string source = "int main() { return 0++; }";
		EXPECT_THROW(compile(source, ss), semantic_error);
	}

	TEST_F(CompilerTest, TestInvalidPostfixDecrement) {
		std::string source = "int main() { return 0--; }";
		EXPECT_THROW(compile(source, ss), semantic_error);
	}

	TEST_F(CompilerTest, TestInvalidAssign) {
		std::string source = "int main() { int a; 1 + (0 = 5); return 0; }";
		EXPECT_THROW(compile(source, ss), semantic_error);
	}

	TEST_F(CompilerTest, TestNotLvalue) {
		std::string source = "int main() { int a = 0; -a = 1; return a; }";
		EXPECT_THROW(compile(source, ss), semantic_error);
	}

	TEST_F(CompilerTest, TestCompoundAdd) {
		std::string source = "int main() { int a = 0; a += 5; return a; }";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 5);
	}

	TEST_F(CompilerTest, TestIntegerOverflow) {
		std::string source = std::format("int main() {{ int a = {}; a += 1; return a; }}", (std::numeric_limits<int>::max)());
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), (std::numeric_limits<int>::min)());
	}

	// Prefix Operator Tests

	TEST_F(CompilerTest, TestChainedPrefixOperators) {
		std::string source = "int main() { int a = 0; return ++(++a); }";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 2);
	}

	TEST_F(CompilerTest, TestPrefixOperatorsInExpressions) {
		std::string source = "int main() { int a = 1; int b = 2; return ++a * ++b; }";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 6);
	}

	TEST_F(CompilerTest, TestInvalidPrefixOnExpressions) {
		std::string source = "int main() { int a = 1; int b = 2; return ++(a + b); }";
		EXPECT_THROW(compile(source, ss), semantic_error);
	}

	TEST_F(CompilerTest, TestPrefixWithAssignment) {
		std::string source = "int main() { int a = 0; int b = ++(a = 5); return b; }";
		EXPECT_THROW(compile(source, ss), semantic_error);
	}

	TEST_F(CompilerTest, TestSideEffectsWithPrefix) {
		std::string source = "int main() { int a = 1; int b = ++a + ++a; return b; }";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 6);  // Could be 2+3=5 depending on evaluation order
	}

	// Postfix Operator Tests

	TEST_F(CompilerTest, TestChainedPostfixOperators) {
		std::string source = "int main() { int a = 0; return (a++)++; }";
		EXPECT_THROW(compile(source, ss), semantic_error);
	}

	TEST_F(CompilerTest, TestPostfixInComplexExpressions) {
		std::string source = "int main() { int a = 1; int b = 2; return a++ * b++; }";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 2);
	}

	TEST_F(CompilerTest, TestSideEffectsWithPostfix) {
		std::string source = "int main() { int a = 1; int b = a++ + a++; return b; }";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 3);  // Could be 1+2=3 depending on evaluation order
	}

	TEST_F(CompilerTest, TestMixedPrefixAndPostfix) {
		std::string source = "int main() { int a = 5; return ++a + a++; }";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 13);  // Could be 6+6=12 depending on evaluation order
	}

	// Compound Assignment Tests

	TEST_F(CompilerTest, TestCompoundSubtract) {
		std::string source = "int main() { int a = 10; a -= 3; return a; }";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 7);
	}

	TEST_F(CompilerTest, TestCompoundMultiply) {
		std::string source = "int main() { int a = 5; a *= 3; return a; }";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 15);
	}

	TEST_F(CompilerTest, TestCompoundDivide) {
		std::string source = "int main() { int a = 10; a /= 2; return a; }";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 5);
	}

	TEST_F(CompilerTest, TestCompoundModulo) {
		std::string source = "int main() { int a = 10; a %= 3; return a; }";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 1);
	}

	TEST_F(CompilerTest, TestCompoundBitwiseAnd) {
		std::string source = "int main() { int a = 5; a &= 3; return a; }";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 1);
	}

	TEST_F(CompilerTest, TestCompoundBitwiseOr) {
		std::string source = "int main() { int a = 5; a |= 2; return a; }";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 7);
	}

	TEST_F(CompilerTest, TestCompoundBitwiseXor) {
		std::string source = "int main() { int a = 5; a ^= 3; return a; }";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 6);
	}

	TEST_F(CompilerTest, TestCompoundLeftShift) {
		std::string source = "int main() { int a = 5; a <<= 2; return a; }";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 20);
	}

	TEST_F(CompilerTest, TestCompoundRightShift) {
		std::string source = "int main() { int a = 20; a >>= 2; return a; }";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 5);
	}

	TEST_F(CompilerTest, TestCompoundAssignmentsAsExpressions) {
		std::string source = "int main() { int a = 5; int b = 2; return (a += 3) * (b -= 1); }";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 8);
	}

	TEST_F(CompilerTest, TestChainedCompoundAssignments) {
		std::string source = "int main() { int a = 0; int b = 2; int c = 3; a += b += c; return a; }";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 5);
	}

	TEST_F(CompilerTest, TestInvalidCompoundTargets) {
		std::string source = "int main() { int a = 5; (a + 2) += 3; return a; }";
		EXPECT_THROW(compile(source, ss), semantic_error);
	}

	// Mixed Complex Test Cases

	TEST_F(CompilerTest, TestPrefixWithCompoundAssignment) {
		std::string source = "int main() { int a = 1; return ++(a += 2); }";
		EXPECT_THROW(compile(source, ss), semantic_error);
	}

	TEST_F(CompilerTest, TestPostfixWithCompoundAssignment) {
		std::string source = "int main() { int a = 1; return (a += 2)++; }";
		EXPECT_THROW(compile(source, ss), semantic_error);
	}

	TEST_F(CompilerTest, TestPrefixInCompoundAssignment) {
		std::string source = "int main() { int a = 1; int b = 2; a += ++b; return a; }";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 4);
	}

	TEST_F(CompilerTest, TestPostfixInCompoundAssignment) {
		std::string source = "int main() { int a = 1; int b = 2; a += b++; return a; }";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 3);
	}

	TEST_F(CompilerTest, TestMultipleOperationsInOneStatement) {
		std::string source = R"(
int main() {
	int a = 1;
	return a = ++a + a++ + (a += 2);
})";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 10);  // Result depends on evaluation order
	}

	TEST_F(CompilerTest, TestOrderOfEvaluation) {
		std::string source = "int main() { int a = 1; int b = 1; return (a += b) += ++b; }";
		EXPECT_THROW(compile(source, ss), semantic_error);
	}

	TEST_F(CompilerTest, TestNestedPrefixOperators) {
		std::string source = "int main() { int a = 5; return ++(++a); }";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 7);
	}

	TEST_F(CompilerTest, TestUnaryPlusWithIncrement) {
		std::string source = "int main() { int a = 5; return +(+(++a)); }";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 6);
	}

	TEST_F(CompilerTest, TestInvalidUnaryPlusWithIncrement) {
		std::string source = "int main() { int a = 5; return (+a)++; }";
		EXPECT_THROW(compile(source, ss), semantic_error);
	}
	TEST_F(CompilerTest, TestIncrementOverflow) {
		std::string source = std::format("int main() {{ int a = {}; return ++a; }}", (std::numeric_limits<int>::max)());
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), (std::numeric_limits<int>::min)());
	}

	TEST_F(CompilerTest, TestCompoundAddOverflow) {
		std::string source = std::format("int main() {{ int a = {}; a += 1; return a; }}", (std::numeric_limits<int>::max)());
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), (std::numeric_limits<int>::min)());
	}

	TEST_F(CompilerTest, TestDecrementOverflow) {
		std::string source = std::format("int main() {{ int a = {}; return --a; }}", (std::numeric_limits<int>::min)());
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), (std::numeric_limits<int>::max)());
	}

	TEST_F(CompilerTest, TestCompoundSubtractOverflow) {
		std::string source = std::format("int main() {{ int a = {}; a -= 1; return a; }}", (std::numeric_limits<int>::min)());
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), (std::numeric_limits<int>::max)());
	}

	TEST_F(CompilerTest, TestPrefixAsLvalueForCompoundAssign) {
		std::string source = "int main() { int a = 5; return ++a += 2; }";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 8);
	}
}
