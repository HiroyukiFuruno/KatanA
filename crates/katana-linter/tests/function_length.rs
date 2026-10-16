use katana_linter::Violation;
use katana_linter::rules::FunctionLengthOps;
use std::path::Path;

fn function_source(line_span: usize) -> String {
    format!(
        "fn example() {{\n{}}}",
        "    let _ = 0;\n".repeat(line_span - 1)
    )
}

fn lint(source: &str) -> Vec<Violation> {
    let syntax = syn::parse_file(source).expect("Valid Rust fixture");
    FunctionLengthOps::lint(Path::new("function_length_fixture.rs"), &syntax)
}

#[test]
fn ast_linter_function_length_accepts_thirty_line_span() {
    assert!(lint(&function_source(30)).is_empty());
}

#[test]
fn ast_linter_function_length_rejects_thirty_one_line_span() {
    let violations = lint(&function_source(31));
    assert_eq!(violations.len(), 1);
}

#[test]
fn ast_linter_function_length_reports_measured_span_and_name_location() {
    let violations = lint(&function_source(31));
    let violation = violations.first().expect("Long function violation");
    assert_eq!(
        (
            violation.file.as_path(),
            violation.line,
            violation.column,
            violation.message.as_str(),
        ),
        (
            Path::new("function_length_fixture.rs"),
            1,
            4,
            "Function `example` exceeds 30-line limit (current: 31). Extract helper methods.",
        )
    );
}

#[test]
fn ast_linter_function_length_checks_impl_methods() {
    let source = format!(
        "struct Example;\nimpl Example {{\n{}\n}}",
        function_source(31)
    );
    assert_eq!(lint(&source).len(), 1);
}

#[test]
fn ast_linter_function_length_checks_nested_functions_independently() {
    let source = format!("fn outer() {{\n{}\n}}", function_source(31));
    assert_eq!(lint(&source).len(), 2);
}

#[test]
fn ast_linter_function_length_does_not_count_signature_lines() {
    let source = function_source(30).replace("fn example()", "fn example(\n    _: (),\n)");
    assert!(lint(&source).is_empty());
}

#[test]
fn ast_linter_function_length_accepts_single_line_empty_function() {
    assert!(lint("fn example() {}").is_empty());
}
