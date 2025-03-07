#include <gtest/gtest.h>
#include "simulator.hpp"
#include "compiler.hpp"

TEST(BasicCompile, TestSuccess) {
	std::stringstream ss;
	std::string source = R"(
int main() {
	return 2;
}
)";
	compile(source, ss);
	X86Simulator simulator;
	simulator.loadProgram(ss.str());
	int64_t output = simulator.execute();
	EXPECT_EQ(2, output);
}

int main(int argc, char** argv) {
	testing::InitGoogleTest(&argc, argv);
	return RUN_ALL_TESTS();
}
