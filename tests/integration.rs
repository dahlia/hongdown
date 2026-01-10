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

    // trailing_spaces=2, so "N.  " format
    assert!(result.contains("1.  First"));
    assert!(result.contains("2.  Second"));
    assert!(result.contains("3.  Third"));
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

mod cli_tests {
    use std::io::Write;
    use std::process::{Command, Stdio};

    /// Helper function to run hongdown CLI with given args and stdin input.
    fn run_hongdown(args: &[&str], stdin_input: Option<&str>) -> (String, String, i32) {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_hongdown"));
        cmd.args(args);

        if stdin_input.is_some() {
            cmd.stdin(Stdio::piped());
        }
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd.spawn().expect("Failed to spawn hongdown");

        if let Some(input) = stdin_input {
            let mut stdin = child.stdin.take().expect("Failed to get stdin");
            // Ignore broken pipe errors - the process may have exited early
            // (e.g., due to argument validation failure) before reading stdin
            let _ = stdin.write_all(input.as_bytes());
        }

        let output = child.wait_with_output().expect("Failed to wait for output");
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);

        (stdout, stderr, exit_code)
    }

    /// Test --diff flag shows no output when input is already formatted.
    #[test]
    fn test_diff_no_changes() {
        let formatted_input = "Title\n=====\n\nA paragraph.\n";
        let (stdout, _stderr, exit_code) =
            run_hongdown(&["--diff", "--stdin"], Some(formatted_input));

        // No diff output when already formatted
        assert!(stdout.is_empty(), "No diff expected for formatted input");
        assert_eq!(exit_code, 0);
    }

    /// Test --diff flag shows unified diff when input needs formatting.
    #[test]
    fn test_diff_with_changes() {
        let unformatted_input = "# Title\n\nA paragraph.";
        let (stdout, _stderr, exit_code) =
            run_hongdown(&["--diff", "--stdin"], Some(unformatted_input));

        // Should show diff output
        assert!(stdout.contains("---"), "Diff should contain --- header");
        assert!(stdout.contains("+++"), "Diff should contain +++ header");
        assert!(stdout.contains("-# Title"), "Diff should show removed line");
        assert!(stdout.contains("+Title"), "Diff should show added line");
        assert!(
            stdout.contains("+====="),
            "Diff should show added underline"
        );
        assert_eq!(exit_code, 0);
    }

    /// Test --diff with file input.
    #[test]
    fn test_diff_with_file() {
        use std::fs;
        use tempfile::NamedTempFile;

        // Create a temporary file with unformatted content
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        writeln!(temp_file, "# Test Heading").expect("Failed to write to temp file");
        writeln!(temp_file).expect("Failed to write to temp file");
        writeln!(temp_file, "A paragraph.").expect("Failed to write to temp file");

        let file_path = temp_file.path().to_str().unwrap();
        let (stdout, _stderr, exit_code) = run_hongdown(&["--diff", file_path], None);

        // Should show diff with filename in header
        assert!(stdout.contains("---"), "Diff should contain --- header");
        assert!(stdout.contains("+++"), "Diff should contain +++ header");
        assert_eq!(exit_code, 0);

        // File should not be modified
        let content = fs::read_to_string(temp_file.path()).expect("Failed to read temp file");
        assert!(
            content.contains("# Test Heading"),
            "File should not be modified"
        );
    }

    /// Test --diff and --check are mutually exclusive.
    #[test]
    fn test_diff_check_mutually_exclusive() {
        let (_stdout, stderr, exit_code) =
            run_hongdown(&["--diff", "--check", "--stdin"], Some("# Test"));

        // Should fail with error about conflicting options
        assert_ne!(exit_code, 0);
        assert!(
            stderr.contains("cannot be used with") || stderr.contains("conflict"),
            "Should report conflicting options"
        );
    }

    /// Test --diff and --write are mutually exclusive.
    #[test]
    fn test_diff_write_mutually_exclusive() {
        let (_stdout, stderr, exit_code) =
            run_hongdown(&["--diff", "--write", "--stdin"], Some("# Test"));

        // Should fail with error about conflicting options
        assert_ne!(exit_code, 0);
        assert!(
            stderr.contains("cannot be used with") || stderr.contains("conflict"),
            "Should report conflicting options"
        );
    }

    /// Test --write reports which files were changed.
    #[test]
    fn test_write_reports_changed_files() {
        use std::fs;
        use tempfile::TempDir;

        // Create a temporary directory with test files
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create a file that needs formatting
        let unformatted_path = temp_dir.path().join("unformatted.md");
        fs::write(&unformatted_path, "# Needs Formatting\n\nA paragraph.")
            .expect("Failed to write unformatted file");

        // Create a file that is already formatted
        let formatted_path = temp_dir.path().join("formatted.md");
        fs::write(
            &formatted_path,
            "Already Formatted\n=================\n\nA paragraph.\n",
        )
        .expect("Failed to write formatted file");

        let (stdout, _stderr, exit_code) = run_hongdown(
            &[
                "--write",
                unformatted_path.to_str().unwrap(),
                formatted_path.to_str().unwrap(),
            ],
            None,
        );

        assert_eq!(exit_code, 0);

        // Should report the unformatted file as changed
        assert!(
            stdout.contains("unformatted.md"),
            "Should report the changed file: got stdout: {}",
            stdout
        );

        // Should NOT report the already formatted file (check for the exact filename)
        // Note: "unformatted.md" contains "formatted.md" as a substring, so we need to
        // check that "formatted.md" only appears as part of "unformatted.md"
        let stdout_without_unformatted = stdout.replace("unformatted.md", "");
        assert!(
            !stdout_without_unformatted.contains("formatted.md"),
            "Should not report unchanged file: got stdout: {}",
            stdout
        );
    }

    /// Test --write does not report files that are unchanged.
    #[test]
    fn test_write_silent_on_unchanged_files() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create a file that is already formatted
        let formatted_path = temp_dir.path().join("already_formatted.md");
        fs::write(&formatted_path, "Title\n=====\n\nA paragraph.\n")
            .expect("Failed to write formatted file");

        let (stdout, _stderr, exit_code) =
            run_hongdown(&["--write", formatted_path.to_str().unwrap()], None);

        assert_eq!(exit_code, 0);

        // Should not report any files since nothing changed
        assert!(
            stdout.is_empty(),
            "Should not report unchanged files: got stdout: {}",
            stdout
        );
    }

    /// Test that running hongdown without files and without --stdin fails.
    #[test]
    fn test_no_input_error() {
        use tempfile::TempDir;

        // Create a temporary directory without .hongdown.toml
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_hongdown"));
        cmd.current_dir(temp_dir.path());
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        let output = cmd.output().expect("Failed to run hongdown");
        let stderr = String::from_utf8_lossy(&output.stderr);
        let exit_code = output.status.code().unwrap_or(-1);

        // Should fail with error about missing input
        assert_ne!(exit_code, 0, "Should exit with error code");
        assert!(
            stderr.contains("no input files") || stderr.contains("No input"),
            "Error message should mention missing input files: got stderr: {}",
            stderr
        );
    }

    /// Test that --stdin explicitly allows stdin input.
    #[test]
    fn test_stdin_flag_works() {
        let input = "# Test\n\nParagraph.";
        let (stdout, _stderr, exit_code) = run_hongdown(&["--stdin"], Some(input));

        assert_eq!(exit_code, 0);
        assert!(stdout.contains("Test\n===="));
    }

    /// Test that `-` explicitly allows stdin input.
    #[test]
    fn test_dash_for_stdin() {
        let input = "# Test\n\nParagraph.";
        let (stdout, _stderr, exit_code) = run_hongdown(&["-"], Some(input));

        assert_eq!(exit_code, 0);
        assert!(stdout.contains("Test\n===="));
    }
}
