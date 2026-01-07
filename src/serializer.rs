//! Serializer for converting comrak AST to formatted Markdown.

use indexmap::IndexMap;

use comrak::nodes::{AlertType, AstNode, ListType, NodeTable, NodeValue, TableAlignment};

use crate::Options;

/// Formatting directives that can be embedded in HTML comments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Directive {
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
    fn parse(html: &str) -> Option<Self> {
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
struct ReferenceLink {
    label: String,
    url: String,
    title: String,
}

/// Serializes a comrak AST node to a formatted Markdown string,
/// with access to the original source for directive handling.
pub fn serialize_with_source<'a>(
    node: &'a AstNode<'a>,
    options: &Options,
    source: Option<&str>,
) -> String {
    let source_lines: Vec<&str> = source.map(|s| s.lines().collect()).unwrap_or_default();
    let mut serializer = Serializer::new(options, source_lines);
    serializer.serialize_node(node);
    serializer.output
}

struct Serializer<'a> {
    output: String,
    options: &'a Options,
    /// Original source lines for extracting unformatted content
    source_lines: Vec<&'a str>,
    /// Current list item index (1-based) for ordered lists
    list_item_index: usize,
    /// Current list type
    list_type: Option<ListType>,
    /// Whether the current list is tight (no blank lines between items)
    list_tight: bool,
    /// Whether we're inside a block quote
    in_block_quote: bool,
    /// Reference links collected for the current section
    /// Key: URL, Value: ReferenceLink (insertion order preserved)
    pending_references: IndexMap<String, ReferenceLink>,
    /// Current list nesting depth (0 = not in list, 1 = top-level, 2+ = nested)
    list_depth: usize,
    /// Formatting is disabled (by `hongdown-disable` or `hongdown-disable-file`)
    formatting_disabled: bool,
    /// Skip formatting for the next block element only
    skip_next_block: bool,
    /// Skip formatting until the next section heading
    skip_until_section: bool,
}

impl<'a> Serializer<'a> {
    fn new(options: &'a Options, source_lines: Vec<&'a str>) -> Self {
        Self {
            output: String::new(),
            options,
            source_lines,
            list_item_index: 0,
            list_type: None,
            list_tight: true,
            in_block_quote: false,
            pending_references: IndexMap::new(),
            list_depth: 0,
            formatting_disabled: false,
            skip_next_block: false,
            skip_until_section: false,
        }
    }

    /// Extract original source text for a node using its sourcepos.
    fn extract_source<'b>(&self, node: &'b AstNode<'b>) -> Option<String> {
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
    fn should_skip_formatting(&self) -> bool {
        self.formatting_disabled || self.skip_next_block || self.skip_until_section
    }

    /// Check if a link/image was originally in reference style by examining the source.
    /// Returns Some((text, label)) if reference style, None if inline style.
    /// The returned text has newlines normalized to spaces for consistent output.
    fn get_reference_style_info<'b>(&self, node: &'b AstNode<'b>) -> Option<(String, String)> {
        let source = self.extract_source(node)?;

        // Reference style patterns:
        // [text][label] or ![text][label] - full reference
        // [text][] or ![text][] - collapsed reference
        // [text] or ![text] - shortcut reference
        //
        // Inline style pattern:
        // [text](url) or ![text](url)
        //
        // Badge pattern (link containing image):
        // [![alt][img-ref]][link-ref]

        // Remove leading ! for images
        let source = source.strip_prefix('!').unwrap_or(&source);

        // Find the position of the first '[' and track brackets to find the matching ']'
        let first_bracket = source.find('[')?;
        let chars: Vec<char> = source.chars().collect();

        // Find the closing bracket at depth 0 (the one that closes the text/content part)
        let mut depth = 0;
        let mut text_end_pos = None;
        for (i, &ch) in chars.iter().enumerate().skip(first_bracket) {
            match ch {
                '[' => depth += 1,
                ']' => {
                    depth -= 1;
                    if depth == 0 {
                        text_end_pos = Some(i);
                        break;
                    }
                }
                _ => {}
            }
        }

        let text_end_pos = text_end_pos?;

        // Convert char indices to strings for safe UTF-8 handling
        let after_close: String = chars[text_end_pos + 1..].iter().collect();
        let text: String = chars[first_bracket + 1..text_end_pos].iter().collect();

        // Normalize newlines to spaces in the text (for idempotency when text spans lines)
        let text = Self::normalize_whitespace(&text);

        // If followed by "(", it's inline style
        if after_close.starts_with('(') {
            return None;
        }

        // If followed by "[", it's full or collapsed reference style
        if let Some(label_content) = after_close.strip_prefix('[') {
            // Find the label between [ and ]
            if let Some(label_end) = label_content.find(']') {
                let label = label_content[..label_end].to_string();
                // Normalize label too
                let label = Self::normalize_whitespace(&label);

                // If label is empty, it's collapsed reference (use text as label)
                let final_label = if label.is_empty() {
                    text.clone()
                } else {
                    label
                };

                return Some((text, final_label));
            }
        }

