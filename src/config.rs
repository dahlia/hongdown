// SPDX-FileCopyrightText: 2025 Hong Minhee <https://hongminhee.org/>
// SPDX-License-Identifier: GPL-3.0-or-later
//! Configuration file support for Hongdown.
//!
//! This module provides functionality for loading and parsing configuration
//! files (`.hongdown.toml`) that control the formatter's behavior.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

/// The default configuration file name.
pub const CONFIG_FILE_NAME: &str = ".hongdown.toml";

/// Configuration for the Hongdown formatter.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(default)]
pub struct Config {
    /// Maximum line width for wrapping (default: 80).
    pub line_width: usize,

    /// Glob patterns for files to include (default: empty, meaning all files
    /// must be specified on command line).
    pub include: Vec<String>,

    /// Glob patterns for files to exclude (default: empty).
    pub exclude: Vec<String>,

    /// Heading formatting options.
    pub heading: HeadingConfig,

    /// Unordered list formatting options.
    pub unordered_list: UnorderedListConfig,

    /// Ordered list formatting options.
    pub ordered_list: OrderedListConfig,

    /// Code block formatting options.
    pub code_block: CodeBlockConfig,

    /// Thematic break (horizontal rule) formatting options.
    pub thematic_break: ThematicBreakConfig,

    /// Punctuation transformation options (SmartyPants-style).
    pub punctuation: PunctuationConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            line_width: 80,
            include: Vec::new(),
            exclude: Vec::new(),
            heading: HeadingConfig::default(),
            unordered_list: UnorderedListConfig::default(),
            ordered_list: OrderedListConfig::default(),
            code_block: CodeBlockConfig::default(),
            thematic_break: ThematicBreakConfig::default(),
            punctuation: PunctuationConfig::default(),
        }
    }
}

/// Heading formatting options.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(default)]
pub struct HeadingConfig {
    /// Use `===` underline for h1 (default: true).
    pub setext_h1: bool,

    /// Use `---` underline for h2 (default: true).
    pub setext_h2: bool,

    /// Convert headings to sentence case (default: false).
    pub sentence_case: bool,

    /// Additional proper nouns to preserve (case-sensitive).
    /// These are merged with built-in proper nouns.
    pub proper_nouns: Vec<String>,

    /// Words to treat as common nouns (case-sensitive).
    /// These are excluded from built-in proper nouns.
    /// Useful for words like "Go" which can be either a programming language
    /// or a common verb depending on context.
    pub common_nouns: Vec<String>,
}

impl Default for HeadingConfig {
    fn default() -> Self {
        Self {
            setext_h1: true,
            setext_h2: true,
            sentence_case: false,
            proper_nouns: Vec::new(),
            common_nouns: Vec::new(),
        }
    }
}

/// Marker character for unordered lists.
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Default)]
pub enum UnorderedMarker {
    /// Hyphen marker (`-`)
    #[default]
    #[serde(rename = "-")]
    Hyphen,
    /// Asterisk marker (`*`)
    #[serde(rename = "*")]
    Asterisk,
    /// Plus marker (`+`)
    #[serde(rename = "+")]
    Plus,
}

impl UnorderedMarker {
    /// Get the character representation of this marker.
    pub fn as_char(self) -> char {
        match self {
            Self::Hyphen => '-',
            Self::Asterisk => '*',
            Self::Plus => '+',
        }
    }
}

/// Leading spaces before a list marker or thematic break (0-3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeadingSpaces(usize);

impl LeadingSpaces {
    /// Maximum allowed leading spaces (CommonMark requirement).
    pub const MAX: usize = 3;

    /// Create a new LeadingSpaces.
    ///
    /// Returns an error if the value is greater than 3.
    pub fn new(value: usize) -> Result<Self, String> {
        if value > Self::MAX {
            Err(format!(
                "leading_spaces must be at most {}, got {}.",
                Self::MAX,
                value
            ))
        } else {
            Ok(Self(value))
        }
    }

    /// Get the inner value.
    pub fn get(self) -> usize {
        self.0
    }
}

impl Default for LeadingSpaces {
    fn default() -> Self {
        Self(1)
    }
}

impl<'de> serde::Deserialize<'de> for LeadingSpaces {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = usize::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

/// Trailing spaces after a list marker (0-3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrailingSpaces(usize);

impl TrailingSpaces {
    /// Maximum allowed trailing spaces (CommonMark requirement).
    pub const MAX: usize = 3;

    /// Create a new TrailingSpaces.
    ///
    /// Returns an error if the value is greater than 3.
    pub fn new(value: usize) -> Result<Self, String> {
        if value > Self::MAX {
            Err(format!(
                "trailing_spaces must be at most {}, got {}.",
                Self::MAX,
                value
            ))
        } else {
            Ok(Self(value))
        }
    }

    /// Get the inner value.
    pub fn get(self) -> usize {
        self.0
    }
}

impl Default for TrailingSpaces {
    fn default() -> Self {
        Self(2)
    }
}

impl<'de> serde::Deserialize<'de> for TrailingSpaces {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = usize::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

/// Unordered list formatting options.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(default)]
pub struct UnorderedListConfig {
    /// Marker character: `-`, `*`, or `+` (default: `-`).
    pub unordered_marker: UnorderedMarker,

