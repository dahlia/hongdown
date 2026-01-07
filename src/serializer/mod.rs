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

pub use state::{ReferenceLink, Serializer};

use comrak::nodes::{AstNode, NodeValue};

use crate::Options;

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

            // Output numeric references (sorted by number)
            for (_, reference) in numeric_refs {
                Self::write_reference(&mut self.output, reference);
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
                        self.output.push('\n');
                    }
                    self.serialize_node(child);
                }
            }
            NodeValue::DescriptionItem(_) => {
                self.serialize_children(node);
            }
            NodeValue::DescriptionTerm => {
                self.serialize_children(node);
                self.output.push('\n');
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
                self.output
                    .push_str(&escape::format_code_span(&code.literal));
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

    fn serialize_children<'b>(&mut self, node: &'b AstNode<'b>) {
        for child in node.children() {
            self.serialize_node(child);
        }
    }
}

#[cfg(test)]
mod tests;
