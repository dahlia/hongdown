// SPDX-FileCopyrightText: 2025 Hong Minhee <https://hongminhee.org/>
// SPDX-License-Identifier: GPL-3.0-or-later
//! Configuration file support for Hongdown.
//!
//! This module provides functionality for loading and parsing configuration
//! files (`.hongdown.toml`) that control the formatter's behavior.

use serde::Deserialize;
use std::path::{Path, PathBuf};

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
    pub list: ListConfig,

    /// Ordered list formatting options.
    pub ordered_list: OrderedListConfig,

    /// Code block formatting options.
    pub code_block: CodeBlockConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            line_width: 80,
            include: Vec::new(),
            exclude: Vec::new(),
            heading: HeadingConfig::default(),
            list: ListConfig::default(),
            ordered_list: OrderedListConfig::default(),
            code_block: CodeBlockConfig::default(),
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
}

impl Default for HeadingConfig {
    fn default() -> Self {
        Self {
            setext_h1: true,
            setext_h2: true,
        }
    }
}

/// Unordered list formatting options.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(default)]
pub struct ListConfig {
    /// Marker character: `-`, `*`, or `+` (default: `-`).
    pub unordered_marker: char,

    /// Spaces before the marker (default: 1).
    pub leading_spaces: usize,

    /// Spaces after the marker (default: 2).
    pub trailing_spaces: usize,

    /// Indentation width for nested items (default: 4).
    pub indent_width: usize,
}

impl Default for ListConfig {
    fn default() -> Self {
        Self {
            unordered_marker: '-',
            leading_spaces: 1,
            trailing_spaces: 2,
            indent_width: 4,
        }
    }
}

/// Ordered list formatting options.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(default)]
pub struct OrderedListConfig {
    /// Marker style at odd nesting levels: `.` for `1.` (default: `.`).
    pub odd_level_marker: char,

    /// Marker style at even nesting levels: `)` for `1)` (default: `)`).
    pub even_level_marker: char,
}

impl Default for OrderedListConfig {
    fn default() -> Self {
        Self {
            odd_level_marker: '.',
            even_level_marker: ')',
        }
    }
}

/// Code block formatting options.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(default)]
pub struct CodeBlockConfig {
    /// Fence character: `~` or `` ` `` (default: `~`).
    pub fence_char: char,

    /// Minimum fence length (default: 4).
    pub min_fence_length: usize,

    /// Add space between fence and language identifier (default: true).
    pub space_after_fence: bool,
}

impl Default for CodeBlockConfig {
    fn default() -> Self {
        Self {
            fence_char: '~',
            min_fence_length: 4,
            space_after_fence: true,
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
        assert_eq!(config.list.unordered_marker, '-');
        assert_eq!(config.list.leading_spaces, 1);
        assert_eq!(config.list.trailing_spaces, 2);
        assert_eq!(config.list.indent_width, 4);
        assert_eq!(config.ordered_list.odd_level_marker, '.');
        assert_eq!(config.ordered_list.even_level_marker, ')');
        assert_eq!(config.code_block.fence_char, '~');
        assert_eq!(config.code_block.min_fence_length, 4);
        assert!(config.code_block.space_after_fence);
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
    fn test_parse_list_config() {
        let config = Config::from_toml(
            r#"
[list]
unordered_marker = "*"
leading_spaces = 0
trailing_spaces = 1
indent_width = 2
"#,
        )
        .unwrap();
        assert_eq!(config.list.unordered_marker, '*');
        assert_eq!(config.list.leading_spaces, 0);
        assert_eq!(config.list.trailing_spaces, 1);
        assert_eq!(config.list.indent_width, 2);
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
        assert_eq!(config.ordered_list.odd_level_marker, ')');
        assert_eq!(config.ordered_list.even_level_marker, '.');
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
        assert_eq!(config.code_block.fence_char, '`');
        assert_eq!(config.code_block.min_fence_length, 3);
        assert!(!config.code_block.space_after_fence);
    }

    #[test]
    fn test_parse_full_config() {
        let config = Config::from_toml(
            r#"
line_width = 80

[heading]
setext_h1 = true
setext_h2 = true

[list]
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
"#,
        )
        .unwrap();
        assert_eq!(config, Config::default());
    }

    #[test]
    fn test_parse_invalid_toml() {
        let result = Config::from_toml("line_width = \"not a number\"");
        assert!(result.is_err());
    }

    #[test]
    fn test_discover_no_config() {
        let temp_dir = std::env::temp_dir().join("hongdown_test_no_config");
        let _ = std::fs::create_dir_all(&temp_dir);
        let result = Config::discover(&temp_dir).unwrap();
        assert!(result.is_none());
        let _ = std::fs::remove_dir_all(&temp_dir);
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
}
