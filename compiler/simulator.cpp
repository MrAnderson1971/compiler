#include "simulator.hpp"

// Memory addresses for emulation
static constexpr uint64_t BASE_ADDRESS = 0x1000;
static constexpr uint64_t STACK_ADDRESS = 0x2000000;
static constexpr size_t STACK_SIZE = 0x20000;  // 128 KB
static constexpr size_t CODE_SIZE = 0x4000;   // 64 KB

void Simulator::loadProgram(const std::string& assemblyCode) {
    program = assemblyCode;
    programLoaded = true;
}

int64_t Simulator::execute() const {
    if (!programLoaded) {
        throw std::runtime_error("No program loaded");
    }
    // Assemble the code using Keystone
    ks_engine* ks;
    ks_err err = ks_open(KS_ARCH_X86, KS_MODE_64, &ks);
    if (err != KS_ERR_OK) {
        throw std::runtime_error("Failed to initialize Keystone assembler");
    }
    // Set AT&T syntax
    ks_option(ks, KS_OPT_SYNTAX, KS_OPT_SYNTAX_ATT);
    unsigned char* encode;
    size_t size;
    size_t count;
    if (ks_asm(ks, program.c_str(), BASE_ADDRESS, &encode, &size, &count) != KS_ERR_OK) {
        std::string errorMsg = "Failed to assemble code: " + std::string(ks_strerror(ks_errno(ks)));
        ks_close(ks);
        throw std::runtime_error(errorMsg);
    }
    // Initialize Unicorn emulator
    uc_engine* uc;
    uc_err uc_err = uc_open(UC_ARCH_X86, UC_MODE_64, &uc);
    if (uc_err != UC_ERR_OK) {
        ks_free(encode);
        ks_close(ks);
        throw std::runtime_error("Failed to initialize Unicorn emulator");
    }
    // Map memory for the code and stack
    uc_err = uc_mem_map(uc, BASE_ADDRESS, CODE_SIZE, UC_PROT_ALL);
    if (uc_err != UC_ERR_OK) {
        uc_close(uc);
        ks_free(encode);
        ks_close(ks);
        throw std::runtime_error("Failed to map code memory");
    }
    uc_err = uc_mem_map(uc, STACK_ADDRESS, STACK_SIZE, UC_PROT_READ | UC_PROT_WRITE);
    if (uc_err != UC_ERR_OK) {
        uc_close(uc);
        ks_free(encode);
        ks_close(ks);
        throw std::runtime_error("Failed to map stack memory");
    }
    // Write the assembled code to memory
    uc_err = uc_mem_write(uc, BASE_ADDRESS, encode, size);
    if (uc_err != UC_ERR_OK) {
        uc_close(uc);
        ks_free(encode);
        ks_close(ks);
        throw std::runtime_error("Failed to write code to memory");
    }

    // Initialize CPU registers
    // Set up stack pointer
    uint64_t rsp = STACK_ADDRESS + STACK_SIZE - 0x100;

    // Add a dummy return address on the stack
    uint64_t return_address = BASE_ADDRESS + size;  // Just past the end of our code
    uc_err = uc_mem_write(uc, rsp, &return_address, sizeof(return_address));
    if (uc_err != UC_ERR_OK) {
        uc_close(uc);
        ks_free(encode);
        ks_close(ks);
        throw std::runtime_error("Failed to write dummy return address to stack");
    }

    uc_err = uc_reg_write(uc, UC_X86_REG_RSP, &rsp);
    if (uc_err != UC_ERR_OK) {
        uc_close(uc);
        ks_free(encode);
        ks_close(ks);
        throw std::runtime_error("Failed to set stack pointer");
    }

    // Set up base pointer (same as stack initially)
    uint64_t rbp = rsp;
    uc_err = uc_reg_write(uc, UC_X86_REG_RBP, &rbp);
    if (uc_err != UC_ERR_OK) {
        uc_close(uc);
        ks_free(encode);
        ks_close(ks);
        throw std::runtime_error("Failed to set base pointer");
    }

    // Start execution
    uc_err = uc_emu_start(uc, BASE_ADDRESS, return_address, 0, 0);
    if (uc_err != UC_ERR_OK) {
        uc_close(uc);
        ks_free(encode);
        ks_close(ks);
        throw std::runtime_error("Failed during emulation: " + std::string(uc_strerror(uc_err)));
    }

    // Read the result from RAX register
    uint64_t rax;
    uc_err = uc_reg_read(uc, UC_X86_REG_RAX, &rax);
    if (uc_err != UC_ERR_OK) {
        uc_close(uc);
        ks_free(encode);
        ks_close(ks);
        throw std::runtime_error("Failed to read result from RAX");
    }

    // Clean up
    uc_close(uc);
    ks_free(encode);
    ks_close(ks);
    return static_cast<int64_t>(rax);
}