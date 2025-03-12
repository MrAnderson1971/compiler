#pragma once
#include <windows.h>
#include <string>
#include <gtest/gtest.h>

class Simulator {
private:
    std::string tempAsmFile;
    std::string tempObjFile;
    std::string tempDllFile;
    HMODULE dllHandle = NULL;

public:
    Simulator();
    ~Simulator();

    void loadProgram(const std::string& asmCode);
    int execute();
};

class CompilerTest : public ::testing::Test {
protected:
    Simulator simulator;
    std::stringstream ss;

    void SetUp() override {}
};
