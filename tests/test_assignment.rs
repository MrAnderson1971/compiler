// tests/test_assignment.rs
mod simulator;

use rstest::*;
use simulator::{CompilerTest, harness};
use compiler::CompilerError;
use std::i32;

#[rstest]
fn test_declaration(mut harness: CompilerTest) {
    let source = "int main() { int a = 5; return a; }";
    harness.assert_runs_ok(source, 5);
}

#[rstest]
fn test_declare_then_assign(mut harness: CompilerTest) {
    let source = "int main() { int a; a = 5; return a; }";
    harness.assert_runs_ok(source, 5);
}

#[rstest]
fn test_non_short_circuit(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            int a = 0;
            0 || (a = 1);
            return a;
        }
    "#;
    harness.assert_runs_ok(source, 1);
}

#[rstest]
fn test_short_circuit(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            int a = 42;
            1 || (a = 1);
            return a;
        }
    "#;
    harness.assert_runs_ok(source, 42);
}

#[rstest]
fn test_short_circuit2(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            int a = 42;
            0 && (a = 1);
            return a;
        }
    "#;
    harness.assert_runs_ok(source, 42);
}

#[rstest]
fn test_non_short_circuit2(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            int a = 0;
            1 && (a = 1);
            return a;
        }
    "#;
    harness.assert_runs_ok(source, 1);
}

#[rstest]
fn test_assignment_precedence(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            int a = 0;
            a = 1 + 2;
            return a;
        }
    "#;
    harness.assert_runs_ok(source, 3);
}

#[rstest]
fn test_variable_part_of_declaration(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            int a = 0 && a;
            return a;
        }
    "#;
    harness.assert_runs_ok(source, 0);
}

#[rstest]
fn test_mixed_precedence(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            int a = 1;
            int b = 0;
            a = 3 * (b = a);
            return a + b;
        }
    "#;
    harness.assert_runs_ok(source, 4);
}

#[rstest]
fn test_expression_then_declaration(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            int a = 999;
            a = a % 2;
            int b = -a;
            return b;
        }
    "#;
    harness.assert_runs_ok(source, -1);
}

#[rstest]
fn test_assign_to_return(harness: CompilerTest) {
    let source = "int main() { int return = 5; return return;}";
    assert_compile_err!(harness, source, CompilerError::SyntaxError(_));
}

#[rstest]
fn test_declaration_in_return(harness: CompilerTest) {
    let source = "int main() { return int a = 5; }";
    assert_compile_err!(harness, source, CompilerError::SyntaxError(_));
}

#[rstest]
fn test_bad_type(harness: CompilerTest) {
    let source = "int main() { ints a = 0; return a; }";
    assert_compile_err!(harness, source, CompilerError::SyntaxError(_));
}

#[rstest]
fn test_bad_precedence(mut harness: CompilerTest) {
    let source = "int main() { int a = 0; a = 3 * a + 1; return a; }";
    harness.assert_runs_ok(source, 1);
}

#[rstest]
fn test_undefined(harness: CompilerTest) {
    let source = "int main() { return a; }";
    assert_compile_err!(harness, source, CompilerError::SemanticError(_));
}

#[rstest]
fn test_assign_before_declare(harness: CompilerTest) {
    let source = "int main() { a = 5; int a; return a; }";
    assert_compile_err!(harness, source, CompilerError::SemanticError(_));
}

#[rstest]
fn test_duplicate_declaration(harness: CompilerTest) {
    let source = "int main() { int a = 1; int a = 2; return a; }";
    assert_compile_err!(harness, source, CompilerError::SemanticError(_));
}

#[rstest]
fn test_prefix_increment(mut harness: CompilerTest) {
    let source = "int main() { int a = 0; return ++a; }";
    harness.assert_runs_ok(source, 1);
}

#[rstest]
fn test_prefix_decrement(mut harness: CompilerTest) {
    let source = "int main() { int a = 0; return --a; }";
    harness.assert_runs_ok(source, -1);
}

#[rstest]
fn test_assignment_in_return(mut harness: CompilerTest) {
    let source = "int main() { int a = 0; return a = (a + 5); }";
    harness.assert_runs_ok(source, 5);
}

#[rstest]
fn test_complex_prefix_increment_decrement_and_assigns(mut harness: CompilerTest) {
    let source = "int main() { int a = 0; return a = ++a + a + a + --a; }";
    harness.assert_runs_ok(source, 3);
}

#[rstest]
fn test_invalid_prefix_increment(harness: CompilerTest) {
    let source = "int main() { int a = 0; return ++0; }";
    assert_compile_err!(harness, source, CompilerError::SemanticError(_));
}

#[rstest]
fn test_invalid_prefix_decrement(harness: CompilerTest) {
    let source = "int main() { int a = 0; return --0; }";
    assert_compile_err!(harness, source, CompilerError::SemanticError(_));
}

