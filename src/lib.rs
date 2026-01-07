//! Hongdown is a Markdown formatter that enforces Hong Minhee's Markdown style
//! conventions.
//!
//! # Example
//!
//! ```
//! use hongdown::{format, Options};
//!
//! let input = "# Hello\nWorld";
//! let options = Options::default();
//! let output = format(input, &options).unwrap();
//! ```

pub mod config;
mod serializer;

pub use serializer::Warning;

use comrak::{Arena, Options as ComrakOptions, parse_document};

/// Formatting options for the Markdown formatter.
#[derive(Debug, Clone)]
pub struct Options {
    /// Line width for wrapping. Default: 80.
    pub line_width: usize,

    /// Use setext-style (underlined) for h1 headings. Default: true.
    pub setext_h1: bool,

    /// Use setext-style (underlined) for h2 headings. Default: true.
    pub setext_h2: bool,

    /// Marker character for unordered lists: `-`, `*`, or `+`. Default: `-`.
    pub unordered_marker: char,

    /// Number of leading spaces before the list marker. Default: 1.
    pub leading_spaces: usize,

    /// Number of trailing spaces after the list marker. Default: 2.
    pub trailing_spaces: usize,

    /// Indentation width for nested list items. Default: 4.
    pub indent_width: usize,

    /// Marker style for ordered lists at odd nesting levels (1st, 3rd, etc.).
    /// Use `.` for `1.` or `)` for `1)`. Default: `.`.
    pub odd_level_marker: char,

    /// Marker style for ordered lists at even nesting levels (2nd, 4th, etc.).
    /// Use `.` for `1.` or `)` for `1)`. Default: `)`.
    pub even_level_marker: char,

    /// Fence character for code blocks: `~` or `` ` ``. Default: `~`.
    pub fence_char: char,

    /// Minimum fence length for code blocks. Default: 4.
    pub min_fence_length: usize,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            line_width: 80,
            setext_h1: true,
            setext_h2: true,
            unordered_marker: '-',
            leading_spaces: 1,
            trailing_spaces: 2,
            indent_width: 4,
            odd_level_marker: '.',
            even_level_marker: ')',
            fence_char: '~',
            min_fence_length: 4,
        }
    }
}

/// Formats a Markdown document according to Hong Minhee's style conventions.
///
/// This function supports formatting directives embedded in HTML comments:
///
/// - `<!-- hongdown-disable-file -->` - Disable formatting for the entire file.
/// - `<!-- hongdown-disable-next-line -->` - Disable formatting for the next block.
/// - `<!-- hongdown-disable-next-section -->` - Disable formatting until the next
///   section heading.
/// - `<!-- hongdown-disable -->` - Disable formatting from this point.
/// - `<!-- hongdown-enable -->` - Re-enable formatting.
///
/// # Arguments
///
/// * `input` - The Markdown source to format.
/// * `options` - Formatting options.
///
/// # Returns
///
/// The formatted Markdown string, or an error if formatting fails.
///
/// # Errors
///
/// Returns an error if the input cannot be parsed or formatted.
pub fn format(input: &str, options: &Options) -> Result<String, FormatError> {
    if input.is_empty() {
        return Ok(String::new());
    }

    let arena = Arena::new();
    let mut comrak_options = ComrakOptions::default();
    comrak_options.extension.front_matter_delimiter = Some("---".to_string());
    comrak_options.extension.table = true;
    comrak_options.extension.description_lists = true;
    comrak_options.extension.alerts = true;
    comrak_options.extension.footnotes = true;
    comrak_options.extension.tasklist = true;

    let root = parse_document(&arena, input, &comrak_options);
    let output = serializer::serialize_with_source(root, options, Some(input));

    Ok(output)
}

/// Result of formatting with warnings.
#[derive(Debug)]
pub struct FormatResult {
    /// The formatted Markdown output.
    pub output: String,
    /// Warnings generated during formatting.
    pub warnings: Vec<Warning>,
}

/// Formats a Markdown document and returns both output and warnings.
///
/// This is similar to [`format`], but also returns any warnings generated
/// during formatting (e.g., inconsistent table column counts).
///
/// # Arguments
///
/// * `input` - The Markdown source to format.
/// * `options` - Formatting options.
///
/// # Returns
///
/// A [`FormatResult`] containing the formatted output and any warnings.
pub fn format_with_warnings(input: &str, options: &Options) -> Result<FormatResult, FormatError> {
    if input.is_empty() {
        return Ok(FormatResult {
            output: String::new(),
            warnings: Vec::new(),
        });
    }

    let arena = Arena::new();
    let mut comrak_options = ComrakOptions::default();
    comrak_options.extension.front_matter_delimiter = Some("---".to_string());
    comrak_options.extension.table = true;
    comrak_options.extension.description_lists = true;
    comrak_options.extension.alerts = true;
    comrak_options.extension.footnotes = true;
    comrak_options.extension.tasklist = true;

    let root = parse_document(&arena, input, &comrak_options);
    let result = serializer::serialize_with_source_and_warnings(root, options, Some(input));

    Ok(FormatResult {
        output: result.output,
        warnings: result.warnings,
    })
}

/// Errors that can occur during formatting.
#[derive(Debug)]
pub enum FormatError {
    /// An error occurred during parsing.
    ParseError(String),
}

impl std::fmt::Display for FormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormatError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for FormatError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_empty_input() {
        let result = format("", &Options::default()).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_format_plain_text() {
        let input = "Hello, world!";
        let result = format(input, &Options::default()).unwrap();
        assert_eq!(result, "Hello, world!\n");
    }
}
