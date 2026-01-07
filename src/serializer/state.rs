//! Serializer state and common types.

use indexmap::IndexMap;

use comrak::nodes::{AstNode, ListType};

use crate::Options;

/// Formatting directives that can be embedded in HTML comments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

        match content {
            "hongdown-disable-next-line" => Some(Directive::DisableNextLine),
            "hongdown-disable-file" => Some(Directive::DisableFile),
            "hongdown-disable-next-section" => Some(Directive::DisableNextSection),
            "hongdown-disable" => Some(Directive::Disable),
            "hongdown-enable" => Some(Directive::Enable),
            _ => None,
        }
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

/// A warning generated during formatting.
#[derive(Debug, Clone)]
pub struct Warning {
    /// Line number where the issue was detected (1-indexed)
    pub line: usize,
    /// Warning message
    pub message: String,
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
    /// Reference links collected for the current section
    /// Key: label, Value: ReferenceLink (insertion order preserved)
    pub pending_references: IndexMap<String, ReferenceLink>,
    /// Reference labels that have already been emitted (to avoid duplicates)
    pub emitted_references: std::collections::HashSet<String>,
    /// Footnote definitions collected for the current section
    /// Key: name, Value: FootnoteDefinition (insertion order preserved)
    pub pending_footnotes: IndexMap<String, FootnoteDefinition>,
    /// Footnote names that have already been emitted (to avoid duplicates)
    pub emitted_footnotes: std::collections::HashSet<String>,
    /// Line numbers where footnotes were referenced (key: footnote name)
    pub footnote_reference_lines: std::collections::HashMap<String, usize>,
    /// Current list nesting depth (0 = not in list, 1 = top-level, 2+ = nested)
    pub list_depth: usize,
    /// Formatting is disabled (by `hongdown-disable` or `hongdown-disable-file`)
    pub formatting_disabled: bool,
    /// Skip formatting for the next block element only
    pub skip_next_block: bool,
    /// Skip formatting until the next section heading
    pub skip_until_section: bool,
    /// Whether we're inside a description details block (for indentation)
    pub in_description_details: bool,
    /// Warnings generated during formatting
    pub warnings: Vec<Warning>,
}

impl<'a> Serializer<'a> {
    pub fn new(options: &'a Options, source_lines: Vec<&'a str>) -> Self {
        Self {
            output: String::new(),
            options,
            source_lines,
            list_item_index: 0,
            list_type: None,
            list_tight: true,
            in_block_quote: false,
            pending_references: IndexMap::new(),
            emitted_references: std::collections::HashSet::new(),
            pending_footnotes: IndexMap::new(),
            emitted_footnotes: std::collections::HashSet::new(),
            footnote_reference_lines: std::collections::HashMap::new(),
            list_depth: 0,
            formatting_disabled: false,
            skip_next_block: false,
            skip_until_section: false,
            in_description_details: false,
            warnings: Vec::new(),
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
                if end_byte <= line.len() {
                    result.push_str(&line[start_byte..end_byte]);
                } else {
                    result.push_str(&line[start_byte..]);
                }
            } else if i == start_idx {
                // First line: from start_col to end
                let start_byte = start_col.saturating_sub(1);
                result.push_str(&line[start_byte..]);
            } else if i == end_idx {
                // Last line: from start to end_col
                let end_byte = end_col.min(line.len());
                result.push_str(&line[..end_byte]);
            } else {
                // Middle lines: full line
                result.push_str(line);
            }
        }
        Some(result)
    }

    /// Check if formatting should be skipped for this node.
    pub fn should_skip_formatting(&self) -> bool {
        self.formatting_disabled || self.skip_next_block || self.skip_until_section
    }

    /// Add a reference link to the pending references.
    pub fn add_reference(&mut self, label: String, url: String, title: String) {
        self.pending_references
            .insert(label.clone(), ReferenceLink { label, url, title });
    }

    /// Check if a URL is external (starts with http:// or https://).
    pub fn is_external_url(url: &str) -> bool {
        url.starts_with("http://") || url.starts_with("https://")
    }
}
