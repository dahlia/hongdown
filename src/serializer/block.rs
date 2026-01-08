//! Block quote and alert serialization logic.

use comrak::nodes::{AlertType, AstNode};

use super::Serializer;

impl<'a> Serializer<'a> {
    pub(super) fn serialize_block_quote<'b>(&mut self, node: &'b AstNode<'b>) {
        let was_in_block_quote = self.in_block_quote;
        self.in_block_quote = true;

        // Save the current list_item_indent as the blockquote's outer indent
        // This is used for list-inside-blockquote to properly prefix continuation lines
        let old_blockquote_outer_indent = std::mem::replace(
            &mut self.blockquote_outer_indent,
            self.list_item_indent.clone(),
        );

        // Save the list depth when entering this blockquote
        let old_blockquote_entry_list_depth =
            std::mem::replace(&mut self.blockquote_entry_list_depth, self.list_depth);

        // Get the indentation prefix for list items (if inside a list)
        let indent = self.list_item_indent.clone();

        // Clear list context for children - the blockquote starts fresh
        // Lists inside the blockquote will set their own context
        let old_list_item_indent = std::mem::take(&mut self.list_item_indent);
        let old_list_type = self.list_type.take();
        let old_list_depth = std::mem::replace(&mut self.list_depth, 0);

        let children: Vec<_> = node.children().collect();
        for (i, child) in children.iter().enumerate() {
            // Add blank quote line between paragraphs
            if i > 0 {
                self.output.push_str(&indent);
                self.output.push_str(">\n");
            }
            self.serialize_node(child);
        }

        self.list_depth = old_list_depth;
        self.list_type = old_list_type;
        self.list_item_indent = old_list_item_indent;
        self.blockquote_outer_indent = old_blockquote_outer_indent;
        self.blockquote_entry_list_depth = old_blockquote_entry_list_depth;
        self.in_block_quote = was_in_block_quote;
    }

    pub(super) fn serialize_alert<'b>(&mut self, node: &'b AstNode<'b>, alert_type: AlertType) {
        // Get the indentation prefix for list items (if inside a list)
        let indent = self.list_item_indent.clone();

        // Output the alert header with list item indent
        let type_str = match alert_type {
            AlertType::Note => "NOTE",
            AlertType::Tip => "TIP",
            AlertType::Important => "IMPORTANT",
            AlertType::Warning => "WARNING",
            AlertType::Caution => "CAUTION",
        };
        self.output.push_str(&indent);
        self.output.push_str("> [!");
        self.output.push_str(type_str);
        self.output.push_str("]\n");

        // Check if original source has a blank line after the alert header
        // by examining the sourcepos of the first child
        let children: Vec<_> = node.children().collect();
        let has_blank_after_header = if let Some(first_child) = children.first() {
            let alert_start = node.data.borrow().sourcepos.start.line;
            let first_child_start = first_child.data.borrow().sourcepos.start.line;
            // If there's more than 1 line gap, there's a blank line
            first_child_start > alert_start + 1
        } else {
            false
        };

        if has_blank_after_header {
            self.output.push_str(&indent);
            self.output.push_str(">\n");
        }

        // Output the alert content with > prefix
        // Use in_block_quote to handle nested content properly
        let was_in_block_quote = self.in_block_quote;
        self.in_block_quote = true;

        // Save the current list_item_indent as the blockquote's outer indent
        let old_blockquote_outer_indent = std::mem::replace(
            &mut self.blockquote_outer_indent,
            self.list_item_indent.clone(),
        );

        // Save the list depth when entering this blockquote
        let old_blockquote_entry_list_depth =
            std::mem::replace(&mut self.blockquote_entry_list_depth, self.list_depth);

        // Clear list context for children - the blockquote starts fresh
        // Lists inside the blockquote will set their own context
        let old_list_item_indent = std::mem::take(&mut self.list_item_indent);
        let old_list_type = self.list_type.take();
        let old_list_depth = std::mem::replace(&mut self.list_depth, 0);

        for (i, child) in children.iter().enumerate() {
            if i > 0 {
                self.output.push_str(&indent);
                self.output.push_str(">\n");
            }
            self.serialize_node(child);
        }

        self.list_depth = old_list_depth;
        self.list_type = old_list_type;
        self.list_item_indent = old_list_item_indent;
        self.blockquote_outer_indent = old_blockquote_outer_indent;
        self.blockquote_entry_list_depth = old_blockquote_entry_list_depth;
        self.in_block_quote = was_in_block_quote;
    }
}
