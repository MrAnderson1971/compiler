mod simulator;

use crate::simulator::{CompilerTest, harness};
use rstest::rstest;
use compiler::CompilerError;

#[rstest]
fn test_function(mut harness: CompilerTest) {
    let source = r#"
    int foo(int a) {
    return a;
    }
    
    int main() {
    return foo(1);
    }
    "#;
    harness.assert_runs_ok(source, 1);
}

#[rstest]
fn test_duplicate_definition(harness: CompilerTest) {
    let source = r#"
    int foo(int a) {
    return a;
    }

    int foo(int a) {
    return a;
    }

    int main () {
    return foo(1);
    }
    "#;
    assert_compile_err!(harness, source, CompilerError::SemanticError(_));
}