    /// Spaces before the marker (default: 1).
    pub leading_spaces: LeadingSpaces,

    /// Spaces after the marker (default: 2).
    pub trailing_spaces: TrailingSpaces,

    /// Indentation width for nested items (default: 4).
    pub indent_width: usize,
}

impl Default for UnorderedListConfig {
    fn default() -> Self {
        Self {
            unordered_marker: UnorderedMarker::default(),
            leading_spaces: LeadingSpaces::default(),
            trailing_spaces: TrailingSpaces::default(),
            indent_width: 4,
        }
    }
}

/// Marker character for ordered lists.
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Default)]
pub enum OrderedMarker {
    /// Period marker (`.`) - `1.`, `2.`, etc.
    #[default]
    #[serde(rename = ".")]
    Period,
    /// Parenthesis marker (`)`) - `1)`, `2)`, etc.
    #[serde(rename = ")")]
    Parenthesis,
}

impl OrderedMarker {
    /// Get the character representation of this marker.
    pub fn as_char(self) -> char {
        match self {
            Self::Period => '.',
            Self::Parenthesis => ')',
        }
    }
}

/// Padding style for ordered list numbers.
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum OrderedListPad {
    /// Pad before the number (default): `  1.`, `  2.`, ..., ` 10.`
    #[default]
    Start,
    /// Pad after the number: `1. `, `2. `, ..., `10.`
    End,
}

/// Ordered list formatting options.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(default)]
pub struct OrderedListConfig {
    /// Marker style at odd nesting levels: `.` for `1.` (default: `.`).
    pub odd_level_marker: OrderedMarker,

    /// Marker style at even nesting levels: `)` for `1)` (default: `)`).
    pub even_level_marker: OrderedMarker,

    /// Padding style for aligning numbers of different widths (default: `start`).
    pub pad: OrderedListPad,

    /// Indentation width for nested ordered list items (default: 4).
    pub indent_width: usize,
}

impl Default for OrderedListConfig {
    fn default() -> Self {
        Self {
            odd_level_marker: OrderedMarker::default(),
            even_level_marker: OrderedMarker::Parenthesis,
            pad: OrderedListPad::Start,
            indent_width: 4,
        }
    }
}

/// Default timeout for external formatters in seconds.
fn default_formatter_timeout() -> u64 {
    5
}

/// External formatter configuration for a single language.
///
/// Can be specified in two formats:
/// - Simple: `["command", "arg1", "arg2"]`
/// - Full: `{ command = ["command", "arg1"], timeout = 10 }`
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum FormatterConfig {
    /// Simple array format: `["deno", "fmt", "-"]`
    Simple(Vec<String>),
    /// Full format with command and options.
    Full {
        /// Command and arguments as a vector.
        command: Vec<String>,
        /// Timeout in seconds (default: 5).
        #[serde(default = "default_formatter_timeout")]
        timeout: u64,
    },
}

impl FormatterConfig {
    /// Get the command as a slice.
    pub fn command(&self) -> &[String] {
        match self {
            FormatterConfig::Simple(cmd) => cmd,
            FormatterConfig::Full { command, .. } => command,
        }
    }

    /// Get the timeout in seconds.
    pub fn timeout(&self) -> u64 {
        match self {
            FormatterConfig::Simple(_) => default_formatter_timeout(),
            FormatterConfig::Full { timeout, .. } => *timeout,
        }
    }

    /// Validate the configuration.
    ///
    /// Returns an error message if the configuration is invalid.
    pub fn validate(&self) -> Result<(), String> {
        if self.command().is_empty() {
            return Err("formatter command cannot be empty".to_string());
        }
        Ok(())
    }
}

/// Fence character for code blocks.
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Default)]
pub enum FenceChar {
    /// Tilde fence (`~`)
    #[default]
    #[serde(rename = "~")]
    Tilde,
    /// Backtick fence (`` ` ``)
    #[serde(rename = "`")]
    Backtick,
}

impl FenceChar {
    /// Get the character representation of this fence character.
    pub fn as_char(self) -> char {
        match self {
            Self::Tilde => '~',
            Self::Backtick => '`',
        }
    }
}

/// Minimum fence length for code blocks (must be at least 3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MinFenceLength(usize);

impl MinFenceLength {
    /// Minimum allowed fence length (CommonMark requirement).
    pub const MIN: usize = 3;

    /// Create a new MinFenceLength.
    ///
    /// Returns an error if the value is less than 3.
    pub fn new(value: usize) -> Result<Self, String> {
        if value < Self::MIN {
            Err(format!(
                "min_fence_length must be at least {}, got {}.",
                Self::MIN,
                value
            ))
        } else {
            Ok(Self(value))
        }
    }

    /// Get the inner value.
    pub fn get(self) -> usize {
        self.0
    }
}

impl Default for MinFenceLength {
    fn default() -> Self {
        Self(4)
    }
}

