//! Block quote and alert serialization logic.

use comrak::nodes::{AlertType, AstNode};

use super::Serializer;

impl<'a> Serializer<'a> {
    pub(super) fn serialize_block_quote<'b>(&mut self, node: &'b AstNode<'b>) {
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

    pub(super) fn serialize_alert<'b>(&mut self, node: &'b AstNode<'b>, alert_type: AlertType) {
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
            self.output.push_str(">\n");
        }

        // Output the alert content with > prefix
        // Use in_block_quote to handle nested content properly
        let was_in_block_quote = self.in_block_quote;
        self.in_block_quote = true;

        for (i, child) in children.iter().enumerate() {
            if i > 0 {
                self.output.push_str(">\n");
            }
            self.serialize_node(child);
        }

        self.in_block_quote = was_in_block_quote;
    }
}
