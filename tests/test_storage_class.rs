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
