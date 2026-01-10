//! Serializer for converting comrak AST to formatted Markdown.

mod block;
mod code;
mod document;
mod escape;
mod inline;
mod link;
mod list;
mod state;
mod table;
mod wrap;

pub use state::{ReferenceLink, Serializer, Warning};

use comrak::nodes::{AstNode, NodeValue};
use unicode_width::UnicodeWidthStr;

use crate::Options;

/// Result of serialization including output and any warnings.
pub struct SerializeResult {
    /// The formatted Markdown output.
    pub output: String,
    /// Warnings generated during formatting.
    pub warnings: Vec<Warning>,
}

/// Serializes a comrak AST node to a formatted Markdown string,
/// with access to the original source for directive handling.
pub fn serialize_with_source<'a>(
    node: &'a AstNode<'a>,
    options: &Options,
    source: Option<&str>,
) -> String {
    serialize_with_source_and_warnings(node, options, source).output
}

/// Serializes a comrak AST node to a formatted Markdown string,
/// returning both the output and any warnings generated.
pub fn serialize_with_source_and_warnings<'a>(
    node: &'a AstNode<'a>,
    options: &Options,
    source: Option<&str>,
) -> SerializeResult {
    let source_lines: Vec<&str> = source.map(|s| s.lines().collect()).unwrap_or_default();
    let source_ends_with_newline = source.is_some_and(|s| s.ends_with('\n'));
    let mut serializer = Serializer::new(options, source_lines, source_ends_with_newline);
    serializer.serialize_node(node);
    SerializeResult {
        output: serializer.output,
        warnings: serializer.warnings,
    }
}

impl<'a> Serializer<'a> {
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
        let text = escape::normalize_whitespace(&text);

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
                let label = escape::normalize_whitespace(&label);