        // Shortcut reference: just [text] with nothing following
        // or followed by something that's not ( or [
        Some((text.clone(), text))
    }

    /// Normalize whitespace in text: convert newlines and multiple spaces to single space.
    fn normalize_whitespace(text: &str) -> String {
        text.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    /// Escape special Markdown characters in text content.
    /// Characters that could be misinterpreted as Markdown syntax need escaping.
    fn escape_text(text: &str) -> String {
        let mut result = String::with_capacity(text.len());
        let chars: Vec<char> = text.chars().collect();

        for (i, &ch) in chars.iter().enumerate() {
            match ch {
                // Asterisk always needs escaping (can create emphasis anywhere)
                '*' => {
                    result.push('\\');
                    result.push(ch);
                }
                // Underscore only needs escaping at word boundaries
                // In CommonMark, intraword underscores don't create emphasis
                '_' => {
                    let prev_is_alnum = i > 0 && chars[i - 1].is_alphanumeric();
                    let next_is_alnum = i + 1 < chars.len() && chars[i + 1].is_alphanumeric();

                    // Only escape if at word boundary (could start/end emphasis)
                    if prev_is_alnum && next_is_alnum {
                        // Intraword underscore - no escape needed
                        result.push(ch);
                    } else {
                        result.push('\\');
                        result.push(ch);
                    }
                }
                // Square brackets - only escape if they could form a link
                // A '[' at the end can't start a link, ']' at the start can't close one
                // Adjacent brackets like '[[' or ']]' also don't need escaping
                '[' => {
                    let next_is_bracket = i + 1 < chars.len() && chars[i + 1] == '[';
                    let at_end = i + 1 >= chars.len();
                    if next_is_bracket || at_end {
                        result.push(ch);
                    } else {
                        result.push('\\');
                        result.push(ch);
                    }
                }
                ']' => {
                    let prev_is_bracket = i > 0 && chars[i - 1] == ']';
                    let at_start = i == 0;
                    let at_end = i + 1 >= chars.len();
                    // A ']' can only close a link if followed by '(' or '['
                    // At end of text or followed by other chars, it's just text
                    let next_could_continue_link =
                        i + 1 < chars.len() && (chars[i + 1] == '(' || chars[i + 1] == '[');
                    if prev_is_bracket || at_start || at_end || !next_could_continue_link {
                        result.push(ch);
                    } else {
                        result.push('\\');
                        result.push(ch);
                    }
                }
                // Backslash itself needs escaping
                '\\' => {
                    result.push('\\');
                    result.push(ch);
                }
                // Backtick could start code spans
                '`' => {
                    result.push('\\');
                    result.push(ch);
                }
                // Other characters pass through unchanged
                _ => result.push(ch),
            }
        }
        result
    }

    /// Format a code span with the appropriate number of backticks.
    /// According to CommonMark spec, if the content contains N consecutive backticks,
    /// the delimiter must use at least N+1 backticks. Spaces are added if the content
    /// starts or ends with a backtick or space (and content is not all spaces).
    fn format_code_span(content: &str) -> String {
        // Find the maximum number of consecutive backticks in the content
        let mut max_consecutive = 0;
        let mut current_consecutive = 0;
        for ch in content.chars() {
            if ch == '`' {
                current_consecutive += 1;
                max_consecutive = max_consecutive.max(current_consecutive);
            } else {
                current_consecutive = 0;
            }
        }

        // Use max_consecutive + 1 backticks as delimiters
        let delimiter_count = if max_consecutive > 0 {
            max_consecutive + 1
        } else {
            1
        };
        let delimiter: String = "`".repeat(delimiter_count);

        // Determine if we need space padding.
        // Per CommonMark: if the content begins or ends with a backtick or space
        // (and is not only spaces), add space padding.
        let needs_space = if content.is_empty() {
            false
        } else if content.chars().all(|c| c == ' ') {
            // Content is only spaces - no padding needed
            false
        } else {
            let first = content.chars().next().unwrap();
            let last = content.chars().last().unwrap();
            first == '`' || first == ' ' || last == '`' || last == ' '
        };

        if needs_space {
            format!("{} {} {}", delimiter, content, delimiter)
        } else {
            format!("{}{}{}", delimiter, content, delimiter)
        }
    }

    /// Check if a URL is external (http:// or https://)
    fn is_external_url(url: &str) -> bool {
        url.starts_with("http://") || url.starts_with("https://")
    }

    /// Output pending reference definitions and clear them
    fn flush_references(&mut self) {
        if self.pending_references.is_empty() {
            return;
        }

        // Take ownership of references to avoid borrow issues
        let refs: Vec<ReferenceLink> = self.pending_references.values().cloned().collect();
        self.pending_references.clear();

        // Count numeric references to decide sorting strategy
        let numeric_count = refs
            .iter()
            .filter(|r| Self::extract_numeric_label(&r.label).is_some())
            .count();

        // Add a blank line before references if not already present
        if !self.output.ends_with("\n\n") {
            if self.output.ends_with('\n') {
                self.output.push('\n');
            } else {
                self.output.push_str("\n\n");
            }
        }

        if numeric_count < 2 {
            // Less than 2 numeric refs: output all in insertion order
            for reference in &refs {
                Self::write_reference(&mut self.output, reference);
            }
        } else {
            // 2+ numeric refs: separate, sort numeric ones, output regular first
            let mut regular_refs: Vec<&ReferenceLink> = Vec::new();
            let mut numeric_refs: Vec<(u64, &ReferenceLink)> = Vec::new();

            for reference in &refs {
                if let Some(num) = Self::extract_numeric_label(&reference.label) {
                    numeric_refs.push((num, reference));
                } else {
                    regular_refs.push(reference);
                }
            }

            // Sort numeric references by their numeric value
            numeric_refs.sort_by_key(|(num, _)| *num);

            // Output regular references first (in insertion order)
            for reference in regular_refs {
                Self::write_reference(&mut self.output, reference);
            }

            // Output numeric references at the end (sorted by number)
            for (_, reference) in numeric_refs {
                Self::write_reference(&mut self.output, reference);
            }
        }
    }

    /// Extract numeric value from a label like "123", "#123", "#456"
    /// Returns None if the label is not numeric
    fn extract_numeric_label(label: &str) -> Option<u64> {
        let trimmed = label.strip_prefix('#').unwrap_or(label);
        trimmed.parse::<u64>().ok()
    }

    /// Write a single reference definition to output
    fn write_reference(output: &mut String, reference: &ReferenceLink) {
        output.push('[');
        output.push_str(&reference.label);
        output.push_str("]: ");
        output.push_str(&reference.url);
        if !reference.title.is_empty() {
            output.push_str(" \"");
            output.push_str(&reference.title);
            output.push('"');
        }
        output.push('\n');
    }

    fn serialize_node<'b>(&mut self, node: &'b AstNode<'b>) {
        match &node.data.borrow().value {
            NodeValue::Document => {
                self.serialize_document(node);
            }
            NodeValue::Heading(heading) => {
                self.serialize_heading(node, heading.level);
            }
            NodeValue::List(list) => {
                self.serialize_list(node, list.list_type, list.tight);
            }
            NodeValue::CodeBlock(code_block) => {
                self.serialize_code_block(&code_block.info, &code_block.literal);
            }
            NodeValue::BlockQuote => {
                self.serialize_block_quote(node);
            }
            NodeValue::HtmlBlock(html_block) => {
                // Preserve HTML blocks (like comments) as-is
                self.output.push_str(&html_block.literal);
            }
            NodeValue::HtmlInline(html) => {
                // Preserve inline HTML as-is
                self.output.push_str(html);
            }
            NodeValue::FrontMatter(content) => {
                self.serialize_front_matter(content);
            }
            NodeValue::Table(table) => {
                self.serialize_table(node, table);
            }
            NodeValue::TableRow(is_header) => {
                self.serialize_table_row(node, *is_header);
            }
            NodeValue::TableCell => {
                self.serialize_children(node);
            }
            NodeValue::DescriptionList => {
                self.serialize_children(node);
            }
            NodeValue::DescriptionItem(_) => {
                self.serialize_children(node);
            }
            NodeValue::DescriptionTerm => {
                self.serialize_children(node);
                self.output.push('\n');
            }
            NodeValue::DescriptionDetails => {
                self.output.push_str(":   ");
                // Collect inline content for the definition
                let mut content = String::new();
                for child in node.children() {
                    self.collect_inline_node(child, &mut content);
                }
                self.output.push_str(content.trim());
                self.output.push('\n');
            }
            NodeValue::Alert(alert) => {
                self.serialize_alert(node, alert.alert_type);
            }
            NodeValue::Item(_) => {
                self.serialize_list_item(node);
            }
            NodeValue::Paragraph => {
                self.serialize_paragraph(node);
            }
            NodeValue::Text(text) => {
                self.output.push_str(&Self::escape_text(text));
            }
            NodeValue::SoftBreak => {
                self.output.push(' ');
            }
            NodeValue::LineBreak => {
                self.output.push('\n');
            }
            NodeValue::Emph => {
                self.output.push('*');
                self.serialize_children(node);
                self.output.push('*');
            }
            NodeValue::Strong => {
                self.output.push_str("**");
                self.serialize_children(node);
                self.output.push_str("**");
            }
            NodeValue::Code(code) => {
                self.output.push_str(&Self::format_code_span(&code.literal));
            }
            NodeValue::Link(link) => {
                self.serialize_link(node, &link.url, &link.title);
            }
            NodeValue::Image(image) => {
                self.serialize_image(node, &image.url, &image.title);
            }
            NodeValue::FootnoteReference(footnote_ref) => {
                self.output.push_str("[^");
                self.output.push_str(&footnote_ref.name);
                self.output.push(']');
            }
            NodeValue::FootnoteDefinition(footnote_def) => {
                self.output.push_str("[^");
                self.output.push_str(&footnote_def.name);
                self.output.push_str("]: ");
                // Collect content
                let mut content = String::new();
                for child in node.children() {
                    self.collect_inline_node(child, &mut content);
                }
                self.output.push_str(content.trim());
                self.output.push('\n');
            }
            _ => {
                // For now, just recurse into children for unhandled nodes
                self.serialize_children(node);
            }
        }
    }

    fn serialize_document<'b>(&mut self, node: &'b AstNode<'b>) {
        let children: Vec<_> = node.children().collect();
        for (i, child) in children.iter().enumerate() {
            // Check for directives in HTML blocks
            if let NodeValue::HtmlBlock(html_block) = &child.data.borrow().value
                && let Some(directive) = Directive::parse(&html_block.literal)
            {
                match directive {
                    Directive::DisableFile => {
                        // Output the directive comment, then output remaining content as-is
                        self.output.push_str(&html_block.literal);
                        for remaining_child in children.iter().skip(i + 1) {
                            self.output.push('\n');
                            if let Some(source) = self.extract_source(remaining_child) {
                                self.output.push_str(&source);
                            } else {
                                self.serialize_node(remaining_child);
                            }
                        }
                        self.flush_references();
                        return;
                    }
                    Directive::DisableNextLine => {
                        self.skip_next_block = true;
                        // Output the directive comment
                        if i > 0 {
                            self.output.push('\n');
                        }
                        self.output.push_str(&html_block.literal);
                        continue;
                    }
                    Directive::DisableNextSection => {
                        self.skip_until_section = true;
                        // Output the directive comment
                        if i > 0 {
                            self.output.push('\n');
                        }
                        self.output.push_str(&html_block.literal);
                        continue;
                    }
                    Directive::Disable => {
                        self.formatting_disabled = true;
                        // Output the directive comment
                        if i > 0 {
                            self.output.push('\n');
                        }
                        self.output.push_str(&html_block.literal);
                        continue;
                    }
                    Directive::Enable => {
                        self.formatting_disabled = false;
                        // Output the directive comment
                        if i > 0 {
                            self.output.push('\n');
                        }
                        self.output.push_str(&html_block.literal);
                        continue;
                    }
                }
            }

            // Check if we're about to start a new section (h2 heading)
            // If so, flush any pending references first
            let is_h2 = matches!(
                &child.data.borrow().value,
                NodeValue::Heading(h) if h.level == 2
            );
            if is_h2 && i > 0 {
                self.flush_references();
            }

            // Add blank line between block elements (except after front matter)
            if i > 0 {
                let prev_is_front_matter = matches!(
                    &children[i - 1].data.borrow().value,
                    NodeValue::FrontMatter(_)
                );
                // Also skip blank line after directive comments
                let prev_is_directive =
                    if let NodeValue::HtmlBlock(html) = &children[i - 1].data.borrow().value {
                        Directive::parse(&html.literal).is_some()
                    } else {
                        false
                    };
                // Check if previous element was a heading (empty section case)
                let prev_is_heading =
                    matches!(&children[i - 1].data.borrow().value, NodeValue::Heading(_));
                if !prev_is_front_matter && !prev_is_directive {
                    self.output.push('\n');
                    // Add extra blank line before h2 only if previous wasn't a heading
                    // (i.e., only when there's content between headings)
                    if is_h2 && !prev_is_heading {
                        self.output.push('\n');
                    }
                }
            }

            // Handle formatting based on directive state
            if self.should_skip_formatting() {
                // Check if this is a heading that ends skip_until_section
                // (we skip the first heading in the section, but the next heading ends it)
                if self.skip_until_section && is_h2 {
                    // This heading ends the skipped section
                    // Check if this is the second heading after disable-next-section
                    // by looking at whether we've already output content in skip mode
                    let in_skipped_section =
                        self.output.ends_with('\n') && !self.output.ends_with("-->\n");

                    if in_skipped_section {
                        // Second heading: end skip mode and format normally
                        self.skip_until_section = false;
                        self.serialize_node(child);
                    } else {
                        // First heading: keep skip mode, output as-is
                        if let Some(source) = self.extract_source(child) {
                            self.output.push_str(&source);
                            self.output.push('\n');
                        } else {
                            self.serialize_node(child);
                        }
                    }
                } else {
                    // Output original source if available, otherwise serialize normally
                    if let Some(source) = self.extract_source(child) {
                        self.output.push_str(&source);
                        self.output.push('\n');
                    } else {
                        self.serialize_node(child);
                    }
                }
                // Reset skip_next_block after processing one block
                if self.skip_next_block {
                    self.skip_next_block = false;
                }
            } else {
                self.serialize_node(child);
            }
        }
        // Flush any remaining references at the end of the document
        self.flush_references();
    }

    fn serialize_heading<'b>(&mut self, node: &'b AstNode<'b>, level: u8) {
        // Collect heading text first
        let heading_text = self.collect_text(node);

        if level == 1 {
            // Setext-style with '='
            self.output.push_str(&heading_text);
            self.output.push('\n');
            self.output
                .push_str(&"=".repeat(heading_text.chars().count()));
            self.output.push('\n');
        } else if level == 2 {
            // Setext-style with '-'
            self.output.push_str(&heading_text);
            self.output.push('\n');
            self.output
                .push_str(&"-".repeat(heading_text.chars().count()));
            self.output.push('\n');
        } else {
            // ATX-style for level 3+
            self.output.push_str(&"#".repeat(level as usize));
            self.output.push(' ');
            self.output.push_str(&heading_text);
            self.output.push('\n');
        }
    }

    fn serialize_link<'b>(&mut self, node: &'b AstNode<'b>, url: &str, title: &str) {
        // Check if link contains an image (badge-style link)
        let contains_image = node
            .children()
            .any(|child| matches!(&child.data.borrow().value, NodeValue::Image(_)));

        // Check if this is an autolink (link text equals URL)
        let raw_text = self.collect_raw_text(node);
        let is_autolink = title.is_empty() && raw_text == url;

        // Check if original was reference style
        if let Some((text, label)) = self.get_reference_style_info(node) {
            // Preserve reference style
            if contains_image {
                // Badge-style with reference: [![alt][img-ref]][link-ref]
                self.output.push('[');
                for child in node.children() {
                    self.serialize_node(child);
                }
                self.output.push_str("][");
                self.output.push_str(&label);
                self.output.push(']');
            } else if text == label {
                // Shortcut reference: [text]
                self.output.push('[');
                self.output.push_str(&text);
                self.output.push(']');
            } else {
                // Full reference: [text][label]
                self.output.push('[');
                self.output.push_str(&text);
                self.output.push_str("][");
                self.output.push_str(&label);
                self.output.push(']');
            }

            // Store the reference definition for later output
            self.pending_references.insert(
                url.to_string(),
                ReferenceLink {
                    label,
                    url: url.to_string(),
                    title: title.to_string(),
                },
            );
        } else if contains_image {
            // Badge-style inline: [![alt](img-url)](link-url)
            self.output.push('[');
            for child in node.children() {
                self.serialize_node(child);
            }
            self.output.push_str("](");
            self.output.push_str(url);
            if !title.is_empty() {
                self.output.push_str(" \"");
                self.output.push_str(title);
                self.output.push('"');
            }
            self.output.push(')');
        } else if is_autolink {
            // Autolink: use <url> format
            self.output.push('<');
            self.output.push_str(url);
            self.output.push('>');
        } else if Self::is_external_url(url) {
            // External URL: use reference link style
            let link_text = self.collect_text(node);
            let label = link_text.clone();

            // Output the reference: [text]
            self.output.push('[');
            self.output.push_str(&link_text);
            self.output.push(']');

            // Store the reference definition for later output
            self.pending_references.insert(
                url.to_string(),
                ReferenceLink {
                    label,
                    url: url.to_string(),
                    title: title.to_string(),
                },
            );
        } else {
            // Relative/local URL: keep as inline link
            let link_text = self.collect_text(node);
            self.output.push('[');
            self.output.push_str(&link_text);
            self.output.push_str("](");
            self.output.push_str(url);
            if !title.is_empty() {
                self.output.push_str(" \"");
                self.output.push_str(title);
                self.output.push('"');
            }
            self.output.push(')');
        }
    }

    fn serialize_image<'b>(&mut self, node: &'b AstNode<'b>, url: &str, title: &str) {
        // Collect the alt text
        let alt_text = self.collect_text(node);

        // Check if original was reference style
        if let Some((text, label)) = self.get_reference_style_info(node) {
            // Preserve reference style
            if text == label {
                // Shortcut reference: ![alt]
                self.output.push_str("![");
                self.output.push_str(&text);
                self.output.push(']');
            } else {
                // Full reference: ![alt][label]
                self.output.push_str("![");
                self.output.push_str(&text);
                self.output.push_str("][");
                self.output.push_str(&label);
                self.output.push(']');
            }

            // Store the reference definition for later output
            self.pending_references.insert(
                url.to_string(),
                ReferenceLink {
                    label,
                    url: url.to_string(),
                    title: title.to_string(),
                },
            );
        } else {
            // Inline style: ![alt](url)
            self.output.push_str("![");
            self.output.push_str(&alt_text);
            self.output.push_str("](");
            self.output.push_str(url);
            if !title.is_empty() {
                self.output.push_str(" \"");
                self.output.push_str(title);
                self.output.push('"');
            }
            self.output.push(')');
        }
    }

    fn collect_text<'b>(&self, node: &'b AstNode<'b>) -> String {
        let mut text = String::new();
        self.collect_text_recursive(node, &mut text);
        text
    }

    /// Collect raw text without escaping (for comparison purposes)
    fn collect_raw_text<'b>(&self, node: &'b AstNode<'b>) -> String {
        let mut text = String::new();
        self.collect_raw_text_recursive(node, &mut text);
        text
    }

    fn collect_raw_text_recursive<'b>(&self, node: &'b AstNode<'b>, text: &mut String) {
        match &node.data.borrow().value {
            NodeValue::Text(t) => {
                text.push_str(t);
            }
            NodeValue::SoftBreak => {
                text.push(' ');
            }
            _ => {
                for child in node.children() {
                    self.collect_raw_text_recursive(child, text);
                }
            }
        }
    }

    fn collect_text_recursive<'b>(&self, node: &'b AstNode<'b>, text: &mut String) {
        match &node.data.borrow().value {
            NodeValue::Text(t) => {
                text.push_str(&Self::escape_text(t));
            }
            NodeValue::Code(code) => {
                text.push_str(&Self::format_code_span(&code.literal));
            }
            NodeValue::SoftBreak => {
                text.push(' ');
            }
            _ => {
                for child in node.children() {
                    self.collect_text_recursive(child, text);
                }
            }
        }
    }

    fn serialize_paragraph<'b>(&mut self, node: &'b AstNode<'b>) {
        // Collect all inline content first
        let mut inline_content = String::new();
        self.collect_inline_content(node, &mut inline_content);

        let prefix = if self.in_block_quote { "> " } else { "" };

        if self.list_type.is_some() {
            // Inside a list item, wrap with proper continuation indent
            // First line has no prefix (marker already output)
            // Continuation lines need 4-space indent per nesting level
            // (to align with list item content at each level)
            let base_indent = "    ".repeat(self.list_depth);
            let continuation = if self.in_block_quote {
                format!("> {}", base_indent)
            } else {
                base_indent
            };
            let wrapped = self.wrap_text_first_line(&inline_content, "", &continuation);
            self.output.push_str(&wrapped);
        } else {
            // Wrap the paragraph at line_width
            let wrapped = self.wrap_text(&inline_content, prefix);
            self.output.push_str(&wrapped);
            self.output.push('\n');
        }
    }

    fn collect_inline_content<'b>(&mut self, node: &'b AstNode<'b>, content: &mut String) {
        for child in node.children() {
            self.collect_inline_node(child, content);
        }
    }

    fn collect_inline_node<'b>(&mut self, node: &'b AstNode<'b>, content: &mut String) {
        match &node.data.borrow().value.clone() {
            NodeValue::Text(text) => {
                content.push_str(&Self::escape_text(text));
            }
            NodeValue::SoftBreak => {
                // Use a special marker to preserve original line breaks
                // This will be processed by wrap_text to decide whether to keep them
                content.push('\x00');
            }
            NodeValue::LineBreak => {
                content.push('\n');
            }
            NodeValue::Emph => {
                content.push('*');
                for child in node.children() {
                    self.collect_inline_node(child, content);
                }
                content.push('*');
            }
            NodeValue::Strong => {
                content.push_str("**");
                for child in node.children() {
                    self.collect_inline_node(child, content);
                }
                content.push_str("**");
            }
            NodeValue::Code(code) => {
                content.push_str(&Self::format_code_span(&code.literal));
            }
            NodeValue::Link(link) => {
                // Check if link contains an image (badge-style link)
                let contains_image = node
                    .children()
                    .any(|child| matches!(&child.data.borrow().value, NodeValue::Image(_)));

                // Check if this is an autolink (link text equals URL)
                let raw_text = self.collect_raw_text(node);
                let is_autolink = link.title.is_empty() && raw_text == link.url;

                // Check if original was reference style
                if let Some((text, label)) = self.get_reference_style_info(node) {
                    // Preserve reference style
                    if contains_image {
                        // Badge-style with reference: [![alt][img-ref]][link-ref]
                        content.push('[');
                        for child in node.children() {
                            self.collect_inline_node(child, content);
                        }
                        content.push_str("][");
                        content.push_str(&label);
                        content.push(']');
                    } else if text == label {
                        // Shortcut reference: [text]
                        content.push('[');
                        content.push_str(&text);
                        content.push(']');
                    } else {
                        // Full reference: [text][label]
                        content.push('[');
                        content.push_str(&text);
                        content.push_str("][");
                        content.push_str(&label);
                        content.push(']');
                    }

                    // Store the reference definition
                    self.pending_references.insert(
                        link.url.clone(),
                        ReferenceLink {
                            label,
                            url: link.url.clone(),
                            title: link.title.clone(),
                        },
                    );
                } else if contains_image {
                    // Badge-style inline: [![alt](img-url)](link-url)
                    content.push('[');
                    for child in node.children() {
                        self.collect_inline_node(child, content);
                    }
                    content.push_str("](");
                    content.push_str(&link.url);
                    if !link.title.is_empty() {
                        content.push_str(" \"");
                        content.push_str(&link.title);
                        content.push('"');
                    }
                    content.push(')');
                } else if is_autolink {
                    // Autolink: use <url> format
                    content.push('<');
                    content.push_str(&link.url);
                    content.push('>');
                } else if Self::is_external_url(&link.url) {
                    // External URL: use reference link style
                    let mut link_text = String::new();
                    for child in node.children() {
                        self.collect_inline_node(child, &mut link_text);
                    }
                    content.push('[');
                    content.push_str(&link_text);
                    content.push(']');

                    // Store the reference definition
                    self.pending_references.insert(
                        link.url.clone(),
                        ReferenceLink {
                            label: link_text,
                            url: link.url.clone(),
                            title: link.title.clone(),
                        },
                    );
                } else {
                    // Relative/local URL: keep as inline link
                    let mut link_text = String::new();
                    for child in node.children() {
                        self.collect_inline_node(child, &mut link_text);
                    }
                    content.push('[');
                    content.push_str(&link_text);
                    content.push_str("](");
                    content.push_str(&link.url);
                    if !link.title.is_empty() {
                        content.push_str(" \"");
                        content.push_str(&link.title);
                        content.push('"');
                    }
                    content.push(')');
                }
            }
            NodeValue::Image(image) => {
                // Check if original was reference style
                if let Some((text, label)) = self.get_reference_style_info(node) {
                    // Preserve reference style
                    if text == label {
                        // Shortcut reference: ![alt]
                        content.push_str("![");
                        content.push_str(&text);
                        content.push(']');
                    } else {
                        // Full reference: ![alt][label]
                        content.push_str("![");
                        content.push_str(&text);
                        content.push_str("][");
                        content.push_str(&label);
                        content.push(']');
                    }

                    // Store the reference definition
                    self.pending_references.insert(
                        image.url.clone(),
                        ReferenceLink {
                            label,
                            url: image.url.clone(),
                            title: image.title.clone(),
                        },
                    );
                } else {
                    // Inline style: collect alt text and use inline syntax
                    let mut alt_text = String::new();
                    for child in node.children() {
                        self.collect_inline_node(child, &mut alt_text);
                    }

                    content.push_str("![");
                    content.push_str(&alt_text);
                    content.push_str("](");
                    content.push_str(&image.url);
                    if !image.title.is_empty() {
                        content.push_str(" \"");
                        content.push_str(&image.title);
                        content.push('"');
                    }
                    content.push(')');
                }
            }
            NodeValue::HtmlInline(html) => {
                // Preserve inline HTML as-is
                content.push_str(html);
            }
            NodeValue::FootnoteReference(footnote_ref) => {
                content.push_str("[^");
                content.push_str(&footnote_ref.name);
                content.push(']');
            }
            _ => {
                for child in node.children() {
                    self.collect_inline_node(child, content);
                }
            }
        }
    }

    fn wrap_text(&self, text: &str, prefix: &str) -> String {
        let line_width = self.options.line_width;

        // Split by soft break markers (original line breaks)
        // \x00 represents where the original document had line breaks
        let original_lines: Vec<&str> = text.split('\x00').collect();

        if original_lines.len() == 1 {
            // No original line breaks, just wrap normally
            return self.wrap_single_segment(text, prefix, prefix);
        }

        // Process lines: keep short lines as-is, merge and rewrap long lines
        let mut result = String::new();
        let mut i = 0;

        while i < original_lines.len() {
            let line = original_lines[i].trim();
            let line_with_prefix_len = prefix.len() + line.len();

            if line_with_prefix_len <= line_width {
                // Line fits within limit, keep it as-is
                if !result.is_empty() {
                    result.push('\n');
                }
                result.push_str(prefix);
                result.push_str(line);
                i += 1;
            } else {
                // Line exceeds limit, merge with following lines and rewrap
                let mut merged = String::from(line);

                // Keep merging while current merged content exceeds limit
                // or until we run out of lines
                i += 1;
                while i < original_lines.len() {
                    let next_line = original_lines[i].trim();
                    merged.push(' ');
                    merged.push_str(next_line);
                    i += 1;

                    // Check if the last "line" of wrapped content would fit
                    // If so, we can stop merging
                    let wrapped = self.wrap_single_segment(&merged, "", "");
                    if let Some(last_line) = wrapped.lines().last()
                        && prefix.len() + last_line.len() <= line_width
                    {
                        break;
                    }
                }

                // Wrap the merged content
                let wrapped = self.wrap_single_segment(&merged, prefix, prefix);

                if !result.is_empty() {
                    result.push('\n');
                }
                result.push_str(&wrapped);
            }
        }

        result
    }

    /// Wrap a single segment of text (no original line break markers)
    fn wrap_single_segment(&self, text: &str, first_prefix: &str, prefix: &str) -> String {
        let line_width = self.options.line_width;
        let mut result = String::new();
        let mut current_line = String::new();
        let mut is_first_line = true;
        let first_prefix_len = first_prefix.len();

        // Add prefix to first line
        current_line.push_str(first_prefix);

        // Split into "tokens" where each token is either:
        // - A word (non-space characters) followed by optional spaces
        // - Content inside backticks (treated as a single unbreakable unit)
        // We preserve double spaces after periods.
        let chars = text.chars();
        let mut current_token = String::new();
        let mut trailing_spaces = String::new();
        let mut in_backticks = false;

        for ch in chars {
            if ch == '`' {
                if in_backticks {
                    // End of backtick region
                    current_token.push(ch);
                    in_backticks = false;
                } else {
                    // Start of backtick region - include any accumulated content first
                    if !current_token.is_empty() && !trailing_spaces.is_empty() {
                        // We have a previous word, output it
                        Self::add_token_to_line_with_prefix(
                            &mut result,
                            &mut current_line,
                            &current_token,
                            &trailing_spaces,
                            first_prefix_len,
                            prefix,
                            line_width,
                            &mut is_first_line,
                        );
                        current_token.clear();
                        trailing_spaces.clear();
                    }
                    current_token.push(ch);
                    in_backticks = true;
                }
            } else if in_backticks {
                // Inside backticks, everything is part of the token
                current_token.push(ch);
            } else if ch == ' ' {
                trailing_spaces.push(ch);
            } else {
                // Regular character outside backticks
                if !current_token.is_empty() && !trailing_spaces.is_empty() {
                    // We have a previous word with trailing spaces, output it
                    Self::add_token_to_line_with_prefix(
                        &mut result,
                        &mut current_line,
                        &current_token,
                        &trailing_spaces,
                        first_prefix_len,
                        prefix,
                        line_width,
                        &mut is_first_line,
                    );
                    current_token.clear();
                    trailing_spaces.clear();
                }
                current_token.push(ch);
            }
        }

        // Handle the last token
        if !current_token.is_empty() {
            Self::add_token_to_line_with_prefix(
                &mut result,
                &mut current_line,
                &current_token,
                "",
                first_prefix_len,
                prefix,
                line_width,
                &mut is_first_line,
            );
        }

        // Add the last line (trim trailing spaces)
        let final_line = current_line.trim_end();
        if !final_line.is_empty() {
            result.push_str(final_line);
        }

        result
    }

    #[allow(clippy::too_many_arguments)]
    fn add_token_to_line_with_prefix(
        result: &mut String,
        current_line: &mut String,
        token: &str,
        trailing_spaces: &str,
        first_prefix_len: usize,
        prefix: &str,
        line_width: usize,
        is_first_line: &mut bool,
    ) {
        let token_len = token.len();
        let spaces_len = trailing_spaces.len();
        let current_prefix_len = if *is_first_line {
            first_prefix_len
        } else {
            prefix.len()
        };

        if current_line.len() == current_prefix_len {
            // First word on this line (prefix already added)
            current_line.push_str(token);
            current_line.push_str(trailing_spaces);
        } else if current_line.len() + token_len + spaces_len <= line_width {
            // Token fits on current line
            current_line.push_str(token);
            current_line.push_str(trailing_spaces);
        } else {
            // Start a new line - trim trailing spaces from previous line
            let trimmed = current_line.trim_end();
            result.push_str(trimmed);
            result.push('\n');
            *current_line = String::from(prefix);
            current_line.push_str(token);
            current_line.push_str(trailing_spaces);
            *is_first_line = false;
        }
    }

    /// Wrap text where the first line has a different prefix than continuation lines.
    /// This is used for list items where the marker is already output and continuation
    /// lines need indentation.
    fn wrap_text_first_line(
        &self,
        text: &str,
        first_prefix: &str,
        continuation_prefix: &str,
    ) -> String {
        let line_width = self.options.line_width;

        // Split by soft break markers (original line breaks)
        let original_lines: Vec<&str> = text.split('\x00').collect();

        if original_lines.len() == 1 {
            // No original line breaks, just wrap normally
            return self.wrap_single_segment(text, first_prefix, continuation_prefix);
        }

        // Process lines: keep short lines as-is, merge and rewrap long lines
        let mut result = String::new();
        let mut i = 0;
        let mut is_first_line = true;

        while i < original_lines.len() {
            let line = original_lines[i].trim();
            let current_prefix = if is_first_line {
                first_prefix
            } else {
                continuation_prefix
            };
            let line_with_prefix_len = current_prefix.len() + line.len();

            if line_with_prefix_len <= line_width {
                // Line fits within limit, keep it as-is
                if !result.is_empty() {
                    result.push('\n');
                }
                result.push_str(current_prefix);
                result.push_str(line);
                is_first_line = false;
                i += 1;
            } else {
                // Line exceeds limit, merge with following lines and rewrap
                let mut merged = String::from(line);

                i += 1;
                while i < original_lines.len() {
                    let next_line = original_lines[i].trim();
                    merged.push(' ');
                    merged.push_str(next_line);
                    i += 1;

                    // Check if the last line of wrapped content would fit
                    let test_prefix = if is_first_line {
                        first_prefix
                    } else {
                        continuation_prefix
                    };
                    let wrapped =
                        self.wrap_single_segment(&merged, test_prefix, continuation_prefix);
                    if let Some(last_line) = wrapped.lines().last()
                        && continuation_prefix.len() + last_line.trim_start().len() <= line_width
                    {
                        break;
                    }
                }

                // Wrap the merged content
                let wrapped =
                    self.wrap_single_segment(&merged, current_prefix, continuation_prefix);

                if !result.is_empty() {
                    result.push('\n');
                }
                result.push_str(&wrapped);
                is_first_line = false;
            }
        }

        result
    }

    fn serialize_alert<'b>(&mut self, node: &'b AstNode<'b>, alert_type: AlertType) {
        // Output the alert header
        let type_str = match alert_type {
            AlertType::Note => "NOTE",
            AlertType::Tip => "TIP",
            AlertType::Important => "IMPORTANT",
            AlertType::Warning => "WARNING",
            AlertType::Caution => "CAUTION",
        };
        self.output.push_str("> [!");
        self.output.push_str(type_str);
        self.output.push_str("]\n");

        // Output the alert content with > prefix
        // Use in_block_quote to handle nested content properly
        let was_in_block_quote = self.in_block_quote;
        self.in_block_quote = true;

        let children: Vec<_> = node.children().collect();
        for (i, child) in children.iter().enumerate() {
            if i > 0 {
                self.output.push_str(">\n");
            }
            self.serialize_node(child);
        }

        self.in_block_quote = was_in_block_quote;
    }

    fn serialize_front_matter(&mut self, content: &str) {
        // Front matter content from comrak includes the delimiters,
        // so we preserve it verbatim and add a trailing blank line
        self.output.push_str(content.trim());
        self.output.push_str("\n\n");
    }

    fn serialize_table<'b>(&mut self, node: &'b AstNode<'b>, table: &NodeTable) {
        let alignments = &table.alignments;
        // Collect all rows and cells first to calculate column widths
        let rows: Vec<_> = node.children().collect();
        if rows.is_empty() {
            return;
        }

        // Collect cell contents (with full inline formatting) and calculate max widths
        let mut all_cells: Vec<Vec<String>> = Vec::new();
        let mut col_widths: Vec<usize> = vec![0; alignments.len()];

        for row in &rows {
            let mut row_cells: Vec<String> = Vec::new();
            for (i, cell) in row.children().enumerate() {
                // Use collect_inline_content to preserve links and formatting
                let mut content = String::new();
                self.collect_inline_content(cell, &mut content);
                if i < col_widths.len() {
                    col_widths[i] = col_widths[i].max(content.len());
                }
                row_cells.push(content);
            }
            all_cells.push(row_cells);
        }

        // Ensure minimum column width for alignment markers
        for width in &mut col_widths {
            *width = (*width).max(3);
        }

        // Output header row
        if let Some(header_cells) = all_cells.first() {
            self.output.push('|');
            for (i, cell) in header_cells.iter().enumerate() {
                self.output.push(' ');
                let width = col_widths.get(i).copied().unwrap_or(3);
                self.output
                    .push_str(&format!("{:width$}", cell, width = width));
                self.output.push_str(" |");
            }
            self.output.push('\n');
        }

        // Output separator row with alignment
        self.output.push('|');
        for (i, alignment) in alignments.iter().enumerate() {
            self.output.push(' ');
            let width = col_widths.get(i).copied().unwrap_or(3);
            match alignment {
                TableAlignment::Left => {
                    self.output.push(':');
                    self.output.push_str(&"-".repeat(width - 1));
                }
                TableAlignment::Right => {
                    self.output.push_str(&"-".repeat(width - 1));
                    self.output.push(':');
                }
                TableAlignment::Center => {
                    self.output.push(':');
                    self.output.push_str(&"-".repeat(width - 2));
                    self.output.push(':');
                }
                TableAlignment::None => {
                    self.output.push_str(&"-".repeat(width));
                }
            }
            self.output.push_str(" |");
        }
        self.output.push('\n');

        // Output data rows (skip header)
        for row_cells in all_cells.iter().skip(1) {
            self.output.push('|');
            for (i, cell) in row_cells.iter().enumerate() {
                self.output.push(' ');
                let width = col_widths.get(i).copied().unwrap_or(3);
                self.output
                    .push_str(&format!("{:width$}", cell, width = width));
                self.output.push_str(" |");
            }
            self.output.push('\n');
        }
    }

    fn serialize_table_row<'b>(&mut self, _node: &'b AstNode<'b>, _is_header: bool) {
        // Table rows are handled by serialize_table
    }

    fn serialize_block_quote<'b>(&mut self, node: &'b AstNode<'b>) {
        let was_in_block_quote = self.in_block_quote;
        self.in_block_quote = true;

        let children: Vec<_> = node.children().collect();
        for (i, child) in children.iter().enumerate() {
            // Add blank quote line between paragraphs
            if i > 0 {
                self.output.push_str(">\n");
            }
            self.serialize_node(child);
        }

        self.in_block_quote = was_in_block_quote;
    }

    fn serialize_code_block(&mut self, info: &str, literal: &str) {
        // Determine the minimum fence length (at least 4)
        let min_fence_length = 4;

        // Find the longest sequence of tildes in the content
        let max_tildes_in_content = literal
            .lines()
            .filter_map(|line| {
                let trimmed = line.trim_start();
                if trimmed.starts_with('~') {
                    Some(trimmed.chars().take_while(|&c| c == '~').count())
                } else {
                    None
                }
            })
            .max()
            .unwrap_or(0);

        // Fence length must be greater than any tilde sequence in content
        let fence_length = std::cmp::max(min_fence_length, max_tildes_in_content + 1);
        let fence = "~".repeat(fence_length);

        // Use "text" as default if no language specified
        let language = if info.is_empty() { "text" } else { info };

        self.output.push_str(&fence);
        self.output.push(' ');
        self.output.push_str(language);
        self.output.push('\n');
        self.output.push_str(literal);
        self.output.push_str(&fence);
        self.output.push('\n');
    }

    /// Serialize a code block with indentation prefix on each line.
    /// Used for code blocks inside list items.
    fn serialize_code_block_indented(&mut self, info: &str, literal: &str, indent: &str) {
        // Determine the minimum fence length (at least 4)
        let min_fence_length = 4;

        // Find the longest sequence of tildes in the content
        let max_tildes_in_content = literal
            .lines()
            .filter_map(|line| {
                let trimmed = line.trim_start();
                if trimmed.starts_with('~') {
                    Some(trimmed.chars().take_while(|&c| c == '~').count())
                } else {
                    None
                }
            })
            .max()
            .unwrap_or(0);

        // Fence length must be greater than any tilde sequence in content
        let fence_length = std::cmp::max(min_fence_length, max_tildes_in_content + 1);
        let fence = "~".repeat(fence_length);

        // Output opening fence with optional language
        self.output.push_str(&fence);
        if !info.is_empty() {
            self.output.push(' ');
            self.output.push_str(info);
        }
        self.output.push('\n');

        // Output content with indentation (skip indent for empty lines)
        for line in literal.lines() {
            if self.in_block_quote {
                self.output.push_str("> ");
            }
            if !line.is_empty() {
                self.output.push_str(indent);
                self.output.push_str(line);
            }
            self.output.push('\n');
        }

        // Output closing fence with indentation
        if self.in_block_quote {
            self.output.push_str("> ");
        }
        self.output.push_str(indent);
        self.output.push_str(&fence);
        self.output.push('\n');
    }

    fn serialize_list<'b>(&mut self, node: &'b AstNode<'b>, list_type: ListType, tight: bool) {
        let old_list_type = self.list_type;
        let old_list_tight = self.list_tight;
        let old_index = self.list_item_index;

        self.list_type = Some(list_type);
        self.list_tight = tight;
        self.list_item_index = 0;
        self.list_depth += 1;

        self.serialize_children(node);

        self.list_depth -= 1;
        self.list_type = old_list_type;
        self.list_tight = old_list_tight;
        self.list_item_index = old_index;
    }

    fn serialize_list_item<'b>(&mut self, node: &'b AstNode<'b>) {
        self.list_item_index += 1;

        // For loose lists, add a blank line before items (except the first)
        if !self.list_tight && self.list_item_index > 1 {
            self.output.push('\n');
        }

        // Add block quote prefix if we're inside a block quote
        if self.in_block_quote {
            self.output.push_str("> ");
        }

        // Calculate indentation for nested lists
        // Level 1: " -  " (1 leading space + hyphen + 2 trailing spaces)
        // Level 2+: 4 spaces per additional level
        // This gives: level 1 = " -  ", level 2 = "    -  " (4 spaces), etc.
        if self.list_depth > 1 {
            let indent = "    ".repeat(self.list_depth - 1);
            self.output.push_str(&indent);
        }

        match self.list_type {
            Some(ListType::Bullet) => {
                if self.list_depth > 1 {
                    // Nested bullets: "-  " (no leading space, hyphen, two trailing spaces)
                    self.output.push_str("-  ");
                } else {
                    // Top-level bullets: " -  " (one leading space)
                    self.output.push_str(" -  ");
                }
            }
            Some(ListType::Ordered) => {
                if self.list_depth > 1 {
                    // Nested ordered: "N. "
                    self.output.push_str(&self.list_item_index.to_string());
                    self.output.push_str(". ");
                } else {
                    // Top-level ordered: " N. " format
                    self.output.push(' ');
                    self.output.push_str(&self.list_item_index.to_string());
                    self.output.push_str(". ");
                }
            }
            None => {}
        }

        // Serialize children, handling nested lists and multiple paragraphs
        let children: Vec<_> = node.children().collect();
        let base_indent = "    ".repeat(self.list_depth);

        for (i, child) in children.iter().enumerate() {
            let is_first = i == 0;
            match &child.data.borrow().value {
                NodeValue::List(_) => {
                    // Check if there's a blank line before this nested list in the original
                    let has_blank_line_before = if i > 0 {
                        let prev_child = children[i - 1];
                        let prev_end_line = prev_child.data.borrow().sourcepos.end.line;
                        let curr_start_line = child.data.borrow().sourcepos.start.line;
                        // More than one line difference means there's a blank line
                        curr_start_line > prev_end_line + 1
                    } else {
                        false
                    };

                    if has_blank_line_before {
                        // Blank line to separate from preceding paragraph
                        self.output.push_str("\n\n");
                    } else {
                        self.output.push('\n');
                    }
                    self.serialize_node(child);
                }
                NodeValue::Paragraph => {
                    // For paragraphs after the first, add blank line with proper indentation
                    if !is_first {
                        // First \n ends the previous paragraph, second \n creates blank line
                        self.output.push_str("\n\n");
                        if self.in_block_quote {
                            self.output.push_str("> ");
                        }
                        self.output.push_str(&base_indent);
                    }
                    self.serialize_node(child);
                }
                NodeValue::CodeBlock(code_block) => {
                    // Code blocks in list items need blank line and indentation
                    self.output.push_str("\n\n");
                    if self.in_block_quote {
                        self.output.push_str("> ");
                    }
                    self.output.push_str(&base_indent);
                    self.serialize_code_block_indented(
                        &code_block.info,
                        &code_block.literal,
                        &base_indent,
                    );
                }
                _ => {
                    self.serialize_node(child);
                }
            }
        }

        // Only add newline if we didn't just serialize a nested list
        // (nested lists add their own newlines)
        let last_child_is_list = node
            .children()
            .last()
            .is_some_and(|child| matches!(&child.data.borrow().value, NodeValue::List(_)));
        if !last_child_is_list {
            self.output.push('\n');
        }
    }

    fn serialize_children<'b>(&mut self, node: &'b AstNode<'b>) {
        for child in node.children() {
            self.serialize_node(child);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use comrak::{Arena, Options as ComrakOptions, parse_document};

    fn parse_and_serialize(input: &str) -> String {
        let arena = Arena::new();
        let options = ComrakOptions::default();
        let root = parse_document(&arena, input, &options);
        let format_options = Options::default();
        serialize_with_source(root, &format_options, None)
    }

    fn parse_and_serialize_with_source(input: &str) -> String {
        let arena = Arena::new();
        let options = ComrakOptions::default();
        let root = parse_document(&arena, input, &options);
        let format_options = Options::default();
        serialize_with_source(root, &format_options, Some(input))
    }

    #[test]
    fn test_serialize_plain_text() {
        let result = parse_and_serialize("Hello, world!");
        assert_eq!(result, "Hello, world!\n");
    }

    #[test]
    fn test_serialize_multiline_paragraph() {
        // Original line breaks are preserved when lines are under 80 chars
        let result = parse_and_serialize("Hello\nworld!");
        assert_eq!(result, "Hello\nworld!\n");
    }

    #[test]
    fn test_serialize_h1_setext() {
        let result = parse_and_serialize("# Document Title");
        assert_eq!(result, "Document Title\n==============\n");
    }

    #[test]
    fn test_serialize_h2_setext() {
        let result = parse_and_serialize("## Section Name");
        assert_eq!(result, "Section Name\n------------\n");
    }

    #[test]
    fn test_serialize_h3_atx() {
        let result = parse_and_serialize("### Subsection");
        assert_eq!(result, "### Subsection\n");
    }

    #[test]
    fn test_serialize_h4_atx() {
        let result = parse_and_serialize("#### Deep Subsection");
        assert_eq!(result, "#### Deep Subsection\n");
    }

    #[test]
    fn test_serialize_unordered_list_single_item() {
        let result = parse_and_serialize("- Item one");
        assert_eq!(result, " -  Item one\n");
    }

    #[test]
    fn test_serialize_unordered_list_multiple_items() {
        let result = parse_and_serialize("- Item one\n- Item two\n- Item three");
        assert_eq!(result, " -  Item one\n -  Item two\n -  Item three\n");
    }

    #[test]
    fn test_serialize_ordered_list_single_item() {
        let result = parse_and_serialize("1. First item");
        assert_eq!(result, " 1. First item\n");
    }

    #[test]
    fn test_serialize_ordered_list_multiple_items() {
        let result = parse_and_serialize("1. First\n2. Second\n3. Third");
        assert_eq!(result, " 1. First\n 2. Second\n 3. Third\n");
    }

    #[test]
    fn test_serialize_tight_list() {
        // Tight list: no blank lines between items
        let input = " -  Item one\n -  Item two\n -  Item three";
        let result = parse_and_serialize(input);
        assert_eq!(result, " -  Item one\n -  Item two\n -  Item three\n");
    }

    #[test]
    fn test_serialize_loose_list() {
        // Loose list: blank lines between items should be preserved
        let input = " -  Item one\n\n -  Item two\n\n -  Item three";
        let result = parse_and_serialize(input);
        assert_eq!(
            result, " -  Item one\n\n -  Item two\n\n -  Item three\n",
            "Loose list should have blank lines between items"
        );
    }

    #[test]
    fn test_serialize_loose_list_with_content() {
        // Loose list with multi-line content
        let input = " -  *Zero dependencies*: LogTape has zero dependencies.\n\n -  *Library support*: Designed for libraries.";
        let result = parse_and_serialize(input);
        assert!(
            result.contains(" -  *Zero dependencies*"),
            "Should contain first item"
        );
        assert!(
            result.contains("\n\n -  *Library support*"),
            "Should have blank line before second item, got:\n{}",
            result
        );
    }

    #[test]
    fn test_serialize_fenced_code_block() {
        let result = parse_and_serialize("```rust\nfn main() {}\n```");
        assert_eq!(result, "~~~~ rust\nfn main() {}\n~~~~\n");
    }

    #[test]
    fn test_serialize_fenced_code_block_no_language() {
        let result = parse_and_serialize("```\nsome code\n```");
        assert_eq!(result, "~~~~ text\nsome code\n~~~~\n");
    }

    #[test]
    fn test_serialize_fenced_code_block_with_tildes_inside() {
        // When code contains ~~~~, use more tildes for the fence
        let result = parse_and_serialize("```\n~~~~\ninner fence\n~~~~\n```");
        assert_eq!(result, "~~~~~ text\n~~~~\ninner fence\n~~~~\n~~~~~\n");
    }

    #[test]
    fn test_serialize_block_quote_single_line() {
        let result = parse_and_serialize("> This is a quote.");
        assert_eq!(result, "> This is a quote.\n");
    }

    #[test]
    fn test_serialize_block_quote_multiple_lines() {
        // Original line breaks are preserved when lines are under 80 chars
        let result = parse_and_serialize("> Line one.\n> Line two.");
        assert_eq!(result, "> Line one.\n> Line two.\n");
    }

    #[test]
    fn test_serialize_block_quote_multiple_paragraphs() {
        let result = parse_and_serialize("> First paragraph.\n>\n> Second paragraph.");
        assert_eq!(result, "> First paragraph.\n>\n> Second paragraph.\n");
    }

    #[test]
    fn test_serialize_emphasis() {
        let result = parse_and_serialize("This is *emphasized* text.");
        assert_eq!(result, "This is *emphasized* text.\n");
    }

    #[test]
    fn test_serialize_strong() {
        let result = parse_and_serialize("This is **strong** text.");
        assert_eq!(result, "This is **strong** text.\n");
    }

    #[test]
    fn test_serialize_inline_code() {
        let result = parse_and_serialize("Use the `format()` function.");
        assert_eq!(result, "Use the `format()` function.\n");
    }

    #[test]
    fn test_serialize_external_link_becomes_reference() {
        // External links (https://) are converted to reference style
        let result = parse_and_serialize("Visit [Rust](https://www.rust-lang.org/).");
        assert!(result.contains("Visit [Rust]."));
        assert!(result.contains("[Rust]: https://www.rust-lang.org/"));
    }

    #[test]
    fn test_serialize_external_link_with_title_becomes_reference() {
        // External links with titles are also converted to reference style
        let result =
            parse_and_serialize("Visit [Rust](https://www.rust-lang.org/ \"The Rust Language\").");
        assert!(result.contains("Visit [Rust]."));
        assert!(result.contains("[Rust]: https://www.rust-lang.org/ \"The Rust Language\""));
    }

    #[test]
    fn test_reference_order_preserved() {
        // Regular references should maintain insertion order
        let input =
            "See [foo](https://foo.com), [bar](https://bar.com), and [baz](https://baz.com).";
        let result = parse_and_serialize(input);
        // Find positions of references
        let foo_pos = result.find("[foo]:").unwrap();
        let bar_pos = result.find("[bar]:").unwrap();
        let baz_pos = result.find("[baz]:").unwrap();
        assert!(
            foo_pos < bar_pos && bar_pos < baz_pos,
            "References should be in insertion order, got:\n{}",
            result
        );
    }

    #[test]
    fn test_numeric_references_sorted_at_end() {
        // Numeric references should be sorted by number and placed at the end
        let input = "See [foo](https://foo.com), [2](https://2.com), [bar](https://bar.com), [1](https://1.com).";
        let result = parse_and_serialize(input);
        // foo and bar should come before numeric refs
        let foo_pos = result.find("[foo]:").unwrap();
        let bar_pos = result.find("[bar]:").unwrap();
        let one_pos = result.find("[1]:").unwrap();
        let two_pos = result.find("[2]:").unwrap();
        // Regular refs first, in order
        assert!(foo_pos < bar_pos, "foo should come before bar");
        // Numeric refs at end, sorted by number
        assert!(
            bar_pos < one_pos,
            "Regular refs should come before numeric refs"
        );
        assert!(
            one_pos < two_pos,
            "Numeric refs should be sorted: 1 before 2, got:\n{}",
            result
        );
    }

    #[test]
    fn test_single_numeric_reference_not_sorted() {
        // A single numeric reference should stay in insertion order
        let input = "See [foo](https://foo.com), [1](https://1.com), [bar](https://bar.com).";
        let result = parse_and_serialize(input);
        let foo_pos = result.find("[foo]:").unwrap();
        let one_pos = result.find("[1]:").unwrap();
        let bar_pos = result.find("[bar]:").unwrap();
        // With only one numeric ref, it stays in insertion order
        assert!(
            foo_pos < one_pos && one_pos < bar_pos,
            "Single numeric ref should stay in insertion order, got:\n{}",
            result
        );
    }

    #[test]
    fn test_hash_numeric_references_sorted() {
        // References like #123 should also be sorted numerically
        let input = "See [#456](https://issue/456) and [#123](https://issue/123).";
        let result = parse_and_serialize(input);
        let pos_123 = result.find("[#123]:").unwrap();
        let pos_456 = result.find("[#456]:").unwrap();
        assert!(
            pos_123 < pos_456,
            "#123 should come before #456, got:\n{}",
            result
        );
    }

    fn parse_and_serialize_with_frontmatter(input: &str) -> String {
        let arena = Arena::new();
        let mut options = ComrakOptions::default();
        options.extension.front_matter_delimiter = Some("---".to_string());
        let root = parse_document(&arena, input, &options);
        let format_options = Options::default();
        serialize_with_source(root, &format_options, None)
    }

    #[test]
    fn test_serialize_yaml_front_matter() {
        let input = "---\ntitle: Hello\nauthor: World\n---\n\n# Heading";
        let result = parse_and_serialize_with_frontmatter(input);
        assert_eq!(
            result,
            "---\ntitle: Hello\nauthor: World\n---\n\nHeading\n=======\n"
        );
    }

    #[test]
    fn test_serialize_yaml_front_matter_only() {
        let input = "---\ntitle: Test\n---\n\nSome content.";
        let result = parse_and_serialize_with_frontmatter(input);
        assert_eq!(result, "---\ntitle: Test\n---\n\nSome content.\n");
    }

    #[test]
    fn test_serialize_two_blank_lines_before_h2() {
        let input = "# Title\n\nParagraph.\n\n## Section";
        let result = parse_and_serialize(input);
        // Should have two blank lines before h2 (one after paragraph + one extra)
        assert!(result.contains("Paragraph.\n\n\nSection"));
    }

    #[test]
    fn test_serialize_one_blank_line_for_empty_section() {
        // When h1 is immediately followed by h2 (empty section), only one blank line
        let input = "# Title\n\n## Section\n\nContent.";
        let result = parse_and_serialize(input);
        // Should have only one blank line between headings
        assert_eq!(result, "Title\n=====\n\nSection\n-------\n\nContent.\n");
    }

    #[test]
    fn test_serialize_consecutive_h2_sections() {
        // When h2 is immediately followed by another h2 (empty section)
        let input = "## Section 1\n\n## Section 2\n\nContent.";
        let result = parse_and_serialize(input);
        // Should have only one blank line between headings
        assert_eq!(
            result,
            "Section 1\n---------\n\nSection 2\n---------\n\nContent.\n"
        );
    }

    fn parse_and_serialize_with_width(input: &str, line_width: usize) -> String {
        let arena = Arena::new();
        let options = ComrakOptions::default();
        let root = parse_document(&arena, input, &options);
        let format_options = Options { line_width };
        serialize_with_source(root, &format_options, None)
    }

    #[test]
    fn test_heading_with_inline_code() {
        // Inline code in headings should be preserved
        let input = "# Heading with `code`";
        let result = parse_and_serialize(input);
        assert_eq!(result, "Heading with `code`\n===================\n");
    }

    #[test]
    fn test_heading_with_multiple_inline_codes() {
        // Multiple inline codes in headings
        let input = "### Looking at the `to`, `cc`, and `bcc` fields";
        let result = parse_and_serialize(input);
        assert_eq!(result, "### Looking at the `to`, `cc`, and `bcc` fields\n");
    }

    #[test]
    fn test_korean_in_link() {
        // Korean text in links should not cause panic
        let input = "[한국어](https://example.com)";
        let result = parse_and_serialize(input);
        assert!(result.contains("[한국어]"));
        assert!(result.contains("https://example.com"));
    }

    #[test]
    fn test_serialize_paragraph_wrap_at_80() {
        // A long line that should wrap at approximately 80 characters
        let input = "This is a very long paragraph that should be wrapped at approximately eighty characters to maintain readability.";
        let result = parse_and_serialize_with_width(input, 80);
        // The line should be wrapped
        assert!(result.contains('\n'));
        // Each line should be at most 80 characters (approximately)
        for line in result.lines() {
            assert!(line.len() <= 85, "Line too long: {} chars", line.len());
        }
    }

    #[test]
    fn test_serialize_paragraph_no_wrap_short() {
        // A short line that should not be wrapped
        let input = "Short paragraph.";
        let result = parse_and_serialize_with_width(input, 80);
        assert_eq!(result, "Short paragraph.\n");
    }

    #[test]
    fn test_serialize_paragraph_wrap_preserves_words() {
        // Words should not be broken
        let input = "Word1 Word2 Word3 Word4 Word5 Word6 Word7 Word8 Word9 Word10 Word11 Word12 Word13 Word14 Word15";
        let result = parse_and_serialize_with_width(input, 40);
        // Check that words are not broken
        for line in result.lines() {
            assert!(!line.ends_with('-'), "Words should not be hyphenated");
        }
    }

    #[test]
    fn test_selective_rewrap_short_lines_preserved() {
        // Short lines (under 80 chars) should be preserved as-is
        let input = "Line one.\nLine two.\nLine three.";
        let result = parse_and_serialize(input);
        // Each line should stay on its own line
        assert_eq!(
            result, "Line one.\nLine two.\nLine three.\n",
            "Short lines should be preserved"
        );
    }

    #[test]
    fn test_selective_rewrap_long_line_wrapped() {
        // A line over 80 chars should be rewrapped
        let input = "This is a very long line that definitely exceeds the eighty character limit and should be wrapped to the next line properly.";
        let result = parse_and_serialize_with_width(input, 80);
        // Should be wrapped
        let lines: Vec<&str> = result.lines().collect();
        assert!(
            lines.len() > 1,
            "Long line should be wrapped, got:\n{}",
            result
        );
        // Each line should be under 80 chars
        for line in &lines {
            assert!(line.len() <= 80, "Line should be under 80 chars: {}", line);
        }
    }

    #[test]
    fn test_selective_rewrap_mixed_lines() {
        // Mix of short and long lines - short should be preserved, long rewrapped
        let input = "Short line one.\nShort line two.\nThis is a very long line that definitely exceeds the eighty character limit and needs to be wrapped.";
        let result = parse_and_serialize_with_width(input, 80);
        // Short lines should be preserved
        assert!(
            result.starts_with("Short line one.\nShort line two.\n"),
            "Short lines should be preserved at start, got:\n{}",
            result
        );
    }

    fn parse_and_serialize_with_table(input: &str) -> String {
        let arena = Arena::new();
        let mut options = ComrakOptions::default();
        options.extension.table = true;
        let root = parse_document(&arena, input, &options);
        let format_options = Options::default();
        serialize_with_source(root, &format_options, None)
    }

    #[test]
    fn test_serialize_simple_table() {
        let input = "| A | B |\n|---|---|\n| 1 | 2 |";
        let result = parse_and_serialize_with_table(input);
        assert!(result.contains("| A"));
        assert!(result.contains("| B"));
        assert!(result.contains("| 1"));
        assert!(result.contains("| 2"));
    }

    #[test]
    fn test_serialize_table_with_alignment() {
        let input = "| Left | Center | Right |\n|:-----|:------:|------:|\n| L | C | R |";
        let result = parse_and_serialize_with_table(input);
        // Should contain alignment markers
        assert!(result.contains(":--"));
        assert!(result.contains("--:"));
    }

    #[test]
    fn test_serialize_table_aligned_columns() {
        let input = "| Name | Age |\n|------|-----|\n| Alice | 30 |\n| Bob | 25 |";
        let result = parse_and_serialize_with_table(input);
        // Columns should be aligned with padding
        let lines: Vec<&str> = result.lines().collect();
        // All rows should have the same pipe positions (aligned)
        if lines.len() >= 3 {
            // Find pipe positions in first data row
            let first_pipes: Vec<_> = lines[0].match_indices('|').map(|(i, _)| i).collect();
            // Verify other rows have pipes in similar positions (allowing for padding)
            for line in &lines[1..] {
                let pipes: Vec<_> = line.match_indices('|').map(|(i, _)| i).collect();
                assert_eq!(
                    first_pipes.len(),
                    pipes.len(),
                    "All rows should have same number of pipes"
                );
            }
        }
    }

    #[test]
    fn test_serialize_table_with_links() {
        // Table cells containing links should preserve the links
        let input =
            "| Package | Link |\n|---------|------|\n| [foo](/foo) | [bar](https://bar.com) |";
        let result = parse_and_serialize_with_table(input);
        // Links should be preserved in table cells
        assert!(
            result.contains("[foo](/foo)"),
            "Relative link should be preserved in table, got:\n{}",
            result
        );
        assert!(
            result.contains("[bar]"),
            "External link text should be preserved in table, got:\n{}",
            result
        );
    }

    #[test]
    fn test_serialize_table_with_reference_links() {
        // Table cells containing reference-style links should preserve them
        let input = "| Package | JSR |\n|---------|-----|\n| [*@pkg/core*](/packages/core/) | [JSR][jsr:@pkg/core] |\n\n[jsr:@pkg/core]: https://jsr.io/@pkg/core";
        let result = parse_and_serialize_with_source(input);
        // Reference links should be preserved in table cells
        assert!(
            result.contains("[*@pkg/core*](/packages/core/)"),
            "Link with emphasis should be preserved in table, got:\n{}",
            result
        );
        assert!(
            result.contains("[JSR][jsr:@pkg/core]"),
            "Reference-style link should be preserved in table, got:\n{}",
            result
        );
    }

    fn parse_and_serialize_with_description_list(input: &str) -> String {
        let arena = Arena::new();
        let mut options = ComrakOptions::default();
        options.extension.description_lists = true;
        let root = parse_document(&arena, input, &options);
        let format_options = Options::default();
        serialize_with_source(root, &format_options, None)
    }

    #[test]
    fn test_serialize_definition_list_single() {
        let input = "Term\n:   Definition";
        let result = parse_and_serialize_with_description_list(input);
        assert!(result.contains("Term\n"));
        assert!(result.contains(":   Definition"));
    }

    #[test]
    fn test_serialize_definition_list_multiple_definitions() {
        let input = "Term\n:   First definition\n:   Second definition";
        let result = parse_and_serialize_with_description_list(input);
        assert!(result.contains("Term\n"));
        assert!(result.contains(":   First definition"));
        assert!(result.contains(":   Second definition"));
    }

    fn parse_and_serialize_with_alerts(input: &str) -> String {
        let arena = Arena::new();
        let mut options = ComrakOptions::default();
        options.extension.alerts = true;
        let root = parse_document(&arena, input, &options);
        let format_options = Options::default();
        serialize_with_source(root, &format_options, None)
    }

    #[test]
    fn test_serialize_github_note_alert() {
        let input = "> [!NOTE]\n> This is a note.";
        let result = parse_and_serialize_with_alerts(input);
        assert!(result.contains("> [!NOTE]"));
        assert!(result.contains("> This is a note."));
    }

    #[test]
    fn test_serialize_github_warning_alert() {
        let input = "> [!WARNING]\n> This is a warning.";
        let result = parse_and_serialize_with_alerts(input);
        assert!(result.contains("> [!WARNING]"));
        assert!(result.contains("> This is a warning."));
    }

    #[test]
    fn test_serialize_github_caution_alert() {
        let input = "> [!CAUTION]\n> Be careful!";
        let result = parse_and_serialize_with_alerts(input);
        assert!(result.contains("> [!CAUTION]"));
        assert!(result.contains("> Be careful!"));
    }

    fn parse_and_serialize_with_footnotes(input: &str) -> String {
        let arena = Arena::new();
        let mut options = ComrakOptions::default();
        options.extension.footnotes = true;
        let root = parse_document(&arena, input, &options);
        let format_options = Options::default();
        serialize_with_source(root, &format_options, None)
    }

    #[test]
    fn test_serialize_footnote_reference() {
        let input = "This has a footnote[^1].\n\n[^1]: The footnote text.";
        let result = parse_and_serialize_with_footnotes(input);
        assert!(result.contains("[^1]"));
    }

    #[test]
    fn test_serialize_footnote_definition() {
        let input = "Text[^note].\n\n[^note]: A named footnote.";
        let result = parse_and_serialize_with_footnotes(input);
        assert!(result.contains("[^note]"));
    }

    #[test]
    fn test_serialize_double_space_after_period() {
        // Hong's style uses two spaces after periods
        let input = "First sentence.  Second sentence.";
        let result = parse_and_serialize(input);
        // Should preserve double spaces
        assert_eq!(result, "First sentence.  Second sentence.\n");
    }

    #[test]
    fn test_serialize_long_list_item_wrapping() {
        // Long list items should wrap with 4-space continuation indent
        let input = " -  This is a very long list item that should wrap to the next line with proper indentation to maintain readability.";
        let result = parse_and_serialize_with_width(input, 80);
        // Should contain wrapped content with proper indent
        assert!(result.contains(" -  This is a very long list item"));
        assert!(result.contains("\n    ")); // Continuation with 4 spaces
    }

    fn parse_and_serialize_with_alerts_and_width(input: &str, line_width: usize) -> String {
        let arena = Arena::new();
        let mut options = ComrakOptions::default();
        options.extension.alerts = true;
        let root = parse_document(&arena, input, &options);
        let format_options = Options { line_width };
        serialize_with_source(root, &format_options, None)
    }

    #[test]
    fn test_serialize_list_in_alert() {
        // Lists inside alerts should have proper prefixing
        let input = "> [!NOTE]\n>  -  First item\n>  -  Second item";
        let result = parse_and_serialize_with_alerts(input);
        assert!(result.contains("> [!NOTE]"));
        assert!(result.contains(">  -  First item"));
        assert!(result.contains(">  -  Second item"));
    }

    #[test]
    fn test_serialize_long_list_item_in_alert() {
        // Long list items in alerts should wrap with proper continuation prefix
        let input = "> [!NOTE]\n>  -  This is a very long list item that should wrap properly inside the alert block.";
        let result = parse_and_serialize_with_alerts_and_width(input, 60);
        // Should wrap with ">     " continuation (> + 4 spaces)
        assert!(result.contains(">  -  This is a very long"));
        assert!(result.contains("\n>     ")); // Continuation line with > and 4 spaces
    }

    #[test]
    fn test_serialize_external_link_as_reference() {
        // External URLs should be converted to reference links
        let input = "Visit [Rust](https://www.rust-lang.org/) for more info.";
        let result = parse_and_serialize(input);
        // Should use reference style, not inline
        assert!(result.contains("[Rust]"));
        assert!(!result.contains("](https://"));
        assert!(result.contains("[Rust]: https://www.rust-lang.org/"));
    }

    #[test]
    fn test_serialize_relative_link_stays_inline() {
        // Relative paths should stay as inline links
        let input = "See the [README](./README.md) for details.";
        let result = parse_and_serialize(input);
        // Should keep inline style for relative paths
        assert!(result.contains("[README](./README.md)"));
    }

    #[test]
    fn test_serialize_reference_links_at_section_end() {
        // Reference definitions should appear at the end of each section
        let input = r#"# Title

See [Example](https://example.com/) here.

## Section One

Visit [Rust](https://www.rust-lang.org/) and [Cargo](https://doc.rust-lang.org/cargo/).

## Section Two

Check [Python](https://python.org/) too.
"#;
        let result = parse_and_serialize(input);
        // Each section should have its references at the end
        assert!(result.contains("[Rust]: https://www.rust-lang.org/"));
        assert!(result.contains("[Cargo]: https://doc.rust-lang.org/cargo/"));
        assert!(result.contains("[Python]: https://python.org/"));
        // References should come before the next section
        let rust_def_pos = result.find("[Rust]: ").unwrap();
        let section_two_pos = result.find("Section Two").unwrap();
        assert!(rust_def_pos < section_two_pos);
    }

    #[test]
    fn test_serialize_shortcut_reference_when_text_matches_label() {
        // When link text matches a sensible label, use shortcut reference [text]
        let input = "Use [comrak](https://docs.rs/comrak) for parsing.";
        let result = parse_and_serialize(input);
        // Should use shortcut reference style
        assert!(result.contains("[comrak]"));
        assert!(result.contains("[comrak]: https://docs.rs/comrak"));
    }

    #[test]
    fn test_serialize_escaped_asterisk_in_emphasis() {
        // Escaped asterisks inside emphasis should be preserved
        let input = r"*\*.ts*";
        let result = parse_and_serialize(input);
        assert_eq!(result, "*\\*.ts*\n");
    }

    #[test]
    fn test_serialize_escaped_underscore() {
        // Escaped underscores should be preserved
        let input = r"\_\_init\_\_";
        let result = parse_and_serialize(input);
        assert_eq!(result, "\\_\\_init\\_\\_\n");
    }

    #[test]
    fn test_serialize_escaped_brackets() {
        // Escaped brackets should be preserved (not treated as links)
        // The closing ] at end of text doesn't need escaping (can't close a link)
        let input = r"\[not a link\]";
        let result = parse_and_serialize(input);
        assert_eq!(result, "\\[not a link]\n");
    }

    #[test]
    fn test_double_brackets_preserved() {
        // Double brackets around references (common in changelogs) should not be escaped
        let input = "See [[#123]] for details.\n\n[#123]: https://example.com/123\n";
        let result = parse_and_serialize(input);
        assert_eq!(
            result,
            "See [[#123]] for details.\n\n[#123]: https://example.com/123\n"
        );
    }

    #[test]
    fn test_double_brackets_with_multiple_refs() {
        // Double brackets with multiple references and text
        let input = "[[#120], [#121] by Author]\n\n[#120]: https://example.com/120\n[#121]: https://example.com/121\n";
        let result = parse_and_serialize(input);
        assert_eq!(
            result,
            "[[#120], [#121] by Author]\n\n[#120]: https://example.com/120\n[#121]: https://example.com/121\n"
        );
    }

    #[test]
    fn test_serialize_escaped_backslash() {
        // Escaped backslash should be preserved
        let input = r"path\\to\\file";
        let result = parse_and_serialize(input);
        assert_eq!(result, "path\\\\to\\\\file\n");
    }

    #[test]
    fn test_multi_paragraph_list_item() {
        // Multiple paragraphs within a single list item should be separated by blank lines
        let input = " -  First paragraph.\n\n    Second paragraph.\n\n    Third paragraph.\n";
        let result = parse_and_serialize(input);
        assert_eq!(
            result,
            " -  First paragraph.\n\n    Second paragraph.\n\n    Third paragraph.\n"
        );
    }

    #[test]
    fn test_tight_nested_list() {
        // Nested list directly following text (tight) - no blank line
        // 4-space indent for nested lists
        let input = " -  Item:\n    -  Nested 1\n    -  Nested 2\n";
        let result = parse_and_serialize(input);
        assert_eq!(result, " -  Item:\n    -  Nested 1\n    -  Nested 2\n");
    }

    #[test]
    fn test_loose_nested_list() {
        // Nested list after blank line (loose) - preserve blank line
        // 4-space indent for nested lists
        let input = " -  Item.\n\n    -  Nested 1\n    -  Nested 2\n";
        let result = parse_and_serialize(input);
        assert_eq!(result, " -  Item.\n\n    -  Nested 1\n    -  Nested 2\n");
    }

    #[test]
    fn test_serialize_asterisk_in_text_not_emphasis() {
        // Asterisks in plain text that aren't emphasis should be escaped
        let input = "5 * 3 = 15";
        let result = parse_and_serialize(input);
        // The asterisk in "5 * 3" should be escaped to prevent misinterpretation
        assert_eq!(result, "5 \\* 3 = 15\n");
    }

    #[test]
    fn test_serialize_image_inside_link_badge_style() {
        // Badge-style: image inside a link, both using reference style
        // Input: [![alt][img-ref]][link-ref] with definitions
        // Should output fully inline: [![alt](img-url)](link-url)
        let input = r#"[![JSR][JSR badge]][JSR]

[JSR]: https://jsr.io/
[JSR badge]: https://jsr.io/badge.svg
"#;
        let result = parse_and_serialize(input);
        // The output should have a clickable image linking to JSR
        assert!(
            result.contains("[![JSR](https://jsr.io/badge.svg)](https://jsr.io/)"),
            "Should output fully inline badge-style link"
        );
        assert!(
            !result.contains("[![JSR](https://jsr.io/badge.svg)]:"),
            "Should not create malformed reference definition"
        );
    }

    #[test]
    fn test_serialize_underscore_in_word_not_escaped() {
        // Underscores in the middle of words (like ALL_CAPS) should not be escaped
        // because they don't create emphasis in CommonMark
        let input = "Use ALL_CAPS for constants.";
        let result = parse_and_serialize(input);
        assert_eq!(result, "Use ALL_CAPS for constants.\n");
    }

    #[test]
    fn test_serialize_underscore_emphasis_boundary() {
        // Underscores at word boundaries should be escaped to prevent emphasis
        let input = r"\_start and end\_";
        let result = parse_and_serialize(input);
        assert_eq!(result, "\\_start and end\\_\n");
    }

    #[test]
    fn test_serialize_autolink_preserved() {
        // Autolinks <url> should be preserved as autolink format, not reference style
        let input = "Visit <https://example.com/> for more info.";
        let result = parse_and_serialize(input);
        assert_eq!(result, "Visit <https://example.com/> for more info.\n");
    }

    #[test]
    fn test_serialize_nested_list_wrap_continuation() {
        // Nested list items should wrap with proper continuation indent
        // accounting for the parent list's indentation
        let input = " 1. First\n     -  This is a very long nested item that should wrap with proper eight-space continuation.";
        let result = parse_and_serialize_with_width(input, 80);
        // Continuation should have 8 spaces (4 for parent + 4 for list item content)
        assert!(
            result.contains("\n        "),
            "Nested list continuation should have 8-space indent, got:\n{}",
            result
        );
    }

    #[test]
    fn test_directive_disable_next_line() {
        // hongdown-disable-next-line should preserve the next block element as-is
        let input = "<!-- hongdown-disable-next-line -->\n[![Badge][badge-img]][badge-url]\n\n[badge-img]: https://example.com/badge.svg\n[badge-url]: https://example.com";
        let result = parse_and_serialize_with_source(input);
        // The badge line should be preserved exactly as-is (not converted to inline)
        assert!(
            result.contains("[![Badge][badge-img]][badge-url]"),
            "disable-next-line should preserve the next line as-is, got:\n{}",
            result
        );
    }

    #[test]
    fn test_directive_disable_file() {
        // hongdown-disable-file should preserve the entire file as-is
        let input = "<!-- hongdown-disable-file -->\n\nTitle\n===\n\nSome paragraph with *emphasis* that would normally be reformatted.";
        let result = parse_and_serialize_with_source(input);
        // The entire content after the directive should be preserved
        assert!(
            result.contains("Title\n==="),
            "disable-file should preserve file content as-is, got:\n{}",
            result
        );
    }

    #[test]
    fn test_directive_disable_next_section() {
        // hongdown-disable-next-section should preserve content until the next heading
        let input = "First section\n-------------\n\nNormal content.\n\n<!-- hongdown-disable-next-section -->\n\nSecond section\n--------------\n\n[![Badge][img]][url]\n\n[img]: https://example.com/img.svg\n[url]: https://example.com\n\nThird section\n-------------\n\nThis should be formatted normally.";
        let result = parse_and_serialize_with_source(input);
        // Second section should be preserved as-is
        assert!(
            result.contains("[![Badge][img]][url]"),
            "disable-next-section should preserve section content as-is, got:\n{}",
            result
        );
    }

    #[test]
    fn test_directive_disable_enable() {
        // hongdown-disable and hongdown-enable should bracket unformatted regions
        let input = "Normal paragraph.\n\n<!-- hongdown-disable -->\n\n[![Badge][img]][url]\n\nAnother unformatted line.\n\n<!-- hongdown-enable -->\n\nBack to normal formatting.\n\n[img]: https://example.com/img.svg\n[url]: https://example.com";
        let result = parse_and_serialize_with_source(input);
        // Content between disable/enable should be preserved
        assert!(
            result.contains("[![Badge][img]][url]"),
            "disable/enable should preserve bracketed content as-is, got:\n{}",
            result
        );
        assert!(
            result.contains("Another unformatted line."),
            "disable/enable should preserve all bracketed content, got:\n{}",
            result
        );
    }

    #[test]
    fn test_preserve_reference_style_badge() {
        // Reference-style badge links should be preserved as reference style
        let input = "[![JSR][JSR badge]][JSR]\n\n[JSR]: https://jsr.io/@optique\n[JSR badge]: https://jsr.io/badges/@optique/core";
        let result = parse_and_serialize_with_source(input);
        // Should preserve reference style, not convert to inline
        assert!(
            result.contains("[![JSR][JSR badge]][JSR]"),
            "Reference-style badge should be preserved, got:\n{}",
            result
        );
    }

    #[test]
    fn test_preserve_reference_style_image() {
        // Reference-style images should be preserved as reference style
        let input = "![Logo][logo]\n\n[logo]: https://example.com/logo.png";
        let result = parse_and_serialize_with_source(input);
        // Should preserve reference style
        assert!(
            result.contains("![Logo][logo]"),
            "Reference-style image should be preserved, got:\n{}",
            result
        );
    }

    #[test]
    fn test_preserve_reference_style_link() {
        // Reference-style links should be preserved as reference style
        let input =
            "Check the [documentation][docs] for more info.\n\n[docs]: https://example.com/docs";
        let result = parse_and_serialize_with_source(input);
        // Should preserve reference style
        assert!(
            result.contains("[documentation][docs]"),
            "Reference-style link should be preserved, got:\n{}",
            result
        );
    }

    #[test]
    fn test_code_span_with_backticks() {
        // Code spans containing backticks should use double backticks as delimiters
        let input = "Here is `` `code` `` in text.";
        let result = parse_and_serialize_with_source(input);
        assert!(
            result.contains("`` `code` ``"),
            "Code span with backtick should use double backtick delimiters, got:\n{}",
            result
        );
    }

    #[test]
    fn test_code_span_with_multiple_backticks() {
        // Code spans containing double backticks should use triple backticks
        let input = "Use ``` `` ``` for double backticks.";
        let result = parse_and_serialize_with_source(input);
        assert!(
            result.contains("``` `` ```"),
            "Code span with double backticks should use triple backtick delimiters, got:\n{}",
            result
        );
    }

    #[test]
    fn test_code_span_simple() {
        // Simple code spans without backticks should use single backticks
        let input = "Use `code` for inline code.";
        let result = parse_and_serialize_with_source(input);
        assert!(
            result.contains("`code`"),
            "Simple code span should use single backticks, got:\n{}",
            result
        );
    }

    #[test]
    fn test_code_span_starting_with_backtick() {
        // Code starting with backtick needs space padding
        let input = "The code `` `foo `` starts with a backtick.";
        let result = parse_and_serialize_with_source(input);
        assert!(
            result.contains("`` `foo ``"),
            "Code starting with backtick should have space padding, got:\n{}",
            result
        );
    }

    #[test]
    fn test_code_span_ending_with_backtick() {
        // Code ending with backtick needs space padding
        let input = "The code `` foo` `` ends with a backtick.";
        let result = parse_and_serialize_with_source(input);
        assert!(
            result.contains("`` foo` ``"),
            "Code ending with backtick should have space padding, got:\n{}",
            result
        );
    }

    #[test]
    fn test_reference_link_multiline_text_normalized() {
        // Reference link text spanning multiple lines should be normalized to single line
        let input = "Click [here for\nmore info][1].\n\n[1]: https://example.com";
        let result = parse_and_serialize_with_source(input);
        // The link text should be normalized (newline -> space)
        assert!(
            result.contains("[here for more info]"),
            "Reference link text should be normalized to single line, got:\n{}",
            result
        );
    }

    #[test]
    fn test_reference_link_idempotent() {
        // Reference style link should be idempotent after formatting
        let input = "Click [here for more info][1].\n\n[1]: https://example.com";
        let result = parse_and_serialize_with_source(input);
        let result2 = parse_and_serialize_with_source(&result);
        assert_eq!(
            result, result2,
            "Reference link should be idempotent.\nFirst pass:\n{}\nSecond pass:\n{}",
            result, result2
        );
    }

    #[test]
    fn test_code_block_in_list_item() {
        // Code block inside a list item should be properly indented
        let input = " -  Example:\n\n    ~~~~\n    code here\n    ~~~~";
        let result = parse_and_serialize_with_source(input);
        // Code block should be on a new line with proper indentation
        assert!(
            result.contains("Example:\n\n    ~~~~"),
            "Code block in list item should have blank line and indentation, got:\n{}",
            result
        );
        assert!(
            result.contains("    code here"),
            "Code block content should be indented, got:\n{}",
            result
        );
    }

    #[test]
    fn test_code_block_in_list_item_no_language() {
        // Code block without language identifier should not add one
        let input = " -  Item:\n\n    ~~~~\n    code\n    ~~~~";
        let result = parse_and_serialize_with_source(input);
        // Should use ~~~~ without language identifier
        assert!(
            result.contains("~~~~\n"),
            "Code block should not have language identifier added, got:\n{}",
            result
        );
    }
}
