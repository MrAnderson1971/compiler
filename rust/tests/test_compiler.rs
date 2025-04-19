// tests/compiler_tests.rs

// Declare the module where CompilerTest and the harness fixture are defined
mod simulator;

// Import necessary items
use rstest::*;
use simulator::{CompilerTest, harness}; // Import the type and the fixture function
use compiler::CompilerError; // Import your library's error enum for matching

// Helper macro for asserting specific compiler errors
// (Define locally or make public in test_setup and import)
macro_rules! assert_compile_err {
    ($harness:expr, $source:expr, $pattern:pat) => {
        // Note: harness.assert_compile_error now panics, doesn't return Result
        $harness.assert_compile_error($source, |e| matches!(e, $pattern))
    };
}

#[rstest]
fn test_success(mut harness: CompilerTest) {
    let source = r#"
int main() {
    return 42;
}
"#;
    harness.assert_runs_ok(source, 42);
}

#[rstest]
fn test_missing_closing_brace(harness: CompilerTest) {
    let source = r#"
int main() {
    return 0;
"#;
    // Assuming this is caught by the parser
    assert_compile_err!(harness, source, CompilerError::SyntaxError(_));
}

#[rstest]
fn test_missing_opening_brace(harness: CompilerTest) {
    let source = r#"
int main()
    return 0;
}
"#;
    // Assuming this is caught by the parser
    assert_compile_err!(harness, source, CompilerError::SyntaxError(_));
}

#[rstest]
fn test_missing_main_function_name(harness: CompilerTest) {
    let source = r#"
int () {
    return 0;
}
"#;
    // Assuming this is caught by the parser (missing identifier)
    assert_compile_err!(harness, source, CompilerError::SyntaxError(_));
}

#[rstest]
fn test_missing_return_value(harness: CompilerTest) {
    let source = r#"
int main() {
    return ;
}
"#;
    // Assuming this is caught by the parser (expected expression after return)
    assert_compile_err!(harness, source, CompilerError::SyntaxError(_));
}

#[rstest]
fn test_statement_without_return(mut harness: CompilerTest) {
    let source = r#"
int main() {
    0;
}
"#;
    // Behavior depends on your compiler:
    // Option 1: It implicitly returns 0 (like the C++ test seemed to imply)
    harness.assert_runs_ok(source, 0);
    // Option 2: It requires an explicit return (more common in simple C compilers)
    // assert_compile_err!(harness, source, CompilerError::SemanticError(_)); // Or a more specific error
}

#[rstest]
fn test_missing_semicolon(harness: CompilerTest) {
    let source = r#"
int main() {
    return 0
}
"#;
    // Assuming this is caught by the parser
    assert_compile_err!(harness, source, CompilerError::SyntaxError(_));
}

#[rstest]
fn test_missing_space_return_literal(harness: CompilerTest) {
    let source = r#"
int main() {
    return0;
}
"#;
    // Likely caught by the lexer trying to parse "return0" as one token
    assert_compile_err!(harness, source, CompilerError::SemanticError(_));
    // Or, if the lexer splits it, the parser might fail:
    // assert_compile_err!(harness, source, CompilerError::ParserError(_));
}

#[rstest]
fn test_no_entry_point(mut harness: CompilerTest) {
    let source = r#"
int foo() {
    return 0;
}
"#;
    // Option 1: Compiler performs semantic check for 'main' function
    // assert_compile_err!(harness, source, CompilerError::MissingMain); // Or SemanticError

    // Option 2: Compiler succeeds, but execution fails because simulator expects _runAsm (from main)
    // This requires compile() to succeed first.
    match compiler::compile(source.to_string()) {
        Ok(asm) => harness.assert_asm_execution_fails(&asm),
        Err(e) => panic!("Compilation failed unexpectedly when testing no entry point: {}", e),
    }
}

#[rstest]
fn test_unknown_symbol_char(harness: CompilerTest) {
    let source = r#"
int main() {
    #; // '#' is not standard C syntax
    return 0;
}
"#;
    // Likely caught by the lexer
    assert_compile_err!(harness, source, CompilerError::SyntaxError(_));
}

#[rstest]
fn test_many_semicolons(mut harness: CompilerTest) {
    let source = r#"
int main() {
    ;;;;; // Empty statements
    return 0;
}
"#;
    // Empty statements are generally valid in C-like languages
    harness.assert_runs_ok(source, 0);
}