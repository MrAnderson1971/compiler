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

	TEST_F(CompilerTest, TestFor) {
		std::string source = R"(
int main() {
    int result = 0;
    for (int i = 0; i <= 10; i++) {
        result += i;
    }
    return result;
}
		)";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 55);
	}

	TEST_F(CompilerTest, TestForWithNonDeclarationInit) {
		std::string source = R"(
int main() {
	int result = 0;
	int i;
	for (i = 0; i <= 10; i++) {
		result += i;
	}
	return result;
	})";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 55);
	}

	TEST_F(CompilerTest, TestContinueInFor) {
		std::string source = R"(
int main() {
    int result = 0;
    for (int i = 0; i <= 10; i++) {
        if (i % 2 == 1) {
            continue;
        }
        result += i;
    }
    return result;
})";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 30);
	}

	TEST_F(CompilerTest, TestBreakInFor) {
		std::string source = R"(
int main() {
	int result = 0;
	for (int i = 0; i <= 10; i++) {
		if (i >= 5) {
			break;
		}
		result += i;
	}
	return result;})";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 10);
	}

	TEST_F(CompilerTest, TestForInitProperScope) {
		std::string source = R"(
int main() {
    int i = -100;
    for (int i = 69420;;) {
        return i;
    }
    return i;
})";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), 69420);
	}

	TEST_F(CompilerTest, TestForInitProperScope2) {
		std::string source = R"(int main() {
    int i = -100;
    for (int i = 69420; i < 69420 + 10; i++);
    return i;
})";
		compile(source, ss);
		simulator.loadProgram(ss.str());
		EXPECT_EQ(simulator.execute(), -100);
	}

    // Test nested while loops
    TEST_F(CompilerTest, TestNestedWhileLoops) {
        std::string code = R"(
            int main() {
                int i = 0;
                int j = 0;
                int sum = 0;
                
                while (i < 3) {
                    j = 0;
                    while (j < 4) {
                        sum += i * j;
                        j++;
                    }
                    i++;
                }
                return sum;
            }
        )";
        compile(code, ss);
        simulator.loadProgram(ss.str());
        EXPECT_EQ(simulator.execute(), 18); // Sum: 0*0 + 0*1 + 0*2 + 0*3 + 1*0 + 1*1 + 1*2 + 1*3 + 2*0 + 2*1 + 2*2 + 2*3 = 18
    }

    // Test while loop with initially false condition
    TEST_F(CompilerTest, TestWhileWithInitiallyFalseCondition) {
        std::string code = R"(
            int main() {
                int i = 10;
                while (i < 10) {
                    i++;
                }
                return i;
            }
        )";
        compile(code, ss);
        simulator.loadProgram(ss.str());
        EXPECT_EQ(simulator.execute(), 10); // Loop should never execute
    }

    // Test while with complex condition
    TEST_F(CompilerTest, TestWhileWithComplexCondition) {
        std::string code = R"(
            int main() {
                int i = 0;
                int j = 10;
                while (i < 5 && j > 5) {
                    i++;
                    j--;
                }
                return i * 100 + j;
            }
        )";
        compile(code, ss);
        simulator.loadProgram(ss.str());
        EXPECT_EQ(simulator.execute(), 505); // i = 5, j = 5, result = 5*100 + 5 = 505
    }

    // Test do-while loop
    //TEST_F(CompilerTest, TestDoWhile) {
    //    std::string code = R"(
    //        int main() {
    //            int i = 10;
    //            do {
    //                i++;
    //            } while (i < 10);
    //            return i;
    //        }
    //    )";
    //    compile(code, ss);
    //    simulator.loadProgram(ss.str());
    //    EXPECT_EQ(simulator.execute(), 11); // Loop should execute once even though condition is false initially
    //}

    // Test break in nested loops
    TEST_F(CompilerTest, TestBreakInNestedLoops) {
        std::string code = R"(
            int main() {
                int i = 0;
                int j = 0;
                int sum = 0;
                
                while (i < 5) {
                    j = 0;
                    while (j < 5) {
                        sum++;
                        if (j == 2) {
                            break; // Should only break inner loop
                        }
                        j++;
                    }
                    if (i == 3) {
                        break; // Should break outer loop
                    }
                    i++;
                }
                return sum;
            }
        )";
        compile(code, ss);
        simulator.loadProgram(ss.str());
        EXPECT_EQ(simulator.execute(), 12);
    }

    // Test continue in nested loops
    TEST_F(CompilerTest, TestContinueInNestedLoops) {
        std::string code = R"(
            int main() {
                int i = 0;
                int sum = 0;
                
                while (i < 3) {
                    i++;
                    if (i == 2) {
                        continue; // Skip when i == 2
                    }
                    
                    int j = 0;
                    while (j < 3) {
                        j++;
                        if (j == 2) {
                            continue; // Skip when j == 2
                        }
                        sum += i * j;
                    }
                }
                return sum;
            }
        )";
        compile(code, ss);
        simulator.loadProgram(ss.str());
        EXPECT_EQ(simulator.execute(), 16);
    }

    // Test for loop with all empty parts (infinite loop with break)
    TEST_F(CompilerTest, TestForWithAllPartsEmpty) {
        std::string code = R"(
            int main() {
                int i = 0;
                for (;;) {
                    i++;
                    if (i >= 10) {
                        break;
                    }
                }
                return i;
            }
        )";
        compile(code, ss);
        simulator.loadProgram(ss.str());
        EXPECT_EQ(simulator.execute(), 10);
    }

    // Test for loop with empty condition (treated as true)
    TEST_F(CompilerTest, TestForWithEmptyCondition) {
        std::string code = R"(
            int main() {
                int i = 0;
                for (i = 0; ; i++) {
                    if (i >= 10) {
                        break;
                    }
                }
                return i;
            }
        )";
        compile(code, ss);
        simulator.loadProgram(ss.str());
        EXPECT_EQ(simulator.execute(), 10);
    }

    // Test for loop with empty update
    TEST_F(CompilerTest, TestForWithEmptyUpdate) {
        std::string code = R"(
            int main() {
                int i = 0;
                for (i = 0; i < 10;) {
                    i += 2; // Update inside the loop
                }
                return i;
            }
        )";
        compile(code, ss);
        simulator.loadProgram(ss.str());
        EXPECT_EQ(simulator.execute(), 10);
    }

    // Test for with initially false condition
    TEST_F(CompilerTest, TestForWithInitiallyFalseCondition) {
        std::string code = R"(
            int main() {
                int counter = 0;
                for (int i = 10; i < 10; i++) {
                    counter++;
                }
                return counter;
            }
        )";
        compile(code, ss);
        simulator.loadProgram(ss.str());
        EXPECT_EQ(simulator.execute(), 0); // Loop should never execute
    }

    // Test nested for loops
    TEST_F(CompilerTest, TestNestedForLoops) {
        std::string code = R"(
            int main() {
                int sum = 0;
                for (int i = 0; i < 3; i++) {
                    for (int j = 0; j < 3; j++) {
                        sum += i * j;
                    }
                }
                return sum;
            }
        )";
        compile(code, ss);
        simulator.loadProgram(ss.str());
        EXPECT_EQ(simulator.execute(), 9); // 0*0 + 0*1 + 0*2 + 1*0 + 1*1 + 1*2 + 2*0 + 2*1 + 2*2 = 9
    }

    // Test for with complex update expression
    TEST_F(CompilerTest, TestForWithComplexUpdate) {
        std::string code = R"(
            int main() {
                int sum = 0;
                for (int i = 0; i < 10; i += 2) {
                    sum += i;
                }
                return sum;
            }
        )";
        compile(code, ss);
        simulator.loadProgram(ss.str());
        EXPECT_EQ(simulator.execute(), 20); // 0 + 2 + 4 + 6 + 8 = 20
    }

    // Test access to loop variables after loop execution
    TEST_F(CompilerTest, TestLoopVariableAccessAfterExecution) {
        std::string code = R"(
            int main() {
                int sum = 0;
                int i;
                for (i = 0; i < 5; i++) {
                    sum += i;
                }
                return i * 10 + sum;
            }
        )";
        compile(code, ss);
        simulator.loadProgram(ss.str());
        EXPECT_EQ(simulator.execute(), 60); // i = 5, sum = 0+1+2+3+4 = 10, result = 5*10 + 10 = 60
    }
}
