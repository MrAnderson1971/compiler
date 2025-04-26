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
fn test_truncate_at_return(mut harness: CompilerTest) {
    let source = r#"
    int main() {
    long l = 9223372036854775807l;
    return l;
}"#;
    harness.assert_runs_ok(source, -1);
}

#[rstest]
fn test_sign_extend(mut harness: CompilerTest) {
    let source = r#"
    int main() {
    int i = -42;
    long l = i;
    return l == -42;
    }
    "#;
    harness.assert_runs_ok(source, 1);
}
