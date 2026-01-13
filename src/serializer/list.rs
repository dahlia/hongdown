//! List serialization logic.

use comrak::nodes::{AstNode, ListType, NodeValue};

use super::Serializer;

impl<'a> Serializer<'a> {
    /// Count the number of items in a list node.
    fn count_list_items<'b>(node: &'b AstNode<'b>) -> usize {
        node.children()
            .filter(|child| matches!(child.data.borrow().value, NodeValue::Item(_)))
            .count()
    }

    /// Calculate the width of a list item marker.
    /// This is used to determine the indentation for continuation lines.
    fn calculate_marker_width(&self) -> usize {
        match self.list_type {
            Some(ListType::Bullet) => {
                // " -  " = leading_spaces + 1 (marker) + trailing_spaces
                self.options.leading_spaces + 1 + self.options.trailing_spaces
            }
            Some(ListType::Ordered) => {
                // Fixed width based on ordered_list_indent_width (default 4)
                // e.g., "1.  " (4), "10. " (4), "100." (4, min 1 trailing space)
                self.options.ordered_list_indent_width
            }
            None => 0,
        }
    }

    pub(super) fn serialize_list<'b>(
        &mut self,
        node: &'b AstNode<'b>,
        list_type: ListType,
        tight: bool,
    ) {
        let old_list_type = self.list_type;
        let old_list_tight = self.list_tight;
        let old_index = self.list_item_index;
        let old_max_items = self.ordered_list_max_items;

        self.list_type = Some(list_type);
        self.list_tight = tight;
        self.list_item_index = 0;
        self.list_depth += 1;

        // For ordered lists, count items to determine padding width
        if matches!(list_type, ListType::Ordered) {
            self.ordered_list_max_items = Self::count_list_items(node);
        }

        self.serialize_children(node);

        self.list_depth -= 1;
        self.list_type = old_list_type;
        self.list_tight = old_list_tight;
        self.list_item_index = old_index;
        self.ordered_list_max_items = old_max_items;
    }

    /// Serialize a list item, optionally with a task list checkbox.
    ///
    /// # Arguments
    ///
    /// * `node` - The list item node.
    /// * `task_marker` - For task list items: `Some(Some('x'))` for checked,
    ///   `Some(None)` for unchecked, `None` for regular list items.
    pub(super) fn serialize_list_item<'b>(
        &mut self,
        node: &'b AstNode<'b>,
        task_marker: Option<Option<char>>,
    ) {
        self.list_item_index += 1;

        // For loose lists, add a blank line before items (except the first)
        if !self.list_tight && self.list_item_index > 1 {
            self.output.push('\n');
        }

        // Add block quote prefix if we're inside a block quote
        if self.in_block_quote {
            self.output.push_str("> ");
        }

        // Check if this is the first item of a list that starts on the same line as `:` in
        // definition details. In that case, skip base indentation for the first item only.
        let is_first_item_on_colon_line = self.description_details_first_list
            && self.list_item_index == 1
            && self.list_depth == 1;

        // Add extra indentation if inside a description details block
        // (lists inside definition list details need 5-space base indent: "     ")
        // Skip this for the first item when it's on the same line as the colon
        let desc_base_indent = if self.in_description_details && !is_first_item_on_colon_line {
            "     "
        } else {
            ""
        };

        // Calculate indentation for nested lists
        // Level 1: " -  " (leading space + hyphen + trailing spaces)
        // Level 2+: indent_width spaces per nesting level, then " -  " prefix
        // Use different indent_width for ordered vs unordered lists
        let indent_width = match self.list_type {
            Some(ListType::Ordered) => self.options.ordered_list_indent_width,
            _ => self.options.indent_width,
        };
        if self.list_depth > 1 {
            let indent = format!(
                "{}{}",
                desc_base_indent,
                " ".repeat(indent_width * (self.list_depth - 1))
            );
            self.output.push_str(&indent);
        } else {
            self.output.push_str(desc_base_indent);
        }

        match self.list_type {
            Some(ListType::Bullet) => {
                let marker = self.options.unordered_marker;
                let leading = " ".repeat(self.options.leading_spaces);
                let trailing = " ".repeat(self.options.trailing_spaces);
                if self.in_description_details && self.list_depth == 1 {
                    // Inside description details at top level: "-  " (no leading space)
                    self.output.push(marker);
                    self.output.push_str(&trailing);
                } else {
                    // All other cases: " -  " (leading spaces + marker + trailing spaces)
                    self.output.push_str(&leading);
                    self.output.push(marker);
                    self.output.push_str(&trailing);
                }
            }
            Some(ListType::Ordered) => {
                // Determine marker based on nesting level (odd=1,3,5..., even=2,4,6...)
                let marker = if self.list_depth % 2 == 1 {
                    self.options.odd_level_marker
                } else {
                    self.options.even_level_marker
                };

                let current_num = self.list_item_index.to_string();
                let current_num_width = current_num.len();

                // Calculate trailing spaces to maintain fixed marker width
                // marker_width = number + marker_char + trailing
                // trailing = marker_width - number - 1 (minimum 1)
                let marker_width = self.options.ordered_list_indent_width;
                let trailing_count = marker_width.saturating_sub(current_num_width + 1).max(1);

                self.output.push_str(&current_num);
                self.output.push(marker);
                self.output.push_str(&" ".repeat(trailing_count));
            }
            None => {}
        }

        // Add task list checkbox if this is a task item
        if let Some(checked) = task_marker {
            if checked.is_some() {
                self.output.push_str("[x] ");
            } else {
                self.output.push_str("[ ] ");
            }
        }

        // Serialize children, handling nested lists and multiple paragraphs
        let children: Vec<_> = node.children().collect();
        // Calculate base indentation for continuation lines (paragraphs, code blocks, etc.)
        // This should match the marker width so content aligns properly
        let marker_width = self.calculate_marker_width();
        // Inside description details at top-level, the marker has no leading space,
        // so we need to use marker_width without leading_spaces for base_indent calculation.
        let marker_width_for_indent = if self.in_description_details && self.list_depth == 1 {
            match self.list_type {
                Some(ListType::Bullet) => {
                    // "-  " = 1 (marker) + trailing_spaces (no leading space)
                    1 + self.options.trailing_spaces
                }
                Some(ListType::Ordered) => {
                    // For ordered lists in description details, still use full width
                    self.options.ordered_list_indent_width
                }
                None => 0,
            }
        } else {
            marker_width
        };
        let base_indent = if self.in_description_details {
            // Inside description details, add extra 5-space indent
            format!(
                "{}{}",
                " ".repeat(5 + indent_width * (self.list_depth - 1)),
                " ".repeat(marker_width_for_indent)
            )
        } else if self.list_depth > 1 {
            // Nested list: outer indent + marker width
            format!(
                "{}{}",
                " ".repeat(indent_width * (self.list_depth - 1)),
                " ".repeat(marker_width)
            )
        } else {
            // Top-level list: just marker width
            " ".repeat(marker_width)
        };

        // Store the base indent for use by nested block elements (blockquotes, alerts, etc.)
        let old_list_item_indent =
            std::mem::replace(&mut self.list_item_indent, base_indent.clone());

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
                        // Check if previous child ends with a newline (code blocks, nested lists)
                        let prev_ends_with_newline = i > 0
                            && matches!(
                                &children[i - 1].data.borrow().value,
                                NodeValue::CodeBlock(_) | NodeValue::List(_)
                            );
                        if prev_ends_with_newline {
                            // Previous element already ends with \n, so just add one more \n
                            self.output.push('\n');
                        } else {
                            // First \n ends the previous paragraph, second \n creates blank line
                            self.output.push_str("\n\n");
                        }
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
                NodeValue::BlockQuote | NodeValue::Alert(_) => {
                    // Block quotes and alerts in list items need blank line
                    // The indentation is handled by the blockquote/alert serialization itself
                    if !is_first {
                        self.output.push_str("\n\n");
                    } else {
                        self.output.push('\n');
                    }
                    if self.in_block_quote {
                        self.output.push_str("> ");
                    }
                    self.serialize_node(child);
                }
                _ => {
                    self.serialize_node(child);
                }
            }
        }

        // Restore the old list item indent
        self.list_item_indent = old_list_item_indent;

        // Only add newline if the last child doesn't already end with one
        // (nested lists, code blocks, and blockquotes add their own newlines)
        let last_child = node.children().last();
        let last_child_ends_with_newline = last_child.is_some_and(|child| {
            matches!(
                &child.data.borrow().value,
                NodeValue::List(_)
                    | NodeValue::CodeBlock(_)
                    | NodeValue::BlockQuote
                    | NodeValue::Alert(_)
            )
        });
        if !last_child_ends_with_newline {
            self.output.push('\n');
        }
    }
}
