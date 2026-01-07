//! Serializer for converting comrak AST to formatted Markdown.

mod escape;
mod list;
mod state;
mod table;
mod wrap;

pub use state::{Directive, ReferenceLink, Serializer};

use comrak::nodes::{AlertType, AstNode, NodeValue};

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

    /// Format a reference-style link and write to output buffer.
    fn format_reference_link(
        &mut self,
        output: &mut String,
        text: &str,
        label: &str,
        url: &str,
        title: &str,
    ) {
        if label.starts_with('\x01') {
            // Collapsed reference: [text][]
            let actual_label = label.strip_prefix('\x01').unwrap();
            output.push('[');
            output.push_str(text);
            output.push_str("][]");

            self.add_reference(actual_label.to_string(), url.to_string(), title.to_string());
        } else if text == label {
            // Shortcut reference: [text]
            output.push('[');
            output.push_str(text);
            output.push(']');

            self.add_reference(label.to_string(), url.to_string(), title.to_string());
        } else {
            // Full reference: [text][label]
            output.push('[');
            output.push_str(text);
            output.push_str("][");
            output.push_str(label);
            output.push(']');

            self.add_reference(label.to_string(), url.to_string(), title.to_string());
        }
    }

    /// Format an inline-style link and write to output buffer.
    fn format_inline_link(output: &mut String, text: &str, url: &str, title: &str) {
        output.push('[');
        output.push_str(text);
        output.push_str("](");
        output.push_str(url);
        if !title.is_empty() {
            output.push_str(" \"");
            output.push_str(title);
            output.push('"');
        }
        output.push(')');
    }

    /// Format an autolink and write to output buffer.
    fn format_autolink(output: &mut String, url: &str) {
        output.push('<');
        output.push_str(url);
        output.push('>');
    }

    /// Format an external link as reference style and write to output buffer.
    fn format_external_link_as_reference(
        &mut self,
        output: &mut String,
        text: &str,
        url: &str,
        title: &str,
    ) {
        // Normalize: replace SoftBreak markers with spaces for shortcut refs
        let normalized_text = text.replace('\x00', " ");
        output.push('[');
        output.push_str(&normalized_text);
        output.push(']');

        self.add_reference(normalized_text, url.to_string(), title.to_string());
    }

    /// Format a reference-style image and write to output buffer.
    fn format_reference_image(
        &mut self,
        output: &mut String,
        text: &str,
        label: &str,
        url: &str,
        title: &str,
    ) {
        if label.starts_with('\x01') {
            // Collapsed reference: ![alt][]
            let actual_label = label.strip_prefix('\x01').unwrap();
            output.push_str("![");
            output.push_str(text);
            output.push_str("][]");

            self.add_reference(actual_label.to_string(), url.to_string(), title.to_string());
        } else if text == label {
            // Shortcut reference: ![alt]
            output.push_str("![");
            output.push_str(text);
            output.push(']');

            self.add_reference(label.to_string(), url.to_string(), title.to_string());
        } else {
            // Full reference: ![alt][label]
            output.push_str("![");
            output.push_str(text);
            output.push_str("][");
            output.push_str(label);
            output.push(']');

            self.add_reference(label.to_string(), url.to_string(), title.to_string());
        }
    }

    /// Format an inline-style image and write to output buffer.
    fn format_inline_image(output: &mut String, alt_text: &str, url: &str, title: &str) {
        output.push_str("![");
        output.push_str(alt_text);
        output.push_str("](");
        output.push_str(url);
        if !title.is_empty() {
            output.push_str(" \"");
            output.push_str(title);
            output.push('"');
        }
        output.push(')');
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
                self.serialize_list_item(node);
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
                // For skip_next_block, reset the flag after this block
                let was_skip_next_block = self.skip_next_block;
                if was_skip_next_block {
                    self.skip_next_block = false;
                }

                // For skip_until_section, check if this is a heading to reset
                if self.skip_until_section
                    && let NodeValue::Heading(h) = &child.data.borrow().value
                    && h.level <= 2
                {
                    self.skip_until_section = false;
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

        self.flush_references();
    }

    fn serialize_description_details<'b>(&mut self, node: &'b AstNode<'b>) {
        let children: Vec<_> = node.children().collect();

        for (i, child) in children.iter().enumerate() {
            let child_value = &child.data.borrow().value;

            if i == 0 {
                // First child: start with `:   ` marker
                match child_value {
                    NodeValue::Paragraph => {
                        self.output.push_str(":   ");
                        let mut content = String::new();
                        self.collect_inline_content(child, &mut content);
                        let wrapped = wrap::wrap_text_first_line(
                            content.trim(),
                            "",
                            "    ",
                            self.options.line_width,
                        );
                        self.output.push_str(&wrapped);
                        self.output.push('\n');
                    }
                    NodeValue::CodeBlock(code) => {
                        // Code block as first child (unusual but possible)
                        self.output.push_str(":   ");
                        self.output.push('\n');
                        self.output.push_str("    ");
                        self.serialize_code_block_with_indent(code, "    ");
                    }
                    _ => {
                        // Other block types: serialize normally with indent
                        self.output.push_str(":   ");
                        self.serialize_node(child);
                    }
                }
            } else {
                // Subsequent children: need blank line and 4-space indent
                self.output.push('\n');
                match child_value {
                    NodeValue::Paragraph => {
                        self.output.push_str("    ");
                        let mut content = String::new();
                        self.collect_inline_content(child, &mut content);
                        let wrapped = wrap::wrap_text_first_line(
                            content.trim(),
                            "",
                            "    ",
                            self.options.line_width,
                        );
                        self.output.push_str(&wrapped);
                        self.output.push('\n');
                    }
                    NodeValue::CodeBlock(code) => {
                        self.output.push_str("    ");
                        self.serialize_code_block_with_indent(code, "    ");
                    }
                    _ => {
                        // Other block types
                        self.output.push_str("    ");
                        self.serialize_node(child);
                    }
                }
            }
        }
    }

    fn serialize_code_block_with_indent(
        &mut self,
        code: &comrak::nodes::NodeCodeBlock,
        indent: &str,
    ) {
        let fence = if code.literal.contains("~~~~") {
            "~~~~~"
        } else {
            "~~~~"
        };
        self.output.push_str(fence);
        if !code.info.is_empty() {
            self.output.push(' ');
            self.output.push_str(&code.info);
        }
        self.output.push('\n');
        // Add indent to each line of code
        for line in code.literal.lines() {
            self.output.push_str(indent);
            self.output.push_str(line);
            self.output.push('\n');
        }
        // Handle trailing newline in literal
        if !code.literal.ends_with('\n') && !code.literal.is_empty() {
            self.output.push('\n');
        }
        self.output.push_str(indent);
        self.output.push_str(fence);
        self.output.push('\n');
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
            // For badge-style, serialize children first to get image content
            if contains_image {
                // Badge-style with reference: [![alt][img-ref]][link-ref]
                self.output.push('[');
                for child in node.children() {
                    self.serialize_node(child);
                }
                self.output.push_str("][");
                let actual_label = label.strip_prefix('\x01').unwrap_or(&label);
                self.output.push_str(actual_label);
                self.output.push(']');
                self.add_reference(actual_label.to_string(), url.to_string(), title.to_string());
            } else {
                // Use helper for non-badge reference links
                let mut output = String::new();
                self.format_reference_link(&mut output, &text, &label, url, title);
                self.output.push_str(&output);
            }
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
            Self::format_autolink(&mut self.output, url);
        } else if Self::is_external_url(url) {
            let link_text = self.collect_text(node);
            let mut output = String::new();
            self.format_external_link_as_reference(&mut output, &link_text, url, title);
            self.output.push_str(&output);
        } else {
            // Relative/local URL: keep as inline link
            let link_text = self.collect_text(node);
            Self::format_inline_link(&mut self.output, &link_text, url, title);
        }
    }

    fn serialize_image<'b>(&mut self, node: &'b AstNode<'b>, url: &str, title: &str) {
        // Collect the alt text
        let alt_text = self.collect_text(node);

        // Check if original was reference style
        if let Some((text, label)) = self.get_reference_style_info(node) {
            // Use a temporary buffer to avoid double borrow
            let mut output = String::new();
            self.format_reference_image(&mut output, &text, &label, url, title);
            self.output.push_str(&output);
            return;
        }

        // Inline style: ![alt](url)
        Self::format_inline_image(&mut self.output, &alt_text, url, title);
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
                text.push_str(&escape::escape_text(t));
            }
            NodeValue::Code(code) => {
                text.push_str(&escape::format_code_span(&code.literal));
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
            let wrapped = wrap::wrap_text_first_line(
                &inline_content,
                "",
                &continuation,
                self.options.line_width,
            );
            self.output.push_str(&wrapped);
        } else {
            // Wrap the paragraph at line_width
            let wrapped = wrap::wrap_text(&inline_content, prefix, self.options.line_width);
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
        match &node.data.borrow().value {
            NodeValue::Text(text) => {
                content.push_str(&escape::escape_text(text));
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
                content.push_str(&escape::format_code_span(&code.literal));
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
                        let actual_label = label.strip_prefix('\x01').unwrap_or(&label);
                        content.push('[');
                        for child in node.children() {
                            self.collect_inline_node(child, content);
                        }
                        content.push_str("][");
                        content.push_str(actual_label);
                        content.push(']');

                        self.add_reference(
                            actual_label.to_string(),
                            link.url.clone(),
                            link.title.clone(),
                        );
                    } else {
                        // Non-badge reference links: use helper
                        self.format_reference_link(content, &text, &label, &link.url, &link.title);
                    }
                } else if contains_image {
                    // Badge-style inline: [![alt](img-url)](link-url)
                    // Need to iterate children, so can't use helper directly
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
                    Self::format_autolink(content, &link.url);
                } else if Self::is_external_url(&link.url) {
                    // External URL: collect link text first
                    let mut link_text = String::new();
                    for child in node.children() {
                        self.collect_inline_node(child, &mut link_text);
                    }
                    self.format_external_link_as_reference(
                        content,
                        &link_text,
                        &link.url,
                        &link.title,
                    );
                } else {
                    // Relative/local URL: keep as inline link
                    let mut link_text = String::new();
                    for child in node.children() {
                        self.collect_inline_node(child, &mut link_text);
                    }
                    Self::format_inline_link(content, &link_text, &link.url, &link.title);
                }
            }
            NodeValue::Image(image) => {
                // Check if original was reference style
                if let Some((text, label)) = self.get_reference_style_info(node) {
                    self.format_reference_image(content, &text, &label, &image.url, &image.title);
                } else {
                    // Inline style: collect alt text and use inline syntax
                    let mut alt_text = String::new();
                    for child in node.children() {
                        self.collect_inline_node(child, &mut alt_text);
                    }
                    Self::format_inline_image(content, &alt_text, &image.url, &image.title);
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

        // Opening fence
        if self.in_block_quote {
            self.output.push_str("> ");
        }
        self.output.push_str(&fence);
        self.output.push(' ');
        self.output.push_str(language);
        self.output.push('\n');

        // Content lines
        for line in literal.lines() {
            if self.in_block_quote {
                self.output.push_str("> ");
            }
            self.output.push_str(line);
            self.output.push('\n');
        }

        // Closing fence
        if self.in_block_quote {
            self.output.push_str("> ");
        }
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

    fn serialize_children<'b>(&mut self, node: &'b AstNode<'b>) {
        for child in node.children() {
            self.serialize_node(child);
        }
    }
}

#[cfg(test)]
mod tests;
