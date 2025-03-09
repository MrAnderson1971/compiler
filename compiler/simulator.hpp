#pragma once

#include <string>
#include <sstream>
#include <stdexcept>
#include <gtest/gtest.h>

// Include Keystone headers
#include <keystone/keystone.h>

// Include Unicorn headers
#include <unicorn/unicorn.h>

// Simulator class that can execute AT&T x86 assembly
class Simulator {
private:
    // Memory addresses for emulation
    static constexpr uint64_t BASE_ADDRESS = 0x1000000;
    static constexpr uint64_t STACK_ADDRESS = 0x2000000;
    static constexpr size_t STACK_SIZE = 0x20000;  // 128 KB
    static constexpr size_t CODE_SIZE = 0x10000;   // 64 KB

    std::string program;
    bool programLoaded = false;

public:
    // Load an assembly program (AT&T syntax)
    void loadProgram(const std::string& assemblyCode);

    // Execute the loaded program and return the value in RAX/EAX
    int64_t execute() const;
};

// Google Test fixture for compiler tests
class CompilerTest : public ::testing::Test {
protected:
    Simulator simulator;
    std::stringstream ss;

    void SetUp() override {
        // Reset stringstream before each test
        ss.str("");
        ss.clear();
    }

    void TearDown() override {
        // Clean up after each test if needed
    }
};
