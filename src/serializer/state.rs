//! Serializer state and common types.

use indexmap::IndexMap;

use comrak::nodes::{AstNode, ListType, NodeValue};

use crate::Options;

/// The current formatting skip mode.
///
/// Controls whether and how formatting should be skipped for content.
/// Only one mode can be active at a time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FormatSkipMode {
    /// Normal formatting is active.
    #[default]
    None,
    /// Skip formatting for the next block element only.
    /// Automatically resets to `None` after the block is processed.
    NextBlock,
    /// Skip formatting until the next section heading (h2 or lower).
    /// Automatically resets to `None` when a heading is encountered.
    UntilSection,
    /// Formatting is disabled (by `hongdown-disable` directive).
    /// Remains active until `hongdown-enable` directive is encountered.
    Disabled,
}

/// Formatting directives that can be embedded in HTML comments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Directive {
    /// Disable formatting for the next block element only.
    DisableNextLine,
    /// Disable formatting for the entire file.
    DisableFile,
    /// Disable formatting for the next section (until next heading).
    DisableNextSection,
    /// Disable formatting from this point until `Enable`.
    Disable,
    /// Re-enable formatting after `Disable`.
    Enable,
    /// Define proper nouns for sentence case (case-sensitive).
    ProperNouns(Vec<String>),
    /// Define common nouns for sentence case (case-sensitive).
    CommonNouns(Vec<String>),
}

impl Directive {
    /// Parse a directive from an HTML comment.
    /// Returns `Some(Directive)` if the comment contains a valid directive.
    pub fn parse(html: &str) -> Option<Self> {
        let trimmed = html.trim();
        // Check if it's an HTML comment
        if !trimmed.starts_with("<!--") || !trimmed.ends_with("-->") {
            return None;
        }
        // Extract the content between <!-- and -->
        let content = trimmed.strip_prefix("<!--")?.strip_suffix("-->")?.trim();

        // Check for directives without arguments
        match content {
            "hongdown-disable-next-line" => return Some(Directive::DisableNextLine),
            "hongdown-disable-file" => return Some(Directive::DisableFile),
            "hongdown-disable-next-section" => return Some(Directive::DisableNextSection),
            "hongdown-disable" => return Some(Directive::Disable),
            "hongdown-enable" => return Some(Directive::Enable),
            _ => {}
        }

        // Check for directives with arguments
        if let Some(args) = content.strip_prefix("hongdown-proper-nouns:") {
            let nouns = args
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            return Some(Directive::ProperNouns(nouns));
        }

        if let Some(args) = content.strip_prefix("hongdown-common-nouns:") {
            let nouns = args
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            return Some(Directive::CommonNouns(nouns));
        }

        None
    }
}

/// A reference link definition: label -> (url, title)
#[derive(Debug, Clone)]
pub struct ReferenceLink {
    pub label: String,
    pub url: String,
    pub title: String,
}

/// A footnote definition: name -> content
#[derive(Debug, Clone)]
pub struct FootnoteDefinition {
    pub name: String,
    pub content: String,
    /// Line number where the footnote was referenced (1-indexed)
    pub reference_line: usize,
}

/// Manages footnote definitions and their reference tracking.
///
/// This struct encapsulates all state related to footnote processing:
/// - Pending footnote definitions waiting to be emitted
/// - Tracking which footnotes have been emitted
/// - Line numbers where footnotes are referenced
/// - Reference links found within footnote content
#[derive(Debug, Default)]
pub struct FootnoteSet {
    /// Footnote definitions collected for the current section.
    /// Key: name, Value: FootnoteDefinition (insertion order preserved)
    pub pending: IndexMap<String, FootnoteDefinition>,
    /// Footnote names that have already been emitted (to avoid duplicates)
    pub emitted: std::collections::HashSet<String>,
    /// Line numbers where footnotes were referenced (key: footnote name)
    pub reference_lines: std::collections::HashMap<String, usize>,
    /// Whether we're currently collecting footnote content.
    /// When true, reference links are added to `pending_references` instead of
    /// the main reference collection.
    pub collecting_content: bool,
    /// The reference line of the footnote currently being collected.
    /// Used to associate references with their parent footnote's timing.
    pub current_reference_line: usize,
    /// Reference links collected from within footnote definitions.
    /// Value is (ReferenceLink, footnote_reference_line) to track when to flush.
    pub pending_references: IndexMap<String, (ReferenceLink, usize)>,
}