impl<'de> serde::Deserialize<'de> for MinFenceLength {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = usize::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

/// Code block formatting options.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(default)]
pub struct CodeBlockConfig {
    /// Fence character: `~` or `` ` `` (default: `~`).
    pub fence_char: FenceChar,

    /// Minimum fence length (default: 4).
    pub min_fence_length: MinFenceLength,

    /// Add space between fence and language identifier (default: true).
    pub space_after_fence: bool,

    /// Default language identifier for code blocks without one (default: empty).
    /// When empty, code blocks without a language identifier remain without one.
    /// Set to e.g. "text" to add a default language identifier.
    pub default_language: String,

    /// External formatters for code blocks by language.
    ///
    /// Key: language identifier (exact match only).
    /// Value: formatter configuration.
    pub formatters: HashMap<String, FormatterConfig>,
}

impl Default for CodeBlockConfig {
    fn default() -> Self {
        Self {
            fence_char: FenceChar::default(),
            min_fence_length: MinFenceLength::default(),
            space_after_fence: true,
            default_language: String::new(),
            formatters: HashMap::new(),
        }
    }
}

/// Thematic break (horizontal rule) formatting options.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(default)]
pub struct ThematicBreakConfig {
    /// The style string for thematic breaks (default: `*  *  *`).
    pub style: String,

    /// Number of leading spaces before the thematic break (0-3, default: 3).
    /// CommonMark allows 0-3 leading spaces for thematic breaks.
    pub leading_spaces: LeadingSpaces,
}

impl Default for ThematicBreakConfig {
    fn default() -> Self {
        Self {
            style: "- - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -"
                .to_string(),
            leading_spaces: LeadingSpaces::new(3).unwrap(),
        }
    }
}

/// Dash transformation setting.
/// Can be `false` (disabled) or a string pattern to match.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum DashSetting {
    /// Dash transformation is disabled.
    #[default]
    Disabled,
    /// Transform the given pattern to a dash character.
    Pattern(String),
}

impl<'de> Deserialize<'de> for DashSetting {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, Visitor};

        struct DashSettingVisitor;

        impl<'de> Visitor<'de> for DashSettingVisitor {
            type Value = DashSetting;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("false or a string pattern")
            }

            fn visit_bool<E>(self, value: bool) -> Result<DashSetting, E>
            where
                E: de::Error,
            {
                if value {
                    Err(de::Error::custom(
                        "expected false or a string pattern, got true",
                    ))
                } else {
                    Ok(DashSetting::Disabled)
                }
            }

            fn visit_str<E>(self, value: &str) -> Result<DashSetting, E>
            where
                E: de::Error,
            {
                Ok(DashSetting::Pattern(value.to_string()))
            }

            fn visit_string<E>(self, value: String) -> Result<DashSetting, E>
            where
                E: de::Error,
            {
                Ok(DashSetting::Pattern(value))
            }
        }

        deserializer.deserialize_any(DashSettingVisitor)
    }
}

/// Punctuation transformation options (SmartyPants-style).
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(default)]
pub struct PunctuationConfig {
    /// Convert straight double quotes to curly quotes (default: true).
    /// `"text"` becomes `"text"` (U+201C and U+201D).
    pub curly_double_quotes: bool,

    /// Convert straight single quotes to curly quotes (default: true).
    /// `'text'` becomes `'text'` (U+2018 and U+2019).
    pub curly_single_quotes: bool,

    /// Convert straight apostrophes to curly apostrophes (default: false).
    /// `it's` becomes `it's` (U+2019).
    pub curly_apostrophes: bool,

    /// Convert three dots to ellipsis character (default: true).
    /// `...` becomes `…` (U+2026).
    pub ellipsis: bool,

    /// Convert a pattern to en-dash (default: disabled).
    /// Set to a string like `"--"` to enable.
    /// The pattern is replaced with `–` (U+2013).
    pub en_dash: DashSetting,

    /// Convert a pattern to em-dash (default: `"--"`).
    /// Set to `false` to disable, or a string like `"---"` for a different pattern.
    /// The pattern is replaced with `—` (U+2014).
    pub em_dash: DashSetting,
}

impl Default for PunctuationConfig {
    fn default() -> Self {
        Self {
            curly_double_quotes: true,
            curly_single_quotes: true,
            curly_apostrophes: false,
            ellipsis: true,
            en_dash: DashSetting::Disabled,
            em_dash: DashSetting::Pattern("--".to_string()),
        }
    }
}

impl Config {
    /// Parse a configuration from a TOML string.
    pub fn from_toml(toml_str: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(toml_str)
    }

    /// Load configuration from a file.
    pub fn from_file(path: &Path) -> Result<Self, ConfigError> {
        let content =
            std::fs::read_to_string(path).map_err(|e| ConfigError::Io(path.to_path_buf(), e))?;
        Self::from_toml(&content).map_err(|e| ConfigError::Parse(path.to_path_buf(), e))
    }

