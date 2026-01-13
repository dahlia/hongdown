//! Document-level serialization logic.

use comrak::nodes::{AstNode, NodeValue};
use regex::Regex;
use unicode_width::UnicodeWidthStr;

use super::Serializer;
use super::state::{Directive, FormatSkipMode};
use super::wrap;

impl<'a> Serializer<'a> {
    pub(super) fn serialize_document<'b>(&mut self, node: &'b AstNode<'b>) {
        let children: Vec<_> = node.children().collect();

        // Check for undefined reference links using AST
        self.check_undefined_references_ast(node);

        // First pass: collect all footnote reference lines
        // This is needed because FootnoteDefinition nodes come at the end of the AST,
        // but we need to know reference lines before flushing at section boundaries
        self.collect_footnote_reference_lines(node);

        // Second pass: process all FootnoteDefinition nodes first
        // This ensures pending_footnotes is populated before we flush at section boundaries
        for child in &children {
            if let NodeValue::FootnoteDefinition(_) = &child.data.borrow().value {
                self.serialize_node(child);
            }
        }

        // Identify trailing HTML blocks (non-directive comments at the end of document)
        // These should be output after reference definitions to maintain their position
        let trailing_html_start = self.find_trailing_html_blocks(&children);

        for (i, child) in children.iter().enumerate() {
            // Skip trailing HTML blocks for now - they'll be output after references
            if i >= trailing_html_start
                && let NodeValue::HtmlBlock(_) = &child.data.borrow().value
            {
                continue;
            }
            // Skip FootnoteDefinition nodes (already processed above)
            if let NodeValue::FootnoteDefinition(_) = &child.data.borrow().value {
                continue;
            }
            // Check for directives in HTML blocks
            if let NodeValue::HtmlBlock(html_block) = &child.data.borrow().value
                && let Some(directive) = Directive::parse(&html_block.literal)
            {
                match directive {
                    Directive::DisableFile => {
                        // Output the directive comment, then output remaining content as-is
                        self.output.push_str(html_block.literal.trim_end());
                        // Get the line after the directive block ends
                        let directive_end_line = child.data.borrow().sourcepos.end.line;
                        // Extract everything from the next line to the end of file
                        if let Some(remaining) =
                            self.extract_source_from_line(directive_end_line + 1)
                        {
                            self.output.push('\n');
                            self.output.push_str(&remaining);
                        }
                        return;
                    }
                    Directive::DisableNextLine => {
                        self.skip_mode = FormatSkipMode::NextBlock;
                        // Output the directive comment
                        if i > 0 {
                            self.output.push('\n');
                        }
                        self.output.push_str(&html_block.literal);
                        continue;
                    }
                    Directive::DisableNextSection => {
                        self.skip_mode = FormatSkipMode::UntilSection;
                        // Output the directive comment
                        if i > 0 {
                            self.output.push('\n');
                        }
                        self.output.push_str(&html_block.literal);
                        continue;
                    }
                    Directive::Disable => {
                        self.skip_mode = FormatSkipMode::Disabled;
                        // Output the directive comment
                        if i > 0 {
                            self.output.push('\n');
                        }
                        self.output.push_str(&html_block.literal);
                        continue;
                    }
                    Directive::Enable => {
                        self.skip_mode = FormatSkipMode::None;
                        // Output the directive comment
                        if i > 0 {
                            self.output.push('\n');
                        }
                        self.output.push_str(&html_block.literal);
                        continue;
                    }
                    Directive::ProperNouns(nouns) => {
                        // Add to directive proper nouns list
                        self.directive_proper_nouns.extend(nouns);
                        // Output the directive comment
                        if i > 0 {
                            self.output.push('\n');
                        }
                        self.output.push_str(&html_block.literal);
                        continue;
                    }
                    Directive::CommonNouns(nouns) => {
                        // Add to directive common nouns list
                        self.directive_common_nouns.extend(nouns);
                        // Output the directive comment
                        if i > 0 {
                            self.output.push('\n');
                        }
                        self.output.push_str(&html_block.literal);
                        continue;
                    }
                }
            }

            // Check if we're about to start a new section (h2 or h3 heading)
            // If so, flush any pending references and footnotes first
            let heading_level = match &child.data.borrow().value {
                NodeValue::Heading(h) => Some(h.level),
                _ => None,
            };
            let is_h2 = heading_level == Some(2);
            let is_h2_or_h3 = matches!(heading_level, Some(2) | Some(3));

            if is_h2_or_h3 && i > 0 {
                // Get the source line of the heading to flush only earlier footnotes
                let heading_line = child.data.borrow().sourcepos.start.line;
                // Footnotes come before link reference definitions
                self.flush_footnotes_before(Some(heading_line));
                self.flush_references();
                self.flush_footnote_references_before(Some(heading_line));
            }

            // Add blank line between block elements (except after front matter)
            if i > 0 {
                let prev_is_front_matter = matches!(
                    &children[i - 1].data.borrow().value,
                    NodeValue::FrontMatter(_)
                );
                if prev_is_front_matter {
                    // No extra blank line needed after front matter
                } else if is_h2 {
                    // Check if previous element was a heading (empty section)
                    let prev_is_heading =
                        matches!(&children[i - 1].data.borrow().value, NodeValue::Heading(_));
                    if prev_is_heading {
                        // Just one blank line between consecutive headings
                        self.output.push('\n');
                    } else {
                        // Two blank lines before h2 sections (one after content + one extra)
                        self.output.push_str("\n\n");
                    }
                } else {
                    self.output.push('\n');
                }
            }

            // Check if this block should be output as-is (skip formatting)
            if self.should_skip_formatting() {
                // For NextBlock mode, reset after this block
                let was_next_block = self.skip_mode == FormatSkipMode::NextBlock;
                if was_next_block {
                    self.skip_mode = FormatSkipMode::None;
                }

                // For UntilSection mode, check if this is a heading to reset
                if self.skip_mode == FormatSkipMode::UntilSection
                    && let NodeValue::Heading(h) = &child.data.borrow().value
                    && h.level <= 2
                {
                    self.skip_mode = FormatSkipMode::None;
                    // Continue with normal formatting for this heading
                    self.serialize_node(child);
                    continue;
                }

                // Output the original source
                if let Some(source) = self.extract_source(child) {
                    self.output.push_str(&source);
                    self.output.push('\n');
                } else {
                    self.serialize_node(child);
                }
                continue;
            }

            self.serialize_node(child);
        }

