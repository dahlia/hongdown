//! Block quote and alert serialization logic.

use comrak::nodes::{AlertType, AstNode};

use super::Serializer;

/// Saved state for blockquote context, used for restoration after processing.
struct BlockquoteState {
    was_in_block_quote: bool,
    old_blockquote_prefix: String,
    old_blockquote_outer_indent: String,
    old_blockquote_entry_list_depth: usize,
    old_list_item_indent: String,
    old_list_type: Option<comrak::nodes::ListType>,
    old_list_depth: usize,
    /// The list item indent before entering blockquote context (for separator lines).
    indent: String,
}

impl<'a> Serializer<'a> {
    /// Enter blockquote context: save current state and set up for blockquote processing.
    fn enter_blockquote_context(&mut self) -> BlockquoteState {
        let was_in_block_quote = self.in_block_quote;
        self.in_block_quote = true;

        let old_blockquote_prefix = self.blockquote_prefix.clone();
        self.blockquote_prefix.push_str("> ");

        let old_blockquote_outer_indent = std::mem::replace(
            &mut self.blockquote_outer_indent,
            self.list_item_indent.clone(),
        );

        let old_blockquote_entry_list_depth =
            std::mem::replace(&mut self.blockquote_entry_list_depth, self.list_depth);

        let indent = self.list_item_indent.clone();

        let old_list_item_indent = std::mem::take(&mut self.list_item_indent);
        let old_list_type = self.list_type.take();
        let old_list_depth = std::mem::replace(&mut self.list_depth, 0);

        BlockquoteState {
            was_in_block_quote,
            old_blockquote_prefix,
            old_blockquote_outer_indent,
            old_blockquote_entry_list_depth,
            old_list_item_indent,
            old_list_type,
            old_list_depth,
            indent,
        }
    }

    /// Exit blockquote context: restore the saved state.
    fn exit_blockquote_context(&mut self, state: BlockquoteState) {
        self.list_depth = state.old_list_depth;
        self.list_type = state.old_list_type;
        self.list_item_indent = state.old_list_item_indent;
        self.blockquote_outer_indent = state.old_blockquote_outer_indent;
        self.blockquote_entry_list_depth = state.old_blockquote_entry_list_depth;
        self.blockquote_prefix = state.old_blockquote_prefix;
        self.in_block_quote = state.was_in_block_quote;
    }

    /// Serialize children within blockquote context, adding blank quote lines between them.
    fn serialize_blockquote_children<'b>(
        &mut self,
        children: &[&'b AstNode<'b>],
        state: &BlockquoteState,
    ) {
        for (i, child) in children.iter().enumerate() {
            if i > 0 {
                self.output.push_str(&state.indent);
                self.output.push_str(&state.old_blockquote_prefix);
                self.output.push_str(">\n");
            }
            self.serialize_node(child);
        }
    }

    pub(super) fn serialize_block_quote<'b>(&mut self, node: &'b AstNode<'b>) {
        let state = self.enter_blockquote_context();
        let children: Vec<_> = node.children().collect();
        self.serialize_blockquote_children(&children, &state);
        self.exit_blockquote_context(state);
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
        self.output.push_str(&self.list_item_indent);
        self.output.push_str(&self.blockquote_prefix);
        self.output.push_str("> [!");
        self.output.push_str(type_str);
        self.output.push_str("]\n");

        // Check if original source has a blank line after the alert header
        let children: Vec<_> = node.children().collect();
        let has_blank_after_header = if let Some(first_child) = children.first() {
            let alert_start = node.data.borrow().sourcepos.start.line;
            let first_child_start = first_child.data.borrow().sourcepos.start.line;
            first_child_start > alert_start + 1
        } else {
            false
        };

        if has_blank_after_header {
            self.output.push_str(&self.list_item_indent);
            self.output.push_str(&self.blockquote_prefix);
            self.output.push_str(">\n");
        }

        let state = self.enter_blockquote_context();
        self.serialize_blockquote_children(&children, &state);
        self.exit_blockquote_context(state);
    }
}
