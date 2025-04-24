mod simulator;

use crate::simulator::{CompilerTest, harness};
use compiler::CompilerError;
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

#[rstest]
fn test_duplicate_definition(harness: CompilerTest) {
    let source = r#"
    int foo(int a) {
    return a;
    }

    int foo(int a) {
    return a;
    }

    int main () {
    return foo(1);
    }
    "#;
    assert_compile_err!(harness, source, CompilerError::SemanticError(_));
}

#[rstest]
fn test_two_functions(mut harness: CompilerTest) {
    let source = r#"
    int foo(int a) {
    return a;
    }

    int bar(int b) {
    return foo(b);
    }

    int main() {
    return foo(1);
    }
    "#;
    harness.assert_runs_ok(source, 1);
}

#[rstest]
fn test_nested_calls(mut harness: CompilerTest) {
    let source = r#"
    int foo(int a) {
    return 2 * a;
    }
    int main() {
    return foo(foo(1));
    }
    "#;
    harness.assert_runs_ok(source, 4);
}

#[rstest]
fn test_undefined(harness: CompilerTest) {
    let source = r#"
    int main() {
    return foo(1);
    }
    "#;
    assert_compile_err!(harness, source, CompilerError::SemanticError(_));
}

#[rstest]
fn test_many_parameters(mut harness: CompilerTest) {
    let source = r#"
    int calculate_check_digit(int a, int b, int c, int d, int e, int f, int g, int h, int i, int j) {
        int sum = 3*a + b + 3*c + d + 3*e + f + 3*g + h + 3*i + j;
        return (10 - (sum % 10)) % 10;
    }

    int main() {
        return calculate_check_digit(1, 2, 3, 4, 5, 6, 7, 8, 9, 0);
    }
    "#;
    harness.assert_runs_ok(source, 5);
}

#[rstest]
fn test_passing_too_few_args(harness: CompilerTest) {
    let source = r#"
    int foo(int a) {
    return a;
    }
    int main() {
    return foo();
    }
    "#;
    assert_compile_err!(harness, source, CompilerError::SemanticError(_));
}

#[rstest]
fn test_passing_too_many_args(harness: CompilerTest) {
    let source = r#"
    int foo(int a) {
    return a;
    }
    int main() {
    return foo(1, 2);
}"#;
    assert_compile_err!(harness, source, CompilerError::SemanticError(_));
}

#[rstest]
fn test_recursion(mut harness: CompilerTest) {
    let source = r#"
    int factorial(int n) {
        if (n <= 1) {
            return 1;
        }
        return n * factorial(n - 1);
    }

    int main() {
        return factorial(5);
    }
    "#;
    harness.assert_runs_ok(source, 120);
}

#[rstest]
fn test_mutual_recursion(mut harness: CompilerTest) {
    let source = r#"
    int is_even(int n);

    int is_odd(int n) {
        if (n == 0) return 0;
        return is_even(n - 1);
    }

    int is_even(int n) {
        if (n == 0) return 1;
        return is_odd(n - 1);
    }

    int main() {
        return is_even(10);
    }
    "#;
    harness.assert_runs_ok(source, 1);
}

#[rstest]
fn test_forward_reference(mut harness: CompilerTest) {
    let source = r#"
    int helper();

    int main() {
        return helper();
    }

    int helper() {
        return 42;
    }
    "#;
    harness.assert_runs_ok(source, 42);
}

#[rstest]
fn test_inner_function(harness: CompilerTest) {
    let source = r#"
    int foo(int a) {
        int bar(int b) {
            return a + b;
        }
        return bar(1);
    }
    int main() {
        return foo(1);
    }
    "#;
    assert_compile_err!(harness, source, CompilerError::SemanticError(_));
}

#[rstest]
fn test_duplicate_forward_reference(mut harness: CompilerTest) {
    let source = r#"
    int helper();
    int helper();

    int main() {
        return helper();
    }

    int helper() {
        return 42;
    }
    "#;
    harness.assert_runs_ok(source, 42);
}

#[rstest]
fn test_function_declaration_in_for_loop(harness: CompilerTest) {
    let source = r#"
int main() {
    for (int f(); ; ) {
        return 0;
    }"#;
    assert_compile_err!(harness, source, CompilerError::SyntaxError(_));
}

#[rstest]
fn test_assign_variable_to_function(harness: CompilerTest) {
    let source = r#"
            int f() {
            return 0;
        }
    int main() {

        f = 1;
        return 0;
    }"#;
    assert_compile_err!(harness, source, CompilerError::SemanticError(_));
}

#[rstest]
fn test_assignment_in_param(harness: CompilerTest) {
    let source = r#"
    int foo(int a = 2) {
        a = 1;
        return a;
    }
    int main() {
        return foo(1);
    }"#;
    assert_compile_err!(harness, source, CompilerError::SyntaxError(_));
}
