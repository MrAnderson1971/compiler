// tests/test_condition.rs
mod simulator;

use rstest::*;
use simulator::{CompilerTest, harness};
use compiler::CompilerError;

#[rstest]
fn test_ternary(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            int a = 1;
            int b = 2;
            return a > b ? a : b;
        }
    "#;
    harness.assert_runs_ok(source, 2);
}

#[rstest]
fn test_ternary_other_side(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            int a = 1;
            int b = 2;
            return a < b ? a : b;
        }
    "#;
    harness.assert_runs_ok(source, 1);
}

#[rstest]
fn test_single_if_true(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            if (2 + 3 == 5) return 6;
        }
    "#;
    harness.assert_runs_ok(source, 6);
}

#[rstest]
fn test_single_if_false(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            if (2 + 3 == 6) return 6;
        }
    "#;
    harness.assert_runs_ok(source, 0);
}

#[rstest]
fn test_if_else_into_if(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            int a = 1;
            if (a) return 2;
            else return 3;
        }
    "#;
    harness.assert_runs_ok(source, 2);
}

#[rstest]
fn test_if_else_into_else(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            int a = 0;
            if (a) return 2;
            else return 3;
        }
    "#;
    harness.assert_runs_ok(source, 3);
}

#[rstest]
fn test_else_without_if(harness: CompilerTest) {
    let source = r#"
        int main() {
            else return 3;
    "#;
    assert_compile_err!(harness, source, CompilerError::SyntaxError(_));
}

#[rstest]
fn test_ternary_without_condition(harness: CompilerTest) {
    let source = r#"
        int main() {
            return ? 1 : 2;
        }
    "#;
    assert_compile_err!(harness, source, CompilerError::SyntaxError(_));
}

#[rstest]
fn test_ternary_without_first_case(harness: CompilerTest) {
    let source = r#"
        int main() {
            return ? : 2;
        }
    "#;
    assert_compile_err!(harness, source, CompilerError::SyntaxError(_));
}

#[rstest]
fn test_ternary_without_second_case(harness: CompilerTest) {
    let source = r#"
        int main() {
            return 0 ? 1 :;
        }
    "#;
    assert_compile_err!(harness, source, CompilerError::SyntaxError(_));
}

#[rstest]
fn test_ternary_without_question(harness: CompilerTest) {
    let source = r#"
        int main() {
            return 1 : 2;
        }
    "#;
    assert_compile_err!(harness, source, CompilerError::SyntaxError(_));
}

#[rstest]
fn test_ternary_without_colon(harness: CompilerTest) {
    let source = r#"
        int main() {
            return 0 ? 1  2;
        }
    "#;
    assert_compile_err!(harness, source, CompilerError::SyntaxError(_));
}

#[rstest]
fn test_if_empty_condition(harness: CompilerTest) {
    let source = r#"
        int main() {
            if () return 1;
        }
    "#;
    assert_compile_err!(harness, source, CompilerError::SyntaxError(_));
}

#[rstest]
fn test_if_empty_body(harness: CompilerTest) {
    let source = r#"
        int main() {
            if (1)
        }
    "#;
    assert_compile_err!(harness, source, CompilerError::SyntaxError(_));
}

#[rstest]
fn test_if_else_if_else(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            if (0) return 1;
            else if (1) return 2;
            else return 3;
        }
    "#;
    harness.assert_runs_ok(source, 2);
}

#[rstest]
fn test_nested_if(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            int a = 1;
            int b = 2;
            if (a < b)
                if (a > 0) return 10;
                else return 20;
            return 30;
        }
    "#;
    harness.assert_runs_ok(source, 10);
}

#[rstest]
fn test_nested_if_else(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            int a = 1;
            int b = 2;
            if (a > b)
                return 10;
            else
                if (a > 0) return 20;
                else return 30;
        }
    "#;
    harness.assert_runs_ok(source, 20);
}

#[rstest]
fn test_if_else_if_no_final_else(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            int a = 1;
            if (a > 2) return 10;
            else if (a > 0) return 20;
            return 30;
        }
    "#;
    harness.assert_runs_ok(source, 20);
}

#[rstest]
fn test_logical_and_in_condition(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            int a = 1;
            int b = 2;
            if (a > 0 && b > 0) return 10;
            return 20;
        }
    "#;
    harness.assert_runs_ok(source, 10);
}

#[rstest]
fn test_logical_or_in_condition(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            int a = 0;
            int b = 2;
            if (a > 0 || b > 0) return 10;
            return 20;
        }
    "#;
    harness.assert_runs_ok(source, 10);
}

#[rstest]
fn test_logical_not_in_condition(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            int a = 0;
            if (!a) return 10;
            return 20;
        }
    "#;
    harness.assert_runs_ok(source, 10);
}

#[rstest]
fn test_nested_ternary(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            int a = 1;
            int b = 2;
            int c = 3;
            return a > b ? a : (b > c ? b : c);
        }
    "#;
    harness.assert_runs_ok(source, 3);
}

#[rstest]
fn test_if_with_assignment(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            int a = 0;
            if (a < 1) a = 10;
            return a;
        }
    "#;
    harness.assert_runs_ok(source, 10);
}

#[rstest]
fn test_missing_parentheses_in_if(harness: CompilerTest) {
    let source = r#"
        int main() {
            if 1 > 0 return 10;
        }
    "#;
    assert_compile_err!(harness, source, CompilerError::SyntaxError(_));
}

#[rstest]
fn test_double_else(harness: CompilerTest) {
    let source = r#"
        int main() {
            if (1 > 0) return 10;
            else return 20;
            else return 30;
        }
    "#;
    assert_compile_err!(harness, source, CompilerError::SyntaxError(_));
}

#[rstest]
fn test_nested_if_without_statement(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            if (1 > 0)
                if (1 > 2)
            return 10;
        }
    "#;
    harness.assert_runs_ok(source, 0);
}

#[rstest]
fn test_else_if_without_condition(harness: CompilerTest) {
    let source = r#"
        int main() {
            if (1 > 0) return 10;
            else if return 20;
        }
    "#;
    assert_compile_err!(harness, source, CompilerError::SyntaxError(_));
}

#[rstest]
fn test_missing_semicolon_in_if(harness: CompilerTest) {
    let source = r#"
        int main() {
            int a = 0;
            if (1 > 0) a = 10
            return a;
        }
    "#;
    assert_compile_err!(harness, source, CompilerError::SyntaxError(_));
}

#[rstest]
fn test_if_with_multiple_statements(harness: CompilerTest) {
    let source = r#"
        int main() {
            if (1 > 0) int a = 10; return a;
        }
    "#;
    assert_compile_err!(harness, source, CompilerError::SyntaxError(_));
}

#[rstest]
fn test_ternary_in_condition(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            int a = 1;
            int b = 2;
            if (a < b ? 1 : 0) return 10;
            return 20;
        }
    "#;
    harness.assert_runs_ok(source, 10);
}

#[rstest]
fn test_chained_else_if(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            int a = 2;
            if (a > 3) return 10;
            else if (a > 2) return 20;
            else if (a > 1) return 30;
            else if (a > 0) return 40;
            else return 50;
        }
    "#;
    harness.assert_runs_ok(source, 30);
}