#[rstest]
fn test_postfix_increment(mut harness: CompilerTest) {
    let source = "int main() { int a = 0; return a++; }";
    harness.assert_runs_ok(source, 0);
}

#[rstest]
fn test_get_value_of_postfix_increment(mut harness: CompilerTest) {
    let source = "int main() { int a = 0; a++; return a; }";
    harness.assert_runs_ok(source, 1);
}

#[rstest]
fn test_postfix_decrement(mut harness: CompilerTest) {
    let source = "int main() { int a = 0; return a--; }";
    harness.assert_runs_ok(source, 0);
}

#[rstest]
fn test_get_value_of_postfix_decrement(mut harness: CompilerTest) {
    let source = "int main() { int a = 0; a--; return a; }";
    harness.assert_runs_ok(source, -1);
}

#[rstest]
fn test_invalid_postfix_increment(harness: CompilerTest) {
    let source = "int main() { return 0++; }";
    assert_compile_err!(harness, source, CompilerError::SemanticError(_));
}

#[rstest]
fn test_invalid_postfix_decrement(harness: CompilerTest) {
    let source = "int main() { return 0--; }";
    assert_compile_err!(harness, source, CompilerError::SemanticError(_));
}

#[rstest]
fn test_invalid_assign(harness: CompilerTest) {
    let source = "int main() { int a; 1 + (0 = 5); return 0; }";
    assert_compile_err!(harness, source, CompilerError::SemanticError(_));
}

#[rstest]
fn test_not_lvalue(harness: CompilerTest) {
    let source = "int main() { int a = 0; -a = 1; return a; }";
    assert_compile_err!(harness, source, CompilerError::SemanticError(_));
}

#[rstest]
fn test_compound_add(mut harness: CompilerTest) {
    let source = "int main() { int a = 0; a += 5; return a; }";
    harness.assert_runs_ok(source, 5);
}

#[rstest]
fn test_integer_overflow(mut harness: CompilerTest) {
    let source = format!("int main() {{ int a = {}; a += 1; return a; }}", i32::MAX);
    harness.assert_runs_ok(&*source, i32::MIN);
}

#[rstest]
fn test_chained_prefix_operators(mut harness: CompilerTest) {
    let source = "int main() { int a = 0; return ++(++a); }";
    harness.assert_runs_ok(source, 2);
}

#[rstest]
fn test_prefix_operators_in_expressions(mut harness: CompilerTest) {
    let source = "int main() { int a = 1; int b = 2; return ++a * ++b; }";
    harness.assert_runs_ok(source, 6);
}

#[rstest]
fn test_invalid_prefix_on_expressions(harness: CompilerTest) {
    let source = "int main() { int a = 1; int b = 2; return ++(a + b); }";
    assert_compile_err!(harness, source, CompilerError::SemanticError(_));
}

#[rstest]
fn test_prefix_with_assignment(harness: CompilerTest) {
    let source = "int main() { int a = 0; int b = ++(a = 5); return b; }";
    assert_compile_err!(harness, source, CompilerError::SemanticError(_));
}

#[rstest]
fn test_side_effects_with_prefix(mut harness: CompilerTest) {
    let source = "int main() { int a = 1; int b = ++a + ++a; return b; }";
    harness.assert_runs_ok(source, 6);
}

#[rstest]
fn test_chained_postfix_operators(harness: CompilerTest) {
    let source = "int main() { int a = 0; return (a++)++; }";
    assert_compile_err!(harness, source, CompilerError::SemanticError(_));
}

#[rstest]
fn test_postfix_in_complex_expressions(mut harness: CompilerTest) {
    let source = "int main() { int a = 1; int b = 2; return a++ * b++; }";
    harness.assert_runs_ok(source, 2);
}

#[rstest]
fn test_side_effects_with_postfix(mut harness: CompilerTest) {
    let source = "int main() { int a = 1; int b = a++ + a++; return b; }";
    harness.assert_runs_ok(source, 3);
}

#[rstest]
fn test_mixed_prefix_and_postfix(mut harness: CompilerTest) {
    let source = "int main() { int a = 5; return ++a + a++; }";
    harness.assert_runs_ok(source, 13);
}

#[rstest]
fn test_compound_subtract(mut harness: CompilerTest) {
    let source = "int main() { int a = 10; a -= 3; return a; }";
    harness.assert_runs_ok(source, 7);
}

#[rstest]
fn test_compound_multiply(mut harness: CompilerTest) {
    let source = "int main() { int a = 5; a *= 3; return a; }";
    harness.assert_runs_ok(source, 15);
}

#[rstest]
fn test_compound_divide(mut harness: CompilerTest) {
    let source = "int main() { int a = 10; a /= 2; return a; }";
    harness.assert_runs_ok(source, 5);
}

#[rstest]
fn test_compound_modulo(mut harness: CompilerTest) {
    let source = "int main() { int a = 10; a %= 3; return a; }";
    harness.assert_runs_ok(source, 1);
}

