#include "simulator.hpp"
#include "compiler.hpp"

TEST_F(CompilerTest, TestSuccess) {
	std::string source = R"(
int main() {
	return 2;
}
)";
	compile(source, ss);
	simulator.loadProgram(ss.str());
	int64_t output = simulator.execute();
	EXPECT_EQ(2, output);
}

TEST_F(CompilerTest, TestMissingClosingBrace) {
	std::string source = R"(
int main() {
return 0;
)";
	EXPECT_THROW(compile(source, ss), compiler_error);
}

TEST_F(CompilerTest, TestMissingOpeningBrace) {
	std::string source = R"(
int main()
return 0;
})";
	EXPECT_THROW(compile(source, ss), compiler_error);
}

TEST_F(CompilerTest, TestMissingMainFunction) {
	std::string source = R"(
int () {
return 0;
})";
	EXPECT_THROW(compile(source, ss), compiler_error);
}

TEST_F(CompilerTest, TestMissingReturnStatement) {
	std::string source = R"(
int main() {
	0;
})";
	EXPECT_THROW(compile(source, ss), compiler_error);
}

TEST_F(CompilerTest, TestInvalidReturnStatement) {
	std::string source = R"(
int main() {
	return ;
})";
	EXPECT_THROW(compile(source, ss), compiler_error);
}

TEST_F(CompilerTest, TestMissingSemicolon) {
	std::string source = R"(
int main() {
	return 0
})";
	EXPECT_THROW(compile(source, ss), compiler_error);
}

TEST_F(CompilerTest, TestMissingSpace) {
	std::string source = R"(
int main() {
	return0;
})";
	EXPECT_THROW(compile(source, ss), compiler_error);
}

int main(int argc, char** argv) {
	testing::InitGoogleTest(&argc, argv);
	return RUN_ALL_TESTS();
}
