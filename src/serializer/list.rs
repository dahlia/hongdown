//! List serialization logic.

use comrak::nodes::{AstNode, ListType, NodeValue};

use super::Serializer;

impl<'a> Serializer<'a> {
    pub(super) fn serialize_list<'b>(
        &mut self,
        node: &'b AstNode<'b>,
        list_type: ListType,
        tight: bool,
    ) {
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

    pub(super) fn serialize_list_item<'b>(&mut self, node: &'b AstNode<'b>) {
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
}