impl FootnoteSet {
    /// Create a new empty FootnoteSet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a footnote definition.
    pub fn add(&mut self, name: String, content: String, reference_line: usize) {
        self.pending.insert(
            name.clone(),
            FootnoteDefinition {
                name,
                content,
                reference_line,
            },
        );
    }

    /// Record the line where a footnote is referenced.
    pub fn record_reference_line(&mut self, name: String, line: usize) {
        self.reference_lines.entry(name).or_insert(line);
    }

    /// Get the reference line for a footnote.
    pub fn get_reference_line(&self, name: &str) -> Option<usize> {
        self.reference_lines.get(name).copied()
    }

    /// Start collecting content for a footnote.
    pub fn start_collecting(&mut self, reference_line: usize) {
        self.collecting_content = true;
        self.current_reference_line = reference_line;
    }

    /// Stop collecting content for a footnote.
    pub fn stop_collecting(&mut self) {
        self.collecting_content = false;
        self.current_reference_line = 0;
    }

    /// Add a reference link found within footnote content.
    pub fn add_reference(&mut self, label: String, reference: ReferenceLink) {
        self.pending_references
            .insert(label, (reference, self.current_reference_line));
    }
}

/// A warning generated during formatting.
#[derive(Debug, Clone)]
pub struct Warning {
    /// Line number where the issue was detected (1-indexed)
    pub line: usize,
    /// Warning message
    pub message: String,
}

/// Safely slice a string, ensuring the indices are valid UTF-8 boundaries.
/// If the indices are not valid boundaries, adjusts to the nearest valid boundary.
fn safe_str_slice(s: &str, start: usize, end: usize) -> &str {
    let safe_start = if start >= s.len() {
        s.len()
    } else if s.is_char_boundary(start) {
        start
    } else {
        // Find the previous valid boundary
        (0..start)
            .rev()
            .find(|&i| s.is_char_boundary(i))
            .unwrap_or(0)
    };

    let safe_end = if end >= s.len() {
        s.len()
    } else if s.is_char_boundary(end) {
        end
    } else {
        // Find the next valid boundary
        (end..=s.len())
            .find(|&i| s.is_char_boundary(i))
            .unwrap_or(s.len())
    };

    &s[safe_start..safe_end]
}

/// The main serializer state for converting comrak AST to formatted Markdown.
pub struct Serializer<'a> {
    pub output: String,
    pub options: &'a Options,
    /// Original source lines for extracting unformatted content
    pub source_lines: Vec<&'a str>,
    /// Current list item index (1-based) for ordered lists
    pub list_item_index: usize,
    /// Current list type
    pub list_type: Option<ListType>,
    /// Whether the current list is tight (no blank lines between items)
    pub list_tight: bool,
    /// Whether we're inside a block quote
    pub in_block_quote: bool,
    /// Accumulated blockquote prefix for nested blockquotes (e.g., "> " or "> > ")
    pub blockquote_prefix: String,
    /// Reference links collected for the current section
    /// Key: label, Value: ReferenceLink (insertion order preserved)
    pub pending_references: IndexMap<String, ReferenceLink>,
    /// Reference labels that have already been emitted (to avoid duplicates)
    pub emitted_references: std::collections::HashSet<String>,
    /// Footnote definitions and their reference tracking
    pub footnotes: FootnoteSet,
    /// Current list nesting depth (0 = not in list, 1 = top-level, 2+ = nested)
    pub list_depth: usize,
    /// Current formatting skip mode
    pub skip_mode: FormatSkipMode,
    /// Whether we're inside a description details block (for indentation)
    pub in_description_details: bool,
    /// Warnings generated during formatting
    pub warnings: Vec<Warning>,
    /// Maximum number of items in the current ordered list (for padding calculation)
    pub ordered_list_max_items: usize,
    /// Whether the original source ends with a newline
    pub source_ends_with_newline: bool,
    /// Current indentation prefix for list item content (e.g., "     " for ` 1.  `)
    /// Used by blockquotes and other block elements inside list items.
    pub list_item_indent: String,
    /// Indentation prefix for content inside a blockquote that's nested inside a list.
    /// This is the outer list's indent that should appear before each `>` in the blockquote.
    pub blockquote_outer_indent: String,
    /// The list depth when entering the current blockquote.
    /// Used to determine if a list exists inside vs outside the blockquote.
    pub blockquote_entry_list_depth: usize,
    /// Proper nouns defined via directives for sentence case (merged with config)
    pub directive_proper_nouns: Vec<String>,
    /// Common nouns defined via directives for sentence case (merged with config)
    pub directive_common_nouns: Vec<String>,
}

