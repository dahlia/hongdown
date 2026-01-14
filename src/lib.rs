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

use std::collections::HashMap;

pub mod config;
mod serializer;

#[cfg(feature = "wasm")]
mod wasm;

pub use config::{
    DashSetting, FenceChar, LeadingSpaces, MinFenceLength, OrderedListPad, OrderedMarker,
    TrailingSpaces, UnorderedMarker,
};
pub use serializer::Warning;
pub use serializer::punctuation::{PunctuationError, validate_dash_settings};

use comrak::{Arena, Options as ComrakOptions, parse_document};

/// External code formatter configuration.
#[derive(Debug, Clone)]
pub struct CodeFormatter {
    /// Command and arguments as a vector.
    pub command: Vec<String>,
    /// Timeout in seconds.
    pub timeout_secs: u64,
}

/// Formatting options for the Markdown formatter.
#[derive(Debug, Clone)]
pub struct Options {
    /// Line width for wrapping. Default: 80.
    pub line_width: usize,

    /// Use setext-style (underlined) for h1 headings. Default: true.
    pub setext_h1: bool,

    /// Use setext-style (underlined) for h2 headings. Default: true.
    pub setext_h2: bool,

    /// Convert headings to sentence case. Default: false.
    pub heading_sentence_case: bool,

    /// Additional proper nouns to preserve (case-sensitive).
    /// These are merged with built-in proper nouns.
    pub heading_proper_nouns: Vec<String>,

    /// Words to treat as common nouns (case-sensitive).
    /// These are excluded from built-in proper nouns.
    pub heading_common_nouns: Vec<String>,

    /// Marker character for unordered lists: `-`, `*`, or `+`. Default: `-`.
    pub unordered_marker: UnorderedMarker,

    /// Number of leading spaces before the list marker. Default: 1.
    pub leading_spaces: LeadingSpaces,

    /// Number of trailing spaces after the list marker. Default: 2.
    pub trailing_spaces: TrailingSpaces,

    /// Indentation width for nested list items. Default: 4.
    pub indent_width: usize,

    /// Marker style for ordered lists at odd nesting levels (1st, 3rd, etc.).
    /// Use `.` for `1.` or `)` for `1)`. Default: `.`.
    pub odd_level_marker: OrderedMarker,

    /// Marker style for ordered lists at even nesting levels (2nd, 4th, etc.).
    /// Use `.` for `1.` or `)` for `1)`. Default: `)`.
    pub even_level_marker: OrderedMarker,

    /// Padding style for ordered list numbers. Default: `Start`.
    pub ordered_list_pad: OrderedListPad,

    /// Indentation width for nested ordered list items. Default: 4.
    pub ordered_list_indent_width: usize,

    /// Fence character for code blocks: `~` or `` ` ``. Default: `~`.
    pub fence_char: FenceChar,

    /// Minimum fence length for code blocks. Default: 4.
    pub min_fence_length: MinFenceLength,

    /// Add space between fence and language identifier. Default: true.
    pub space_after_fence: bool,

    /// Default language identifier for code blocks without one. Default: empty string.
    /// When empty, code blocks without a language identifier remain without one.
    /// Set to e.g. "text" to add a default language identifier.
    pub default_language: String,

    /// The style string for thematic breaks. Default: 37 spaced dashes.
    pub thematic_break_style: String,

    /// Number of leading spaces before thematic breaks (0-3). Default: 3.
    /// CommonMark allows 0-3 leading spaces for thematic breaks.
    pub thematic_break_leading_spaces: LeadingSpaces,

    /// Convert straight double quotes to curly quotes. Default: true.
    /// `"text"` becomes `"text"` (U+201C and U+201D).
    pub curly_double_quotes: bool,

    /// Convert straight single quotes to curly quotes. Default: true.
    /// `'text'` becomes `'text'` (U+2018 and U+2019).
    pub curly_single_quotes: bool,

    /// Convert straight apostrophes to curly apostrophes. Default: false.
    /// `it's` becomes `it's` (U+2019).
    pub curly_apostrophes: bool,

    /// Convert three dots to ellipsis character. Default: true.
    /// `...` becomes `…` (U+2026).
    pub ellipsis: bool,

    /// Convert a pattern to en-dash. Default: disabled.
    /// Set to a string like `"--"` to enable.
    /// The pattern is replaced with `–` (U+2013).
    pub en_dash: DashSetting,

    /// Convert a pattern to em-dash. Default: `"--"`.
    /// Set to `Disabled` to disable, or a string like `"---"` for a different pattern.
    /// The pattern is replaced with `—` (U+2014).
    pub em_dash: DashSetting,

    /// External code formatters by language.
    ///
    /// Key: language identifier (exact match only).
    /// Value: formatter configuration with command and timeout.
    ///
    /// When a code block with a matching language is encountered, the formatter
    /// command is executed with the code content passed via stdin. The formatted
    /// output is read from stdout.
    ///
    /// If the formatter fails (non-zero exit, timeout, etc.), the original code
    /// is preserved and a warning is emitted.
    pub code_formatters: HashMap<String, CodeFormatter>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            line_width: 80,
            setext_h1: true,
            setext_h2: true,
            heading_sentence_case: false,
            heading_proper_nouns: Vec::new(),
            heading_common_nouns: Vec::new(),
            unordered_marker: UnorderedMarker::default(),
            leading_spaces: LeadingSpaces::default(),
            trailing_spaces: TrailingSpaces::default(),
            indent_width: 4,
            odd_level_marker: OrderedMarker::default(),
            even_level_marker: OrderedMarker::Parenthesis,
            ordered_list_pad: OrderedListPad::Start,
            ordered_list_indent_width: 4,
            fence_char: FenceChar::default(),
            min_fence_length: MinFenceLength::default(),
            space_after_fence: true,
            default_language: String::new(),
            thematic_break_style:
                "- - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -"
                    .to_string(),
            thematic_break_leading_spaces: LeadingSpaces::new(3).unwrap(),
            curly_double_quotes: true,
            curly_single_quotes: true,
            curly_apostrophes: false,
            ellipsis: true,
            en_dash: DashSetting::Disabled,
            em_dash: DashSetting::Pattern("--".to_string()),
            code_formatters: HashMap::new(),
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
    use config::DashSetting;

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

    #[test]
    fn test_options_default_punctuation() {
        let options = Options::default();
        assert!(options.curly_double_quotes);
        assert!(options.curly_single_quotes);
        assert!(!options.curly_apostrophes);
        assert!(options.ellipsis);
        assert_eq!(options.en_dash, DashSetting::Disabled);
        assert_eq!(options.em_dash, DashSetting::Pattern("--".to_string()));
    }
}
