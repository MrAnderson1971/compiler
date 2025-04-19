use rstest::*;
// tests/test_helpers/simulator.rs
use std::ffi::CString;
use std::fs;
use std::io::{self};
use std::path::PathBuf;
use std::process::Command;

// Windows-specific imports
use compiler::{CompilerError, compile};
use uuid::Uuid;
use winapi::shared::minwindef::{DWORD, HMODULE};
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::fileapi::GetTempPathA;
use winapi::um::libloaderapi::{FreeLibrary, GetProcAddress, LoadLibraryA};
use winapi::um::processthreadsapi::GetCurrentProcessId;

const MAX_PATH: usize = 260; // Windows MAX_PATH constant

#[derive(Debug)]
pub struct Simulator {
    temp_asm_file: PathBuf,
    temp_obj_file: PathBuf,
    temp_dll_file: PathBuf,
    dll_handle: Option<HMODULE>,
}

impl Simulator {
    pub fn new() -> Self {
        // Create unique filenames using process ID
        let pid = Uuid::new_v4().to_string();

        // Get Windows temp path
        let mut temp_path_buf = [0u8; MAX_PATH];
        let temp_path_len =
            unsafe { GetTempPathA(MAX_PATH as DWORD, temp_path_buf.as_mut_ptr() as *mut i8) };

        if temp_path_len == 0 {
            let error = unsafe { GetLastError() };
            panic!("Failed to get temp path: {}", error);
        }

        // Convert the Windows temp path to a Rust string
        let temp_path = String::from_utf8_lossy(&temp_path_buf[..temp_path_len as usize]);

        // Create the file paths
        let temp_asm_file = PathBuf::from(format!("{}asm_{}.s", temp_path, pid));
        let temp_obj_file = PathBuf::from(format!("{}asm_{}.o", temp_path, pid));
        let temp_dll_file = PathBuf::from(format!("{}asm_{}.dll", temp_path, pid));

        Simulator {
            temp_asm_file,
            temp_obj_file,
            temp_dll_file,
            dll_handle: None,
        }
    }

