#include "simulator.hpp"
#include <windows.h>
#include <fstream>
#include <stdexcept>
#include <string>
#include <iostream>
#include "type.hpp"

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

void Simulator::loadProgram(const std::string& asmCode) const {
    std::string cleanedCode;

    std::cout << "Compiling assembly code:\n" << asmCode << std::endl;

    // Clean the code if in debug mode
    if constexpr (DEBUG) {
        std::istringstream stream(asmCode);
        std::string line;

        while (std::getline(stream, line)) {
            // Skip blank lines or comments
            if (line.empty() || line.find_first_not_of(" \t") == std::string::npos || line.find(';') != std::string::npos) {
                continue;
            }

            cleanedCode += line + "\n";
        }
    } else {
        cleanedCode = asmCode;
    }

    // Write the assembly code to a temporary file
    std::ofstream asmFile(tempAsmFile);
    if (!asmFile) {
        throw std::runtime_error("Failed to create assembly file");
    }

    // Write the assembly code, renaming main to _runAsm for Windows
    std::string modifiedCode = cleanedCode;
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

    std::cout << "Wrote assembly to temporary file: " << tempAsmFile << std::endl;

    // Helper function to execute command and capture output
    auto executeCommand = [](const std::string& cmd) -> std::pair<int, std::string> {
        std::string output;
        FILE* pipe = _popen((cmd + " 2>&1").c_str(), "r");
        if (!pipe) {
            return { -1, "Failed to execute command" };
        }

        char buffer[256];
        while (!feof(pipe)) {
            if (fgets(buffer, sizeof(buffer), pipe) != NULL) {
                output += buffer;
            }
        }

        int status = _pclose(pipe);
        return { status, output };
        };

    // Compile assembly to object file with verbose output
    std::string compileCmd = "gcc -v -c \"" + tempAsmFile + "\" -o \"" + tempObjFile + "\"";
    std::cout << "Running compile command: " << compileCmd << std::endl;

    auto [compileStatus, compileOutput] = executeCommand(compileCmd);
    std::cout << "Compilation output: " << compileOutput << std::endl;

    if (compileStatus != 0) {
        // Create a more detailed error message
        std::ostringstream errorMsg;
        errorMsg << "Failed to compile assembly (status code: " << compileStatus << ")" << std::endl;
        errorMsg << "Command: " << compileCmd << std::endl;
        errorMsg << "Output: " << compileOutput << std::endl;

        // Get temp path for debug file
        char tempPathBuffer[MAX_PATH];
        GetTempPathA(MAX_PATH, tempPathBuffer);

        // Save the assembly file for debugging (in case it gets deleted)
        std::string debugFileName = std::string(tempPathBuffer) + "asm_debug_" + std::to_string(GetCurrentProcessId()) + ".s";
        std::ofstream debugFile(debugFileName);
        if (debugFile) {
            debugFile << modifiedCode;
            debugFile.close();
            errorMsg << "Assembly code saved to: " << debugFileName << std::endl;
        }

        throw std::runtime_error(errorMsg.str());
    }

    // Link object file to create DLL with verbose output
    std::string linkCmd = "gcc -v -shared \"" + tempObjFile + "\" -o \"" + tempDllFile + "\" -Wl,--export-all-symbols";
    std::cout << "Running link command: " << linkCmd << std::endl;

    auto [linkStatus, linkOutput] = executeCommand(linkCmd);
    std::cout << "Linking output: " << linkOutput << std::endl;

    if (linkStatus != 0) {
        // Create a more detailed error message
        std::ostringstream errorMsg;
        errorMsg << "Failed to create DLL (status code: " << linkStatus << ")" << std::endl;
        errorMsg << "Command: " << linkCmd << std::endl;
        errorMsg << "Output: " << linkOutput << std::endl;

        throw std::runtime_error(errorMsg.str());
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