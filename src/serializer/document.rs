//! Document-level serialization logic.

use comrak::nodes::{AstNode, NodeValue};

use super::Serializer;
use super::state::Directive;
use super::wrap;

impl<'a> Serializer<'a> {
    pub(super) fn serialize_document<'b>(&mut self, node: &'b AstNode<'b>) {
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

    pub(super) fn serialize_description_details<'b>(&mut self, node: &'b AstNode<'b>) {
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

    pub(super) fn serialize_heading<'b>(&mut self, node: &'b AstNode<'b>, level: u8) {
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

    pub(super) fn serialize_front_matter(&mut self, content: &str) {
        // Front matter content from comrak includes the delimiters,
        // so we preserve it verbatim and add a trailing blank line
        self.output.push_str(content.trim());
        self.output.push_str("\n\n");
    }
}
