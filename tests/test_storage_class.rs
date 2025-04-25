mod simulator;

use rstest::rstest;
use crate::simulator::{harness, CompilerTest};

#[rstest]
fn test_static(mut harness: CompilerTest) {
    let source = r#"
    static int foo() {
        return 0;
    }
    int main() {
        return foo();
    }"#;
    harness.assert_runs_ok(source, 0);
}

#[rstest]
fn test_static_in_function(mut harness: CompilerTest) {
    let source = r#"
    int foo() {
        static int bar = 0;
        bar += 1;
        return bar;
    }
    int main() {
        foo();
        return foo();
    }"#;
    harness.assert_runs_ok(source, 2);
}
