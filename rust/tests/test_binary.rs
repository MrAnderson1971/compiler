// tests/test_binary.rs
mod simulator;

use rstest::*;
use simulator::{CompilerTest, harness};
use compiler::CompilerError;

#[rstest]
fn test_addition(mut harness: CompilerTest) {
    let source = "int main() { return 1 + 2; }";
    harness.assert_runs_ok(source, 3);
}

#[rstest]
fn test_missing_operand(mut harness: CompilerTest) {
    let source = "int main() { return 1 +; }";
    assert_compile_err!(harness, source, CompilerError::SyntaxError(_));
}

#[rstest]
fn test_subtraction(mut harness: CompilerTest) {
    let source = "int main() { return 3 - 2; }";
    harness.assert_runs_ok(source, 1);
}

#[rstest]
fn test_multiplication(mut harness: CompilerTest) {
    let source = "int main() { return 2 * 3; }";
    harness.assert_runs_ok(source, 6);
}

#[rstest]
fn test_division(mut harness: CompilerTest) {
    let source = "int main() { return 6 / 2; }";
    harness.assert_runs_ok(source, 3);
}

#[rstest]
fn test_modulus(mut harness: CompilerTest) {
    let source = "int main() { return 7 % 3; }";
    harness.assert_runs_ok(source, 1);
}

#[rstest]
fn test_precedence(mut harness: CompilerTest) {
    let source = "int main() { return 1 + 2 * 3; }";
    harness.assert_runs_ok(source, 7);
}

#[rstest]
fn test_parentheses(mut harness: CompilerTest) {
    let source = "int main() { return (1 + 2) * 3; }";
    harness.assert_runs_ok(source, 9);
}

#[rstest]
fn test_associativity(mut harness: CompilerTest) {
    let source = "int main() { return 1 - 2 - 3; }";
    harness.assert_runs_ok(source, -4);
}

#[rstest]
fn test_associativity_and_precedence(mut harness: CompilerTest) {
    let source = r#"int main() {
    return 5 * 4 / 2 -
        3 % (2 + 1);
}"#;
    harness.assert_runs_ok(source, 5 * 4 / 2 - 3 % (2 + 1));
}

#[rstest]
fn test_divide_negative(mut harness: CompilerTest) {
    let source = r#"int main() {
    return (-12) / 5;
}"#;
    harness.assert_runs_ok(source, -2);
}

#[rstest]
fn test_unary_and_binary(mut harness: CompilerTest) {
    let source = r#"int main() {
    return ~(1+1);
}"#;
    harness.assert_runs_ok(source, !(1 + 1));
}

#[rstest]
fn test_bitwise_and(mut harness: CompilerTest) {
    let source = r#"int main() {
    return 3 & 5;
}"#;
    harness.assert_runs_ok(source, 3 & 5);
}

#[rstest]
fn test_complicated(mut harness: CompilerTest) {
    let source = r#"int main() {
    return ((42 * 3) - (15 / 5) % 4 + (7 << 2)) & ~(255 - 128) | ((16 >> 2) ^ 10);
}"#;
    harness.assert_runs_ok(source, ((42 * 3) - (15 / 5) % 4 + (7 << 2)) & !(255 - 128) | ((16 >> 2) ^ 10));
}

#[rstest]
fn test_divide_by_zero(mut harness: CompilerTest) {
    let source = r#"int main() {
    return 1/0;
}"#;
    match compiler::compile(source.to_string()) {
        Ok(asm) => harness.assert_asm_execution_fails(&asm),
        Err(e) => panic!("Compilation failed unexpectedly when testing divide by zero: {}", e),
    }
}

#[rstest]
fn test_mod_by_zero(mut harness: CompilerTest) {
    let source = r#"int main() {
    return 1 % 0;
}"#;
    match compiler::compile(source.to_string()) {
        Ok(asm) => harness.assert_asm_execution_fails(&asm),
        Err(e) => panic!("Compilation failed unexpectedly when testing modulo by zero: {}", e),
    }
}