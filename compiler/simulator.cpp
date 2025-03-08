#include "simulator.hpp"
#include <iostream>
#include <sstream>
#include <regex>
#include <algorithm>
#include <stdexcept>

// Register names as constexpr string_view
namespace {
    constexpr std::string_view REG_RAX = "rax";
    constexpr std::string_view REG_RBX = "rbx";
    constexpr std::string_view REG_RCX = "rcx";
    constexpr std::string_view REG_RDX = "rdx";
    constexpr std::string_view REG_RSI = "rsi";
    constexpr std::string_view REG_RDI = "rdi";
    constexpr std::string_view REG_RBP = "rbp";
    constexpr std::string_view REG_RSP = "rsp";
    constexpr std::string_view REG_R8 = "r8";
    constexpr std::string_view REG_R9 = "r9";
    constexpr std::string_view REG_R10 = "r10";
    constexpr std::string_view REG_R11 = "r11";
    constexpr std::string_view REG_R12 = "r12";
    constexpr std::string_view REG_R13 = "r13";
    constexpr std::string_view REG_R14 = "r14";
    constexpr std::string_view REG_R15 = "r15";
    constexpr std::string_view REG_EAX = "eax";
    constexpr std::string_view REG_EBX = "ebx";
    constexpr std::string_view REG_ECX = "ecx";
    constexpr std::string_view REG_EDX = "edx";
    constexpr std::string_view REG_ESI = "esi";
    constexpr std::string_view REG_EDI = "edi";
    constexpr std::string_view REG_EBP = "ebp";
    constexpr std::string_view REG_ESP = "esp";
    constexpr std::string_view REG_R8D = "r8d";
    constexpr std::string_view REG_R9D = "r9d";
    constexpr std::string_view REG_R10D = "r10d";
    constexpr std::string_view REG_R11D = "r11d";
    constexpr std::string_view REG_R12D = "r12d";
    constexpr std::string_view REG_R13D = "r13d";
    constexpr std::string_view REG_R14D = "r14d";
    constexpr std::string_view REG_R15D = "r15d";
}

X86Simulator::X86Simulator() :
    memory(1024 * 1024, 0),
    pc(0),
    debug(false) {

    // Initialize registers to zero
    const std::vector<std::string_view> regNames = {
        REG_RAX, REG_RBX, REG_RCX, REG_RDX, REG_RSI, REG_RDI, REG_RBP, REG_RSP,
        REG_R8, REG_R9, REG_R10, REG_R11, REG_R12, REG_R13, REG_R14, REG_R15,
        REG_EAX, REG_EBX, REG_ECX, REG_EDX, REG_ESI, REG_EDI, REG_EBP, REG_ESP,
        REG_R8D, REG_R9D, REG_R10D, REG_R11D, REG_R12D, REG_R13D, REG_R14D, REG_R15D
    };

    for (const auto& reg : regNames) {
        registers[std::string(reg)] = 0;
    }

    // Initialize stack pointer
    registers[std::string(REG_RSP)] = static_cast<int64_t>(STACK_START);
    registers[std::string(REG_ESP)] = static_cast<int32_t>(STACK_START);

    // Initialize instruction handlers
    initInstructionHandlers();
}

void X86Simulator::reset() {
    // Clear state
    for (auto& reg : registers) {
        reg.second = 0;
    }

    registers[std::string(REG_RSP)] = static_cast<int64_t>(STACK_START);
    registers[std::string(REG_RBP)] = static_cast<int32_t>(STACK_START);

    std::fill(memory.begin(), memory.end(), 0);
    pc = 0;
    instructions.clear();
    labels.clear();
}

