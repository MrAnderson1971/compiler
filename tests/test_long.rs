mod simulator;

use rstest::rstest;
use crate::simulator::{harness, CompilerTest};

#[rstest]
fn test_long(mut harness: CompilerTest) {
    let source = r#"
    int main() {
    long l = 9223372036854775807l;
    return (l - 2l == 9223372036854775805l);
}"#;
    harness.assert_runs_ok(source, 1);
}

#[rstest]
fn test_cast_at_return(mut harness: CompilerTest) {
    let source = r#"
    int main() {
    long l = 100l;
    return l;
}"#;
    harness.assert_runs_ok(source, 100);
}
