// tests/compiler_tests.rs
mod simulator;

use rstest::*;
use simulator::{CompilerTest, harness};
use compiler::CompilerError;

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
    assert_compile_err!(harness, source, CompilerError::SyntaxError(_));
}

#[rstest]
fn test_missing_opening_brace(harness: CompilerTest) {
    let source = r#"
int main()
    return 0;
}
"#;
    assert_compile_err!(harness, source, CompilerError::SyntaxError(_));
}

#[rstest]
fn test_missing_main_function_name(harness: CompilerTest) {
    let source = r#"
int () {
    return 0;
}
"#;
    assert_compile_err!(harness, source, CompilerError::SyntaxError(_));
}

#[rstest]
fn test_missing_return_value(harness: CompilerTest) {
    let source = r#"
int main() {
    return ;
}
"#;
    assert_compile_err!(harness, source, CompilerError::SyntaxError(_));
}

#[rstest]
fn test_statement_without_return(mut harness: CompilerTest) {
    let source = r#"
int main() {
    0;
}
"#;
    harness.assert_runs_ok(source, 0);
}

#[rstest]
fn test_missing_semicolon(harness: CompilerTest) {
    let source = r#"
int main() {
    return 0
}
"#;
    assert_compile_err!(harness, source, CompilerError::SyntaxError(_));
}

#[rstest]
fn test_missing_space_return_literal(harness: CompilerTest) {
    let source = r#"
int main() {
    return0;
}
"#;
    assert_compile_err!(harness, source, CompilerError::SemanticError(_));
}

#[rstest]
fn test_no_entry_point(mut harness: CompilerTest) {
    let source = r#"
int foo() {
    return 0;
}
"#;
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
    harness.assert_runs_ok(source, 0);
}