    /// Discover and load configuration by searching up the directory tree.
    ///
    /// Starting from `start_dir`, searches for `.hongdown.toml` in each parent
    /// directory until the filesystem root is reached. Returns `None` if no
    /// configuration file is found.
    pub fn discover(start_dir: &Path) -> Result<Option<(PathBuf, Self)>, ConfigError> {
        let mut current = start_dir.to_path_buf();
        loop {
            let config_path = current.join(CONFIG_FILE_NAME);
            if config_path.exists() {
                let config = Self::from_file(&config_path)?;
                return Ok(Some((config_path, config)));
            }
            if !current.pop() {
                break;
            }
        }
        Ok(None)
    }

    /// Collect files matching the include patterns, excluding those matching
    /// exclude patterns.
    ///
    /// The `base_dir` is used as the starting point for glob pattern matching.
    /// Returns an empty list if no include patterns are configured.
    pub fn collect_files(&self, base_dir: &Path) -> Result<Vec<PathBuf>, ConfigError> {
        use glob::{MatchOptions, glob_with};

        if self.include.is_empty() {
            return Ok(Vec::new());
        }

        let options = MatchOptions {
            case_sensitive: true,
            require_literal_separator: false,
            require_literal_leading_dot: false,
        };

        let mut files = Vec::new();

        // Collect files matching include patterns
        for pattern in &self.include {
            let full_pattern = base_dir.join(pattern);
            let pattern_str = full_pattern.to_string_lossy();
            let matches = glob_with(&pattern_str, options)
                .map_err(|e| ConfigError::Glob(pattern.clone(), e))?;

            for entry in matches {
                let path = entry.map_err(ConfigError::GlobIo)?;
                if path.is_file() {
                    files.push(path);
                }
            }
        }

        // Remove duplicates
        files.sort();
        files.dedup();

        // Filter out excluded files
        if !self.exclude.is_empty() {
            let exclude_patterns: Vec<glob::Pattern> = self
                .exclude
                .iter()
                .filter_map(|p| {
                    let full_pattern = base_dir.join(p);
                    glob::Pattern::new(&full_pattern.to_string_lossy()).ok()
                })
                .collect();

            files.retain(|path| {
                let path_str = path.to_string_lossy();
                !exclude_patterns
                    .iter()
                    .any(|pattern| pattern.matches(&path_str))
            });
        }

        Ok(files)
    }
}