#[rstest]
fn test_compound_bitwise_and(mut harness: CompilerTest) {
    let source = "int main() { int a = 5; a &= 3; return a; }";
    harness.assert_runs_ok(source, 1);
}

#[rstest]
fn test_compound_bitwise_or(mut harness: CompilerTest) {
    let source = "int main() { int a = 5; a |= 2; return a; }";
    harness.assert_runs_ok(source, 7);
}

#[rstest]
fn test_compound_bitwise_xor(mut harness: CompilerTest) {
    let source = "int main() { int a = 5; a ^= 3; return a; }";
    harness.assert_runs_ok(source, 6);
}

#[rstest]
fn test_compound_left_shift(mut harness: CompilerTest) {
    let source = "int main() { int a = 5; a <<= 2; return a; }";
    harness.assert_runs_ok(source, 20);
}

#[rstest]
fn test_compound_right_shift(mut harness: CompilerTest) {
    let source = "int main() { int a = 20; a >>= 2; return a; }";
    harness.assert_runs_ok(source, 5);
}

#[rstest]
fn test_compound_assignments_as_expressions(mut harness: CompilerTest) {
    let source = "int main() { int a = 5; int b = 2; return (a += 3) * (b -= 1); }";
    harness.assert_runs_ok(source, 8);
}

#[rstest]
fn test_chained_compound_assignments(mut harness: CompilerTest) {
    let source = "int main() { int a = 0; int b = 2; int c = 3; a += b += c; return a; }";
    harness.assert_runs_ok(source, 5);
}

#[rstest]
fn test_invalid_compound_targets(harness: CompilerTest) {
    let source = "int main() { int a = 5; (a + 2) += 3; return a; }";
    assert_compile_err!(harness, source, CompilerError::SemanticError(_));
}

#[rstest]
fn test_prefix_with_compound_assignment(harness: CompilerTest) {
    let source = "int main() { int a = 1; return ++(a += 2); }";
    assert_compile_err!(harness, source, CompilerError::SemanticError(_));
}

#[rstest]
fn test_postfix_with_compound_assignment(harness: CompilerTest) {
    let source = "int main() { int a = 1; return (a += 2)++; }";
    assert_compile_err!(harness, source, CompilerError::SemanticError(_));
}

#[rstest]
fn test_prefix_in_compound_assignment(mut harness: CompilerTest) {
    let source = "int main() { int a = 1; int b = 2; a += ++b; return a; }";
    harness.assert_runs_ok(source, 4);
}

#[rstest]
fn test_postfix_in_compound_assignment(mut harness: CompilerTest) {
    let source = "int main() { int a = 1; int b = 2; a += b++; return a; }";
    harness.assert_runs_ok(source, 3);
}

#[rstest]
fn test_multiple_operations_in_one_statement(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            int a = 1;
            return a = ++a + a++ + (a += 2);
        }
    "#;
    harness.assert_runs_ok(source, 10);
}

#[rstest]
fn test_order_of_evaluation(harness: CompilerTest) {
    let source = "int main() { int a = 1; int b = 1; return (a += b) += ++b; }";
    assert_compile_err!(harness, source, CompilerError::SemanticError(_));
}

#[rstest]
fn test_nested_prefix_operators(mut harness: CompilerTest) {
    let source = "int main() { int a = 5; return ++(++a); }";
    harness.assert_runs_ok(source, 7);
}

#[rstest]
fn test_unary_plus_with_increment(mut harness: CompilerTest) {
    let source = "int main() { int a = 5; return +(+(++a)); }";
    harness.assert_runs_ok(source, 6);
}

#[rstest]
fn test_invalid_unary_plus_with_increment(harness: CompilerTest) {
    let source = "int main() { int a = 5; return (+a)++; }";
    assert_compile_err!(harness, source, CompilerError::SemanticError(_));
}

#[rstest]
fn test_increment_overflow(mut harness: CompilerTest) {
    let source = &format!("int main() {{ int a = {}; return ++a; }}", i32::MAX);
    harness.assert_runs_ok(source, i32::MIN);
}

#[rstest]
fn test_compound_add_overflow(mut harness: CompilerTest) {
    let source = &format!("int main() {{ int a = {}; a += 1; return a; }}", i32::MAX);
    harness.assert_runs_ok(source, i32::MIN);
}

#[rstest]
fn test_decrement_overflow(mut harness: CompilerTest) {
    let source = &format!("int main() {{ int a = {}; return --a; }}", i32::MIN);
    harness.assert_runs_ok(source, i32::MAX);
}

#[rstest]
fn test_compound_subtract_overflow(mut harness: CompilerTest) {
    let source = &format!("int main() {{ int a = {}; a -= 1; return a; }}", i32::MIN);
    harness.assert_runs_ok(source, i32::MAX);
}

#[rstest]
fn test_prefix_as_lvalue_for_compound_assign(mut harness: CompilerTest) {
    let source = "int main() { int a = 5; return ++a += 2; }";
    harness.assert_runs_ok(source, 8);
}