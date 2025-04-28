mod simulator;

use crate::simulator::{CompilerTest, harness};
use compiler::CompilerError::{SemanticError, SyntaxError};
use rstest::rstest;

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
    let source = format!(
        r#"
    int main() {{
    long max = {};
    max++;
    return max == {};
    }}"#,
        i64::MAX,
        i64::MIN
    );
    harness.assert_runs_ok(&source, 1);
}

#[rstest]
fn test_long_underflow(mut harness: CompilerTest) {
    let source = format!(
        r#"
    int main() {{
    long min = {};
    min--;
    return min == {};
    }}"#,
        i64::MIN,
        i64::MAX
    );
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

#[rstest]
fn test_overflow(harness: CompilerTest) {
    let source = format!(
        r#"
    int main() {{
    long l = {};
    return 0;
    }}"#,
        i128::MAX
    );
    assert_compile_err!(harness, &*source, SyntaxError(_));
}

#[rstest]
fn test_too_many_suffixes(harness: CompilerTest) {
    let source = r#"
    int main() {
    long l = 1000000lL;
    return l;
    }"#;
    assert_compile_err!(harness, &*source, SyntaxError(_));
}

#[rstest]
fn test_long_as_identifier(harness: CompilerTest) {
    let source = r#"
    int long() {
    return 0;
    }
    int main() {
    return 0;
    }"#;
    assert_compile_err!(harness, &*source, SyntaxError(_));
}

#[rstest]
fn test_conflicting_global_types(harness: CompilerTest) {
    let source = r#"
    int a;
    long a;
    int main() {
    return a;
    }"#;
    assert_compile_err!(harness, &*source, SemanticError(_));
}

#[rstest]
fn test_conflicting_extern_types(harness: CompilerTest) {
    let source = r#"
long a;

int main() {
    extern int a;
    return 0;
    }"#;
    assert_compile_err!(harness, &*source, SemanticError(_));
}

#[rstest]
fn test_conflicting_function_param_types(harness: CompilerTest) {
    let source = r#"int foo(int a);

int main() {
    return 0;
}

int foo(long a);"#;
    assert_compile_err!(harness, &*source, SemanticError(_));
}

#[rstest]
fn test_conflicting_function_return_types(harness: CompilerTest) {
    let source = r#"int foo(int a);

int main() {
    return 0;
}

long foo(int a);"#;
    assert_compile_err!(harness, &*source, SemanticError(_));
}

#[rstest]
fn test_static_long_with_init(mut harness: CompilerTest) {
    let source = r#"
    int foo() {
        static long a = 1000000l;
        a++;
        return a;
    }
    int main() {
        foo();
        foo();
        return foo();
    }
    "#;
    harness.assert_runs_ok(source, 1000003);
}

#[rstest]
fn test_top_level_static_long_with_init(mut harness: CompilerTest) {
    let source = r#"
    static long a = 1000000l;
    int foo() {
        a++;
        return a;
    }
    int main() {
        foo();
        foo();
        return foo();
    }
    "#;
    harness.assert_runs_ok(source, 1000003);
}

#[rstest]
fn test_static_long_without_init(mut harness: CompilerTest) {
    let source = r#"
    int foo() {
        static long a;
        a++;
        return a;
    }
    int main() {
        foo();
        foo();
        return foo();
    }
    "#;
    harness.assert_runs_ok(source, 3);
}

#[rstest]
fn test_top_level_static_long_without_init(mut harness: CompilerTest) {
    let source = r#"
    static long a;
    int foo() {
        a++;
        return a;
    }
    int main() {
        foo();
        foo();
        return foo();
    }
    "#;
    harness.assert_runs_ok(source, 3);
}

#[rstest]
fn test_invalid_specifier(harness: CompilerTest) {
    let source = r#"
    int main() {
        long long a;
        return 0;
    }
    "#;
    assert_compile_err!(harness, &*source, SyntaxError(_));
}

#[rstest]
fn test_cast_without_parentheses(harness: CompilerTest) {
    let source = r#"
    int main() {
        long a = long 1000000l;
        return a;
    }
    "#;
    assert_compile_err!(harness, &*source, SyntaxError(_));
}

#[rstest]
fn test_invalid_cast(harness: CompilerTest) {
    let source = r#"
    int main() {
        long a = (static long) 1000000l;
        return a;
    }
    "#;
    assert_compile_err!(harness, &*source, SyntaxError(_));
}

#[rstest]
fn test_cast_long_int(mut harness: CompilerTest) {
    let source = r#"
    int main() {
    if ((long int) 1000000 != 1000000l) {
    return 1;
    }

    if (-1000000l != (long int) -1000000) {
    return 2;
    }
    return 0;
    }
    "#;
    harness.assert_runs_ok(source, 0);
}

#[rstest]
fn test_cast_with_sign_extend(mut harness: CompilerTest) {
    let source = r#"
long sign_extend(int i, long expected) {
    long extended = (long) i;
    return (extended == expected);
}

int main() {
    if (!sign_extend(10, 10l)) {
        return 1;
    }

    if (!sign_extend(-10, -10l)) {
        return 2;
    }

    long l = (long) 100;
    if (l != 100l) {
        return 3;
    }
    return 0;
}"#;
    harness.assert_runs_ok(source, 0);
}

#[rstest]
fn test_cast_with_truncate(mut harness: CompilerTest) {
    let source = r#"int truncate(long l, int expected) {
    int result = (int) l;
    return (result == expected);
}

int main()
{

    if (!truncate(10l, 10)) {
        return 1;
    }

    if (!truncate(-10l, -10)) {
        return 2;
    }

    if (!truncate(17179869189l, // 2^34 + 5
                  5)) {
        return 3;
    }

    if (!truncate(-17179869179l, // (-2^34) + 5
                  5l)) {
        return 4;
    }

    int i = (int)17179869189l; // 2^34 + 5
    if (i != 5)
        return 5;

    return 0;
}"#;
    harness.assert_runs_ok(source, 0);
}

