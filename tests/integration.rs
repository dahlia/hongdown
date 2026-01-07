//! Integration tests for Hongdown formatter.

use hongdown::{Options, format};

/// Test that formatting is idempotent (formatting twice produces same result).
#[test]
fn test_idempotent_formatting() {
    let input = r#"# Title

This is a paragraph with some **bold** and *italic* text.

## Section

 -  List item one
 -  List item two

~~~~ rust
fn main() {}
~~~~
"#;

    let options = Options::default();
    let first_pass = format(input, &options).unwrap();
    let second_pass = format(&first_pass, &options).unwrap();

    assert_eq!(first_pass, second_pass, "Formatting should be idempotent");
}

/// Test formatting a complete document with various elements.
#[test]
fn test_complete_document() {
    let input = r#"# Document Title

This is the introduction paragraph.

## First Section

Here is some content with *emphasis* and **strong** text.

 -  First item
 -  Second item
 -  Third item

### Subsection

> This is a block quote.

~~~~ python
def hello():
    print("Hello!")
~~~~

## Second Section

Visit [Rust](https://www.rust-lang.org/) for more info.
"#;

    let options = Options::default();
    let result = format(input, &options).unwrap();

    // Verify key formatting rules
    assert!(result.contains("Document Title\n="));
    assert!(result.contains("First Section\n-"));
    assert!(result.contains("### Subsection"));
    assert!(result.contains(" -  First item"));
    assert!(result.contains("~~~~ python"));
}

/// Test that inline code is not broken across lines.
#[test]
fn test_inline_code_not_broken() {
    let input = "This is a paragraph with `some_very_long_function_name_that_should_not_be_broken()` inline code.";
    let options = Options {
        line_width: 40,
        ..Options::default()
    };
    let result = format(input, &options).unwrap();

    // The inline code should appear intact on some line
    assert!(
        result.contains("`some_very_long_function_name_that_should_not_be_broken()`"),
        "Inline code should not be broken"
    );
}

/// Test heading underline length matches heading text.
#[test]
fn test_heading_underline_length() {
    let input = "# Short";
    let options = Options::default();
    let result = format(input, &options).unwrap();

    let lines: Vec<&str> = result.lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "Short");
    assert_eq!(lines[1], "=====");
    assert_eq!(lines[0].len(), lines[1].len());
}

/// Test ordered list numbering.
#[test]
fn test_ordered_list_numbering() {
    let input = "1. First\n2. Second\n3. Third";
    let options = Options::default();
    let result = format(input, &options).unwrap();

    assert!(result.contains(" 1. First"));
    assert!(result.contains(" 2. Second"));
    assert!(result.contains(" 3. Third"));
}

/// Test empty input produces empty output.
#[test]
fn test_empty_input() {
    let result = format("", &Options::default()).unwrap();
    assert_eq!(result, "");
}

/// Test whitespace-only input.
#[test]
fn test_whitespace_only() {
    let result = format("   \n\n   ", &Options::default()).unwrap();
    // Should produce empty or minimal output
    assert!(result.trim().is_empty() || result.is_empty());
}
