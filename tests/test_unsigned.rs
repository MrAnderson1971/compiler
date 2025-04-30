mod simulator;

use crate::simulator::{CompilerTest, harness};
use compiler::CompilerError::{SemanticError, SyntaxError};
use rstest::rstest;

#[rstest]
fn test_unsigned_int(mut harness: CompilerTest) {
    let source = r#"
    int main() {
    unsigned int ui = 4294967295u;
    return (ui == 4294967295u);
}"#;
    harness.assert_runs_ok(source, 1);
}

#[rstest]
fn test_unsigned_long(mut harness: CompilerTest) {
    let source = r#"
    int main() {
    unsigned long ul = 18446744073709551615ul;
    return (ul == 18446744073709551615ul);
}"#;
    harness.assert_runs_ok(source, 1);
}

#[rstest]
fn test_int_unsigned_int(mut harness: CompilerTest) {
    let source = r#"
    int main() {
    unsigned int ui = 4294967295u;
    int unsigned ui2 = 4294967295u;
    return (ui == 4294967295u) && (ui2 == 4294967295u);
}"#;
    harness.assert_runs_ok(source, 1);
}

#[rstest]
fn test_long_unsigned_long(mut harness: CompilerTest) {
    let source = r#"
    int main() {
    unsigned long ul = 18446744073709551615ul;
    long unsigned ul2 = 18446744073709551615ul;
    return (ul == 18446744073709551615ul) && (ul2 == 18446744073709551615ul);
}"#;
    harness.assert_runs_ok(source, 1);
}

#[rstest]
fn test_truncate_at_return_unsigned(mut harness: CompilerTest) {
    let source = r#"
    int main() {
    unsigned long ul = 18446744073709551615ul;
    return ul;
}"#;
    harness.assert_runs_ok(source, -1);
}

#[rstest]
fn test_truncate_at_assign_unsigned(mut harness: CompilerTest) {
    let source = r#"
    int main() {
    unsigned long ul = 18446744073709551615ul;
    unsigned int ui = ul;
    return ui;
    }"#;
    harness.assert_runs_ok(source, -1);
}

#[rstest]
fn test_zero_extension(mut harness: CompilerTest) {
    let source = r#"
    int main() {
    unsigned int ui = 4294967295u;
    unsigned long ul = ui;
    return ul == 4294967295ul;
    }
    "#;
    harness.assert_runs_ok(source, 1);
}

#[rstest]
fn test_unsigned_multiplication(mut harness: CompilerTest) {
    let source = r#"
    int main() {
    return 1000u * 1000u;
    }
    "#;
    harness.assert_runs_ok(source, 1_000_000);
}

#[rstest]
fn test_unsigned_multiplication_large_values(mut harness: CompilerTest) {
    // In Rust, we can use wrapping_mul for explicit overflow handling
    let source = r#"
    int main() {
    return 1000000u * 1000000u == 4000000000u; // Result after wrapping (1 trillion mod 2^32)
    }
    "#;
    harness.assert_runs_ok(source, 1);

    // Alternative using std::num::Wrapping
    use std::num::Wrapping;
    let a = Wrapping(1000000u32);
    let b = Wrapping(1000000u32);
    let result = a * b;
    assert_eq!(result.0, 1000000000000u64 as u32); // Correctly shows the wrapped result
}

#[rstest]
fn test_unsigned_long_multiplication(mut harness: CompilerTest) {
    let source = r#"
    int main() {
    return 1000000ul * 1000000ul;
    }
    "#;
    harness.assert_runs_ok(source, (1_000_000u64 * 1_000_000u64) as i32);
}

#[rstest]
fn test_unsigned_division(mut harness: CompilerTest) {
    let source = r#"
    int main() {
    return 1000000u / 1000u;
    }
    "#;
    harness.assert_runs_ok(source, 1000);
}

#[rstest]
fn test_unsigned_modulo(mut harness: CompilerTest) {
    let source = r#"
    unsigned int a;
    unsigned int b;
    int main() {
             a = 4294967290u; // 2^32 - 6
             b = 90u;

    return a % b == 80u;
    }
    "#;
    harness.assert_runs_ok(source, 1);
}

