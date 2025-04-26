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
fn test_truncate_at_assign(mut harness: CompilerTest) {
    let source = r#"
    int main() {
    long l = 9223372036854775807l;
    int i = l;
    return i;
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

#[rstest]
fn test_long_multiplication(mut harness: CompilerTest) {
    let source = r#"
    int main() {
    return 1000000l * 1000000l;
    }
    "#;
    harness.assert_runs_ok(source, (1_000_000i64 * 1_000_000i64) as i32);
}

#[rstest]
fn test_long_division(mut harness: CompilerTest) {
    let source = r#"
    int main() {
    return 1000000l / 1000000l;
    }
    "#;
    harness.assert_runs_ok(source, 1);
}

#[rstest]
fn test_long_subtraction(mut harness: CompilerTest) {
    let source = r#"
    long a;
    long b;
    int main() {
             a = -4294967290l; // 2^32 - 6
             b = 90l;

    return a - b == -4294967380l;
    }
    "#;
    harness.assert_runs_ok(source, 1);
}

#[rstest]
fn test_long_operations(mut harness: CompilerTest) {
    let source = r#"
long a;
long b;
int addition() {
    // a == 4294967290l, i.e. 2^32 - 6
    // b = 5
    return (a + b == 4294967295l);
}

int subtraction() {
    // a = -4294967290l;
    // b = 90l;
    return (a - b == -4294967380l);
}

int multiplication() {
    // a = 4294967290l;
    return (a * 4l == 17179869160l);
}

int division() {
    // a = 4294967290l;
    b = a / 128l;
    return (b == 33554431l);
}

int remaind() {
    // a = 8589934585l, i.e. 2^33 - 7
    b = -a % 4294967290l;
    return (b == -5l);
}

int complement() {
    // a = 9223372036854775806l, i.e. LONG_MAX - 1
    return (~a == -9223372036854775807l);
}

int main() {

    a = 4294967290l; // 2^32 - 6
    b = 5l;
    if (!addition()) {
        return 1;
    }

    a = -4294967290l;
    b = 90l;
    if (!subtraction()) {
        return 2;
    }

    a = 4294967290l;
    if (!multiplication()) {
        return 3;
    }

    a = 4294967290l;
    if (!division()) {
        return 4;
    }

    a = 8589934585l; // 2^33 - 7
    if (!remaind()) {
        return 5;
    }

    a = 9223372036854775806l; //LONG_MAX - 1
    if (!complement()) {
        return 6;
    }

    return 0;
}"#;
    harness.assert_runs_ok(source, 0);
}
