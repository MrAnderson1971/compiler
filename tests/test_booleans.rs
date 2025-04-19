// tests/test_booleans.rs
mod simulator;

use rstest::*;
use simulator::{CompilerTest, harness};

#[rstest]
fn test_equality(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            return 0 == 0;
        }
    "#;
    harness.assert_runs_ok(source, 1);
}

#[rstest]
fn test_inequality(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            return 0 != 0;
        }
    "#;
    harness.assert_runs_ok(source, 0);
}

#[rstest]
fn test_less_than(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            return 0 < 1;
        }
    "#;
    harness.assert_runs_ok(source, 1);
}

#[rstest]
fn test_boolean_and_arithmetic(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            return 100 == 36 + 64;
        }
    "#;
    harness.assert_runs_ok(source, 1);
}

#[rstest]
fn test_logical_and(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            return 1 && 1;
        }
    "#;
    harness.assert_runs_ok(source, 1);
}

#[rstest]
fn test_false_logical_and1(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            return 1 && 0;
        }
    "#;
    harness.assert_runs_ok(source, 0);
}

#[rstest]
fn test_false_logical_and2(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            return 0 && 1;
        }
    "#;
    harness.assert_runs_ok(source, 0);
}

#[rstest]
fn test_false_logical_and_both(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            return 0 && 0;
        }
    "#;
    harness.assert_runs_ok(source, 0);
}

#[rstest]
fn test_logical_or(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            return 1 || 1;
        }
    "#;
    harness.assert_runs_ok(source, 1);
}

#[rstest]
fn test_true_logical_or1(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            return 1 || 0;
        }
    "#;
    harness.assert_runs_ok(source, 1);
}

#[rstest]
fn test_true_logical_or2(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            return 0 || 1;
        }
    "#;
    harness.assert_runs_ok(source, 1);
}

#[rstest]
fn test_false_logical_or(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            return 0 || 0;
        }
    "#;
    harness.assert_runs_ok(source, 0);
}

#[rstest]
fn test_logical_not_true(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            return !0;
        }
    "#;
    harness.assert_runs_ok(source, 1);
}

#[rstest]
fn test_logical_not_false(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            return !1;
        }
    "#;
    harness.assert_runs_ok(source, 0);
}

#[rstest]
fn test_complex_true(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            return (!0 && (5 > 3 || 2 < 1)) && (!(1 == 0) || (3 >= 4 && 2 != 2));
        }
    "#;
    harness.assert_runs_ok(source, 1);
}

#[rstest]
fn test_complex_false(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            return (!(3 < 7) || (0 && !0)) && ((2 == 2) && (4 > 5 || 1 > 3));
        }
    "#;
    harness.assert_runs_ok(source, 0);
}