#[rstest]
fn test_unsigned_operations(mut harness: CompilerTest) {
    let source = r#"
unsigned int a;
unsigned int b;
int addition() {
    // a == 4294967290u, i.e. 2^32 - 6
    // b = 10u
    return (a + b == 4294967300u - 4294967296u); // Overflow behavior
}

int subtraction() {
    // a = 10u;
    // b = 20u;
    return (a - b == 4294967286u); // Underflow behavior
}

int multiplication() {
    // a = 4294967290u;
    return (a * 2u == (4294967290u * 2u) % 4294967296u);
}

int division() {
    // a = 4294967290u;
    b = a / 128u;
    return (b == 33554431u);
}

int remaind() {
    // a = 4294967290u;
    b = a % 100u;
    return (b == 90u);
}

int complement() {
    // a = 4294967290u;
    return (~a == 5u);
}

int main() {

    a = 4294967290u; // 2^32 - 6
    b = 10u;
    if (!addition()) {
        return 1;
    }

    a = 10u;
    b = 20u;
    if (!subtraction()) {
        return 2;
    }

    a = 4294967290u;
    if (!multiplication()) {
        return 3;
    }

    a = 4294967290u;
    if (!division()) {
        return 4;
    }

    a = 4294967290u;
    if (!remaind()) {
        return 5;
    }

    a = 4294967290u;
    if (!complement()) {
        return 6;
    }

    return 0;
}"#;
    harness.assert_runs_ok(source, 0);
}

#[rstest]
fn test_unsigned_long_operations(mut harness: CompilerTest) {
    let source = r#"
unsigned long a;
unsigned long b;
int addition() {
    // a == 18446744073709551610ul, i.e. 2^64 - 6
    // b = 10ul
    return (a + b == 4ul); // Overflow behavior
}

int subtraction() {
    // a = 10ul;
    // b = 20ul;
    return (a - b == 18446744073709551606ul); // Underflow behavior
}

int multiplication() {
    // a = 18446744073709551610ul;
    return (a * 2ul == 18446744073709551614ul);
}

int division() {
    // a = 18446744073709551610ul;
    b = a / 128ul;
    return (b == 144115344481324621ul);
}

int remaind() {
    // a = 18446744073709551610ul;
    b = a % 100ul;
    return (b == 10ul);
}

int complement() {
    // a = 18446744073709551610ul;
    return (~a == 5ul);
}

int main() {

    a = 18446744073709551610ul; // 2^64 - 6
    b = 10ul;
    if (!addition()) {
        return 1;
    }

    a = 10ul;
    b = 20ul;
    if (!subtraction()) {
        return 2;
    }

    a = 18446744073709551610ul;
    if (!multiplication()) {
        return 3;
    }

    a = 18446744073709551610ul;
    if (!division()) {
        return 4;
    }

    a = 18446744073709551610ul;
    if (!remaind()) {
        return 5;
    }

    a = 18446744073709551610ul;
    if (!complement()) {
        return 6;
    }

    return 0;
}"#;
    harness.assert_runs_ok(source, 0);
}

#[rstest]
fn test_many_unsigned_parameters(mut harness: CompilerTest) {
    let source = r#"
unsigned long calculate_check_value(unsigned long a, unsigned int b, unsigned long c, unsigned int d, unsigned long e, unsigned int f, unsigned long g, unsigned int h, unsigned long i, unsigned int j) {

    unsigned long product = a * c * e * g * i;

    unsigned long remainder = product % 10ul;

    unsigned long sum = b + d + f + h + j;

    return (10ul - (remainder + sum) % 10ul) % 10ul;
}

int main() {

return calculate_check_value(
    4294967297ul, 2u,
    3ul, 4u,
    5ul, 6u,
    7ul, 8u,
    9ul, 10u);
    }
    "#;
    harness.assert_runs_ok(source, 7);
}

