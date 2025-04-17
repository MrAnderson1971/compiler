// compiler.cpp : This file contains the 'main' function. Program execution begins and ends there.
//

#include <iostream>
#include <filesystem>
#include <fstream>
#include "compiler.hpp"
#include "exceptions.hpp"

int main(int argc, char* argv[]) {
	if (argc < 2) {
		std::cerr << "Usage: " << argv[0] << " <input file>\n";
		return 1;
	}

	std::filesystem::path inputFile = argv[1];
	if (!exists(inputFile)) {
		std::cerr << "File not found: " << inputFile << "\n";
		return 1;
	}

	std::string source;
	{
		std::ifstream istream(inputFile);
		source = std::string(std::istreambuf_iterator<char>(istream), std::istreambuf_iterator<char>());
	}
	try {
		std::filesystem::path outputFile = inputFile;
		outputFile.replace_extension(".asm");
		std::ofstream ostream(outputFile);
		compile(source, ostream);
	}
	catch (const compiler_error& e) {
		std::cerr << e.what() << "\n";
	}

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