static std::vector<std::string> parseOperands(const std::string& args) {
    std::vector<std::string> result;
    std::string trimmedArgs = args;
    trimmedArgs.erase(0, trimmedArgs.find_first_not_of(" \t"));

    // Handle empty args
    if (trimmedArgs.empty()) {
        return result;
    }

    // Split by commas
    size_t start = 0;
    size_t end = 0;
    while ((end = trimmedArgs.find(',', start)) != std::string::npos) {
        std::string operand = trimmedArgs.substr(start, end - start);

        // Trim whitespace
        operand.erase(0, operand.find_first_not_of(" \t"));
        operand.erase(operand.find_last_not_of(" \t") + 1);

        result.push_back(operand);
        start = end + 1;
    }

    // Add the last operand
    std::string lastOperand = trimmedArgs.substr(start);
    lastOperand.erase(0, lastOperand.find_first_not_of(" \t"));
    lastOperand.erase(lastOperand.find_last_not_of(" \t") + 1);
    result.push_back(lastOperand);

    return result;
}

// Updated instruction handlers with improved argument parsing

void X86Simulator::initInstructionHandlers() {
    using Handler = std::function<void(const std::vector<std::string>&)>;

    std::unordered_map<std::string, Handler> handlers;

    handlers["movq"] = [this](const std::vector<std::string>& operands) {
        if (operands.size() != 2) {
            throw std::runtime_error("movq requires exactly 2 operands");
        }

        const std::string& src = operands[0];
        const std::string& dst = operands[1];

        auto srcVal = parseOperand(src);
        if (srcVal.isMemoryAddress) {
            srcVal.value = static_cast<int64_t>(readMemory(static_cast<uint32_t>(srcVal.value), 8));
        }

        if (!dst.empty() && dst[0] == '%') {
            // Register destination
            std::string regName = dst.substr(1);
            registers[regName] = srcVal.value;
            syncRegisters(regName);
        }
        else {
            // Memory destination
            uint32_t destAddr = parseMemoryAddress(dst);
            writeMemory(destAddr, static_cast<uint64_t>(srcVal.value), 8);
        }
        };

    handlers["movl"] = [this](const std::vector<std::string>& operands) {
        if (operands.size() != 2) {
            throw std::runtime_error("movl requires exactly 2 operands");
        }

        const std::string& src = operands[0];
        const std::string& dst = operands[1];

        auto srcVal = parseOperand(src);
        if (srcVal.isMemoryAddress) {
            srcVal.value = static_cast<int64_t>(readMemory(static_cast<uint32_t>(srcVal.value), 4));
        }

        // Truncate to 32 bits
        int32_t val32 = static_cast<int32_t>(srcVal.value);

        if (!dst.empty() && dst[0] == '%') {
            // Register destination
            std::string regName = dst.substr(1);
            registers[regName] = static_cast<int64_t>(val32);
            syncRegisters(regName);
        }
        else {
            // Memory destination
            uint32_t destAddr = parseMemoryAddress(dst);
            writeMemory(destAddr, static_cast<uint64_t>(val32), 4);
        }
        };

    handlers["subq"] = [this](const auto& operands) {
        if (operands.size() != 2) {
            throw std::runtime_error("subq requires exactly 2 operands");
        }

		const std::string& src = operands[0];
		const std::string& dst = operands[1];

		auto srcVal = parseOperand(src);
        if (srcVal.isMemoryAddress) {
            srcVal.value = static_cast<int64_t>(readMemory(static_cast<uint32_t>(srcVal.value), 8));
        }

        if (!dst.empty() && dst[0] == '%') {
			// Register destination
			std::string regName = dst.substr(1);
			registers[regName] -= srcVal.value;
			syncRegisters(regName);
		}
        else {
            // Memory destination
            uint32_t destAddr = parseMemoryAddress(dst);
            writeMemory(destAddr, readMemory(destAddr, 8) - srcVal.value, 8);
        }
        };

    handlers["addl"] = [this](const std::vector<std::string>& operands) {
        if (operands.size() != 2) {
            throw std::runtime_error("addl requires exactly 2 operands");
        }

        const std::string& src = operands[0];
        const std::string& dst = operands[1];

        auto srcVal = parseOperand(src);
        if (srcVal.isMemoryAddress) {
            srcVal.value = static_cast<int64_t>(readMemory(static_cast<uint32_t>(srcVal.value), 4));
        }

        // Truncate to 32 bits
        int32_t val32 = static_cast<int32_t>(srcVal.value);

        if (!dst.empty() && dst[0] == '%') {
            // Register destination
            std::string regName = dst.substr(1);
            int32_t dstVal = static_cast<int32_t>(registers[regName]);
            registers[regName] = static_cast<int64_t>(dstVal + val32);
            syncRegisters(regName);
        }
        else {
            // Memory destination
            uint32_t destAddr = parseMemoryAddress(dst);
            int32_t dstVal = static_cast<int32_t>(readMemory(destAddr, 4));
            writeMemory(destAddr, static_cast<uint64_t>(dstVal + val32), 4);
        }
        };

    handlers["negl"] = [this](const std::vector<std::string>& operands) {
        if (operands.size() != 1) {
            throw std::runtime_error("negl requires exactly 1 operand");
        }

        const std::string& operand = operands[0];

        if (!operand.empty() && operand[0] == '%') {
            // Register operand
            std::string regName = operand.substr(1);
            int32_t val = static_cast<int32_t>(registers[regName]);
            registers[regName] = static_cast<int64_t>(-val); // Maintain 32-bit behavior
            syncRegisters(regName);
        }
        else {
            // Memory operand
            uint32_t addr = parseMemoryAddress(operand);
            int32_t val = static_cast<int32_t>(readMemory(addr, 4));
            writeMemory(addr, static_cast<uint64_t>(-val), 4);
        }
        };

    handlers["notl"] = [this](const std::vector<std::string>& operands) {
        if (operands.size() != 1) {
            throw std::runtime_error("notl requires exactly 1 operand");
        }

        const std::string& operand = operands[0];

        if (!operand.empty() && operand[0] == '%') {
            // Register operand
            std::string regName = operand.substr(1);
            registers[regName] = static_cast<int64_t>(~static_cast<int32_t>(registers[regName]));
            syncRegisters(regName);
        }
        else {
            // Memory operand
            uint32_t addr = parseMemoryAddress(operand);
            int32_t val = static_cast<int32_t>(readMemory(addr, 4));
            writeMemory(addr, static_cast<uint64_t>(~val), 4);
        }
        };

    handlers["pushq"] = [this](const std::vector<std::string>& operands) {
        if (operands.size() != 1) {
            throw std::runtime_error("pushq requires exactly 1 operand");
        }

        const std::string& operand = operands[0];

        auto srcVal = parseOperand(operand);
        if (srcVal.isMemoryAddress) {
            srcVal.value = static_cast<int64_t>(readMemory(static_cast<uint32_t>(srcVal.value), 8));
        }

        // Decrement stack pointer first
        registers[std::string(REG_RSP)] -= 8;
        registers[std::string(REG_ESP)] = static_cast<int32_t>(registers[std::string(REG_RSP)]);

        // Write value to stack
        writeMemory(static_cast<uint32_t>(registers[std::string(REG_RSP)]),
            static_cast<uint64_t>(srcVal.value), 8);
        };

    handlers["popq"] = [this](const std::vector<std::string>& operands) {
        if (operands.size() != 1) {
            throw std::runtime_error("popq requires exactly 1 operand");
        }

        const std::string& operand = operands[0];

        // Read value from stack
        int64_t value = static_cast<int64_t>(
            readMemory(static_cast<uint32_t>(registers[std::string(REG_RSP)]), 8)
            );

        // Increment stack pointer
        registers[std::string(REG_RSP)] += 8;
        registers[std::string(REG_ESP)] = static_cast<int32_t>(registers[std::string(REG_RSP)]);

        if (!operand.empty() && operand[0] == '%') {
            // Register destination
            std::string regName = operand.substr(1);
            registers[regName] = value;
            syncRegisters(regName);
        }
        else {
            // Memory destination (rare, but possible)
            uint32_t destAddr = parseMemoryAddress(operand);
            writeMemory(destAddr, static_cast<uint64_t>(value), 8);
        }
        };

    handlers["ret"] = [this](const std::vector<std::string>&) {
        // Return just exits - we'll handle this in the execute() function
        pc = static_cast<int>(instructions.size()); // This will end execution
        };

    instructionHandlers = std::move(handlers);
}