    pub fn load_program(&self, asm_code: &str) -> Result<(), io::Error> {
        println!("Compiling assembly code:\n{}", asm_code);

        // Clean the code if in debug mode
        let cleaned_code = if cfg!(debug_assertions) {
            asm_code
                .lines()
                .filter(|line| {
                    let trimmed = line.trim();
                    !trimmed.is_empty() && !trimmed.contains(';')
                })
                .collect::<Vec<&str>>()
                .join("\n")
        } else {
            asm_code.to_string()
        };

        // Modify the code to rename main to _runAsm for Windows
        let modified_code = cleaned_code
            .replace(".global main", ".global _runAsm")
            .replace("main:", "_runAsm:");

        // Write the assembly code to a temporary file
        fs::write(&self.temp_asm_file, &modified_code)?;
        println!("Wrote assembly to temporary file: {:?}", self.temp_asm_file);

        // Helper function to execute a command and get its output
        fn execute_command(
            command: &str,
            args: &[&str],
        ) -> Result<(bool, String, String), io::Error> {
            let output = Command::new(command).args(args).output()?;

            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            Ok((output.status.success(), stdout, stderr))
        }

        // Compile assembly to object file
        let asm_path = self.temp_asm_file.to_str().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "Invalid assembly file path")
        })?;

        let obj_path = self.temp_obj_file.to_str().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "Invalid object file path")
        })?;

        let (compile_success, compile_stdout, compile_stderr) =
            execute_command("gcc", &["-v", "-c", asm_path, "-o", obj_path])?;

        println!("Compilation output: {}", compile_stdout);

        if !compile_success {
            // Create a more detailed error message
            let mut error_msg = format!(
                "Failed to compile assembly (status: failed)\n\
                Command: gcc -v -c \"{}\" -o \"{}\"\n\
                Output: {}\n",
                asm_path, obj_path, compile_stderr
            );

            // Save the assembly file for debugging (in case it gets deleted)
            let pid = unsafe { GetCurrentProcessId() };

            // Get Windows temp path again for debug file
            let mut temp_path_buf = [0u8; MAX_PATH];
            let temp_path_len =
                unsafe { GetTempPathA(MAX_PATH as DWORD, temp_path_buf.as_mut_ptr() as *mut i8) };
            let temp_path = String::from_utf8_lossy(&temp_path_buf[..temp_path_len as usize]);

            let debug_file_name = format!("{}asm_debug_{}.s", temp_path, pid);
            if let Ok(_) = fs::write(&debug_file_name, &modified_code) {
                error_msg += &format!("Assembly code saved to: {}", debug_file_name);
            }

            return Err(io::Error::new(io::ErrorKind::Other, error_msg));
        }

        // Link object file to create DLL
        let dll_path = self
            .temp_dll_file
            .to_str()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid DLL file path"))?;

        let (link_success, link_stdout, link_stderr) = execute_command(
            "gcc",
            &[
                "-v",
                "-shared",
                obj_path,
                "-o",
                dll_path,
                "-Wl,--export-all-symbols",
            ],
        )?;

        println!("Linking output: {}", link_stdout);

        if !link_success {
            // Create a more detailed error message
            let error_msg = format!(
                "Failed to create DLL (status: failed)\n\
                Command: gcc -v -shared \"{}\" -o \"{}\" -Wl,--export-all-symbols\n\
                Output: {}\n",
                obj_path, dll_path, link_stderr
            );

            return Err(io::Error::new(io::ErrorKind::Other, error_msg));
        }

        println!("Successfully compiled and linked assembly");
        Ok(())
    }

    pub fn execute(&mut self) -> Result<i32, io::Error> {
        // Load the DLL
        let dll_path = self
            .temp_dll_file
            .to_str()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid DLL file path"))?;

        let dll_path_c = CString::new(dll_path)?;
        let dll_handle = unsafe { LoadLibraryA(dll_path_c.as_ptr()) };

        if dll_handle.is_null() {
            let error_code = unsafe { GetLastError() };
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to load DLL: {}", error_code),
            ));
        }

        self.dll_handle = Some(dll_handle);

        // Get the function pointer
        type AsmFunction = unsafe extern "C" fn() -> i64;
        let run_asm_name = CString::new("_runAsm")?;
        let mut run_asm: Option<AsmFunction> = None;

        unsafe {
            let proc_addr = GetProcAddress(dll_handle, run_asm_name.as_ptr());
            if !proc_addr.is_null() {
                run_asm = Some(std::mem::transmute(proc_addr));
            } else {
                // Try without underscore as fallback
                let alt_name = CString::new("runAsm")?;
                let alt_proc_addr = GetProcAddress(dll_handle, alt_name.as_ptr());
                if !alt_proc_addr.is_null() {
                    run_asm = Some(std::mem::transmute(alt_proc_addr));
                }
            }
        }

        if let Some(func) = run_asm {
            #[cfg(debug_assertions)]
            println!("Executing assembly function...");

            let result = unsafe { func() };

            #[cfg(debug_assertions)]
            println!("Assembly function returned: {}", result);

            // Cleanup
            unsafe {
                FreeLibrary(dll_handle);
            }
            self.dll_handle = None;

            Ok(result as i32)
        } else {
            let error_code = unsafe { GetLastError() };
            unsafe {
                FreeLibrary(dll_handle);
            }
            self.dll_handle = None;

            Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to get function address: {}", error_code),
            ))
        }
    }
}

impl Drop for Simulator {
    fn drop(&mut self) {
        // Clean up temporary files
        let _ = fs::remove_file(&self.temp_asm_file);
        let _ = fs::remove_file(&self.temp_obj_file);
        let _ = fs::remove_file(&self.temp_dll_file);

        // Free the library if it's loaded
        if let Some(handle) = self.dll_handle {
            unsafe {
                FreeLibrary(handle);
            }
            self.dll_handle = None;
        }
    }
}

#[derive(Debug)]
pub struct CompilerTest {
    pub simulator: Simulator,
}

impl CompilerTest {
    pub fn new() -> Self {
        let simulator = Simulator::new();
        CompilerTest { simulator }
    }

    /// Compiles source code, loads it into the simulator, and executes it.
    /// Returns the exit code or TestError on compiler/simulator failure.
    pub fn compile_and_run(&mut self, source: &str) -> Result<i32, CompilerError> {
        let asm = compile(source.to_string())?;
        match self.simulator.load_program(&asm) {
            Ok(_) => {}
            Err(err) => panic!("{}", err),
        }
        let result = match self.simulator.execute() {
            Ok(code) => code,
            Err(err) => panic!("{}", err),
        };
        Ok(result)
    }

