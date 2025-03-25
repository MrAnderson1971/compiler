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

    TEST_F(CompilerTest, TestNestedIf) {
        std::string source = R"(
int main() {
    int a = 1;
    int b = 2;
    if (a < b)
        if (a > 0) return 10;
        else return 20;
    return 30;
})";
        compile(source, ss);
        simulator.loadProgram(ss.str());
        EXPECT_EQ(simulator.execute(), 10);
    }

    TEST_F(CompilerTest, TestNestedIfElse) {
        std::string source = R"(
int main() {
    int a = 1;
    int b = 2;
    if (a > b)
        return 10;
    else
        if (a > 0) return 20;
        else return 30;
})";
        compile(source, ss);
        simulator.loadProgram(ss.str());
        EXPECT_EQ(simulator.execute(), 20);
    }

    TEST_F(CompilerTest, TestIfElseIfNoFinalElse) {
        std::string source = R"(
int main() {
    int a = 1;
    if (a > 2) return 10;
    else if (a > 0) return 20;
    return 30;
})";
        compile(source, ss);
        simulator.loadProgram(ss.str());
        EXPECT_EQ(simulator.execute(), 20);
    }

    TEST_F(CompilerTest, TestLogicalAndInCondition) {
        std::string source = R"(
int main() {
    int a = 1;
    int b = 2;
    if (a > 0 && b > 0) return 10;
    return 20;
})";
        compile(source, ss);
        simulator.loadProgram(ss.str());
        EXPECT_EQ(simulator.execute(), 10);
    }

    TEST_F(CompilerTest, TestLogicalOrInCondition) {
        std::string source = R"(
int main() {
    int a = 0;
    int b = 2;
    if (a > 0 || b > 0) return 10;
    return 20;
})";
        compile(source, ss);
        simulator.loadProgram(ss.str());
        EXPECT_EQ(simulator.execute(), 10);
    }

    TEST_F(CompilerTest, TestLogicalNotInCondition) {
        std::string source = R"(
int main() {
    int a = 0;
    if (!a) return 10;
    return 20;
})";
        compile(source, ss);
        simulator.loadProgram(ss.str());
        EXPECT_EQ(simulator.execute(), 10);
    }

    TEST_F(CompilerTest, TestNestedTernary) {
        std::string source = R"(
int main() {
    int a = 1;
    int b = 2;
    int c = 3;
    return a > b ? a : (b > c ? b : c);
})";
        compile(source, ss);
        simulator.loadProgram(ss.str());
        EXPECT_EQ(simulator.execute(), 3);
    }

    TEST_F(CompilerTest, TestIfWithAssignment) {
        std::string source = R"(
int main() {
    int a = 0;
    if (a < 1) a = 10;
    return a;
})";
        compile(source, ss);
        simulator.loadProgram(ss.str());
        EXPECT_EQ(simulator.execute(), 10);
    }

    // Invalid test cases

    TEST_F(CompilerTest, TestMissingParenthesesInIf) {
        std::string source = R"(
int main() {
    if 1 > 0 return 10;
})";
        EXPECT_THROW(compile(source, ss), syntax_error);
    }

    TEST_F(CompilerTest, TestDoubleElse) {
        std::string source = R"(
int main() {
    if (1 > 0) return 10;
    else return 20;
    else return 30;
})";
        EXPECT_THROW(compile(source, ss), syntax_error);
    }

    TEST_F(CompilerTest, TestNestedIfWithoutStatement) {
        std::string source = R"(
int main() {
    if (1 > 0)
        if (1 > 2)
    return 10;
})";
        compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 0);
    }

    TEST_F(CompilerTest, TestElseIfWithoutCondition) {
        std::string source = R"(
int main() {
    if (1 > 0) return 10;
    else if return 20;
})";
        EXPECT_THROW(compile(source, ss), syntax_error);
    }

    TEST_F(CompilerTest, TestMissingSemicolonInIf) {
        std::string source = R"(
int main() {
    int a = 0;
    if (1 > 0) a = 10
    return a;
})";
        EXPECT_THROW(compile(source, ss), syntax_error);
    }

    TEST_F(CompilerTest, TestIfWithMultipleStatements) {
        std::string source = R"(
int main() {
    if (1 > 0) int a = 10; return a;
})";
        EXPECT_THROW(compile(source, ss), syntax_error);
    }

    TEST_F(CompilerTest, TestTernaryInCondition) {
        std::string source = R"(
int main() {
    int a = 1;
    int b = 2;
    if (a < b ? 1 : 0) return 10;
    return 20;
})";
        compile(source, ss);
        simulator.loadProgram(ss.str());
        EXPECT_EQ(simulator.execute(), 10);
    }

    TEST_F(CompilerTest, TestChainedElseIf) {
        std::string source = R"(
int main() {
    int a = 2;
    if (a > 3) return 10;
    else if (a > 2) return 20;
    else if (a > 1) return 30;
    else if (a > 0) return 40;
    else return 50;
})";
        compile(source, ss);
        simulator.loadProgram(ss.str());
        EXPECT_EQ(simulator.execute(), 30);
    }
}