// Improved register synchronization to handle r10d/r10 pairs correctly
void X86Simulator::syncRegisters(const std::string& reg) {
    // Handle r10d -> r10 case (extended registers like r8d through r15d)
    if (reg.length() > 2 && reg[0] == 'r' && reg[reg.length() - 1] == 'd') {
        std::string reg64 = reg.substr(0, reg.length() - 1);
        // Sign extend to 64 bits (not zero extend)
        registers[reg64] = static_cast<int64_t>(static_cast<int32_t>(registers[reg]));
        return;
    }

    // Handle eax -> rax case (standard registers)
    if (reg.length() == 3 && reg[0] == 'e') {
        std::string reg64 = "r" + reg.substr(1);
        // Sign extend to 64 bits (not zero extend)
        registers[reg64] = static_cast<int64_t>(static_cast<int32_t>(registers[reg]));
    }
    // Handle rax -> eax case
    else if (reg.length() == 3 && reg[0] == 'r') {
        std::string reg32 = "e" + reg.substr(1);
        if (registers.find(reg32) != registers.end()) {
            // Truncate to 32 bits
            registers[reg32] = static_cast<int64_t>(static_cast<int32_t>(registers[reg]));
        }
    }
    // Handle r10 -> r10d case (for extended registers)
    else if (reg.length() > 1 && reg[0] == 'r' && std::isdigit(reg[1])) {
        std::string reg32 = reg + "d";
        if (registers.find(reg32) != registers.end()) {
            // Truncate to 32 bits
            registers[reg32] = static_cast<int64_t>(static_cast<int32_t>(registers[reg]));
        }
    }
}