    /// Compiles source code and asserts that it runs successfully with the expected exit code.
    /// Panics on compiler/simulator error or if the exit code doesn't match.
    pub fn assert_runs_ok(&mut self, source: &str, expected_code: i32) {
        match self.compile_and_run(source) {
            Ok(actual_code) => {
                assert_eq!(
                    actual_code, expected_code,
                    "Test failed: Expected exit code {}, but got {}",
                    expected_code, actual_code
                );
            }
            Err(e) => {
                panic!(
                    "Test failed: Expected successful run with code {}, but got error: {}",
                    expected_code, e
                );
            }
        }
    }

    /// Compiles source code and asserts that a specific CompilerError occurs.
    /// Panics if compilation succeeds or if a different error occurs.
    pub fn assert_compile_error<F>(&self, source: &str, check: F)
    where
        F: FnOnce(&CompilerError) -> bool,
    {
        match compile(source.to_string()) {
            Ok(asm) => {
                panic!(
                    "Test failed: Expected compiler error, but compilation succeeded.\nAssembly:\n{}",
                    asm
                );
            }
            Err(e) => {
                // Ensure it's a CompilerError
                assert!(
                    check(&e),
                    "Test failed: Compiler error occurred, but it was not the expected type/variant. Got: {:?}",
                    e
                );
            } // If compile can return other error types wrap them or handle here
              // Err(other_error_type) => {
              //     panic!("Test failed: Expected CompilerError, but got a different error type: {:?}", other_error_type);
              // }
        }
    }

    /// Compiles source, loads, and expects execution to fail (e.g., runtime error in asm).
    /// Panics if compilation fails or if execution succeeds.
    pub fn assert_execution_fails(&mut self, source: &str) {
        let asm = match compile(source.to_string()) {
            Ok(a) => a,
            Err(e) => panic!(
                "Test failed: Compilation failed when expecting execution failure. Error: {}",
                e
            ),
        };
        if let Err(e) = self.simulator.load_program(&asm) {
            panic!(
                "Test failed: Simulator failed to load program when expecting execution failure. Error: {}",
                e
            );
        }
        match self.simulator.execute() {
            Ok(code) => {
                panic!(
                    "Test failed: Expected execution failure, but it succeeded with code: {}",
                    code
                );
            }
            Err(_) => {
                // Execution failed as expected, test passes.
            }
        }
    }

    /// Directly loads assembly code and executes it, asserting the expected exit code.
    /// Panics on simulator error or if the exit code doesn't match.
    pub fn assert_asm_runs_ok(&mut self, asm_source: &str, expected_code: i32) {
        if let Err(e) = self.simulator.load_program(asm_source) {
            panic!(
                "Test failed: Simulator failed to load program. Error: {}",
                e
            );
        }
        match self.simulator.execute() {
            Ok(actual_code) => {
                assert_eq!(
                    actual_code, expected_code,
                    "Test failed: Expected ASM exit code {}, but got {}",
                    expected_code, actual_code
                );
            }
            Err(e) => {
                panic!(
                    "Test failed: Expected successful ASM run with code {}, but got error: {}",
                    expected_code, e
                );
            }
        }
    }

    /// Directly loads assembly code and expects execution to fail.
    /// Panics if loading fails or execution succeeds.
    pub fn assert_asm_execution_fails(&mut self, asm_source: &str) {
        if let Err(e) = self.simulator.load_program(asm_source) {
            panic!(
                "Test failed: Simulator failed to load program when expecting execution failure. Error: {}",
                e
            );
        }
        match self.simulator.execute() {
            Ok(code) => {
                panic!(
                    "Test failed: Expected ASM execution failure, but it succeeded with code: {}",
                    code
                );
            }
            Err(_) => {
                // Execution failed as expected, test passes.
            }
        }
    }
}

// Helper macro for asserting specific compiler errors
#[macro_export]
macro_rules! assert_compile_err {
    ($harness:expr, $source:expr, $pattern:pat) => {
        $harness.assert_compile_error($source, |e| matches!(e, $pattern))
    };
}

#[fixture]
pub fn harness() -> CompilerTest {
    // Now calls the new() that returns Self (or panics)
    CompilerTest::new()
}

