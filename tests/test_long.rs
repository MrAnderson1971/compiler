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
fn test_long_int(mut harness: CompilerTest) {
    let source = r#"
    int main() {
    int long l = 9223372036854775807l;
    long int l2 = 9223372036854775807l;
    return (l - 2l == 9223372036854775805l) && (l2 - 2l == 9223372036854775805l);
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
fn test_long_modulo(mut harness: CompilerTest) {
    let source = r#"
    long a;
    long b;
    int main() {
             a = -4294967290l; // 2^32 - 6
             b = 90l;

    return a % b == -70l;
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

#[rstest]
fn test_many_long_parameters(mut harness: CompilerTest) {
    let source = r#"
long calculate_check_digit(long a, int b, long c, int d, long e, int f, long g, int h, long i, int j) {

    long product = a * c * e * g * i;

    long remainder = product % 10;

    long sum = b + d + f + h + j;

    return (10 - (remainder + sum) % 10) % 10;
}

int main() {

return calculate_check_digit(
    -4294967297, 2,
    3, -4,
    5, 6,
    -7, 8,
    9, -10);
    }
    "#;
    harness.assert_runs_ok(source, 3);
}

#[rstest]
fn test_align(mut harness: CompilerTest) {
    let source = r#"int main() {
    // Create alternating int/long variables to stress alignment
    int a = 5;
    long b = 1;
    b = b << 32;    // b = 2^32 (4294967296) - requires full 64 bits
    int c = 10;
    long d = 1;
    d = d << 33;    // d = 2^33 (8589934592)
    int e = 3;

    // Test 1: Can we store and retrieve large values correctly?
    if (b != (1L << 32)) return 1;  // Fails if b is truncated to 32 bits

    // Test 2: 64-bit arithmetic operations
    long sum = b + d;  // 2^32 + 2^33 = 3*2^32
    if (sum != 3 * (1L << 32)) return 2;  // Fails if addition truncates

    // Test 3: Mixed int/long multiplication (tests sign extension)
    long product = a * b;  // 5 * 2^32
    if (product != 5 * (1L << 32)) return 3;  // Fails if conversion is wrong

    // Test 4: Long division with large values
    long quotient = d / c;  // 2^33 / 10
    long remainder = d % c;
    if (quotient * c + remainder != d) return 4;  // Division identity check

    // Test 5: Complex expressions with mixed types
    long complex = (a * b + c * d) / e;
    if (complex != ((a * b) + (c * d)) / e) return 5;

    // All tests passed
    return 6;
}"#;
    harness.assert_runs_ok(source, 6);
}

#[rstest]
fn test_long_overflow(mut harness: CompilerTest) {
    let source = format!(r#"
    int main() {{
    long max = {};
    max++;
    return max == {};
    }}"#, i64::MAX, i64::MIN);
    harness.assert_runs_ok(&source, 1);
}

#[rstest]
fn test_long_underflow(mut harness: CompilerTest) {
    let source = format!(r#"
    int main() {{
    long min = {};
    min--;
    return min == {};
    }}"#, i64::MIN, i64::MAX);
    harness.assert_runs_ok(&source, 1);
}

#[rstest]
fn test_long_prefix(mut harness: CompilerTest) {
    let source = r#"
    int main() {
    long l = 9223372036854775807l;
    return ++l == 9223372036854775808l;
}"#;
    harness.assert_runs_ok(source, 1);
}
