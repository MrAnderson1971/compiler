mod simulator;

use crate::simulator::{CompilerTest, harness};
use rstest::rstest;

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