#[rstest]
fn test_long_bitwise_operations(mut harness: CompilerTest) {
    let source = r#"
    int main() {
        // Using decimal values instead of hex
        long a = 123456789123456789l;
        long b = 987654321987654321l;

        // Bitwise AND
        long and_result = a & b;
        if (and_result != 122892737510904337l) return 1;

        // Bitwise OR
        long or_result = a | b;
        if (or_result != 988218373600206773l) return 2;

        // Bitwise XOR
        long xor_result = a ^ b;
        if (xor_result != 865325636089302436l) return 3;

        // Bitwise NOT
        long not_result = ~a;
        if (not_result != -123456789123456790l) return 4;

        // Bit shifts
        long c = 1l;
        if ((c << 60) != 1152921504606846976l) return 5;
        if ((c << 60) >> 60 != 1l) return 6;

        return 0;
    }"#;
    harness.assert_runs_ok(source, 0);
}

#[rstest]
fn test_long_comparisons(mut harness: CompilerTest) {
    let source = r#"
    int main() {
        long a = 1000000000000l;
        long b = 1000000000001l;

        if (!(a < b)) return 1;
        if (!(b > a)) return 2;
        if (a >= b) return 3;
        if (b <= a) return 4;
        if (a == b) return 5;
        if (!(a != b)) return 6;

        // Test comparison with negative values
        long neg_a = -1000000000000l;
        long neg_b = -1000000000001l;

        if (!(neg_b < neg_a)) return 7;
        if (!(neg_a > neg_b)) return 8;

        return 0;
    }"#;
    harness.assert_runs_ok(source, 0);
}

#[rstest]
fn test_int_long_promotion(mut harness: CompilerTest) {
    let source = r#"
    int main() {
        int a = 1000000;
        long b = 1000000l;
        long result;

        // These should all promote to long
        result = a + b;
        if (result != 2000000l) return 1;

        result = a * b;
        if (result != 1000000000000l) return 2;

        // This should convert a to long before the shift
        result = a << 20;
        if (result != 603979776l) return 3;

        // Mixed expressions with operations
        result = (a * 2) + (b / 2);
        if (result != 2500000l) return 4;

        return 0;
    }"#;
    harness.assert_runs_ok(source, 0);
}

