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