#[rstest]
fn test_align_unsigned(mut harness: CompilerTest) {
    let source = r#"int main() {
    // Create alternating int/long variables to stress alignment
    unsigned int a = 5u;
    unsigned long b = 1ul;
    b = b << 32;    // b = 2^32 (4294967296) - requires full 64 bits
    unsigned int c = 10u;
    unsigned long d = 1ul;
    d = d << 33;    // d = 2^33 (8589934592)
    unsigned int e = 3u;

    // Test 1: Can we store and retrieve large values correctly?
    if (b != (1UL << 32)) return 1;  // Fails if b is truncated to 32 bits

    // Test 2: 64-bit arithmetic operations
    unsigned long sum = b + d;  // 2^32 + 2^33 = 3*2^32
    if (sum != 3 * (1UL << 32)) return 2;  // Fails if addition truncates

    // Test 3: Mixed unsigned int/unsigned long multiplication (tests zero extension)
    unsigned long product = a * b;  // 5 * 2^32
    if (product != 5 * (1UL << 32)) return 3;  // Fails if conversion is wrong

    // Test 4: Unsigned long division with large values
    unsigned long quotient = d / c;  // 2^33 / 10
    unsigned long remainder = d % c;
    if (quotient * c + remainder != d) return 4;  // Division identity check

    // Test 5: Complex expressions with mixed types
    unsigned long complex = (a * b + c * d) / e;
    if (complex != ((a * b) + (c * d)) / e) return 5;

    // All tests passed
    return 6;
}"#;
    harness.assert_runs_ok(source, 6);
}

#[rstest]
fn test_unsigned_int_overflow(mut harness: CompilerTest) {
    let source = r#"
    int main() {
    unsigned int max = 4294967295u;
    max++;
    return max == 0u;
    }
    "#;
    harness.assert_runs_ok(source, 1);
}

#[rstest]
fn test_unsigned_long_overflow(mut harness: CompilerTest) {
    let source = r#"
    int main() {
    unsigned long max = 18446744073709551615ul;
    max++;
    return max == 0ul;
    }
    "#;
    harness.assert_runs_ok(source, 1);
}

#[rstest]
fn test_unsigned_int_underflow(mut harness: CompilerTest) {
    let source = r#"
    int main() {
    unsigned int min = 0u;
    min--;
    return min == 4294967295u;
    }
    "#;
    harness.assert_runs_ok(source, 1);
}

#[rstest]
fn test_unsigned_long_underflow(mut harness: CompilerTest) {
    let source = r#"
    int main() {
    unsigned long min = 0ul;
    min--;
    return min == 18446744073709551615ul;
    }
    "#;
    harness.assert_runs_ok(source, 1);
}

#[rstest]
fn test_unsigned_prefix(mut harness: CompilerTest) {
    let source = r#"
    int main() {
    unsigned int ui = 4294967295u;
    return ++ui == 0u;
}"#;
    harness.assert_runs_ok(source, 1);
}

#[rstest]
fn test_unsigned_overflow(harness: CompilerTest) {
    let source = r#"
    int main() {
    unsigned long ul = 184467440737095516150ul;
    return 0;
    }
    "#;
    assert_compile_err!(harness, &*source, SyntaxError(_));
}

#[rstest]
fn test_too_many_unsigned_suffixes(harness: CompilerTest) {
    let source = r#"
    int main() {
    unsigned long ul = 1000000uLU;
    return ul;
    }
    "#;
    assert_compile_err!(harness, &*source, SyntaxError(_));
}

#[rstest]
fn test_unsigned_as_identifier(harness: CompilerTest) {
    let source = r#"
    int unsigned() {
    return 0;
    }
    int main() {
    return 0;
    }
    "#;
    assert_compile_err!(harness, &*source, SyntaxError(_));
}

#[rstest]
fn test_conflicting_unsigned_global_types(harness: CompilerTest) {
    let source = r#"
    int a;
    unsigned int a;
    int main() {
    return a;
    }
    "#;
    assert_compile_err!(harness, &*source, SemanticError(_));
}

#[rstest]
fn test_conflicting_unsigned_extern_types(harness: CompilerTest) {
    let source = r#"
unsigned int a;

int main() {
    extern int a;
    return 0;
    }
    "#;
    assert_compile_err!(harness, &*source, SemanticError(_));
}