                // If label is empty, it's collapsed reference - mark with special prefix
                // to distinguish from shortcut reference
                let final_label = if label.is_empty() {
                    format!("\x01{}", text) // Use \x01 as marker for collapsed reference
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

    /// Ensure output ends with a blank line (two newlines).
    /// Used before emitting reference definitions and footnotes.
    fn ensure_blank_line(&mut self) {
        if !self.output.ends_with("\n\n") {
            if self.output.ends_with('\n') {
                self.output.push('\n');
            } else {
                self.output.push_str("\n\n");
            }
        }
    }

    /// Output pending reference definitions and clear them
    fn flush_references(&mut self) {
        if self.pending_references.is_empty() {
            return;
        }

        // Take ownership of references to avoid borrow issues
        // Filter out references that have already been emitted
        let refs: Vec<ReferenceLink> = self
            .pending_references
            .values()
            .filter(|r| !self.emitted_references.contains(&r.label))
            .cloned()
            .collect();
        self.pending_references.clear();

        if refs.is_empty() {
            return;
        }

        // Count numeric references to decide sorting strategy
        let numeric_count = refs
            .iter()
            .filter(|r| Self::extract_numeric_label(&r.label).is_some())
            .count();

        self.ensure_blank_line();

        if numeric_count < 2 {
            // Less than 2 numeric refs: output all in insertion order
            for reference in &refs {
                Self::write_reference(&mut self.output, reference);
                self.emitted_references.insert(reference.label.clone());
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
                self.emitted_references.insert(reference.label.clone());
            }

            // Output numeric references (sorted by number)
            for (_, reference) in numeric_refs {
                Self::write_reference(&mut self.output, reference);
                self.emitted_references.insert(reference.label.clone());
            }
        }
    }

    /// Extract numeric value from a reference label like "123" or "#123"
    fn extract_numeric_label(label: &str) -> Option<u64> {
        let label = label.strip_prefix('#').unwrap_or(label);
        label.parse::<u64>().ok()
    }

    /// Write a single reference definition to output
    fn write_reference(output: &mut String, reference: &ReferenceLink) {
        output.push('[');
        // Replace SoftBreak marker with space for reference labels
        // (comrak normalizes whitespace in labels, so this ensures idempotency)
        output.push_str(&reference.label.replace('\x00', " "));
        output.push_str("]: ");
        output.push_str(&reference.url);
        if !reference.title.is_empty() {
            output.push_str(" \"");
            output.push_str(&reference.title);
            output.push('"');
        }
        output.push('\n');
    }

    /// Output pending footnote definitions that were referenced before the given line.
    /// If `before_line` is None, flush all pending footnotes.
    fn flush_footnotes_before(&mut self, before_line: Option<usize>) {
        if self.footnotes.pending.is_empty() {
            return;
        }

        // Collect footnotes to emit (referenced before the given line, not already emitted)
        let mut to_emit: Vec<state::FootnoteDefinition> = Vec::new();
        let mut to_keep: Vec<(String, state::FootnoteDefinition)> = Vec::new();

        for (name, footnote) in self.footnotes.pending.drain(..) {
            if self.footnotes.emitted.contains(&name) {
                continue;
            }
            let should_emit = match before_line {
                Some(line) => footnote.reference_line > 0 && footnote.reference_line < line,
                None => true,
            };
            if should_emit {
                to_emit.push(footnote);
            } else {
                to_keep.push((name, footnote));
            }
        }

        // Put back footnotes that shouldn't be emitted yet
        for (name, footnote) in to_keep {
            self.footnotes.pending.insert(name, footnote);
        }

        if to_emit.is_empty() {
            return;
        }

        self.ensure_blank_line();

        // Count numeric footnotes to decide sorting strategy
        let numeric_count = to_emit
            .iter()
            .filter(|f| Self::extract_numeric_footnote_name(&f.name).is_some())
            .count();

        if numeric_count < 2 {
            // Less than 2 numeric footnotes: output all in insertion order
            for footnote in &to_emit {
                self.write_footnote(footnote);
                self.footnotes.emitted.insert(footnote.name.clone());
            }
        } else {
            // 2+ numeric footnotes: separate, sort numeric ones, output regular first
            let mut regular_footnotes: Vec<&state::FootnoteDefinition> = Vec::new();
            let mut numeric_footnotes: Vec<(u64, &state::FootnoteDefinition)> = Vec::new();

            for footnote in &to_emit {
                if let Some(num) = Self::extract_numeric_footnote_name(&footnote.name) {
                    numeric_footnotes.push((num, footnote));
                } else {
                    regular_footnotes.push(footnote);
                }
            }

            // Sort numeric footnotes by their numeric value
            numeric_footnotes.sort_by_key(|(num, _)| *num);

            // Output regular footnotes first (in insertion order)
            for footnote in regular_footnotes {
                self.write_footnote(footnote);
                self.footnotes.emitted.insert(footnote.name.clone());
            }

            // Output numeric footnotes (sorted by number)
            for (_, footnote) in numeric_footnotes {
                self.write_footnote(footnote);
                self.footnotes.emitted.insert(footnote.name.clone());
            }
        }
    }

    /// Extract numeric value from a footnote name like "1" or "123"
    fn extract_numeric_footnote_name(name: &str) -> Option<u64> {
        name.parse::<u64>().ok()
    }

    /// Output all pending footnote definitions
    fn flush_footnotes(&mut self) {
        self.flush_footnotes_before(None);
    }

    /// Output pending footnote reference definitions whose parent footnote was referenced
    /// before the given line. If `before_line` is None, flush all pending footnote references.
    fn flush_footnote_references_before(&mut self, before_line: Option<usize>) {
        if self.footnotes.pending_references.is_empty() {
            return;
        }

        // Separate references to emit vs keep, based on their parent footnote's reference line
        let mut to_emit: Vec<ReferenceLink> = Vec::new();
        let mut to_keep: Vec<(String, (ReferenceLink, usize))> = Vec::new();

        for (label, (reference, footnote_ref_line)) in self.footnotes.pending_references.drain(..) {
            if self.emitted_references.contains(&label) {
                continue;
            }
            let should_emit = match before_line {
                Some(line) => footnote_ref_line > 0 && footnote_ref_line < line,
                None => true,
            };
            if should_emit {
                to_emit.push(reference);
            } else {
                to_keep.push((label, (reference, footnote_ref_line)));
            }
        }

        // Put back references that shouldn't be emitted yet
        for (label, value) in to_keep {
            self.footnotes.pending_references.insert(label, value);
        }

        if to_emit.is_empty() {
            return;
        }

        self.ensure_blank_line();

        // Output references in insertion order
        for reference in &to_emit {
            Self::write_reference(&mut self.output, reference);
            self.emitted_references.insert(reference.label.clone());
        }
    }

    /// Output all pending footnote reference definitions.
    fn flush_footnote_references(&mut self) {
        self.flush_footnote_references_before(None);
    }

    /// Write a single footnote definition to output, wrapping at 80 characters
    fn write_footnote(&mut self, footnote: &state::FootnoteDefinition) {
        let prefix = format!("[^{}]: ", footnote.name);
        // Continuation indent matches prefix length for alignment
        let continuation_indent = " ".repeat(prefix.len());

        // Wrap content at 80 chars, accounting for prefix on first line
        let first_line_width = 80 - prefix.width();
        let continuation_width = 80 - continuation_indent.len();

        // Replace SoftBreak marker (\x00) with space before processing
        let content = footnote.content.replace('\x00', " ");
        let words: Vec<&str> = content.split_whitespace().collect();
        if words.is_empty() {
            self.output.push_str(&prefix);
            self.output.push('\n');
            return;
        }

        let mut lines: Vec<String> = Vec::new();
        let mut current_line = String::new();
        let mut is_first_line = true;

        for word in words {
            let max_width = if is_first_line {
                first_line_width
            } else {
                continuation_width
            };

            if current_line.is_empty() {
                current_line.push_str(word);
            } else if current_line.width() + 1 + word.width() <= max_width {
                current_line.push(' ');
                current_line.push_str(word);
            } else {
                // Line is full, start a new one
                lines.push(current_line);
                current_line = word.to_string();
                is_first_line = false;
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }

        // Output lines
        for (i, line) in lines.iter().enumerate() {
            if i == 0 {
                self.output.push_str(&prefix);
                self.output.push_str(line);
            } else {
                self.output.push_str(&continuation_indent);
                self.output.push_str(line);
            }
            self.output.push('\n');
        }
    }

    pub fn serialize_node<'b>(&mut self, node: &'b AstNode<'b>) {
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
                // Serialize description list items with blank lines between them
                let children: Vec<_> = node.children().collect();
                for (i, child) in children.iter().enumerate() {
                    if i > 0 {
                        // Add blank line between description items
                        // If inside a blockquote, we need to add the > prefix on the blank line
                        // to keep items within the same blockquote
                        if self.in_block_quote {
                            self.output.push_str(&self.list_item_indent);
                            // Use blockquote_prefix without trailing space for blank lines
                            self.output.push_str(self.blockquote_prefix.trim_end());
                            self.output.push('\n');
                        } else {
                            self.output.push('\n');
                        }
                    }
                    self.serialize_node(child);
                }
            }
            NodeValue::DescriptionItem(_) => {
                // Serialize term and details without extra blank lines between them
                self.serialize_children(node);
            }
            NodeValue::DescriptionTerm => {
                self.serialize_children(node);
                // No extra newline here - DescriptionDetails will handle formatting
            }
            NodeValue::DescriptionDetails => {
                self.serialize_description_details(node);
            }
            NodeValue::Alert(alert) => {
                self.serialize_alert(node, alert.alert_type);
            }
            NodeValue::Item(_) => {
                self.serialize_list_item(node, None);
            }
            NodeValue::TaskItem(task_item) => {
                self.serialize_list_item(node, Some(task_item.symbol));
            }
            NodeValue::Paragraph => {
                self.serialize_paragraph(node);
            }
            NodeValue::ThematicBreak => {
                self.serialize_thematic_break();
            }
            NodeValue::Text(text) => {
                self.output.push_str(&escape::escape_text(text));
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
                // Try to use original source to preserve spacing
                if let Some(source) = self.extract_source(node) {
                    self.output.push_str(&source);
                } else {
                    self.output
                        .push_str(&escape::format_code_span(&code.literal));
                }
            }
            NodeValue::Link(link) => {
                self.serialize_link(node, &link.url, &link.title);
            }
            NodeValue::Image(image) => {
                self.serialize_image(node, &image.url, &image.title);
            }
            NodeValue::FootnoteReference(footnote_ref) => {
                // Record the line where this footnote is referenced
                let ref_line = node.data.borrow().sourcepos.start.line;
                self.footnotes
                    .record_reference_line(footnote_ref.name.clone(), ref_line);
                self.output.push_str("[^");
                self.output.push_str(&footnote_ref.name);
                self.output.push(']');
            }
            NodeValue::FootnoteDefinition(footnote_def) => {
                // Use the reference line (where footnote was used), not definition line
                let reference_line = self
                    .footnotes
                    .get_reference_line(&footnote_def.name)
                    .unwrap_or(0);
                // Collect content from children
                // Set flag so references within footnotes go to pending_footnote_references
                // Also set the current footnote's reference line for proper flush timing
                self.footnotes.start_collecting(reference_line);
                let mut content = String::new();
                for child in node.children() {
                    self.collect_inline_node(child, &mut content);
                }
                self.footnotes.stop_collecting();
                // Add to pending footnotes (will be flushed at section end)
                self.footnotes.add(
                    footnote_def.name.clone(),
                    content.trim().to_string(),
                    reference_line,
                );
            }
            _ => {
                // For now, just recurse into children for unhandled nodes
                self.serialize_children(node);
            }
        }
    }

    fn serialize_children<'b>(&mut self, node: &'b AstNode<'b>) {
        for child in node.children() {
            self.serialize_node(child);
        }
    }
}

#[cfg(test)]
mod tests;