impl<'a> Serializer<'a> {
    pub fn new(
        options: &'a Options,
        source_lines: Vec<&'a str>,
        source_ends_with_newline: bool,
    ) -> Self {
        Self {
            output: String::new(),
            options,
            source_lines,
            list_item_index: 0,
            list_type: None,
            list_tight: true,
            in_block_quote: false,
            blockquote_prefix: String::new(),
            pending_references: IndexMap::new(),
            emitted_references: std::collections::HashSet::new(),
            footnotes: FootnoteSet::new(),
            list_depth: 0,
            skip_mode: FormatSkipMode::None,
            in_description_details: false,
            warnings: Vec::new(),
            ordered_list_max_items: 0,
            source_ends_with_newline,
            list_item_indent: String::new(),
            blockquote_outer_indent: String::new(),
            blockquote_entry_list_depth: 0,
            directive_proper_nouns: Vec::new(),
            directive_common_nouns: Vec::new(),
        }
    }

    /// Add a warning.
    pub fn add_warning(&mut self, line: usize, message: String) {
        self.warnings.push(Warning { line, message });
    }

    /// Extract original source text for a node using its sourcepos.
    pub fn extract_source<'b>(&self, node: &'b AstNode<'b>) -> Option<String> {
        if self.source_lines.is_empty() {
            return None;
        }
        let sourcepos = node.data.borrow().sourcepos;
        let start_line = sourcepos.start.line;
        let end_line = sourcepos.end.line;
        let start_col = sourcepos.start.column;
        let end_col = sourcepos.end.column;

        if start_line == 0 || end_line == 0 {
            return None;
        }

        // Lines and columns are 1-indexed in sourcepos
        let start_idx = start_line - 1;
        let end_idx = end_line - 1;

        if end_idx >= self.source_lines.len() {
            return None;
        }

