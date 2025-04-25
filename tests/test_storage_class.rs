mod simulator;

use crate::simulator::{CompilerTest, harness};
use compiler::CompilerError::{SemanticError, SyntaxError};
use compiler::compile;
use rstest::rstest;

#[rstest]
fn test_static(mut harness: CompilerTest) {
    let source = r#"
    static int foo() {
        return 0;
    }
    int main() {
        return foo();
    }"#;
    let asm = compile(source.to_string()).unwrap();
    harness.assert_isnt_global(&*asm, "foo");
    harness.assert_is_global(&*asm, "main");
    assert_eq!(harness.load_and_run_asm(&*asm), 0);
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
    let asm = compile(source.to_string()).unwrap();
    harness.assert_isnt_global(&*asm, "foo");
    assert_eq!(harness.load_and_run_asm(&*asm), 0);
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
    let asm = compile(source.parse().unwrap()).unwrap();
    harness.assert_is_global(&*asm, "a");
    assert_eq!(harness.load_and_run_asm(&*asm), 5);
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
    let asm = compile(source.parse().unwrap()).unwrap();
    harness.assert_is_global(&*asm, "a");
    harness.assert_is_global(&*asm, "return_a");
    assert_eq!(harness.load_and_run_asm(&*asm), 7);
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
    let asm = compile(source.parse().unwrap()).unwrap();
    harness.assert_is_global(&*asm, "foo");
    harness.assert_is_global(&*asm, "baz");
    assert_eq!(harness.load_and_run_asm(&*asm), 5);
}

#[rstest]
fn test_static_and_extern_variable(harness: CompilerTest) {
    let source = r#"
    static extern foo = 1;
    int main() {
        return foo;
        }"#;
    assert_compile_err!(harness, source, SyntaxError(_));
}

#[rstest]
fn test_static_and_extern_function(harness: CompilerTest) {
    let source = r#"
    static int extern foo(void) {
    return 0;
}

int main(void) {
    return foo();
}"#;
    assert_compile_err!(harness, source, SyntaxError(_));
}

#[rstest]
fn test_extern_in_param(harness: CompilerTest) {
    let source = r#"
    int foo(extern int a) {
    return a;
    }
    int main() {
    return foo(1);
    }"#;
    assert_compile_err!(harness, source, SyntaxError(_));
}

#[rstest]
fn test_static_in_param(harness: CompilerTest) {
    let source = r#"
    int foo(static int a) {
    return a;
    }
    int main() {
    return foo(1);
    }"#;
    assert_compile_err!(harness, source, SyntaxError(_));
}

#[rstest]
fn test_duplicate_static(harness: CompilerTest) {
    let source = r#"
    int main() {
    static int a;
    int a;
    return a;
    }
    "#;
    assert_compile_err!(harness, source, SemanticError(_));
}

#[rstest]
fn test_duplicate_static2(harness: CompilerTest) {
    let source = r#"
    int main() {
        int a;
        static int a;
        return a;
    }
    "#;
    assert_compile_err!(harness, source, SemanticError(_));
}

#[rstest]
fn test_duplicate_extern(harness: CompilerTest) {
    let source = r#"
    int main() {
        extern int a;
        int a;
        return a;
    }
    "#;
    assert_compile_err!(harness, source, SemanticError(_));
}

#[rstest]
fn test_duplicate_extern2(harness: CompilerTest) {
    let source = r#"
    int main() {
        int a;
        extern int a;
        return a;
    }
    "#;
    assert_compile_err!(harness, source, SemanticError(_));
}

#[rstest]
fn test_tentative_definition(mut harness: CompilerTest) {
    let source = r#"
    int a; // Tentative definition (initialized to 0)
    int main() {
        return a;
    }"#;
    harness.assert_runs_ok(source, 0);
}

#[rstest]
fn test_multiple_tentative_definitions(mut harness: CompilerTest) {
    let source = r#"
    int a;
    int a;
    int a; // Multiple tentative definitions are valid
    int main() {
        return a;
    }"#;
    harness.assert_runs_ok(source, 0);
}

#[rstest]
fn test_tentative_then_explicit(mut harness: CompilerTest) {
    let source = r#"
    int a;           // Tentative definition
    int main() {
        return a;
    }
    int a = 42;      // Explicit definition (takes precedence)
    "#;
    harness.assert_runs_ok(source, 42);
}

#[rstest]
fn test_static_no_initializer(mut harness: CompilerTest) {
    let source = r#"
    int main() {
        static int a; // Should be initialized to 0
        return a;
    }"#;
    let asm = compile(source.parse().unwrap()).unwrap();
    harness.assert_isnt_global(&*asm, "a");
    assert_eq!(harness.load_and_run_asm(&*asm), 0);
}

#[rstest]
fn test_extern_with_initializer_file_scope(mut harness: CompilerTest) {
    let source = r#"
    extern int a = 10; // Valid at file scope
    int main() {
        return a;
    }"#;
    let asm = compile(source.parse().unwrap()).unwrap();
    harness.assert_is_global(&*asm, "a");
    assert_eq!(harness.load_and_run_asm(&*asm), 10);
}

#[rstest]
fn test_extern_with_initializer_block_scope(harness: CompilerTest) {
    let source = r#"
    int main() {
        extern int a = 10; // Error: extern with initializer at block scope
        return a;
    }"#;
    assert_compile_err!(harness, source, SemanticError(_));
}

#[rstest]
fn test_static_in_for_loop(harness: CompilerTest) {
    let source = r#"
    int main() {
        for (static int i = 0; i < 10; i++) {
            // Empty
        }
        return 0;
    }"#;
    assert_compile_err!(harness, source, SyntaxError(_));
}