#[rstest]
fn test_function_returning_long(mut harness: CompilerTest) {
    let source = r#"
    long get_long() {
        return 1000000000000l;
    }

    long get_long_from_int(int a) {
        return (long)a;
    }

    long calculate_long(int a, long b) {
        return a * b + 42l;
    }

    int main() {
        if (get_long() != 1000000000000l) return 1;
        if (get_long_from_int(-5) != -5l) return 2;
        if (calculate_long(10, 20l) != 200l + 42l) return 3;
        return 0;
    }"#;
    harness.assert_runs_ok(source, 0);
}

#[rstest]
fn test_long_compound_assignments(mut harness: CompilerTest) {
    let source = r#"
    int main() {
        long a = 1000l;

        a += 2000l;
        if (a != 3000l) return 1;

        a -= 1000l;
        if (a != 2000l) return 2;

        a *= 10l;
        if (a != 20000l) return 3;

        a /= 5l;
        if (a != 4000l) return 4;

        a %= 3000l;
        if (a != 1000l) return 5;

        a <<= 10;
        if (a != 1024000l) return 6;

        a >>= 10;
        if (a != 1000l) return 7;

        a &= 1023l;
        if (a != 1000l) return 8;

        a |= 24l;
        if (a != 1016l) return 9;

        a ^= 1000l;
        if (a != 16l) return 10;

        return 0;
    }"#;
    harness.assert_runs_ok(source, 0);
}

#[rstest]
fn test_long_conditionals(mut harness: CompilerTest) {
    let source = r#"
    int main() {
        long zero = 0l;
        long non_zero = 1l;

        // Test if() with longs
        if (zero) return 1;
        if (!non_zero) return 2;

        // Test ternary operator with longs
        long result = non_zero ? 10l : 20l;
        if (result != 10l) return 3;

        result = zero ? 10l : 20l;
        if (result != 20l) return 4;

        // Test with large long values
        long big_num = 9223372036854775807l; // LONG_MAX
        if (!(big_num > 0)) return 5;

        // Test with comparison in condition
        if ((zero < non_zero) ? zero : non_zero) return 6;
        if (!((zero < non_zero) ? non_zero : zero)) return 7;

        return 0;
    }"#;
    harness.assert_runs_ok(source, 0);
}

#[rstest]
fn test_long_literal_edge_cases(mut harness: CompilerTest) {
    let source = r#"
    int main() {
        // Test large long values near MAX
        long big = 9223372036854775807l; // LONG_MAX

        // Test assignment from variables
        long big_copy = big;
        if (big_copy != 9223372036854775807l) return 1;

        // Test negating large values
        long negated = -big;
        if (negated != -9223372036854775807l) return 2;

        // Test calculations with large values (staying in bounds)
        long calc = big - 1000l;
        if (calc != 9223372036854774807l) return 3;

        // Test operations that cause value to wrap
        long plus_one = big;
        plus_one++;
        if (plus_one != -9223372036854775808l) return 4;

        return 0;
    }"#;
    harness.assert_runs_ok(source, 0);
}

#[rstest]
fn test_multiple_casts(mut harness: CompilerTest) {
    let source = r#"
    int main() {
        int i = 42;
        long l = 1000000000000l;

        // Multiple casts in one expression
        int result = (int)(long)(int)(long)i;
        if (result != 42) return 1;

        // Cast in complex expressions
        long complex = (long)i * l + ((long)i + l);
        if (complex != 42l * l + (42l + l)) return 2;

        // Nested casts
        int truncated = (int)((long)i * 100000000l);
        if (truncated != 42 * 100000000 % (1l << 32)) return 3;

        return 0;
    }"#;
    harness.assert_runs_ok(source, 0);
}
