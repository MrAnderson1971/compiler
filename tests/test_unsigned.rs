mod simulator;

use rstest::rstest;
use crate::simulator::{harness, CompilerTest};

#[rstest]
fn test_unsigned_long(mut harness: CompilerTest) {
    let source = r#"
    int main() {
    long a = 1ul;
    return a;
    }"#;
    harness.assert_runs_ok(source, 1);
}
