#pragma once

#include <cstdint>
#include <string>
#include <vector>
#include <unordered_map>
#include <functional>
#include <memory>

class X86Simulator {
public:
    X86Simulator();
    ~X86Simulator() = default;

    // Parse and load assembly code
    void loadProgram(const std::string& assembly);

    // Execute the loaded program
    int64_t execute(bool debug = false);

    // Reset simulator state
    void reset();

    // Get register value by name
    int64_t getRegister(const std::string& regName) const;

    // Set register value
    void setRegister(const std::string& regName, int64_t value);

    // Memory access for testing
    uint64_t readMemory(uint32_t address, int size) const;
    void writeMemory(uint32_t address, uint64_t value, int size);

private:
    struct OperandValue {
        int64_t value;
        bool isMemoryAddress;
    };

    // Register storage
    std::unordered_map<std::string, int64_t> registers;

    // Memory (1MB for simplicity)
    std::vector<uint8_t> memory;

    // Stack pointer initial position
    static constexpr uint32_t STACK_START = 0xF0000;

    // Program counter
    int pc;

    // Label map (label name -> instruction index)
    std::unordered_map<std::string, int> labels;

    // Parsed instructions
    std::vector<std::string> instructions;

    // Debug mode flag
    bool debug;

    // Map of instruction handlers
    std::unordered_map<std::string, std::function<void(const std::string&)>> instructionHandlers;

    // Parse memory operand like '8(%rbp)' or '(%rsp)'
    uint32_t parseMemoryAddress(const std::string& operand);

    // Parse operand (immediate, register, or memory)
    OperandValue parseOperand(const std::string& operand);

    // Sync 32/64 bit registers (when one changes, update the other)
    void syncRegisters(const std::string& reg);

    // Initialize instruction handlers
    void initInstructionHandlers();

    // Execute a single instruction
    void executeInstruction(const std::string& instruction);
};
