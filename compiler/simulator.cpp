#include "simulator.hpp"
#include <windows.h>
#include <fstream>
#include <stdexcept>
#include <string>
#include <iostream>

Simulator::Simulator() {
    // Create unique filenames using process ID
    DWORD pid = GetCurrentProcessId();
    char tempPath[MAX_PATH];
    GetTempPathA(MAX_PATH, tempPath);

    tempAsmFile = std::string(tempPath) + "asm_" + std::to_string(pid) + ".s";
    tempObjFile = std::string(tempPath) + "asm_" + std::to_string(pid) + ".o";
    tempDllFile = std::string(tempPath) + "asm_" + std::to_string(pid) + ".dll";
}

Simulator::~Simulator() {
    // Clean up temporary files
    std::remove(tempAsmFile.c_str());
    std::remove(tempObjFile.c_str());
    std::remove(tempDllFile.c_str());

    // Free the library if it's loaded
    if (dllHandle != NULL) {
        FreeLibrary(dllHandle);
        dllHandle = NULL;
    }
}

void Simulator::loadProgram(const std::string& asmCode) {
    // Write the original assembly code to a file
    std::ofstream asmFile(tempAsmFile);
    if (!asmFile) {
        throw std::runtime_error("Failed to create assembly file");
    }

    // For debugging - print what we're compiling
#ifdef _DEBUG
    std::cout << "Compiling assembly code:\n" << asmCode << std::endl;
#endif

    // Write the assembly code directly, renaming main to _runAsm for Windows
    std::string modifiedCode = asmCode;
    size_t pos = modifiedCode.find(".global main");
    if (pos != std::string::npos) {
        modifiedCode.replace(pos, 12, ".global _runAsm");
    }

    pos = modifiedCode.find("main:");
    if (pos != std::string::npos) {
        modifiedCode.replace(pos, 5, "_runAsm:");
    }

    asmFile << modifiedCode;
    asmFile.close();

    // Compile assembly to object file
    std::string compileCmd = "gcc -c \"" + tempAsmFile + "\" -o \"" + tempObjFile + "\"";
    std::cout << "Running compile command: " << compileCmd << std::endl;
    int compileResult = system(compileCmd.c_str());
    if (compileResult != 0) {
        throw std::runtime_error("Failed to compile assembly: " + std::to_string(compileResult));
    }

    // Link object file to create DLL
    std::string linkCmd = "gcc -shared \"" + tempObjFile + "\" -o \"" + tempDllFile + "\" -Wl,--export-all-symbols";
    std::cout << "Running link command: " << linkCmd << std::endl;
    int linkResult = system(linkCmd.c_str());
    if (linkResult != 0) {
        throw std::runtime_error("Failed to create DLL: " + std::to_string(linkResult));
    }

    std::cout << "Successfully compiled and linked assembly" << std::endl;
}

int Simulator::execute() {
    // Load the DLL
    dllHandle = LoadLibraryA(tempDllFile.c_str());
    if (dllHandle == NULL) {
        throw std::runtime_error("Failed to load DLL: " + std::to_string(GetLastError()));
    }

    // Get the function pointer
    typedef int64_t(*AsmFunction)();
    AsmFunction runAsm = (AsmFunction)GetProcAddress(dllHandle, "_runAsm");

    if (runAsm == NULL) {
        // Try without underscore as fallback
        runAsm = (AsmFunction)GetProcAddress(dllHandle, "runAsm");

        if (runAsm == NULL) {
            FreeLibrary(dllHandle);
            dllHandle = NULL;
            throw std::runtime_error("Failed to get function address: " + std::to_string(GetLastError()));
        }
    }

#ifdef _DEBUG
    // Execute the function
    std::cout << "Executing assembly function..." << std::endl;
#endif
    int64_t result = runAsm();
#ifdef _DEBUG
    std::cout << "Assembly function returned: " << result << std::endl;
#endif

    // Cleanup
    FreeLibrary(dllHandle);
    dllHandle = NULL;

    return static_cast<int>(result);
}