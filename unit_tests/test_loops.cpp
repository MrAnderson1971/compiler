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
}