/// Errors that can occur when loading configuration.
#[derive(Debug)]
pub enum ConfigError {
    /// I/O error reading the configuration file.
    Io(PathBuf, std::io::Error),
    /// Error parsing the TOML configuration.
    Parse(PathBuf, toml::de::Error),
    /// Error parsing a glob pattern.
    Glob(String, glob::PatternError),
    /// I/O error during glob iteration.
    GlobIo(glob::GlobError),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Io(path, err) => {
                write!(f, "failed to read {}: {}", path.display(), err)
            }
            ConfigError::Parse(path, err) => {
                write!(f, "failed to parse {}: {}", path.display(), err)
            }
            ConfigError::Glob(pattern, err) => {
                write!(f, "invalid glob pattern '{}': {}", pattern, err)
            }
            ConfigError::GlobIo(err) => {
                write!(f, "error reading file: {}", err)
            }
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigError::Io(_, err) => Some(err),
            ConfigError::Parse(_, err) => Some(err),
            ConfigError::Glob(_, err) => Some(err),
            ConfigError::GlobIo(err) => Some(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.line_width, 80);
        assert!(config.heading.setext_h1);
        assert!(config.heading.setext_h2);
        assert_eq!(
            config.unordered_list.unordered_marker,
            UnorderedMarker::Hyphen
        );
        assert_eq!(config.unordered_list.leading_spaces.get(), 1);
        assert_eq!(config.unordered_list.trailing_spaces.get(), 2);
        assert_eq!(config.unordered_list.indent_width, 4);
        assert_eq!(config.ordered_list.odd_level_marker, OrderedMarker::Period);
        assert_eq!(
            config.ordered_list.even_level_marker,
            OrderedMarker::Parenthesis
        );
        assert_eq!(config.ordered_list.pad, OrderedListPad::Start);
        assert_eq!(config.ordered_list.indent_width, 4);
        assert_eq!(config.code_block.fence_char, FenceChar::Tilde);
        assert_eq!(config.code_block.min_fence_length.get(), 4);
        assert!(config.code_block.space_after_fence);
        assert_eq!(config.code_block.default_language, "");
        assert_eq!(
            config.thematic_break.style,
            "- - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -"
        );
        assert_eq!(config.thematic_break.leading_spaces.get(), 3);
    }

    #[test]
    fn test_parse_empty_toml() {
        let config = Config::from_toml("").unwrap();
        assert_eq!(config, Config::default());
    }

    #[test]
    fn test_parse_line_width() {
        let config = Config::from_toml("line_width = 100").unwrap();
        assert_eq!(config.line_width, 100);
    }

    #[test]
    fn test_parse_heading_config() {
        let config = Config::from_toml(
            r#"
[heading]
setext_h1 = false
setext_h2 = false
"#,
        )
        .unwrap();
        assert!(!config.heading.setext_h1);
        assert!(!config.heading.setext_h2);
    }

    #[test]
    fn test_parse_heading_sentence_case() {
        let config = Config::from_toml(
            r#"
[heading]
sentence_case = true
"#,
        )
        .unwrap();
        assert!(config.heading.sentence_case);
    }

    #[test]
    fn test_parse_heading_proper_nouns() {
        let config = Config::from_toml(
            r#"
[heading]
proper_nouns = ["Hongdown", "MyCompany", "MyProduct"]
"#,
        )
        .unwrap();
        assert_eq!(
            config.heading.proper_nouns,
            vec!["Hongdown", "MyCompany", "MyProduct"]
        );
    }

    #[test]
    fn test_parse_heading_sentence_case_with_proper_nouns() {
        let config = Config::from_toml(
            r#"
[heading]
sentence_case = true
proper_nouns = ["Hongdown", "MyAPI"]
"#,
        )
        .unwrap();
        assert!(config.heading.sentence_case);
        assert_eq!(config.heading.proper_nouns, vec!["Hongdown", "MyAPI"]);
    }

    #[test]
    fn test_parse_heading_common_nouns() {
        let config = Config::from_toml(
            r#"
[heading]
common_nouns = ["Go", "Swift"]
"#,
        )
        .unwrap();
        assert_eq!(config.heading.common_nouns, vec!["Go", "Swift"]);
    }

    #[test]
    fn test_parse_heading_with_proper_and_common_nouns() {
        let config = Config::from_toml(
            r#"
[heading]
sentence_case = true
proper_nouns = ["MyAPI"]
common_nouns = ["Go"]
"#,
        )
        .unwrap();
        assert!(config.heading.sentence_case);
        assert_eq!(config.heading.proper_nouns, vec!["MyAPI"]);
        assert_eq!(config.heading.common_nouns, vec!["Go"]);
    }

    #[test]
    fn test_parse_unordered_list_config() {
        let config = Config::from_toml(
            r#"
[unordered_list]
unordered_marker = "*"
leading_spaces = 0
trailing_spaces = 1
indent_width = 2
"#,
        )
        .unwrap();
        assert_eq!(
            config.unordered_list.unordered_marker,
            UnorderedMarker::Asterisk
        );
        assert_eq!(config.unordered_list.leading_spaces.get(), 0);
        assert_eq!(config.unordered_list.trailing_spaces.get(), 1);
        assert_eq!(config.unordered_list.indent_width, 2);
    }

    #[test]
    fn test_parse_ordered_list_config() {
        let config = Config::from_toml(
            r#"
[ordered_list]
odd_level_marker = ")"
even_level_marker = "."
"#,
        )
        .unwrap();
        assert_eq!(
            config.ordered_list.odd_level_marker,
            OrderedMarker::Parenthesis
        );
        assert_eq!(config.ordered_list.even_level_marker, OrderedMarker::Period);
        assert_eq!(config.ordered_list.pad, OrderedListPad::Start); // default
    }

    #[test]
    fn test_parse_ordered_list_pad_end() {
        let config = Config::from_toml(
            r#"
[ordered_list]
pad = "end"
"#,
        )
        .unwrap();
        assert_eq!(config.ordered_list.pad, OrderedListPad::End);
    }

    #[test]
    fn test_parse_ordered_list_pad_start() {
        let config = Config::from_toml(
            r#"
[ordered_list]
pad = "start"
"#,
        )
        .unwrap();
        assert_eq!(config.ordered_list.pad, OrderedListPad::Start);
    }

    #[test]
    fn test_parse_code_block_config() {
        let config = Config::from_toml(
            r#"
[code_block]
fence_char = "`"
min_fence_length = 3
space_after_fence = false
"#,
        )
        .unwrap();
        assert_eq!(config.code_block.fence_char, FenceChar::Backtick);
        assert_eq!(config.code_block.min_fence_length.get(), 3);
        assert!(!config.code_block.space_after_fence);
        assert_eq!(config.code_block.default_language, ""); // Default is empty
    }

    #[test]
    fn test_parse_code_block_default_language() {
        let config = Config::from_toml(
            r#"
[code_block]
default_language = "text"
"#,
        )
        .unwrap();
        assert_eq!(config.code_block.default_language, "text");
    }

    #[test]
    fn test_parse_full_config() {
        let config = Config::from_toml(
            r#"
line_width = 80

[heading]
setext_h1 = true
setext_h2 = true

[unordered_list]
unordered_marker = "-"
leading_spaces = 1
trailing_spaces = 2
indent_width = 4

[ordered_list]
odd_level_marker = "."
even_level_marker = ")"

[code_block]
fence_char = "~"
min_fence_length = 4
space_after_fence = true

[thematic_break]
style = "- - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -"
leading_spaces = 3
"#,
        )
        .unwrap();
        assert_eq!(config, Config::default());
    }

    #[test]
    fn test_parse_thematic_break_config() {
        let config = Config::from_toml(
            r#"
[thematic_break]
style = "---"
"#,
        )
        .unwrap();
        assert_eq!(config.thematic_break.style, "---");
    }

    #[test]
    fn test_parse_invalid_toml() {
        let result = Config::from_toml("line_width = \"not a number\"");
        assert!(result.is_err());
    }

    #[test]
    fn test_discover_config_in_current_dir() {
        let temp_dir = std::env::temp_dir().join("hongdown_test_current");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();
        let config_path = temp_dir.join(CONFIG_FILE_NAME);
        std::fs::write(&config_path, "line_width = 120").unwrap();

        let result = Config::discover(&temp_dir).unwrap();
        assert!(result.is_some());
        let (path, config) = result.unwrap();
        assert_eq!(path, config_path);
        assert_eq!(config.line_width, 120);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_discover_config_in_parent_dir() {
        let temp_dir = std::env::temp_dir().join("hongdown_test_parent");
        let _ = std::fs::remove_dir_all(&temp_dir);
        let sub_dir = temp_dir.join("subdir").join("nested");
        std::fs::create_dir_all(&sub_dir).unwrap();
        let config_path = temp_dir.join(CONFIG_FILE_NAME);
        std::fs::write(&config_path, "line_width = 90").unwrap();

        let result = Config::discover(&sub_dir).unwrap();
        assert!(result.is_some());
        let (path, config) = result.unwrap();
        assert_eq!(path, config_path);
        assert_eq!(config.line_width, 90);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_default_include_exclude() {
        let config = Config::default();
        assert!(config.include.is_empty());
        assert!(config.exclude.is_empty());
    }

    #[test]
    fn test_parse_include_patterns() {
        let config = Config::from_toml(
            r#"
include = ["*.md", "docs/**/*.md"]
"#,
        )
        .unwrap();
        assert_eq!(config.include, vec!["*.md", "docs/**/*.md"]);
    }

    #[test]
    fn test_parse_exclude_patterns() {
        let config = Config::from_toml(
            r#"
exclude = ["node_modules/**", "target/**"]
"#,
        )
        .unwrap();
        assert_eq!(config.exclude, vec!["node_modules/**", "target/**"]);
    }

    #[test]
    fn test_parse_include_and_exclude() {
        let config = Config::from_toml(
            r#"
include = ["**/*.md"]
exclude = ["vendor/**"]
"#,
        )
        .unwrap();
        assert_eq!(config.include, vec!["**/*.md"]);
        assert_eq!(config.exclude, vec!["vendor/**"]);
    }

    #[test]
    fn test_collect_files_with_include() {
        let temp_dir = std::env::temp_dir().join("hongdown_test_collect");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();
        std::fs::write(temp_dir.join("README.md"), "# Test").unwrap();
        std::fs::write(temp_dir.join("CHANGELOG.md"), "# Changes").unwrap();
        std::fs::write(temp_dir.join("main.rs"), "fn main() {}").unwrap();

        let config = Config::from_toml(r#"include = ["*.md"]"#).unwrap();
        let files = config.collect_files(&temp_dir).unwrap();

        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|p| p.ends_with("README.md")));
        assert!(files.iter().any(|p| p.ends_with("CHANGELOG.md")));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_collect_files_with_exclude() {
        let temp_dir = std::env::temp_dir().join("hongdown_test_exclude");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();
        std::fs::create_dir_all(temp_dir.join("vendor")).unwrap();
        std::fs::write(temp_dir.join("README.md"), "# Test").unwrap();
        std::fs::write(temp_dir.join("vendor").join("lib.md"), "# Lib").unwrap();

        let config = Config::from_toml(
            r#"
include = ["**/*.md"]
exclude = ["vendor/**"]
"#,
        )
        .unwrap();
        let files = config.collect_files(&temp_dir).unwrap();

        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("README.md"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_collect_files_empty_include() {
        let temp_dir = std::env::temp_dir().join("hongdown_test_empty");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();
        std::fs::write(temp_dir.join("README.md"), "# Test").unwrap();

        let config = Config::default();
        let files = config.collect_files(&temp_dir).unwrap();

        assert!(files.is_empty());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_default_punctuation_config() {
        let config = PunctuationConfig::default();
        assert!(config.curly_double_quotes);
        assert!(config.curly_single_quotes);
        assert!(!config.curly_apostrophes);
        assert!(config.ellipsis);
        assert_eq!(config.en_dash, DashSetting::Disabled);
        assert_eq!(config.em_dash, DashSetting::Pattern("--".to_string()));
    }

    #[test]
    fn test_parse_punctuation_config_all_options() {
        let config = Config::from_toml(
            r#"
[punctuation]
curly_double_quotes = false
curly_single_quotes = false
curly_apostrophes = true
ellipsis = false
en_dash = "--"
em_dash = "---"
"#,
        )
        .unwrap();
        assert!(!config.punctuation.curly_double_quotes);
        assert!(!config.punctuation.curly_single_quotes);
        assert!(config.punctuation.curly_apostrophes);
        assert!(!config.punctuation.ellipsis);
        assert_eq!(
            config.punctuation.en_dash,
            DashSetting::Pattern("--".to_string())
        );
        assert_eq!(
            config.punctuation.em_dash,
            DashSetting::Pattern("---".to_string())
        );
    }

    #[test]
    fn test_parse_dash_setting_disabled() {
        let config = Config::from_toml(
            r#"
[punctuation]
em_dash = false
"#,
        )
        .unwrap();
        assert_eq!(config.punctuation.em_dash, DashSetting::Disabled);
    }

    #[test]
    fn test_parse_dash_setting_pattern() {
        let config = Config::from_toml(
            r#"
[punctuation]
en_dash = "---"
"#,
        )
        .unwrap();
        assert_eq!(
            config.punctuation.en_dash,
            DashSetting::Pattern("---".to_string())
        );
    }

    #[test]
    fn test_punctuation_config_in_full_config() {
        let config = Config::from_toml(
            r#"
line_width = 100

[punctuation]
curly_double_quotes = true
em_dash = "--"
"#,
        )
        .unwrap();
        assert_eq!(config.line_width, 100);
        assert!(config.punctuation.curly_double_quotes);
        assert_eq!(
            config.punctuation.em_dash,
            DashSetting::Pattern("--".to_string())
        );
    }

    #[test]
    fn test_default_code_block_formatters() {
        let config = Config::default();
        assert!(config.code_block.formatters.is_empty());
    }

    #[test]
    fn test_parse_formatter_simple() {
        let config = Config::from_toml(
            r#"
[code_block.formatters]
javascript = ["deno", "fmt", "-"]
"#,
        )
        .unwrap();
        let formatter = config.code_block.formatters.get("javascript").unwrap();
        assert_eq!(formatter.command(), &["deno", "fmt", "-"]);
        assert_eq!(formatter.timeout(), 5);
    }

    #[test]
    fn test_parse_formatter_full() {
        let config = Config::from_toml(
            r#"
[code_block.formatters.python]
command = ["black", "-"]
timeout = 10
"#,
        )
        .unwrap();
        let formatter = config.code_block.formatters.get("python").unwrap();
        assert_eq!(formatter.command(), &["black", "-"]);
        assert_eq!(formatter.timeout(), 10);
    }

    #[test]
    fn test_parse_formatter_full_default_timeout() {
        let config = Config::from_toml(
            r#"
[code_block.formatters.rust]
command = ["rustfmt"]
"#,
        )
        .unwrap();
        let formatter = config.code_block.formatters.get("rust").unwrap();
        assert_eq!(formatter.command(), &["rustfmt"]);
        assert_eq!(formatter.timeout(), 5);
    }

    #[test]
    fn test_parse_multiple_formatters() {
        let config = Config::from_toml(
            r#"
[code_block.formatters]
javascript = ["deno", "fmt", "-"]
typescript = ["deno", "fmt", "-"]

[code_block.formatters.python]
command = ["black", "-"]
timeout = 10
"#,
        )
        .unwrap();
        assert_eq!(config.code_block.formatters.len(), 3);
        assert!(config.code_block.formatters.contains_key("javascript"));
        assert!(config.code_block.formatters.contains_key("typescript"));
        assert!(config.code_block.formatters.contains_key("python"));
    }

    #[test]
    fn test_formatter_empty_command_validation() {
        let config = Config::from_toml(
            r#"
[code_block.formatters]
javascript = []
"#,
        )
        .unwrap();
        assert!(
            config
                .code_block
                .formatters
                .get("javascript")
                .unwrap()
                .validate()
                .is_err()
        );
    }

    #[test]
    fn test_formatter_valid_command_validation() {
        let config = Config::from_toml(
            r#"
[code_block.formatters]
javascript = ["deno", "fmt", "-"]
"#,
        )
        .unwrap();
        assert!(
            config
                .code_block
                .formatters
                .get("javascript")
                .unwrap()
                .validate()
                .is_ok()
        );
    }
}

#[cfg(test)]
mod unordered_marker_tests {
    use super::*;

    #[test]
    fn test_unordered_marker_default() {
        let marker = UnorderedMarker::default();
        assert_eq!(marker, UnorderedMarker::Hyphen);
        assert_eq!(marker.as_char(), '-');
    }

    #[test]
    fn test_unordered_marker_hyphen() {
        let config = Config::from_toml(
            r#"
[unordered_list]
unordered_marker = "-"
"#,
        )
        .unwrap();
        assert_eq!(
            config.unordered_list.unordered_marker,
            UnorderedMarker::Hyphen
        );
        assert_eq!(config.unordered_list.unordered_marker.as_char(), '-');
    }

    #[test]
    fn test_unordered_marker_asterisk() {
        let config = Config::from_toml(
            r#"
[unordered_list]
unordered_marker = "*"
"#,
        )
        .unwrap();
        assert_eq!(
            config.unordered_list.unordered_marker,
            UnorderedMarker::Asterisk
        );
        assert_eq!(config.unordered_list.unordered_marker.as_char(), '*');
    }

    #[test]
    fn test_unordered_marker_plus() {
        let config = Config::from_toml(
            r#"
[unordered_list]
unordered_marker = "+"
"#,
        )
        .unwrap();
        assert_eq!(
            config.unordered_list.unordered_marker,
            UnorderedMarker::Plus
        );
        assert_eq!(config.unordered_list.unordered_marker.as_char(), '+');
    }

    #[test]
    fn test_unordered_marker_invalid_period() {
        let result = Config::from_toml(
            r#"
[unordered_list]
unordered_marker = "."
"#,
        );
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("unordered_marker"));
    }

    #[test]
    fn test_unordered_marker_invalid_letter() {
        let result = Config::from_toml(
            r#"
[unordered_list]
unordered_marker = "x"
"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_unordered_marker_invalid_number() {
        let result = Config::from_toml(
            r#"
[unordered_list]
unordered_marker = "1"
"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_unordered_marker_invalid_empty() {
        let result = Config::from_toml(
            r#"
[unordered_list]
unordered_marker = ""
"#,
        );
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod ordered_marker_tests {
    use super::*;

    #[test]
    fn test_ordered_marker_default() {
        let marker = OrderedMarker::default();
        assert_eq!(marker, OrderedMarker::Period);
        assert_eq!(marker.as_char(), '.');
    }

    #[test]
    fn test_ordered_marker_period() {
        let config = Config::from_toml(
            r#"
[ordered_list]
odd_level_marker = "."
"#,
        )
        .unwrap();
        assert_eq!(config.ordered_list.odd_level_marker, OrderedMarker::Period);
        assert_eq!(config.ordered_list.odd_level_marker.as_char(), '.');
    }

    #[test]
    fn test_ordered_marker_parenthesis() {
        let config = Config::from_toml(
            r#"
[ordered_list]
even_level_marker = ")"
"#,
        )
        .unwrap();
        assert_eq!(
            config.ordered_list.even_level_marker,
            OrderedMarker::Parenthesis
        );
        assert_eq!(config.ordered_list.even_level_marker.as_char(), ')');
    }

    #[test]
    fn test_ordered_marker_invalid_hyphen() {
        let result = Config::from_toml(
            r#"
[ordered_list]
odd_level_marker = "-"
"#,
        );
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("odd_level_marker"));
    }

    #[test]
    fn test_ordered_marker_invalid_asterisk() {
        let result = Config::from_toml(
            r#"
[ordered_list]
even_level_marker = "*"
"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_ordered_marker_invalid_letter() {
        let result = Config::from_toml(
            r#"
[ordered_list]
odd_level_marker = "a"
"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_ordered_marker_invalid_empty() {
        let result = Config::from_toml(
            r#"
[ordered_list]
odd_level_marker = ""
"#,
        );
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod fence_char_tests {
    use super::*;

    #[test]
    fn test_fence_char_default_is_tilde() {
        assert_eq!(FenceChar::default(), FenceChar::Tilde);
    }

    #[test]
    fn test_fence_char_as_char() {
        assert_eq!(FenceChar::Tilde.as_char(), '~');
        assert_eq!(FenceChar::Backtick.as_char(), '`');
    }

    #[test]
    fn test_fence_char_parse_tilde() {
        let config = Config::from_toml(
            r#"
[code_block]
fence_char = "~"
"#,
        )
        .unwrap();
        assert_eq!(config.code_block.fence_char, FenceChar::Tilde);
    }

    #[test]
    fn test_fence_char_parse_backtick() {
        let config = Config::from_toml(
            r#"
[code_block]
fence_char = "`"
"#,
        )
        .unwrap();
        assert_eq!(config.code_block.fence_char, FenceChar::Backtick);
    }

    #[test]
    fn test_fence_char_invalid_char() {
        let result = Config::from_toml(
            r##"
[code_block]
fence_char = "#"
"##,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_fence_char_invalid_empty() {
        let result = Config::from_toml(
            r#"
[code_block]
fence_char = ""
"#,
        );
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod min_fence_length_tests {
    use super::*;

    #[test]
    fn test_min_fence_length_default() {
        assert_eq!(MinFenceLength::default().get(), 4);
    }

    #[test]
    fn test_min_fence_length_valid() {
        assert_eq!(MinFenceLength::new(3).unwrap().get(), 3);
        assert_eq!(MinFenceLength::new(4).unwrap().get(), 4);
        assert_eq!(MinFenceLength::new(10).unwrap().get(), 10);
    }

    #[test]
    fn test_min_fence_length_too_small() {
        assert!(MinFenceLength::new(0).is_err());
        assert!(MinFenceLength::new(1).is_err());
        assert!(MinFenceLength::new(2).is_err());
    }

    #[test]
    fn test_min_fence_length_parse_valid() {
        let config = Config::from_toml(
            r#"
[code_block]
min_fence_length = 3
"#,
        )
        .unwrap();
        assert_eq!(config.code_block.min_fence_length.get(), 3);
    }

    #[test]
    fn test_min_fence_length_parse_invalid() {
        let result = Config::from_toml(
            r#"
[code_block]
min_fence_length = 2
"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_min_fence_length_parse_zero() {
        let result = Config::from_toml(
            r#"
[code_block]
min_fence_length = 0
"#,
        );
        assert!(result.is_err());
    }
}
