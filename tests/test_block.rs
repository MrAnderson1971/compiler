// tests/test_block.rs
mod simulator;

use rstest::*;
use simulator::{CompilerTest, harness};
use compiler::CompilerError;

#[rstest]
fn test_block(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            int a = 2;
            {
                int a = 3;
                int b = a + 2;
            }
            return a;
        }
    "#;
    harness.assert_runs_ok(source, 2);
}

#[rstest]
fn test_duplicate_in_same_block(harness: CompilerTest) {
    let source = r#"
        int main() {
            int a = 2;
            {
                int a = 3;
                int a = 4;
                int b = a + 2;
            }
            return a;
        }
    "#;
    assert_compile_err!(harness, source, CompilerError::SemanticError(_));
}

#[rstest]
fn test_if_block_scoping(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            int a = 2;
            if (1) {
                int a = 3;
                int b = a + 2;
            }
            return a;
        }
    "#;
    harness.assert_runs_ok(source, 2);
}

#[rstest]
fn test_if_else_block_scoping(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            int a = 2;
            int result = 0;
            if (0) {
                int a = 3;
                result = a;
            } else {
                int a = 4;
                result = a;
            }
            return result + a;
        }
    "#;
    harness.assert_runs_ok(source, 6);  // result=4 from else block, a=2 from outer scope
}

#[rstest]
fn test_if_else_if_else_scoping(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            int a = 5;
            int b = 0;

            if (0) {
                int a = 10;
                b = a;
            } else if (1) {
                int a = 20;
                b = a;
            } else {
                int a = 30;
                b = a;
            }

            return a + b;
        }
    "#;
    harness.assert_runs_ok(source, 25);  // a=5 from outer scope, b=20 from else-if block
}

#[rstest]
fn test_duplicate_in_if_block(harness: CompilerTest) {
    let source = r#"
        int main() {
            int a = 5;

            if (1) {
                int b = 10;
                int b = 20;  // Duplicate variable in same block
            }

            return a;
        }
    "#;
    assert_compile_err!(harness, source, CompilerError::SemanticError(_));
}

#[rstest]
fn test_duplicate_in_else_block(harness: CompilerTest) {
    let source = r#"
        int main() {
            int a = 5;

            if (0) {
                int b = 10;
            } else {
                int c = 15;
                int c = 25;  // Duplicate variable in same block
            }

            return a;
        }
    "#;
    assert_compile_err!(harness, source, CompilerError::SemanticError(_));
}

#[rstest]
fn test_nested_blocks_in_if(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            int x = 1;

            if (1) {
                int x = 2;
                {
                    int x = 3;
                    {
                        int x = 4;
                    }
                }
            }

            return x;
        }
    "#;
    harness.assert_runs_ok(source, 1);  // x=1 from outer scope
}

#[rstest]
fn test_complex_nested_scopes(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            int a = 1;
            int result = 0;

            if (1) {
                int a = 2;
                {
                    int a = 3;
                    if (1) {
                        int a = 4;
                        result = a;
                    } else {
                        result = a;
                    }
                }
            } else if (0) {
                int a = 5;
                result = a;
            } else {
                result = a;
            }

            return result;
        }
    "#;
    harness.assert_runs_ok(source, 4);  // result=4 from innermost if block
}

#[rstest]
fn test_variable_access_across_blocks(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            int a = 10;
            int b = 20;

            if (1) {
                int c = a + b;  // Access outer variables
                if (1) {
                    int d = c + a;  // Access variables from multiple outer scopes
                    return d;
                }
            }

            return 0;
        }
    "#;
    harness.assert_runs_ok(source, 40);  // d = c + a = (a + b) + a = 10 + 20 + 10 = 40
}

#[rstest]
fn test_multiple_variables_in_same_scope(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            int a = 5;
            int b = 10;
            int c = 15;

            if (1) {
                int a = 1;
                int b = 2;
                int c = 3;
                return a + b + c;
            }

            return a + b + c;
        }
    "#;
    harness.assert_runs_ok(source, 6);  // a=1, b=2, c=3 from if block
}

