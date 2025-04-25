mod simulator;

use crate::simulator::{CompilerTest, harness};
use rstest::rstest;
use compiler::CompilerError::SyntaxError;

#[rstest]
fn test_static(mut harness: CompilerTest) {
    let source = r#"
    static int foo() {
        return 0;
    }
    int main() {
        return foo();
    }"#;
    harness.assert_runs_ok(source, 0);
}

#[rstest]
fn test_int_static(mut harness: CompilerTest) {
    let source = r#"
    int static foo() {
        return 0;
    }
    int main() {
        return foo();
    }"#;
    harness.assert_runs_ok(source, 0);
}

#[rstest]
fn test_static_in_function(mut harness: CompilerTest) {
    let source = r#"
    int foo() {
        static int bar = 0;
        bar += 1;
        return bar;
    }
    int main() {
        foo();
        return foo();
    }"#;
    harness.assert_runs_ok(source, 2);
}

#[rstest]
fn test_static_in_expression(mut harness: CompilerTest) {
    let source = r#"
    int main() {
    static int i = 2;
    static int j = 3;
    int cmp = i < j; // make sure rewrite cmpl j(%rip), i(%rip)

    if (!cmp)
        return 1;
    return 0;
}"#;
    harness.assert_runs_ok(source, 0);
}

#[rstest]
fn test_top_level_variable(mut harness: CompilerTest) {
    let source = r#"
    int a = 5;
    int main() {
        return a;
    }"#;
    harness.assert_runs_ok(source, 5);
}

#[rstest]
fn test_distinct_local_and_extern(mut harness: CompilerTest) {
    let source = r#"int a = 5;

int return_a() {
    return a;
}

int main() {
    int a = 3;
    {
        extern int a;
        if (a != 5)
            return 1;
        a = 4;
    }
    return a + return_a();
}"#;
    harness.assert_runs_ok(source, 7);
}

#[rstest]
fn test_static_variables_with_same_name_in_different_functions(mut harness: CompilerTest) {
    let source = r#"
    int foo() {
        static int bar = 1;
        return bar;
    }
    int baz() {
        static int bar = 2;
        bar += 1;
        return bar;
    }
    int main() {
        baz();
        return foo() + baz();
    }"#;
    harness.assert_runs_ok(source, 5);
}

#[rstest]
fn test_static_and_extern_variables_with_same_name(harness: CompilerTest) {
    let source = r#"
    static extern foo = 1;
    int main() {
        return foo;
        }"#;
    assert_compile_err!(harness, source, SyntaxError(_));
}
