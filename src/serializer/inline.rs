//! Inline node collection and text extraction logic.

use comrak::nodes::{AstNode, NodeValue};

use super::Serializer;
use super::escape;

impl<'a> Serializer<'a> {
    pub(super) fn collect_text<'b>(&mut self, node: &'b AstNode<'b>) -> String {
        let mut text = String::new();
        self.collect_text_recursive(node, &mut text);
        text
    }

    /// Collect raw text without escaping (for comparison purposes)
    pub(super) fn collect_raw_text<'b>(&self, node: &'b AstNode<'b>) -> String {
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

    fn collect_text_recursive<'b>(&mut self, node: &'b AstNode<'b>, text: &mut String) {
        match &node.data.borrow().value {
            NodeValue::Text(t) => {
                text.push_str(&escape::escape_text(t));
            }
            NodeValue::Code(code) => {
                text.push_str(&escape::format_code_span(&code.literal));
            }
            NodeValue::Emph => {
                text.push('*');
                for child in node.children() {
                    self.collect_text_recursive(child, text);
                }
                text.push('*');
            }
            NodeValue::Strong => {
                text.push_str("**");
                for child in node.children() {
                    self.collect_text_recursive(child, text);
                }
                text.push_str("**");
            }
            NodeValue::SoftBreak => {
                text.push(' ');
            }
            NodeValue::Link(link) => {
                // Handle reference-style links in headings
                if let Some((link_text, label)) = self.get_reference_style_info(node) {
                    self.format_reference_link(text, &link_text, &label, &link.url, &link.title);
                } else {
                    // For inline links, just output plain text (or format as inline?)
                    // In headings, we typically want reference style for external links
                    let link_text = self.collect_raw_text(node);
                    if Self::is_external_url(&link.url) {
                        self.format_external_link_as_reference(
                            text,
                            &link_text,
                            &link.url,
                            &link.title,
                        );
                    } else {
                        Self::format_inline_link(text, &link_text, &link.url, &link.title);
                    }
                }
            }
            _ => {
                for child in node.children() {
                    self.collect_text_recursive(child, text);
                }
            }
        }
    }

    pub(super) fn collect_inline_content<'b>(
        &mut self,
        node: &'b AstNode<'b>,
        content: &mut String,
    ) {
        for child in node.children() {
            self.collect_inline_node(child, content);
        }
    }

    pub(super) fn collect_inline_node<'b>(&mut self, node: &'b AstNode<'b>, content: &mut String) {
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
}