#[rstest]
fn test_unbalanced_braces_missing(harness: CompilerTest) {
    let source = r#"
        int main() {
            int a = 2;
            if (1) {
                int b = 3;
                // Missing closing brace
            return a;
        }
    "#;
    assert_compile_err!(harness, source, CompilerError::SyntaxError(_));
}

#[rstest]
fn test_unbalanced_braces_extra(harness: CompilerTest) {
    let source = r#"
        int main() {
            int a = 2;
            if (1) {
                int b = 3;
            } }  // Extra closing brace
            return a;
        }
    "#;
    assert_compile_err!(harness, source, CompilerError::SyntaxError(_));
}

#[rstest]
fn test_invalid_conditional_syntax(harness: CompilerTest) {
    let source = r#"
        int main() {
            int a = 2;
            if 1 {  // Missing parentheses
                a = 3;
            }
            return a;
        }
    "#;
    assert_compile_err!(harness, source, CompilerError::SyntaxError(_));
}

#[rstest]
fn test_use_after_scope_exit(harness: CompilerTest) {
    let source = r#"
        int main() {
            int a = 5;
            {
                int b = 10;
            }
            return a + b;  // b is not in scope here
        }
    "#;
    assert_compile_err!(harness, source, CompilerError::SemanticError(_));
}

#[rstest]
fn test_use_before_declaration(harness: CompilerTest) {
    let source = r#"
        int main() {
            int a = b + 5;  // Using b before it's declared
            int b = 10;
            return a;
        }
    "#;
    assert_compile_err!(harness, source, CompilerError::SemanticError(_));
}

#[rstest]
fn test_variable_from_if_block_used_outside(harness: CompilerTest) {
    let source = r#"
        int main() {
            int a = 5;
            if (1) {
                int b = 10;
            }
            return a + b;  // b is not in scope here
        }
    "#;
    assert_compile_err!(harness, source, CompilerError::SemanticError(_));
}

#[rstest]
fn test_variable_from_else_block_used_outside(harness: CompilerTest) {
    let source = r#"
        int main() {
            int a = 5;
            if (0) {
                int b = 10;
            } else {
                int c = 15;
            }
            return a + c;  // c is not in scope here
        }
    "#;
    assert_compile_err!(harness, source, CompilerError::SemanticError(_));
}

#[rstest]
fn test_access_across_branches(harness: CompilerTest) {
    let source = r#"
        int main() {
            int a = 5;
            if (1) {
                int b = 10;
            } else {
                return a + b;  // b is not in scope in this branch
            }
            return a;
        }
    "#;
    assert_compile_err!(harness, source, CompilerError::SemanticError(_));
}

#[rstest]
fn test_nested_scope_exit(harness: CompilerTest) {
    let source = r#"
        int main() {
            int a = 5;
            {
                int b = 10;
                {
                    int c = 15;
                }
                return a + b + c;  // c is not in scope here
            }
        }
    "#;
    assert_compile_err!(harness, source, CompilerError::SemanticError(_));
}

#[rstest]
fn test_if_condition_undeclared_variable(harness: CompilerTest) {
    let source = r#"
        int main() {
            if (x > 0) {  // x is not declared
                int a = 5;
            }
            return 0;
        }
    "#;
    assert_compile_err!(harness, source, CompilerError::SemanticError(_));
}

#[rstest]
fn test_multiple_scope_exits(harness: CompilerTest) {
    let source = r#"
        int main() {
            if (1) {
                int a = 5;
                if (1) {
                    int b = 10;
                }
                {
                    int c = 15;
                }
                return a + b + c;  // Both b and c are out of scope
            }
            return 0;
        }
    "#;
    assert_compile_err!(harness, source, CompilerError::SemanticError(_));
}

#[rstest]
fn test_different_scope(harness: CompilerTest) {
    let source = r#"
        int main() {
            {
                int a = 5;
            }
            {
                return a;
            }
        }
    "#;
    assert_compile_err!(harness, source, CompilerError::SemanticError(_));
}