// Improved memory address parsing for AT&T syntax
uint32_t X86Simulator::parseMemoryAddress(const std::string& operand) {
    std::regex memPattern(R"((-?\d*)\(%([a-zA-Z0-9]+)\))");
    std::smatch match;

    if (std::regex_match(operand, match, memPattern)) {
        int32_t offset = match[1].length() > 0 ? std::stoi(match[1].str()) : 0;
        std::string reg = match[2].str();
        return static_cast<uint32_t>(registers[reg] + static_cast<int64_t>(offset));
    }

    throw std::runtime_error("Invalid memory operand: " + operand);
}

void X86Simulator::loadProgram(const std::string& assembly) {
    // Clear previous state
    instructions.clear();
    labels.clear();

    std::istringstream iss(assembly);
    std::string line;

    // First pass: collect labels
    int lineNum = 0;
    while (std::getline(iss, line)) {
        // Trim whitespace
        line.erase(0, line.find_first_not_of(" \t"));
        if (line.empty()) continue;
        line.erase(line.find_last_not_of(" \t") + 1);

        // Skip empty lines and directives
        if (line.empty() || line[0] == '.') {
            continue;
        }

        // Check if line contains a label
        size_t labelEnd = line.find(':');
        if (labelEnd != std::string::npos) {
            std::string labelName = line.substr(0, labelEnd);
            // Trim whitespace from label
            labelName.erase(0, labelName.find_first_not_of(" \t"));
            labelName.erase(labelName.find_last_not_of(" \t") + 1);

            labels[labelName] = lineNum;

            // Check if there's an instruction after the label
            if (line.length() > labelEnd + 1) {
                std::string instruction = line.substr(labelEnd + 1);
                instruction.erase(0, instruction.find_first_not_of(" \t"));

                if (!instruction.empty()) {
                    instructions.push_back(instruction);
                    lineNum++;
                }
            }
        }
        else {
            // Regular instruction
            instructions.push_back(line);
            lineNum++;
        }
    }

    if (debug) {
        std::cout << "Loaded " << instructions.size() << " instructions\n";
        for (const auto& [label, index] : labels) {
            std::cout << "Label '" << label << "' at index " << index << "\n";
        }
    }
}

