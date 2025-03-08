// compiler.cpp : This file contains the 'main' function. Program execution begins and ends there.
//

#include <iostream>
#include <filesystem>
#include <fstream>
#include "compiler.hpp"

// https://norasandler.com/2017/11/29/Write-a-Compiler.html

constexpr bool FILE_TESTS = false;

int main()
{
	//if constexpr (FILE_TESTS) {
	//	std::filesystem::path tests = std::filesystem::current_path() / "tests";

	//	for (const auto& file : std::filesystem::directory_iterator(tests)) {
	//		if (file.is_regular_file()) {
	//			std::string source;
	//			{
	//				std::ifstream fs(file.path());
	//				source = std::string((std::istreambuf_iterator<char>(fs)), std::istreambuf_iterator<char>());
	//			}
	//			std::cout << file.path().filename() << "\n" << source << std::endl;
	//			try {
	//				compile(source);
	//			}
	//			catch (const std::exception& e) {
	//				std::cerr << e.what() << std::endl;
	//			}
	//		}
	//	}
	//}
	//else {
	//	std::string source = R"(int main() { 
	//		return ~-2;
	//})";
	//	try {
	//		compile(source);
	//	}
	//	catch (const compiler_error& e) {
	//		std::cerr << e.what() << std::endl;
	//	}
	//}

    return 0;
}

// Run program: Ctrl + F5 or Debug > Start Without Debugging menu
// Debug program: F5 or Debug > Start Debugging menu

// Tips for Getting Started: 
//   1. Use the Solution Explorer window to add/manage files
//   2. Use the Team Explorer window to connect to source control
//   3. Use the Output window to see build output and other messages
//   4. Use the Error List window to view errors
//   5. Go to Project > Add New Item to create new code files, or Project > Add Existing Item to add existing code files to the project
//   6. In the future, to open this project again, go to File > Open > Project and select the .sln file