        let mut result = String::new();
        for i in start_idx..=end_idx {
            if i > start_idx {
                result.push('\n');
            }
            let line = self.source_lines[i];
            if start_idx == end_idx {
                // Single line: extract from start_col to end_col
                let start_byte = start_col.saturating_sub(1);
                let end_byte = end_col;
                result.push_str(safe_str_slice(line, start_byte, end_byte));
            } else if i == start_idx {
                // First line: from start_col to end
                let start_byte = start_col.saturating_sub(1);
                result.push_str(safe_str_slice(line, start_byte, line.len()));
            } else if i == end_idx {
                // Last line: from start to end_col
                let end_byte = end_col.min(line.len());
                result.push_str(safe_str_slice(line, 0, end_byte));
            } else {
                // Middle lines: full line
                result.push_str(line);
            }
        }
        Some(result)
    }

    /// Extract original source text from a given line to the end of the file.
    /// Line numbers are 1-indexed.
    pub fn extract_source_from_line(&self, start_line: usize) -> Option<String> {
        if self.source_lines.is_empty() || start_line == 0 {
            return None;
        }
        let start_idx = start_line - 1;
        if start_idx >= self.source_lines.len() {
            return None;
        }
        let mut result = String::new();
        for (i, line) in self.source_lines.iter().enumerate().skip(start_idx) {
            if i > start_idx {
                result.push('\n');
            }
            result.push_str(line);
        }
        // Preserve trailing newline if the original source had one
        if self.source_ends_with_newline {
            result.push('\n');
        }
        Some(result)
    }

    /// Check if formatting should be skipped for this node.
    pub fn should_skip_formatting(&self) -> bool {
        self.skip_mode != FormatSkipMode::None
    }

    /// Add a reference link to the pending references.
    /// If collecting_footnote_content is true, adds to pending_footnote_references instead,
    /// along with the current footnote's reference line for proper flush timing.
    pub fn add_reference(&mut self, label: String, url: String, title: String) {
        let reference = ReferenceLink {
            label: label.clone(),
            url,
            title,
        };
        if self.footnotes.collecting_content {
            self.footnotes.add_reference(label, reference);
        } else {
            self.pending_references.insert(label, reference);
        }
    }

    /// Check if a URL is external (starts with http:// or https://).
    pub fn is_external_url(url: &str) -> bool {
        url.starts_with("http://") || url.starts_with("https://")
    }

    /// Get the emphasis delimiter character.
    /// Uses '_' if the content contains '*' (to avoid escaping).
    /// Otherwise, preserves the original delimiter from source, defaulting to '*'.
    pub fn get_emphasis_delimiter<'b>(&self, node: &'b AstNode<'b>) -> char {
        // If content contains '*', use '_' to avoid escaping
        if self.node_text_contains_char(node, '*') {
            return '_';
        }
        // Otherwise, preserve original delimiter or default to '*'
        if let Some(source) = self.extract_source(node)
            && source.starts_with('_')
        {
            return '_';
        }
        '*'
    }

    /// Get the strong emphasis delimiter string.
    /// Uses "__" if the content contains '*' (to avoid escaping).
    /// Otherwise, preserves the original delimiter from source, defaulting to "**".
    pub fn get_strong_delimiter<'b>(&self, node: &'b AstNode<'b>) -> &'static str {
        // If content contains '*', use '__' to avoid escaping
        if self.node_text_contains_char(node, '*') {
            return "__";
        }
        // Otherwise, preserve original delimiter or default to '**'
        if let Some(source) = self.extract_source(node)
            && source.starts_with("__")
        {
            return "__";
        }
        "**"
    }

    /// Check if any text node within the given node contains the specified character.
    fn node_text_contains_char<'b>(&self, node: &'b AstNode<'b>, ch: char) -> bool {
        self.node_text_contains_char_recursive(node, ch)
    }

    fn node_text_contains_char_recursive<'b>(&self, node: &'b AstNode<'b>, ch: char) -> bool {
        match &node.data.borrow().value {
            NodeValue::Text(t) => t.contains(ch),
            _ => node
                .children()
                .any(|child| self.node_text_contains_char_recursive(child, ch)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::safe_str_slice;

    #[test]
    fn test_safe_str_slice_ascii() {
        let s = "hello world";
        assert_eq!(safe_str_slice(s, 0, 5), "hello");
        assert_eq!(safe_str_slice(s, 6, 11), "world");
        assert_eq!(safe_str_slice(s, 0, 11), "hello world");
    }

    #[test]
    fn test_safe_str_slice_valid_utf8_boundaries() {
        // ✅ is 3 bytes: [226, 156, 133]
        let s = "✅ test";
        assert_eq!(safe_str_slice(s, 0, 3), "✅");
        assert_eq!(safe_str_slice(s, 4, 8), "test");
        assert_eq!(safe_str_slice(s, 0, 8), "✅ test");
    }

    #[test]
    fn test_safe_str_slice_invalid_start_boundary() {
        // ✅ is 3 bytes: [226, 156, 133]
        let s = "✅ test";
        // Start at byte 1 (middle of ✅) should adjust to byte 0
        assert_eq!(safe_str_slice(s, 1, 8), "✅ test");
        // Start at byte 2 (middle of ✅) should adjust to byte 0
        assert_eq!(safe_str_slice(s, 2, 8), "✅ test");
    }

    #[test]
    fn test_safe_str_slice_invalid_end_boundary() {
        // ✅ is 3 bytes: [226, 156, 133]
        let s = "✅ test";
        // End at byte 1 (middle of ✅) should adjust to byte 3
        assert_eq!(safe_str_slice(s, 0, 1), "✅");
        // End at byte 2 (middle of ✅) should adjust to byte 3
        assert_eq!(safe_str_slice(s, 0, 2), "✅");
    }

    #[test]
    fn test_safe_str_slice_out_of_bounds() {
        let s = "hello";
        // End beyond string length
        assert_eq!(safe_str_slice(s, 0, 100), "hello");
        // Start beyond string length
        assert_eq!(safe_str_slice(s, 100, 200), "");
    }

    #[test]
    fn test_safe_str_slice_multiple_emoji() {
        // 🚨 is 4 bytes, ✅ is 3 bytes
        let s = "🚨 ✅";
        assert_eq!(safe_str_slice(s, 0, 4), "🚨");
        assert_eq!(safe_str_slice(s, 5, 8), "✅");
        // Invalid boundary in middle of 🚨
        assert_eq!(safe_str_slice(s, 1, 8), "🚨 ✅");
        assert_eq!(safe_str_slice(s, 2, 8), "🚨 ✅");
        assert_eq!(safe_str_slice(s, 3, 8), "🚨 ✅");
    }

    #[test]
    fn test_safe_str_slice_emoji_only() {
        // When the string is just an emoji and we try to slice at byte 1
        let s = "✅";
        // This was the exact case causing the panic: byte index 1 in a 3-byte emoji
        assert_eq!(safe_str_slice(s, 1, 3), "✅");
        assert_eq!(safe_str_slice(s, 0, 1), "✅");
    }
}