int64_t X86Simulator::execute(bool debugMode) {
    debug = debugMode;
    pc = 0;

    // Set stack pointer to initial value
    registers[std::string(REG_RSP)] = static_cast<int64_t>(STACK_START);
    registers[std::string(REG_RBP)] = static_cast<int32_t>(STACK_START);

    while (pc < static_cast<int>(instructions.size())) {
        if (debug) {
            std::cout << "Executing: " << instructions[pc] << std::endl;
        }

        executeInstruction(instructions[pc]);
        pc++;
    }

    // Return value is in RAX
    return registers[std::string(REG_RAX)];
}

void X86Simulator::executeInstruction(const std::string& instruction) {
    std::istringstream iss(instruction);
    std::string opcode;
    iss >> opcode;

    // Get the rest of the instruction (arguments)
    std::string argsStr;
    std::getline(iss, argsStr);
    if (!argsStr.empty()) {
        argsStr.erase(0, argsStr.find_first_not_of(" \t"));
    }

    // Parse operands
    std::vector<std::string> operands = parseOperands(argsStr);

    // Find handler for opcode
    auto it = instructionHandlers.find(opcode);
    if (it != instructionHandlers.end()) {
        it->second(operands);
    }
    else {
        throw std::runtime_error("Unsupported instruction: " + opcode);
    }
}

X86Simulator::OperandValue X86Simulator::parseOperand(const std::string& operand) {
    if (operand.empty()) {
        return { 0, false };
    }

    if (operand[0] == '$') {
        // Immediate value
        return { static_cast<int64_t>(std::stoll(operand.substr(1))), false };
    }
    else if (operand[0] == '%') {
        // Register
        std::string reg = operand.substr(1);
        return { registers[reg], false };
    }
    else {
        // Memory operand
        uint32_t addr = parseMemoryAddress(operand);
        return { static_cast<int64_t>(addr), true }; // Return address + flag indicating memory
    }
}

int64_t X86Simulator::getRegister(const std::string& regName) const {
    auto it = registers.find(regName);
    if (it != registers.end()) {
        return it->second;
    }
    throw std::runtime_error("Invalid register name: " + regName);
}

void X86Simulator::setRegister(const std::string& regName, int64_t value) {
    auto it = registers.find(regName);
    if (it != registers.end()) {
        it->second = value;
        syncRegisters(regName);
    }
    else {
        throw std::runtime_error("Invalid register name: " + regName);
    }
}

uint64_t X86Simulator::readMemory(uint32_t address, int size) const {
    if (address + static_cast<uint32_t>(size) > memory.size()) {
        throw std::runtime_error("Memory access out of bounds: " + std::to_string(address));
    }

    uint64_t result = 0;
    for (int i = 0; i < size; i++) {
        result |= static_cast<uint64_t>(memory[address + static_cast<uint32_t>(i)]) << (i * 8);
    }
    return result;
}

void X86Simulator::writeMemory(uint32_t address, uint64_t value, int size) {
    if (address + static_cast<uint32_t>(size) > memory.size()) {
        throw std::runtime_error("Memory access out of bounds: " + std::to_string(address));
    }

    for (int i = 0; i < size; i++) {
        memory[address + static_cast<uint32_t>(i)] =
            static_cast<uint8_t>((value >> (i * 8)) & 0xFF);
    }
}