// tests/test_loops.rs
mod simulator;

use rstest::*;
use simulator::{CompilerTest, harness};
use compiler::CompilerError;

#[rstest]
fn test_while(mut harness: CompilerTest) {
    let code = r#"
        int main() {
            int i = 0;
            while (i < 10) {
                i = i + 1;
            }
            return i;
        }
    "#;
    harness.assert_runs_ok(code, 10);
}

#[rstest]
fn test_break(mut harness: CompilerTest) {
    let code = r#"
        int main() {
            int i = 0;
            while (i < 10) {
                if (i >= 5) {
                    break;
                }
                i = i + 1;
            }
            return i;
        }
    "#;
    harness.assert_runs_ok(code, 5);
}

#[rstest]
fn test_continue(mut harness: CompilerTest) {
    let code = r#"
        int main() {
            int i = 0;
            int result = 0;
            while (i < 10) {
                i = i + 1;
                if (i % 2 == 1) {
                    continue;
                }
                result += i;
            }
            return result;
        }
    "#;
    harness.assert_runs_ok(code, 30);
}

#[rstest]
fn test_break_outside_loop(harness: CompilerTest) {
    let code = r#"
        int main() {
            break;
            return 0;
        }
    "#;
    assert_compile_err!(harness, code, CompilerError::SemanticError(_));
}

#[rstest]
fn test_continue_outside_loop(harness: CompilerTest) {
    let code = r#"
        int main() {
            continue;
            return 0;
        }
    "#;
    assert_compile_err!(harness, code, CompilerError::SemanticError(_));
}

#[rstest]
fn test_while_without_body(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            int i = 0;
            while (++i && i < 10);
            return i;
        }
    "#;
    harness.assert_runs_ok(source, 10);
}

#[rstest]
fn test_for(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            int result = 0;
            for (int i = 0; i <= 10; i++) {
                result += i;
            }
            return result;
        }
    "#;
    harness.assert_runs_ok(source, 55);
}

#[rstest]
fn test_for_with_non_declaration_init(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            int result = 0;
            int i;
            for (i = 0; i <= 10; i++) {
                result += i;
            }
            return result;
        }
    "#;
    harness.assert_runs_ok(source, 55);
}

#[rstest]
fn test_continue_in_for(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            int result = 0;
            for (int i = 0; i <= 10; i++) {
                if (i % 2 == 1) {
                    continue;
                }
                result += i;
            }
            return result;
        }
    "#;
    harness.assert_runs_ok(source, 30);
}

#[rstest]
fn test_break_in_for(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            int result = 0;
            for (int i = 0; i <= 10; i++) {
                if (i >= 5) {
                    break;
                }
                result += i;
            }
            return result;
        }
    "#;
    harness.assert_runs_ok(source, 10);
}

#[rstest]
fn test_for_init_proper_scope(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            int i = -100;
            for (int i = 69420;;) {
                return i;
            }
            return i;
        }
    "#;
    harness.assert_runs_ok(source, 69420);
}

#[rstest]
fn test_for_init_proper_scope2(mut harness: CompilerTest) {
    let source = r#"
        int main() {
            int i = -100;
            for (int i = 69420; i < 69420 + 10; i++);
            return i;
        }
    "#;
    harness.assert_runs_ok(source, -100);
}

#[rstest]
fn test_nested_while_loops(mut harness: CompilerTest) {
    let code = r#"
        int main() {
            int i = 0;
            int j = 0;
            int sum = 0;

            while (i < 3) {
                j = 0;
                while (j < 4) {
                    sum += i * j;
                    j++;
                }
                i++;
            }
            return sum;
        }
    "#;
    harness.assert_runs_ok(code, 18);
}

#[rstest]
fn test_while_with_initially_false_condition(mut harness: CompilerTest) {
    let code = r#"
        int main() {
            int i = 10;
            while (i < 10) {
                i++;
            }
            return i;
        }
    "#;
    harness.assert_runs_ok(code, 10);
}

#[rstest]
fn test_while_with_complex_condition(mut harness: CompilerTest) {
    let code = r#"
        int main() {
            int i = 0;
            int j = 10;
            while (i < 5 && j > 5) {
                i++;
                j--;
            }
            return i * 100 + j;
        }
    "#;
    harness.assert_runs_ok(code, 505);
}

#[rstest]
fn test_break_in_nested_loops(mut harness: CompilerTest) {
    let code = r#"
        int main() {
            int i = 0;
            int j = 0;
            int sum = 0;

            while (i < 5) {
                j = 0;
                while (j < 5) {
                    sum++;
                    if (j == 2) {
                        break; // Should only break inner loop
                    }
                    j++;
                }
                if (i == 3) {
                    break; // Should break outer loop
                }
                i++;
            }
            return sum;
        }
    "#;
    harness.assert_runs_ok(code, 12);
}

#[rstest]
fn test_continue_in_nested_loops(mut harness: CompilerTest) {
    let code = r#"
        int main() {
            int i = 0;
            int sum = 0;

            while (i < 3) {
                i++;
                if (i == 2) {
                    continue; // Skip when i == 2
                }

                int j = 0;
                while (j < 3) {
                    j++;
                    if (j == 2) {
                        continue; // Skip when j == 2
                    }
                    sum += i * j;
                }
            }
            return sum;
        }
    "#;
    harness.assert_runs_ok(code, 16);
}

#[rstest]
fn test_for_with_all_parts_empty(mut harness: CompilerTest) {
    let code = r#"
        int main() {
            int i = 0;
            for (;;) {
                i++;
                if (i >= 10) {
                    break;
                }
            }
            return i;
        }
    "#;
    harness.assert_runs_ok(code, 10);
}

#[rstest]
fn test_for_with_empty_condition(mut harness: CompilerTest) {
    let code = r#"
        int main() {
            int i = 0;
            for (i = 0; ; i++) {
                if (i >= 10) {
                    break;
                }
            }
            return i;
        }
    "#;
    harness.assert_runs_ok(code, 10);
}

#[rstest]
fn test_for_with_empty_update(mut harness: CompilerTest) {
    let code = r#"
        int main() {
            int i = 0;
            for (i = 0; i < 10;) {
                i += 2; // Update inside the loop
            }
            return i;
        }
    "#;
    harness.assert_runs_ok(code, 10);
}

#[rstest]
fn test_for_with_initially_false_condition(mut harness: CompilerTest) {
    let code = r#"
        int main() {
            int counter = 0;
            for (int i = 10; i < 10; i++) {
                counter++;
            }
            return counter;
        }
    "#;
    harness.assert_runs_ok(code, 0);
}

#[rstest]
fn test_nested_for_loops(mut harness: CompilerTest) {
    let code = r#"
        int main() {
            int sum = 0;
            for (int i = 0; i < 3; i++) {
                for (int j = 0; j < 3; j++) {
                    sum += i * j;
                }
            }
            return sum;
        }
    "#;
    harness.assert_runs_ok(code, 9);
}

#[rstest]
fn test_for_with_complex_update(mut harness: CompilerTest) {
    let code = r#"
        int main() {
            int sum = 0;
            for (int i = 0; i < 10; i += 2) {
                sum += i;
            }
            return sum;
        }
    "#;
    harness.assert_runs_ok(code, 20);
}

#[rstest]
fn test_loop_variable_access_after_execution(mut harness: CompilerTest) {
    let code = r#"
        int main() {
            int sum = 0;
            int i;
            for (i = 0; i < 5; i++) {
                sum += i;
            }
            return i * 10 + sum;
        }
    "#;
    harness.assert_runs_ok(code, 60);
}