        // Footnotes come before link reference definitions
        self.flush_footnotes();
        self.flush_references();
        self.flush_footnote_references();

        // Output trailing HTML blocks after references and footnotes
        self.output_trailing_html_blocks(&children, trailing_html_start);
    }

    /// Find the index where trailing HTML blocks start.
    /// Returns `children.len()` if there are no trailing HTML blocks.
    fn find_trailing_html_blocks<'b>(&self, children: &[&'b AstNode<'b>]) -> usize {
        let mut trailing_start = children.len();

        // Walk backwards from the end, looking for consecutive HTML blocks
        // that are not formatting directives
        for (i, child) in children.iter().enumerate().rev() {
            match &child.data.borrow().value {
                NodeValue::HtmlBlock(html_block) => {
                    // Skip formatting directives - they should stay where they are
                    if Directive::parse(&html_block.literal).is_some() {
                        break;
                    }
                    // This is a regular HTML block (e.g., comment) - mark as trailing
                    trailing_start = i;
                }
                NodeValue::FootnoteDefinition(_) => {
                    // Skip footnote definitions - they're handled separately
                    continue;
                }
                _ => {
                    // Non-HTML block found - stop looking
                    break;
                }
            }
        }

        trailing_start
    }

    /// Output trailing HTML blocks that were deferred until after references.
    fn output_trailing_html_blocks<'b>(
        &mut self,
        children: &[&'b AstNode<'b>],
        start_index: usize,
    ) {
        let mut is_first = true;
        for (i, child) in children.iter().enumerate() {
            if i < start_index {
                continue;
            }

            if let NodeValue::HtmlBlock(html_block) = &child.data.borrow().value {
                // Add a blank line before the first trailing HTML block
                if is_first {
                    if !self.output.ends_with("\n\n") {
                        if self.output.ends_with('\n') {
                            self.output.push('\n');
                        } else {
                            self.output.push_str("\n\n");
                        }
                    }
                    is_first = false;
                }
                self.output.push_str(&html_block.literal);
            }
        }
    }

    pub(super) fn serialize_description_details<'b>(&mut self, node: &'b AstNode<'b>) {
        let children: Vec<_> = node.children().collect();

        // Set flag so nested lists know to add extra indentation
        let was_in_description_details = self.in_description_details;
        self.in_description_details = true;

        // Determine the prefix for blockquote context
        let blockquote_prefix = if self.in_block_quote {
            format!("{}{}", self.blockquote_outer_indent, self.blockquote_prefix)
        } else {
            String::new()
        };

        for (i, child) in children.iter().enumerate() {
            let child_value = &child.data.borrow().value;

            if i == 0 {
                // First child: start with `:   ` marker
                match child_value {
                    NodeValue::Paragraph => {
                        self.output.push_str(&blockquote_prefix);
                        self.output.push_str(":   ");
                        let mut content = String::new();
                        self.collect_inline_content(child, &mut content);
                        let continuation = format!("{}    ", blockquote_prefix);
                        let wrapped = wrap::wrap_text_first_line(
                            content.trim(),
                            "",
                            &continuation,
                            self.options.line_width,
                        );
                        self.output.push_str(&wrapped);
                        self.output.push('\n');
                    }
                    NodeValue::CodeBlock(code) => {
                        // Code block as first child (unusual but possible)
                        self.output.push_str(&blockquote_prefix);
                        self.output.push_str(":   ");
                        self.output.push('\n');
                        self.output.push_str(&blockquote_prefix);
                        self.output.push_str("    ");
                        self.serialize_code_block_with_indent(
                            code,
                            &format!("{}    ", blockquote_prefix),
                        );
                    }
                    NodeValue::List(_) => {
                        // List as first child: output marker, newline, then list
                        // The list will handle its own indentation via in_description_details
                        self.output.push_str(&blockquote_prefix);
                        self.output.push_str(":\n");
                        self.serialize_node(child);
                    }
                    NodeValue::BlockQuote | NodeValue::Alert(_) => {
                        // Block quotes and alerts as first child: output marker, newline,
                        // then serialize with proper list_item_indent for continuation lines
                        self.output.push_str(&blockquote_prefix);
                        self.output.push_str(":\n");
                        let old_list_item_indent =
                            std::mem::replace(&mut self.list_item_indent, "    ".to_string());
                        self.serialize_node(child);
                        self.list_item_indent = old_list_item_indent;
                    }
                    _ => {
                        // Other block types: serialize normally with indent
                        self.output.push_str(&blockquote_prefix);
                        self.output.push_str(":   ");
                        self.serialize_node(child);
                    }
                }
            } else {
                // Subsequent children: need blank line and 4-space indent
                self.output.push('\n');
                match child_value {
                    NodeValue::Paragraph => {
                        self.output.push_str(&blockquote_prefix);
                        self.output.push_str("    ");
                        let mut content = String::new();
                        self.collect_inline_content(child, &mut content);
                        let continuation = format!("{}    ", blockquote_prefix);
                        let wrapped = wrap::wrap_text_first_line(
                            content.trim(),
                            "",
                            &continuation,
                            self.options.line_width,
                        );
                        self.output.push_str(&wrapped);
                        self.output.push('\n');
                    }
                    NodeValue::CodeBlock(code) => {
                        self.output.push_str(&blockquote_prefix);
                        self.output.push_str("    ");
                        self.serialize_code_block_with_indent(
                            code,
                            &format!("{}    ", blockquote_prefix),
                        );
                    }
                    NodeValue::List(_) => {
                        // Lists handle their own indentation via in_description_details flag
                        self.serialize_node(child);
                    }
                    NodeValue::BlockQuote | NodeValue::Alert(_) => {
                        // Block quotes and alerts need list_item_indent to be set
                        // so that their continuation lines are properly indented
                        let old_list_item_indent =
                            std::mem::replace(&mut self.list_item_indent, "    ".to_string());
                        self.serialize_node(child);
                        self.list_item_indent = old_list_item_indent;
                    }
                    _ => {
                        // Other block types
                        self.output.push_str(&blockquote_prefix);
                        self.output.push_str("    ");
                        self.serialize_node(child);
                    }
                }
            }
        }

        self.in_description_details = was_in_description_details;
    }

    pub(super) fn serialize_heading<'b>(&mut self, node: &'b AstNode<'b>, level: u8) {
        // Collect heading text first
        let mut heading_text = self.collect_text(node);

        // Apply sentence case if enabled
        if self.options.heading_sentence_case {
            // Merge config proper nouns with directive proper nouns
            let mut proper_nouns = self.options.heading_proper_nouns.clone();
            proper_nouns.extend(self.directive_proper_nouns.clone());

            // Merge config common nouns with directive common nouns
            let mut common_nouns = self.options.heading_common_nouns.clone();
            common_nouns.extend(self.directive_common_nouns.clone());

            heading_text =
                super::heading::to_sentence_case(&heading_text, &proper_nouns, &common_nouns);
        }

        if level == 1 && self.options.setext_h1 {
            // Setext-style with '='
            self.output.push_str(&heading_text);
            self.output.push('\n');
            self.output.push_str(&"=".repeat(heading_text.width()));
            self.output.push('\n');
        } else if level == 2 && self.options.setext_h2 {
            // Setext-style with '-'
            self.output.push_str(&heading_text);
            self.output.push('\n');
            self.output.push_str(&"-".repeat(heading_text.width()));
            self.output.push('\n');
        } else {
            // ATX-style for level 3+ or when setext is disabled
            self.output.push_str(&"#".repeat(level as usize));
            self.output.push(' ');
            self.output.push_str(&heading_text);
            self.output.push('\n');
        }
    }

    pub(super) fn serialize_paragraph<'b>(&mut self, node: &'b AstNode<'b>) {
        // Check if this is a PHP Markdown Extra abbreviation definition (*[abbr]: ...)
        // These are not parsed by comrak, so we preserve them as-is
        if let Some(source) = self.extract_source(node) {
            let trimmed = source.trim();
            if trimmed.starts_with("*[") && trimmed.contains("]:") {
                self.output.push_str(trimmed);
                self.output.push('\n');
                return;
            }
        }

        // Collect all inline content first
        let mut inline_content = String::new();
        self.collect_inline_content(node, &mut inline_content);

        if self.list_type.is_some() {
            // Inside a list item, wrap with proper continuation indent
            // First line has no prefix (marker already output)
            // Continuation lines need appropriate indent
            //
            // There are two cases:
            // 1. List inside blockquote: list_depth > blockquote_entry_list_depth
            //    - Use base_indent for the inner list's continuation
            // 2. Blockquote inside list: list_depth == blockquote_entry_list_depth
            //    - Don't add base_indent, just use the outer prefix
            let inner_list_depth = self
                .list_depth
                .saturating_sub(self.blockquote_entry_list_depth);
            let base_indent = if self.in_description_details && inner_list_depth > 0 {
                // Inside description details, add extra 5-space indent
                format!("     {}", "    ".repeat(inner_list_depth))
            } else {
                "    ".repeat(inner_list_depth)
            };
            let continuation = if self.in_block_quote {
                // Inside a blockquote, continuation lines need > prefix + indent
                // Use blockquote_outer_indent (the outer list's indent, if any)
                // rather than list_item_indent (which is for the list inside the blockquote)
                format!(
                    "{}{}{}",
                    self.blockquote_outer_indent, self.blockquote_prefix, base_indent
                )
            } else {
                base_indent
            };
            let wrapped = wrap::wrap_text_first_line(
                &inline_content,
                "",
                &continuation,
                self.options.line_width,
            );
            self.output.push_str(&wrapped);
        } else {
            // Not in a list - wrap the paragraph at line_width
            let prefix = if self.in_block_quote {
                format!("{}{}", self.blockquote_outer_indent, self.blockquote_prefix)
            } else {
                String::new()
            };
            let wrapped = wrap::wrap_text(&inline_content, &prefix, self.options.line_width);
            self.output.push_str(&wrapped);
            self.output.push('\n');
        }
    }

    pub(super) fn serialize_front_matter(&mut self, content: &str) {
        // Front matter content from comrak includes the delimiters,
        // so we preserve it verbatim and add a trailing blank line
        self.output.push_str(content.trim());
        self.output.push_str("\n\n");
    }

    /// Recursively collect footnote reference lines from the AST.
    /// This must be called before processing the document to ensure
    /// footnote reference lines are populated for all footnotes.
    fn collect_footnote_reference_lines<'b>(&mut self, node: &'b AstNode<'b>) {
        if let NodeValue::FootnoteReference(footnote_ref) = &node.data.borrow().value {
            let ref_line = node.data.borrow().sourcepos.start.line;
            self.footnotes
                .record_reference_line(footnote_ref.name.clone(), ref_line);
        }
        for child in node.children() {
            self.collect_footnote_reference_lines(child);
        }
    }

    /// Check for undefined reference links using AST traversal.
    ///
    /// This method walks the AST looking for Text nodes that contain `[label]`
    /// patterns. When comrak cannot resolve a reference link, it leaves the
    /// brackets as literal text. We detect these and emit warnings.
    ///
    /// We also check the original source to ensure the bracket wasn't
    /// intentionally escaped (e.g., `\[label]`).
    fn check_undefined_references_ast<'b>(&mut self, node: &'b AstNode<'b>) {
        if self.source_lines.is_empty() {
            return;
        }

        // Collect PHP Markdown Extra abbreviation definitions from source
        let abbreviations = Self::collect_abbreviations(&self.source_lines);

        // Collect reference definitions from source that comrak may not have parsed
        // (e.g., when they follow abbreviation definitions without a blank line)
        let source_ref_defs = Self::collect_source_reference_definitions(&self.source_lines);

        // Collect disabled line ranges based on formatting directives
        let disabled_ranges = Self::collect_disabled_line_ranges(node);

        // Collect warnings first to avoid borrow issues
        let warnings = Self::find_undefined_references_in_ast(
            node,
            &self.source_lines,
            &abbreviations,
            &source_ref_defs,
        );

        // Filter out warnings that fall within disabled regions
        for (line, msg) in warnings {
            if !Self::is_line_in_disabled_ranges(line, &disabled_ranges) {
                self.add_warning(line, msg);
            }
        }
    }

    /// Collect line ranges that should be excluded from warnings due to
    /// formatting directives (hongdown-disable, hongdown-disable-next-line, etc.).
    ///
    /// Returns a vector of (start_line, end_line) tuples representing disabled ranges.
    fn collect_disabled_line_ranges<'b>(node: &'b AstNode<'b>) -> Vec<(usize, usize)> {
        let mut ranges = Vec::new();
        let children: Vec<_> = node.children().collect();

        for (i, child) in children.iter().enumerate() {
            if let NodeValue::HtmlBlock(html_block) = &child.data.borrow().value
                && let Some(directive) = Directive::parse(&html_block.literal)
            {
                match directive {
                    Directive::DisableFile => {
                        // Everything after this directive is disabled
                        let start_line = child.data.borrow().sourcepos.end.line + 1;
                        ranges.push((start_line, usize::MAX));
                    }
                    Directive::DisableNextLine => {
                        // Only the next block is disabled
                        if let Some(next_child) = children.get(i + 1) {
                            // Skip if next child is also a directive
                            if !matches!(
                                &next_child.data.borrow().value,
                                NodeValue::HtmlBlock(hb) if Directive::parse(&hb.literal).is_some()
                            ) {
                                let start_line = next_child.data.borrow().sourcepos.start.line;
                                let end_line = next_child.data.borrow().sourcepos.end.line;
                                ranges.push((start_line, end_line));
                            }
                        }
                    }
                    Directive::DisableNextSection => {
                        // Disabled until next h2 or lower heading
                        let start_line = child.data.borrow().sourcepos.end.line + 1;
                        let mut end_line = usize::MAX;

                        // Find the next section (h2 or lower)
                        for future_child in children.iter().skip(i + 1) {
                            if let NodeValue::Heading(h) = &future_child.data.borrow().value
                                && h.level <= 2
                            {
                                // End just before this heading
                                end_line = future_child.data.borrow().sourcepos.start.line - 1;
                                break;
                            }
                        }
                        ranges.push((start_line, end_line));
                    }
                    Directive::Disable => {
                        // Disabled until corresponding Enable directive
                        let start_line = child.data.borrow().sourcepos.end.line + 1;
                        let mut end_line = usize::MAX;

                        // Find the corresponding Enable directive
                        for future_child in children.iter().skip(i + 1) {
                            if let NodeValue::HtmlBlock(hb) = &future_child.data.borrow().value
                                && let Some(Directive::Enable) = Directive::parse(&hb.literal)
                            {
                                // End just before the Enable directive
                                end_line = future_child.data.borrow().sourcepos.start.line - 1;
                                break;
                            }
                        }
                        ranges.push((start_line, end_line));
                    }
                    Directive::Enable => {
                        // Enable doesn't start a new range, it ends one
                    }
                    Directive::ProperNouns(_) | Directive::CommonNouns(_) => {
                        // These directives don't affect warning ranges
                    }
                }
            }
        }

        ranges
    }

    /// Check if a line number falls within any of the disabled ranges.
    fn is_line_in_disabled_ranges(line: usize, ranges: &[(usize, usize)]) -> bool {
        ranges
            .iter()
            .any(|(start, end)| line >= *start && line <= *end)
    }

    /// Collect PHP Markdown Extra abbreviation definitions from source.
    /// Returns a set of abbreviation names (e.g., "HTML" from "*[HTML]: Hyper Text Markup Language").
    fn collect_abbreviations(source_lines: &[&str]) -> std::collections::HashSet<String> {
        let mut abbreviations = std::collections::HashSet::new();
        let abbr_pattern = Regex::new(r"^\*\[([^\]]+)\]:").unwrap();

        for line in source_lines {
            if let Some(caps) = abbr_pattern.captures(line)
                && let Some(abbr) = caps.get(1)
            {
                abbreviations.insert(abbr.as_str().to_string());
            }
        }

        abbreviations
    }

    /// Collect reference definitions from source that comrak may not have parsed.
    /// This happens when a reference definition follows an abbreviation definition
    /// without a blank line in between.
    /// Returns a set of (label, line_number) tuples.
    fn collect_source_reference_definitions(
        source_lines: &[&str],
    ) -> std::collections::HashSet<String> {
        let mut definitions = std::collections::HashSet::new();
        // Pattern: [label]: URL at start of line (with optional leading whitespace)
        let ref_def_pattern = Regex::new(r"^\s*\[([^\]]+)\]:\s*\S").unwrap();

        for line in source_lines {
            if let Some(caps) = ref_def_pattern.captures(line)
                && let Some(label) = caps.get(1)
            {
                definitions.insert(label.as_str().to_string());
            }
        }

        definitions
    }

    /// Find undefined references by walking the AST.
    /// Returns a vector of (line_number, warning_message) tuples.
    fn find_undefined_references_in_ast<'b>(
        node: &'b AstNode<'b>,
        source_lines: &[&str],
        abbreviations: &std::collections::HashSet<String>,
        source_ref_defs: &std::collections::HashSet<String>,
    ) -> Vec<(usize, String)> {
        let mut warnings = Vec::new();

        // Pattern to find [label] or [text][label] in text nodes
        // This matches text that looks like a reference link but wasn't parsed as one
        // The pattern [^\[\]] ensures the label doesn't start with [ or ]
        let ref_pattern = Regex::new(r"\[([^\[\]][^\]]*)\](?:\[([^\]]*)\])?").unwrap();

        Self::walk_ast_for_undefined_refs(
            node,
            source_lines,
            &ref_pattern,
            abbreviations,
            source_ref_defs,
            &mut warnings,
        );

        warnings
    }

    /// Recursively walk the AST looking for undefined references in Text nodes.
    fn walk_ast_for_undefined_refs<'b>(
        node: &'b AstNode<'b>,
        source_lines: &[&str],
        ref_pattern: &Regex,
        abbreviations: &std::collections::HashSet<String>,
        source_ref_defs: &std::collections::HashSet<String>,
        warnings: &mut Vec<(usize, String)>,
    ) {
        let data = node.data.borrow();

        match &data.value {
            NodeValue::Text(text) => {
                // Look for [label] patterns in text content
                let line_num = data.sourcepos.start.line;

                for caps in ref_pattern.captures_iter(text) {
                    let full_match = caps.get(0).unwrap();
                    let label = if let Some(explicit_label) = caps.get(2) {
                        // [text][label] form - use the explicit label
                        let l = explicit_label.as_str();
                        if l.is_empty() {
                            // [text][] form - use the text as label
                            caps.get(1).map(|m| m.as_str()).unwrap_or("")
                        } else {
                            l
                        }
                    } else {
                        // [text] form - use the text as label
                        caps.get(1).map(|m| m.as_str()).unwrap_or("")
                    };

                    // Skip empty labels
                    if label.is_empty() {
                        continue;
                    }

                    // Skip footnote references [^name]
                    if label.starts_with('^') {
                        continue;
                    }

                    // Skip GitHub alert markers [!NOTE], [!TIP], etc.
                    if label.starts_with('!') {
                        continue;
                    }

                    // Skip PHP Markdown Extra abbreviations
                    if abbreviations.contains(label) {
                        continue;
                    }

                    // Skip reference definitions that exist in source but comrak didn't parse
                    // (e.g., when they follow abbreviation definitions without a blank line)
                    if source_ref_defs.contains(label) {
                        continue;
                    }

                    // Check original source to see if this was escaped
                    if Self::is_escaped_in_source(source_lines, line_num, full_match.as_str()) {
                        continue;
                    }

                    warnings.push((line_num, format!("undefined reference link: [{}]", label)));
                }
            }
            // Skip code blocks and inline code - they don't contain reference links
            NodeValue::CodeBlock(_) | NodeValue::Code(_) => {
                return;
            }
            // Skip other leaf nodes that don't contain text we care about
            NodeValue::HtmlBlock(_) | NodeValue::HtmlInline(_) => {
                return;
            }
            _ => {}
        }

        drop(data);

        // Recurse into children
        for child in node.children() {
            Self::walk_ast_for_undefined_refs(
                child,
                source_lines,
                ref_pattern,
                abbreviations,
                source_ref_defs,
                warnings,
            );
        }
    }

    /// Check if a bracket pattern was escaped in the original source.
    /// Returns true if the pattern appears as `\[...]` in the source.
    fn is_escaped_in_source(source_lines: &[&str], line_num: usize, pattern: &str) -> bool {
        if line_num == 0 || line_num > source_lines.len() {
            return false;
        }

        let line = source_lines[line_num - 1];

        // Look for the pattern in the line and check if it's preceded by backslash
        if let Some(pos) = line.find(pattern)
            && pos > 0
        {
            let bytes = line.as_bytes();
            // Check if preceded by backslash (and not double backslash)
            if bytes[pos - 1] == b'\\' && (pos < 2 || bytes[pos - 2] != b'\\') {
                return true;
            }
        }

        false
    }

    pub(super) fn serialize_thematic_break(&mut self) {
        let style = &self.options.thematic_break_style;
        // Clamp leading spaces to 0-3 for CommonMark compatibility
        let leading_spaces = self.options.thematic_break_leading_spaces.min(3);

        // Determine the prefix based on blockquote context
        if self.in_block_quote {
            let prefix = format!("{}> ", self.blockquote_outer_indent);
            self.output.push_str(&prefix);
        }

        // Add leading spaces
        for _ in 0..leading_spaces {
            self.output.push(' ');
        }

        self.output.push_str(style);
        self.output.push('\n');
    }
}
