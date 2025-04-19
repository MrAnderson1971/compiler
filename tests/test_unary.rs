// tests/test_unary.rs
mod simulator;

use rstest::*;
use simulator::{CompilerTest, harness};
use compiler::CompilerError;

#[rstest]
fn test_bitwise(mut harness: CompilerTest) {
    let source = r#"
int main() {
    return ~12;
}
"#;
    harness.assert_runs_ok(source, !12);
}

#[rstest]
fn test_bitwise0(mut harness: CompilerTest) {
    let source = r#"
int main() {
    return ~0;
}
"#;
    harness.assert_runs_ok(source, !0);
}

#[rstest]
fn test_missing_const(harness: CompilerTest) {
    let source = r#"
int main() {
    return ~;
}
"#;
    assert_compile_err!(harness, source, CompilerError::SyntaxError(_));
}

#[rstest]
fn test_missing_semicolon2(harness: CompilerTest) {
    let source = r#"
int main() {
    return ~12
}
"#;
    assert_compile_err!(harness, source, CompilerError::SyntaxError(_));
}

#[rstest]
fn test_wrong_order(harness: CompilerTest) {
    let source = r#"
int main() {
    return 12~;
}
"#;
    assert_compile_err!(harness, source, CompilerError::SyntaxError(_));
}