pub fn expect_death(source: &str) {
    use compiler::compile;
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use std::process::{Command}; // Import ExitStatus
    use uuid::Uuid; // Ensure this is correctly imported

    const CRASH_EXIT_CODE_SIM: i32 = 101; // Code if simulator catches the error/panic
    const NORMAL_EXIT_CODE_SIM: i32 = 0; // Code if simulator runs successfully (BAD for expect_death)
    const SETUP_ERROR_SIM: i32 = 1; // Code for simulator setup errors (e.g., read file)

    // 1. Compile C to Assembly
    let asm = match compile(source.to_string()) {
        Ok(asm) => asm,
        Err(e) => panic!(
            "Compilation failed unexpectedly when generating code for death test: {}",
            e
        ),
    };
    println!("Generated Assembly for death test:\n{}", asm);

    // --- Get necessary paths ---
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let simulator_source_path = manifest_dir.join("tests").join("simulator.rs");
    if !simulator_source_path.exists() {
        panic!(
            "Simulator source file not found at expected path: {:?}",
            simulator_source_path
        );
    }
    let compiler_crate_root = manifest_dir.clone();
    let simulator_path_escaped = simulator_source_path
        .to_string_lossy()
        .escape_default()
        .to_string();

    // --- 2. Prepare Temporary Cargo Package ---
    let test_pkg_dir = env::temp_dir().join(format!("compiler_test_pkg_{}", Uuid::new_v4()));
    let test_src_dir = test_pkg_dir.join("src");
    fs::create_dir_all(&test_src_dir).expect("Failed to create temp src directory");
    let asm_file = test_pkg_dir.join("test_code.asm");
    fs::write(&asm_file, &asm).expect("Failed to write ASM file");
    let asm_file_path_escaped = asm_file
        .file_name()
        .unwrap()
        .to_string_lossy()
        .escape_default()
        .to_string();

    // --- 3. Create src/main.rs with Revised Exit Codes ---
    let test_main_rs = test_src_dir.join("main.rs");
    let test_code = format!(
        r#"
// --- Crash Tester main.rs ---
use std::fs;
use std::panic::{{catch_unwind, AssertUnwindSafe}};

mod simulator_code {{
    include!("{simulator_path}");
}}
use simulator_code::Simulator;

const EXPECTED_FAIL_CODE: i32 = {crash_code_sim}; // e.g., 101
const UNEXPECTED_SUCCESS_CODE: i32 = {normal_code_sim}; // e.g., 0
const SETUP_ERR_CODE: i32 = {setup_err_sim}; // e.g., 1

fn main() {{
    // We wrap the *logic* because setup errors shouldn't be caught as panics
    match run_setup_and_test() {{
        // Setup ok, test logic finished
        Ok(test_result) => {{
            match test_result {{
                // Test logic succeeded (returned Ok(exit_code)) -> BAD for expect_death
                Ok(exit_code) => {{
                    eprintln!("Crash Tester ERROR: Execution succeeded unexpectedly with code {{}}.", exit_code);
                    // Exit with a code indicating unexpected success
                    std::process::exit(UNEXPECTED_SUCCESS_CODE);
                }},
                // Test logic failed (returned Err(msg)) -> GOOD for expect_death
                Err(e) => {{
                    eprintln!("Crash Tester SUCCESS: Execution failed as expected (simulator returned Err): {{}}", e);
                    // Exit with the code indicating expected failure detected by simulator
                    std::process::exit(EXPECTED_FAIL_CODE);
                }}
            }}
        }},
        // Setup itself failed (e.g. read file)
        Err(setup_err) => {{
             eprintln!("Crash Tester SETUP ERROR: {{}}", setup_err);
             std::process::exit(SETUP_ERR_CODE);
        }}
    }}
}}

// Separated setup from the core logic prone to crashing/panicking
fn run_setup_and_test() -> Result<Result<i32, String>, String> {{ // Outer Result for setup, Inner for test logic
    let mut simulator = Simulator::new(); // Assume this doesn't panic

    let asm = fs::read_to_string("{asm_path}").map_err(|e| format!("Failed to read ASM: {{}}", e))?;
    simulator.load_program(&asm).map_err(|e| format!("Failed to load program: {{}}", e))?;

    // Now, catch panics specifically around the execution
    let exec_result = catch_unwind(AssertUnwindSafe(|| {{
        simulator.execute() // This might panic or return Err/Ok
    }}));

    match exec_result {{
        // Execution completed without Rust panic
        Ok(Ok(exit_code)) => Ok(Ok(exit_code)), // Successful execution -> return Ok(Ok(code))
        Ok(Err(e)) => Ok(Err(e.to_string())), // Simulator returned error -> return Ok(Err(msg))
        // Execution caused Rust panic
        Err(_) => Ok(Err("Execution panicked".to_string())), // Panic -> return Ok(Err(msg))
    }}
}}
// --- End Crash Tester main.rs ---
"#,
        simulator_path = simulator_path_escaped,
        asm_path = asm_file_path_escaped,
        crash_code_sim = CRASH_EXIT_CODE_SIM,
        normal_code_sim = NORMAL_EXIT_CODE_SIM,
        setup_err_sim = SETUP_ERROR_SIM
    );
    fs::write(&test_main_rs, test_code).expect("Failed to write temporary main.rs");

    // --- 4. Create Cargo.toml (Ensure rstest version is correct) ---
    let test_cargo_toml = test_pkg_dir.join("Cargo.toml");
    let compiler_crate_path_escaped = compiler_crate_root.to_string_lossy().replace(r"\", r"\\");

    let cargo_toml_content = format!(
        r#"
[package]
name = "crash-tester"
version = "0.1.0"
edition = "2021"

[dependencies]
uuid = {{ version = "1", features = ["v4"] }}
winapi = {{ version = "0.3", features = ["libloaderapi", "processthreadsapi", "fileapi", "minwindef", "errhandlingapi"] }}
compiler = {{ path = "{}" }}
rstest = "0.25.0"
"#,
        compiler_crate_path_escaped,
    );
    fs::write(&test_cargo_toml, cargo_toml_content).expect("Failed to write temporary Cargo.toml");

    // --- 5. Run the temporary package using `cargo run` ---
    println!(
        "Running crash test package via cargo run in {:?}",
        test_pkg_dir
    );
    let run_output = Command::new("cargo")
        .arg("run")
        .current_dir(&test_pkg_dir)
        // .arg("--quiet")
        .output()
        .expect("Failed to execute cargo run for crash tester");

    println!(
        "Crash Tester (cargo run) status: {:?}\nstdout:\n{}\nstderr:\n{}",
        run_output.status.code(),
        String::from_utf8_lossy(&run_output.stdout),
        String::from_utf8_lossy(&run_output.stderr)
    );

    // --- 6. Assert the outcome (Revised Logic) ---
    let status = run_output.status;
    let exit_code = status.code();

    let crashed_as_expected =
            // Case 1: Simulator caught the error/panic and exited cleanly with the specific code
            (status.success() && exit_code == Some(CRASH_EXIT_CODE_SIM)) ||
                // Case 2: OS terminated the process abnormally (likely due to hardware exception)
                (!status.success() && exit_code != Some(NORMAL_EXIT_CODE_SIM) && exit_code != Some(SETUP_ERROR_SIM) );
    // Note: We exclude the "normal exit" and "setup error" codes from being considered an "abnormal termination" success.

    if !crashed_as_expected {
        // Construct a helpful failure message
        let mut failure_reason = format!("Death test failed. ");
        match exit_code {
            Some(code) if code == NORMAL_EXIT_CODE_SIM => {
                failure_reason
                    .push_str("Process exited normally (code 0), but was expected to crash.");
            }
            Some(code) if code == SETUP_ERROR_SIM => {
                failure_reason.push_str(&format!(
                    "Crash tester setup failed (code {}). Check simulator/ASM loading.",
                    code
                ));
            }
            Some(code) => {
                failure_reason
                    .push_str(&format!("Process exited with unexpected code: {}. ", code));
                if status.success() {
                    failure_reason.push_str("(Process reported success status).");
                } else {
                    failure_reason
                        .push_str("(Process reported failure status, but not a recognized crash).");
                }
            }
            None => {
                failure_reason.push_str("Process terminated by signal (no specific exit code).");
                // On Unix, you could check status.signal() here.
            }
        }
        failure_reason.push_str(&format!(
            "\n--- Crash Tester Stdout:\n{}\n--- Crash Tester Stderr:\n{}",
            String::from_utf8_lossy(&run_output.stdout),
            String::from_utf8_lossy(&run_output.stderr)
        ));
        panic!("{}", failure_reason);
    }

    // --- 7. Cleanup ---
    let _ = fs::remove_dir_all(&test_pkg_dir);
    println!("Death test passed for source:\n{}\n", source);
}