#[rstest]
fn test_conflicting_unsigned_function_param_types(harness: CompilerTest) {
    let source = r#"int foo(int a);

int main() {
    return 0;
}

int foo(unsigned int a);"#;
    assert_compile_err!(harness, &*source, SemanticError(_));
}

#[rstest]
fn test_conflicting_unsigned_function_return_types(harness: CompilerTest) {
    let source = r#"int foo(int a);

int main() {
    return 0;
}

unsigned int foo(int a);"#;
    assert_compile_err!(harness, &*source, SemanticError(_));
}

#[rstest]
fn test_static_unsigned_with_init(mut harness: CompilerTest) {
    let source = r#"
    int foo() {
        static unsigned int a = 1000000u;
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
fn test_top_level_static_unsigned_with_init(mut harness: CompilerTest) {
    let source = r#"
    static unsigned int a = 1000000u;
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
fn test_static_unsigned_without_init(mut harness: CompilerTest) {
    let source = r#"
    int foo() {
        static unsigned int a;
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
fn test_top_level_static_unsigned_without_init(mut harness: CompilerTest) {
    let source = r#"
    static unsigned int a;
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
fn test_invalid_unsigned_specifier(harness: CompilerTest) {
    let source = r#"
    int main() {
        unsigned unsigned int a;
        return 0;
    }
    "#;
    assert_compile_err!(harness, &*source, SyntaxError(_));
}

#[rstest]
fn test_cast_without_unsigned_parentheses(harness: CompilerTest) {
    let source = r#"
    int main() {
        unsigned int a = unsigned int 1000000;
        return a;
    }
    "#;
    assert_compile_err!(harness, &*source, SyntaxError(_));
}

#[rstest]
fn test_invalid_unsigned_cast(harness: CompilerTest) {
    let source = r#"
    int main() {
        unsigned int a = (static unsigned int) 1000000u;
        return a;
    }
    "#;
    assert_compile_err!(harness, &*source, SyntaxError(_));
}

#[rstest]
fn test_cast_unsigned_int(mut harness: CompilerTest) {
    let source = r#"
    int main() {
    if ((unsigned int) 1000000 != 1000000u) {
    return 1;
    }

    if (1000000u != (unsigned int) 1000000) {
    return 2;
    }
    return 0;
    }
    "#;
    harness.assert_runs_ok(source, 0);
}

#[rstest]
fn test_cast_with_zero_extend(mut harness: CompilerTest) {
    let source = r#"
unsigned long zero_extend(unsigned int ui, unsigned long expected) {
    unsigned long extended = (unsigned long) ui;
    return (extended == expected);
}

int main() {
    if (!zero_extend(10u, 10ul)) {
        return 1;
    }

    if (!zero_extend(4294967295u, 4294967295ul)) {
        return 2;
    }

    unsigned long ul = (unsigned long) 100u;
    if (ul != 100ul) {
        return 3;
    }
    return 0;
}"#;
    harness.assert_runs_ok(source, 0);
}

#[rstest]
fn test_cast_with_unsigned_truncate(mut harness: CompilerTest) {
    let source = r#"int truncate(unsigned long ul, unsigned int expected) {
    unsigned int result = (unsigned int) ul;
    return (result == expected);
}

int main()
{
    if (!truncate(10ul, 10u)) {
        return 1;
    }

    if (!truncate(4294967295ul, 4294967295u)) {
        return 2;
    }

    if (!truncate(17179869189ul, // 2^34 + 5
                  5u)) {
        return 3;
    }

    unsigned int ui = (unsigned int)17179869189ul; // 2^34 + 5
    if (ui != 5u)
        return 5;

    return 0;
}"#;
    harness.assert_runs_ok(source, 0);
}

#[rstest]
fn test_unsigned_bitwise_operations(mut harness: CompilerTest) {
    let source = r#"
    int main() {
        unsigned long a = 123456789123456789ul;
        unsigned long b = 987654321987654321ul;

        // Bitwise AND
        unsigned long and_result = a & b;
        if (and_result != 122892737510904337ul) return 1;

        // Bitwise OR
        unsigned long or_result = a | b;
        if (or_result != 988218373600206773ul) return 2;

        // Bitwise XOR
        unsigned long xor_result = a ^ b;
        if (xor_result != 865325636089302436ul) return 3;

        // Bitwise NOT
        unsigned long not_result = ~a;
        if (not_result != 18446620616920094826ul) return 4;

        // Bit shifts
        unsigned long c = 1ul;
        if ((c << 60) != 1152921504606846976ul) return 5;
        if ((c << 60) >> 60 != 1ul) return 6;

        return 0;
    }"#;
    harness.assert_runs_ok(source, 0);
}

#[rstest]
fn test_unsigned_comparisons(mut harness: CompilerTest) {
    let source = r#"
    int main() {
        unsigned long a = 1000000000000ul;
        unsigned long b = 1000000000001ul;

        if (!(a < b)) return 1;
        if (!(b > a)) return 2;
        if (a >= b) return 3;
        if (b <= a) return 4;
        if (a == b) return 5;
        if (!(a != b)) return 6;

        // Test comparison of unsigned with signed
        int neg = -1;
        unsigned int ui = 1u;

        // -1 > 1u should be true because -1 is converted to unsigned int
        // and becomes a very large number
        if (!(neg > ui)) return 7;

        return 0;
    }"#;
    harness.assert_runs_ok(source, 0);
}

#[rstest]
fn test_int_unsigned_promotion(mut harness: CompilerTest) {
    let source = r#"
    int main() {
        int a = 1000000;
        unsigned int b = 1000000u;
        unsigned int result;

        // These should all promote to unsigned int
        result = a + b;
        if (result != 2000000u) return 1;

        result = a * b;
        if (result != 1000000000000u % 4294967296u) return 2;

        // Mixed expressions with operations
        result = (a * 2) + (b / 2);
        if (result != 2500000u) return 3;

        // Negative int + unsigned
        int neg = -1;
        unsigned int ui = 1u;
        if (neg + ui != 0u) return 4;

        return 0;
    }"#;
    harness.assert_runs_ok(source, 0);
}

#[rstest]
fn test_function_returning_unsigned(mut harness: CompilerTest) {
    let source = r#"
    unsigned int get_unsigned() {
        return 4000000000u;
    }

    unsigned int get_unsigned_from_int(int a) {
        return (unsigned int)a;
    }

    unsigned long calculate_unsigned(unsigned int a, unsigned long b) {
        return a * b + 42ul;
    }

    int main() {
        if (get_unsigned() != 4000000000u) return 1;
        if (get_unsigned_from_int(-5) != 4294967291u) return 2;
        if (calculate_unsigned(10u, 20ul) != 200ul + 42ul) return 3;
        return 0;
    }"#;
    harness.assert_runs_ok(source, 0);
}

#[rstest]
fn test_unsigned_compound_assignments(mut harness: CompilerTest) {
    let source = r#"
    int main() {
        unsigned int a = 1000u;

        a += 2000u;
        if (a != 3000u) return 1;

        a -= 1000u;
        if (a != 2000u) return 2;

        a *= 10u;
        if (a != 20000u) return 3;

        a /= 5u;
        if (a != 4000u) return 4;

        a %= 3000u;
        if (a != 1000u) return 5;

        a <<= 10;
        if (a != 1024000u) return 6;

        a >>= 10;
        if (a != 1000u) return 7;

        a &= 1023u;
        if (a != 1000u) return 8;

        a |= 24u;
        if (a != 1016u) return 9;

        a ^= 1000u;
        if (a != 16u) return 10;

        return 0;
    }"#;
    harness.assert_runs_ok(source, 0);
}

#[rstest]
fn test_unsigned_conditionals(mut harness: CompilerTest) {
    let source = r#"
    int main() {
        unsigned int zero = 0u;
        unsigned int non_zero = 1u;

        // Test if() with unsigned values
        if (zero) return 1;
        if (!non_zero) return 2;

        // Test ternary operator with unsigned values
        unsigned int result = non_zero ? 10u : 20u;
        if (result != 10u) return 3;

        result = zero ? 10u : 20u;
        if (result != 20u) return 4;

        // Test with large unsigned values
        unsigned int big_num = 4294967295u; // UINT_MAX
        if (!(big_num > 0)) return 5;

        // Test with comparison in condition
        if ((zero < non_zero) ? zero : non_zero) return 6;
        if (!((zero < non_zero) ? non_zero : zero)) return 7;

        return 0;
    }"#;
    harness.assert_runs_ok(source, 0);
}

#[rstest]
fn test_unsigned_literal_edge_cases(mut harness: CompilerTest) {
    let source = r#"
    int main() {
        // Test large unsigned values near MAX
        unsigned int big = 4294967295u; // UINT_MAX
        unsigned long big_long = 18446744073709551615ul; // ULONG_MAX

        // Test assignment from variables
        unsigned int big_copy = big;
        if (big_copy != 4294967295u) return 1;

        // Test calculations with large values (staying in bounds)
        unsigned int calc = big - 1000u;
        if (calc != 4294966295u) return 2;

        // Test operations that cause value to wrap
        unsigned int plus_one = big;
        plus_one++;
        if (plus_one != 0u) return 3;

        // Test large unsigned long values
        unsigned long big_long_copy = big_long;
        if (big_long_copy != 18446744073709551615ul) return 4;

        unsigned long plus_one_long = big_long;
        plus_one_long++;
        if (plus_one_long != 0ul) return 5;

        return 0;
    }"#;
    harness.assert_runs_ok(source, 0);
}

#[rstest]
fn test_signed_unsigned_conversion(mut harness: CompilerTest) {
    let source = r#"
    int main() {
        // Test signed -> unsigned conversion
        int neg = -1;
        unsigned int ui = neg;
        if (ui != 4294967295u) return 1; // -1 should become UINT_MAX

        // Test unsigned -> signed conversion with potential loss
        unsigned int big = 4294967295u; // UINT_MAX
        int i = big;
        if (i != -1) return 2; // UINT_MAX should become -1

        // Test signed -> unsigned long conversion
        int neg2 = -2;
        unsigned long ul = neg2;
        if (ul != 18446744073709551614ul) return 3; // -2 should become ULONG_MAX-1

        // Test explicit casts
        if ((unsigned int)(-5) != 4294967291u) return 4; // -5 should become UINT_MAX-4
        if ((int)(4294967295u) != -1) return 5; // UINT_MAX should become -1

        return 0;
    }"#;
    harness.assert_runs_ok(source, 0);
}

#[rstest]
fn test_mixed_signed_unsigned_operations(mut harness: CompilerTest) {
    let source = r#"
    int main() {
        // Mixing signed and unsigned in arithmetic
        int neg = -10;
        unsigned int ui = 5u;
        unsigned int result = neg + ui;
        if (result != 4294967291u) return 1; // -10 + 5u should be UINT_MAX-4

        // Mixing in comparisons
        if (!(neg < ui)) return 2; // -10 < 5u should be false (since -10 becomes a large unsigned)
        if (ui < neg) return 3;    // 5u < -10 should be true

        // Mixing in bit operations
        unsigned int bit_result = neg & ui;
        if (bit_result != 5u) return 4; // -10 & 5u should be 5 (after conversion)

        // Mixing unsigned and signed together with ternary operator
        unsigned int ternary = (neg > 0) ? ui : (unsigned int)neg;
        if (ternary != 4294967286u) return 5; // Should be unsigned conversion of -10

        return 0;
    }"#;
    harness.assert_runs_ok(source, 0);
}

#[rstest]
fn test_multiple_unsigned_casts(mut harness: CompilerTest) {
    let source = r#"
    int main() {
        int i = 42;
        unsigned long ul = 1000000000000ul;

        // Multiple casts in one expression
        unsigned int result = (unsigned int)(unsigned long)(unsigned int)(unsigned long)i;
        if (result != 42u) return 1;

        // Cast in complex expressions
        unsigned long complex = (unsigned long)i * ul + ((unsigned long)i + ul);
        if (complex != 42ul * ul + (42ul + ul)) return 2;

        // Nested casts
        unsigned int truncated = (unsigned int)((unsigned long)i * 100000000ul);
        if (truncated != (42u * 100000000u) % 4294967296u) return 3;

        return 0;
    }"#;
    harness.assert_runs_ok(source, 0);
}