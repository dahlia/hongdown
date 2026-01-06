//! Serializer for converting comrak AST to formatted Markdown.

use std::collections::HashMap;

use comrak::nodes::{AlertType, AstNode, ListType, NodeTable, NodeValue, TableAlignment};

use crate::Options;

/// A reference link definition: label -> (url, title)
#[derive(Debug, Clone)]
struct ReferenceLink {
    label: String,
    url: String,
    title: String,
}

/// Serializes a comrak AST node to a formatted Markdown string.
pub fn serialize<'a>(node: &'a AstNode<'a>, options: &Options) -> String {
    let mut serializer = Serializer::new(options);
    serializer.serialize_node(node);
    serializer.output
}

struct Serializer<'a> {
    output: String,
    options: &'a Options,
    /// Current list item index (1-based) for ordered lists
    list_item_index: usize,
    /// Current list type
    list_type: Option<ListType>,
    /// Whether we're inside a block quote
    in_block_quote: bool,
    /// Reference links collected for the current section
    /// Key: URL, Value: ReferenceLink
    pending_references: HashMap<String, ReferenceLink>,
    /// Current list nesting depth (0 = not in list, 1 = top-level, 2+ = nested)
    list_depth: usize,
}

impl<'a> Serializer<'a> {
    fn new(options: &'a Options) -> Self {
        Self {
            output: String::new(),
            options,
            list_item_index: 0,
            list_type: None,
            in_block_quote: false,
            pending_references: HashMap::new(),
            list_depth: 0,
        }
    }

    /// Check if a URL is external (http:// or https://)
    fn is_external_url(url: &str) -> bool {
        url.starts_with("http://") || url.starts_with("https://")
    }

    /// Output pending reference definitions and clear them
    fn flush_references(&mut self) {
        if self.pending_references.is_empty() {
            return;
        }

        // Sort references by label for consistent output
        let mut refs: Vec<_> = self.pending_references.values().collect();
        refs.sort_by(|a, b| a.label.to_lowercase().cmp(&b.label.to_lowercase()));

        // Add a blank line before references if not already present
        if !self.output.ends_with("\n\n") {
            if self.output.ends_with('\n') {
                self.output.push('\n');
            } else {
                self.output.push_str("\n\n");
            }
        }

        for reference in refs {
            self.output.push('[');
            self.output.push_str(&reference.label);
            self.output.push_str("]: ");
            self.output.push_str(&reference.url);
            if !reference.title.is_empty() {
                self.output.push_str(" \"");
                self.output.push_str(&reference.title);
                self.output.push('"');
            }
            self.output.push('\n');
        }

        self.pending_references.clear();
    }

    fn serialize_node<'b>(&mut self, node: &'b AstNode<'b>) {
        match &node.data.borrow().value {
            NodeValue::Document => {
                self.serialize_document(node);
            }
            NodeValue::Heading(heading) => {
                self.serialize_heading(node, heading.level);
            }
            NodeValue::List(list) => {
                self.serialize_list(node, list.list_type);
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
                self.serialize_children(node);
            }
            NodeValue::DescriptionItem(_) => {
                self.serialize_children(node);
            }
            NodeValue::DescriptionTerm => {
                self.serialize_children(node);
                self.output.push('\n');
            }
            NodeValue::DescriptionDetails => {
                self.output.push_str(":   ");
                // Collect inline content for the definition
                let mut content = String::new();
                for child in node.children() {
                    self.collect_inline_node(child, &mut content);
                }
                self.output.push_str(content.trim());
                self.output.push('\n');
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
                self.output.push_str(text);
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
                self.output.push('`');
                self.output.push_str(&code.literal);
                self.output.push('`');
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
                if !prev_is_front_matter {
                    self.output.push('\n');
                    if is_h2 {
                        self.output.push('\n');
                    }
                }
            }
            self.serialize_node(child);
        }
        // Flush any remaining references at the end of the document
        self.flush_references();
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
        // Collect the link text
        let link_text = self.collect_text(node);

        if Self::is_external_url(url) {
            // External URL: use reference link style
            // Use link text as label (shortcut reference style)
            let label = link_text.clone();

            // Output the reference: [text] or [text][label]
            self.output.push('[');
            self.output.push_str(&link_text);
            self.output.push(']');

            // Store the reference definition for later output
            self.pending_references.insert(
                url.to_string(),
                ReferenceLink {
                    label,
                    url: url.to_string(),
                    title: title.to_string(),
                },
            );
        } else {
            // Relative/local URL: keep as inline link
            self.output.push('[');
            self.output.push_str(&link_text);
            self.output.push_str("](");
            self.output.push_str(url);
            if !title.is_empty() {
                self.output.push_str(" \"");
                self.output.push_str(title);
                self.output.push('"');
            }
            self.output.push(')');
        }
    }

    fn serialize_image<'b>(&mut self, node: &'b AstNode<'b>, url: &str, title: &str) {
        // Collect the alt text
        let alt_text = self.collect_text(node);

        // Images always use inline syntax (no reference style for images)
        self.output.push_str("![");
        self.output.push_str(&alt_text);
        self.output.push_str("](");
        self.output.push_str(url);
        if !title.is_empty() {
            self.output.push_str(" \"");
            self.output.push_str(title);
            self.output.push('"');
        }
        self.output.push(')');
    }

    fn collect_text<'b>(&self, node: &'b AstNode<'b>) -> String {
        let mut text = String::new();
        self.collect_text_recursive(node, &mut text);
        text
    }

    fn collect_text_recursive<'b>(&self, node: &'b AstNode<'b>, text: &mut String) {
        match &node.data.borrow().value {
            NodeValue::Text(t) => {
                text.push_str(t);
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
            // Continuation lines need 4-space indent (plus block quote prefix if applicable)
            let continuation = if self.in_block_quote {
                ">     " // > + 4 spaces to align with list item content
            } else {
                "    " // 4 spaces to align with list item content
            };
            let wrapped = self.wrap_text_first_line(&inline_content, "", continuation);
            self.output.push_str(&wrapped);
        } else {
            // Wrap the paragraph at line_width
            let wrapped = self.wrap_text(&inline_content, prefix);
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
        match &node.data.borrow().value.clone() {
            NodeValue::Text(text) => {
                content.push_str(text);
            }
            NodeValue::SoftBreak => {
                content.push(' ');
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
                content.push('`');
                content.push_str(&code.literal);
                content.push('`');
            }
            NodeValue::Link(link) => {
                // Collect link text first
                let mut link_text = String::new();
                for child in node.children() {
                    self.collect_inline_node(child, &mut link_text);
                }

                if Self::is_external_url(&link.url) {
                    // External URL: use reference link style
                    content.push('[');
                    content.push_str(&link_text);
                    content.push(']');

                    // Store the reference definition
                    self.pending_references.insert(
                        link.url.clone(),
                        ReferenceLink {
                            label: link_text,
                            url: link.url.clone(),
                            title: link.title.clone(),
                        },
                    );
                } else {
                    // Relative/local URL: keep as inline link
                    content.push('[');
                    content.push_str(&link_text);
                    content.push_str("](");
                    content.push_str(&link.url);
                    if !link.title.is_empty() {
                        content.push_str(" \"");
                        content.push_str(&link.title);
                        content.push('"');
                    }
                    content.push(')');
                }
            }
            NodeValue::Image(image) => {
                // Collect alt text
                let mut alt_text = String::new();
                for child in node.children() {
                    self.collect_inline_node(child, &mut alt_text);
                }

                // Images always use inline syntax
                content.push_str("![");
                content.push_str(&alt_text);
                content.push_str("](");
                content.push_str(&image.url);
                if !image.title.is_empty() {
                    content.push_str(" \"");
                    content.push_str(&image.title);
                    content.push('"');
                }
                content.push(')');
            }
            NodeValue::HtmlInline(html) => {
                // Preserve inline HTML as-is
                content.push_str(html);
            }
            _ => {
                for child in node.children() {
                    self.collect_inline_node(child, content);
                }
            }
        }
    }

    fn wrap_text(&self, text: &str, prefix: &str) -> String {
        let line_width = self.options.line_width;
        let mut result = String::new();
        let mut current_line = String::new();
        let prefix_len = prefix.len();

        // Add prefix to first line
        current_line.push_str(prefix);

        // Split into "tokens" where each token is either:
        // - A word (non-space characters) followed by optional spaces
        // - Content inside backticks (treated as a single unbreakable unit)
        // We preserve double spaces after periods.
        let chars = text.chars();
        let mut current_token = String::new();
        let mut trailing_spaces = String::new();
        let mut in_backticks = false;

        for ch in chars {
            if ch == '`' {
                if in_backticks {
                    // End of backtick region
                    current_token.push(ch);
                    in_backticks = false;
                } else {
                    // Start of backtick region - include any accumulated content first
                    if !current_token.is_empty() && !trailing_spaces.is_empty() {
                        // We have a previous word, output it
                        Self::add_token_to_line(
                            &mut result,
                            &mut current_line,
                            &current_token,
                            &trailing_spaces,
                            prefix,
                            prefix_len,
                            line_width,
                        );
                        current_token.clear();
                        trailing_spaces.clear();
                    }
                    current_token.push(ch);
                    in_backticks = true;
                }
            } else if in_backticks {
                // Inside backticks, everything is part of the token
                current_token.push(ch);
            } else if ch == ' ' {
                trailing_spaces.push(ch);
            } else {
                // Regular character outside backticks
                if !current_token.is_empty() && !trailing_spaces.is_empty() {
                    // We have a previous word with trailing spaces, output it
                    Self::add_token_to_line(
                        &mut result,
                        &mut current_line,
                        &current_token,
                        &trailing_spaces,
                        prefix,
                        prefix_len,
                        line_width,
                    );
                    current_token.clear();
                    trailing_spaces.clear();
                }
                current_token.push(ch);
            }
        }

        // Handle the last token
        if !current_token.is_empty() {
            Self::add_token_to_line(
                &mut result,
                &mut current_line,
                &current_token,
                "",
                prefix,
                prefix_len,
                line_width,
            );
        }

        // Add the last line (trim trailing spaces)
        let final_line = current_line.trim_end();
        if !final_line.is_empty() && final_line != prefix {
            result.push_str(final_line);
        }

        result
    }

    fn add_token_to_line(
        result: &mut String,
        current_line: &mut String,
        token: &str,
        trailing_spaces: &str,
        prefix: &str,
        prefix_len: usize,
        line_width: usize,
    ) {
        let token_len = token.len();
        let spaces_len = trailing_spaces.len();

        if current_line.len() == prefix_len {
            // First word on this line
            current_line.push_str(token);
            current_line.push_str(trailing_spaces);
        } else if current_line.len() + token_len + spaces_len <= line_width {
            // Token fits on current line
            current_line.push_str(token);
            current_line.push_str(trailing_spaces);
        } else {
            // Start a new line - trim trailing spaces from previous line
            let trimmed = current_line.trim_end();
            result.push_str(trimmed);
            result.push('\n');
            *current_line = String::from(prefix);
            current_line.push_str(token);
            current_line.push_str(trailing_spaces);
        }
    }

    /// Wrap text where the first line has a different prefix than continuation lines.
    /// This is used for list items where the marker is already output and continuation
    /// lines need indentation.
    fn wrap_text_first_line(
        &self,
        text: &str,
        first_prefix: &str,
        continuation_prefix: &str,
    ) -> String {
        let line_width = self.options.line_width;
        let mut result = String::new();
        let mut current_line = String::new();
        let mut is_first_line = true;

        // First line starts empty (marker already output)
        current_line.push_str(first_prefix);
        let first_prefix_len = first_prefix.len();
        let cont_prefix_len = continuation_prefix.len();

        let chars = text.chars();
        let mut current_token = String::new();
        let mut trailing_spaces = String::new();
        let mut in_backticks = false;

        // Helper closure to add a token to the current line
        let add_token = |result: &mut String,
                         current_line: &mut String,
                         token: &str,
                         trailing_spaces: &str,
                         is_first_line: &mut bool| {
            let current_prefix_len = if *is_first_line {
                first_prefix_len
            } else {
                cont_prefix_len
            };
            let token_len = token.len();
            let spaces_len = trailing_spaces.len();

            // Check if token fits on current line (either first word or within width)
            if current_line.len() == current_prefix_len
                || current_line.len() + token_len + spaces_len <= line_width
            {
                current_line.push_str(token);
                current_line.push_str(trailing_spaces);
            } else {
                let trimmed = current_line.trim_end();
                result.push_str(trimmed);
                result.push('\n');
                *current_line = String::from(continuation_prefix);
                current_line.push_str(token);
                current_line.push_str(trailing_spaces);
                *is_first_line = false;
            }
        };

        for ch in chars {
            if ch == '`' {
                if in_backticks {
                    current_token.push(ch);
                    in_backticks = false;
                } else {
                    if !current_token.is_empty() && !trailing_spaces.is_empty() {
                        add_token(
                            &mut result,
                            &mut current_line,
                            &current_token,
                            &trailing_spaces,
                            &mut is_first_line,
                        );
                        current_token.clear();
                        trailing_spaces.clear();
                    }
                    current_token.push(ch);
                    in_backticks = true;
                }
            } else if in_backticks {
                current_token.push(ch);
            } else if ch == ' ' {
                trailing_spaces.push(ch);
            } else {
                if !current_token.is_empty() && !trailing_spaces.is_empty() {
                    add_token(
                        &mut result,
                        &mut current_line,
                        &current_token,
                        &trailing_spaces,
                        &mut is_first_line,
                    );
                    current_token.clear();
                    trailing_spaces.clear();
                }
                current_token.push(ch);
            }
        }

        // Handle the last token
        if !current_token.is_empty() {
            add_token(
                &mut result,
                &mut current_line,
                &current_token,
                "",
                &mut is_first_line,
            );
        }

        // Add the last line (trim trailing spaces)
        let final_line = current_line.trim_end();
        let expected_prefix = if is_first_line {
            first_prefix
        } else {
            continuation_prefix
        };
        if !final_line.is_empty() && final_line != expected_prefix {
            result.push_str(final_line);
        }

        result
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

    fn serialize_table<'b>(&mut self, node: &'b AstNode<'b>, table: &NodeTable) {
        let alignments = &table.alignments;
        // Collect all rows and cells first to calculate column widths
        let rows: Vec<_> = node.children().collect();
        if rows.is_empty() {
            return;
        }

        // Collect cell contents and calculate max widths
        let mut all_cells: Vec<Vec<String>> = Vec::new();
        let mut col_widths: Vec<usize> = vec![0; alignments.len()];

        for row in &rows {
            let mut row_cells: Vec<String> = Vec::new();
            for (i, cell) in row.children().enumerate() {
                let content = self.collect_text(cell);
                if i < col_widths.len() {
                    col_widths[i] = col_widths[i].max(content.len());
                }
                row_cells.push(content);
            }
            all_cells.push(row_cells);
        }

        // Ensure minimum column width for alignment markers
        for width in &mut col_widths {
            *width = (*width).max(3);
        }

        // Output header row
        if let Some(header_cells) = all_cells.first() {
            self.output.push('|');
            for (i, cell) in header_cells.iter().enumerate() {
                self.output.push(' ');
                let width = col_widths.get(i).copied().unwrap_or(3);
                self.output
                    .push_str(&format!("{:width$}", cell, width = width));
                self.output.push_str(" |");
            }
            self.output.push('\n');
        }

        // Output separator row with alignment
        self.output.push('|');
        for (i, alignment) in alignments.iter().enumerate() {
            self.output.push(' ');
            let width = col_widths.get(i).copied().unwrap_or(3);
            match alignment {
                TableAlignment::Left => {
                    self.output.push(':');
                    self.output.push_str(&"-".repeat(width - 1));
                }
                TableAlignment::Right => {
                    self.output.push_str(&"-".repeat(width - 1));
                    self.output.push(':');
                }
                TableAlignment::Center => {
                    self.output.push(':');
                    self.output.push_str(&"-".repeat(width - 2));
                    self.output.push(':');
                }
                TableAlignment::None => {
                    self.output.push_str(&"-".repeat(width));
                }
            }
            self.output.push_str(" |");
        }
        self.output.push('\n');

        // Output data rows (skip header)
        for row_cells in all_cells.iter().skip(1) {
            self.output.push('|');
            for (i, cell) in row_cells.iter().enumerate() {
                self.output.push(' ');
                let width = col_widths.get(i).copied().unwrap_or(3);
                self.output
                    .push_str(&format!("{:width$}", cell, width = width));
                self.output.push_str(" |");
            }
            self.output.push('\n');
        }
    }

    fn serialize_table_row<'b>(&mut self, _node: &'b AstNode<'b>, _is_header: bool) {
        // Table rows are handled by serialize_table
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

        self.output.push_str(&fence);
        self.output.push(' ');
        self.output.push_str(language);
        self.output.push('\n');
        self.output.push_str(literal);
        self.output.push_str(&fence);
        self.output.push('\n');
    }

    fn serialize_list<'b>(&mut self, node: &'b AstNode<'b>, list_type: ListType) {
        let old_list_type = self.list_type;
        let old_index = self.list_item_index;

        self.list_type = Some(list_type);
        self.list_item_index = 0;
        self.list_depth += 1;

        self.serialize_children(node);

        self.list_depth -= 1;
        self.list_type = old_list_type;
        self.list_item_index = old_index;
    }

    fn serialize_list_item<'b>(&mut self, node: &'b AstNode<'b>) {
        self.list_item_index += 1;

        // Calculate indentation for nested lists (4 spaces per level, starting from level 2)
        let indent = if self.list_depth > 1 {
            "    ".repeat(self.list_depth - 1)
        } else {
            String::new()
        };

        // Add block quote prefix if we're inside a block quote
        if self.in_block_quote {
            self.output.push_str("> ");
        }

        self.output.push_str(&indent);

        match self.list_type {
            Some(ListType::Bullet) => {
                // " -  " format: one leading space, hyphen, two trailing spaces
                self.output.push_str(" -  ");
            }
            Some(ListType::Ordered) => {
                // " N. " format for ordered lists
                self.output.push(' ');
                self.output.push_str(&self.list_item_index.to_string());
                self.output.push_str(". ");
            }
            None => {}
        }

        // Serialize children, handling nested lists specially
        for child in node.children() {
            match &child.data.borrow().value {
                NodeValue::List(_) => {
                    // Add newline before nested list
                    self.output.push('\n');
                    self.serialize_node(child);
                }
                _ => {
                    self.serialize_node(child);
                }
            }
        }

        // Only add newline if we didn't just serialize a nested list
        // (nested lists add their own newlines)
        let last_child_is_list = node.children().last().is_some_and(|child| {
            matches!(&child.data.borrow().value, NodeValue::List(_))
        });
        if !last_child_is_list {
            self.output.push('\n');
        }
    }

    fn serialize_children<'b>(&mut self, node: &'b AstNode<'b>) {
        for child in node.children() {
            self.serialize_node(child);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use comrak::{Arena, Options as ComrakOptions, parse_document};

    fn parse_and_serialize(input: &str) -> String {
        let arena = Arena::new();
        let options = ComrakOptions::default();
        let root = parse_document(&arena, input, &options);
        let format_options = Options::default();
        serialize(root, &format_options)
    }

    #[test]
    fn test_serialize_plain_text() {
        let result = parse_and_serialize("Hello, world!");
        assert_eq!(result, "Hello, world!\n");
    }

    #[test]
    fn test_serialize_multiline_paragraph() {
        let result = parse_and_serialize("Hello\nworld!");
        assert_eq!(result, "Hello world!\n");
    }

    #[test]
    fn test_serialize_h1_setext() {
        let result = parse_and_serialize("# Document Title");
        assert_eq!(result, "Document Title\n==============\n");
    }

    #[test]
    fn test_serialize_h2_setext() {
        let result = parse_and_serialize("## Section Name");
        assert_eq!(result, "Section Name\n------------\n");
    }

    #[test]
    fn test_serialize_h3_atx() {
        let result = parse_and_serialize("### Subsection");
        assert_eq!(result, "### Subsection\n");
    }

    #[test]
    fn test_serialize_h4_atx() {
        let result = parse_and_serialize("#### Deep Subsection");
        assert_eq!(result, "#### Deep Subsection\n");
    }

    #[test]
    fn test_serialize_unordered_list_single_item() {
        let result = parse_and_serialize("- Item one");
        assert_eq!(result, " -  Item one\n");
    }

    #[test]
    fn test_serialize_unordered_list_multiple_items() {
        let result = parse_and_serialize("- Item one\n- Item two\n- Item three");
        assert_eq!(result, " -  Item one\n -  Item two\n -  Item three\n");
    }

    #[test]
    fn test_serialize_ordered_list_single_item() {
        let result = parse_and_serialize("1. First item");
        assert_eq!(result, " 1. First item\n");
    }

    #[test]
    fn test_serialize_ordered_list_multiple_items() {
        let result = parse_and_serialize("1. First\n2. Second\n3. Third");
        assert_eq!(result, " 1. First\n 2. Second\n 3. Third\n");
    }

    #[test]
    fn test_serialize_fenced_code_block() {
        let result = parse_and_serialize("```rust\nfn main() {}\n```");
        assert_eq!(result, "~~~~ rust\nfn main() {}\n~~~~\n");
    }

    #[test]
    fn test_serialize_fenced_code_block_no_language() {
        let result = parse_and_serialize("```\nsome code\n```");
        assert_eq!(result, "~~~~ text\nsome code\n~~~~\n");
    }

    #[test]
    fn test_serialize_fenced_code_block_with_tildes_inside() {
        // When code contains ~~~~, use more tildes for the fence
        let result = parse_and_serialize("```\n~~~~\ninner fence\n~~~~\n```");
        assert_eq!(result, "~~~~~ text\n~~~~\ninner fence\n~~~~\n~~~~~\n");
    }

    #[test]
    fn test_serialize_block_quote_single_line() {
        let result = parse_and_serialize("> This is a quote.");
        assert_eq!(result, "> This is a quote.\n");
    }

    #[test]
    fn test_serialize_block_quote_multiple_lines() {
        let result = parse_and_serialize("> Line one.\n> Line two.");
        assert_eq!(result, "> Line one. Line two.\n");
    }

    #[test]
    fn test_serialize_block_quote_multiple_paragraphs() {
        let result = parse_and_serialize("> First paragraph.\n>\n> Second paragraph.");
        assert_eq!(result, "> First paragraph.\n>\n> Second paragraph.\n");
    }

    #[test]
    fn test_serialize_emphasis() {
        let result = parse_and_serialize("This is *emphasized* text.");
        assert_eq!(result, "This is *emphasized* text.\n");
    }

    #[test]
    fn test_serialize_strong() {
        let result = parse_and_serialize("This is **strong** text.");
        assert_eq!(result, "This is **strong** text.\n");
    }

    #[test]
    fn test_serialize_inline_code() {
        let result = parse_and_serialize("Use the `format()` function.");
        assert_eq!(result, "Use the `format()` function.\n");
    }

    #[test]
    fn test_serialize_external_link_becomes_reference() {
        // External links (https://) are converted to reference style
        let result = parse_and_serialize("Visit [Rust](https://www.rust-lang.org/).");
        assert!(result.contains("Visit [Rust]."));
        assert!(result.contains("[Rust]: https://www.rust-lang.org/"));
    }

    #[test]
    fn test_serialize_external_link_with_title_becomes_reference() {
        // External links with titles are also converted to reference style
        let result =
            parse_and_serialize("Visit [Rust](https://www.rust-lang.org/ \"The Rust Language\").");
        assert!(result.contains("Visit [Rust]."));
        assert!(result.contains("[Rust]: https://www.rust-lang.org/ \"The Rust Language\""));
    }

    fn parse_and_serialize_with_frontmatter(input: &str) -> String {
        let arena = Arena::new();
        let mut options = ComrakOptions::default();
        options.extension.front_matter_delimiter = Some("---".to_string());
        let root = parse_document(&arena, input, &options);
        let format_options = Options::default();
        serialize(root, &format_options)
    }

    #[test]
    fn test_serialize_yaml_front_matter() {
        let input = "---\ntitle: Hello\nauthor: World\n---\n\n# Heading";
        let result = parse_and_serialize_with_frontmatter(input);
        assert_eq!(
            result,
            "---\ntitle: Hello\nauthor: World\n---\n\nHeading\n=======\n"
        );
    }

    #[test]
    fn test_serialize_yaml_front_matter_only() {
        let input = "---\ntitle: Test\n---\n\nSome content.";
        let result = parse_and_serialize_with_frontmatter(input);
        assert_eq!(result, "---\ntitle: Test\n---\n\nSome content.\n");
    }

    #[test]
    fn test_serialize_two_blank_lines_before_h2() {
        let input = "# Title\n\nParagraph.\n\n## Section";
        let result = parse_and_serialize(input);
        // Should have two blank lines before h2 (one after paragraph + one extra)
        assert!(result.contains("Paragraph.\n\n\nSection"));
    }

    fn parse_and_serialize_with_width(input: &str, line_width: usize) -> String {
        let arena = Arena::new();
        let options = ComrakOptions::default();
        let root = parse_document(&arena, input, &options);
        let format_options = Options { line_width };
        serialize(root, &format_options)
    }

    #[test]
    fn test_serialize_paragraph_wrap_at_80() {
        // A long line that should wrap at approximately 80 characters
        let input = "This is a very long paragraph that should be wrapped at approximately eighty characters to maintain readability.";
        let result = parse_and_serialize_with_width(input, 80);
        // The line should be wrapped
        assert!(result.contains('\n'));
        // Each line should be at most 80 characters (approximately)
        for line in result.lines() {
            assert!(line.len() <= 85, "Line too long: {} chars", line.len());
        }
    }

    #[test]
    fn test_serialize_paragraph_no_wrap_short() {
        // A short line that should not be wrapped
        let input = "Short paragraph.";
        let result = parse_and_serialize_with_width(input, 80);
        assert_eq!(result, "Short paragraph.\n");
    }

    #[test]
    fn test_serialize_paragraph_wrap_preserves_words() {
        // Words should not be broken
        let input = "Word1 Word2 Word3 Word4 Word5 Word6 Word7 Word8 Word9 Word10 Word11 Word12 Word13 Word14 Word15";
        let result = parse_and_serialize_with_width(input, 40);
        // Check that words are not broken
        for line in result.lines() {
            assert!(!line.ends_with('-'), "Words should not be hyphenated");
        }
    }

    fn parse_and_serialize_with_table(input: &str) -> String {
        let arena = Arena::new();
        let mut options = ComrakOptions::default();
        options.extension.table = true;
        let root = parse_document(&arena, input, &options);
        let format_options = Options::default();
        serialize(root, &format_options)
    }

    #[test]
    fn test_serialize_simple_table() {
        let input = "| A | B |\n|---|---|\n| 1 | 2 |";
        let result = parse_and_serialize_with_table(input);
        assert!(result.contains("| A"));
        assert!(result.contains("| B"));
        assert!(result.contains("| 1"));
        assert!(result.contains("| 2"));
    }

    #[test]
    fn test_serialize_table_with_alignment() {
        let input = "| Left | Center | Right |\n|:-----|:------:|------:|\n| L | C | R |";
        let result = parse_and_serialize_with_table(input);
        // Should contain alignment markers
        assert!(result.contains(":--"));
        assert!(result.contains("--:"));
    }

    #[test]
    fn test_serialize_table_aligned_columns() {
        let input = "| Name | Age |\n|------|-----|\n| Alice | 30 |\n| Bob | 25 |";
        let result = parse_and_serialize_with_table(input);
        // Columns should be aligned with padding
        let lines: Vec<&str> = result.lines().collect();
        // All rows should have the same pipe positions (aligned)
        if lines.len() >= 3 {
            // Find pipe positions in first data row
            let first_pipes: Vec<_> = lines[0].match_indices('|').map(|(i, _)| i).collect();
            // Verify other rows have pipes in similar positions (allowing for padding)
            for line in &lines[1..] {
                let pipes: Vec<_> = line.match_indices('|').map(|(i, _)| i).collect();
                assert_eq!(
                    first_pipes.len(),
                    pipes.len(),
                    "All rows should have same number of pipes"
                );
            }
        }
    }

    fn parse_and_serialize_with_description_list(input: &str) -> String {
        let arena = Arena::new();
        let mut options = ComrakOptions::default();
        options.extension.description_lists = true;
        let root = parse_document(&arena, input, &options);
        let format_options = Options::default();
        serialize(root, &format_options)
    }

    #[test]
    fn test_serialize_definition_list_single() {
        let input = "Term\n:   Definition";
        let result = parse_and_serialize_with_description_list(input);
        assert!(result.contains("Term\n"));
        assert!(result.contains(":   Definition"));
    }

    #[test]
    fn test_serialize_definition_list_multiple_definitions() {
        let input = "Term\n:   First definition\n:   Second definition";
        let result = parse_and_serialize_with_description_list(input);
        assert!(result.contains("Term\n"));
        assert!(result.contains(":   First definition"));
        assert!(result.contains(":   Second definition"));
    }

    fn parse_and_serialize_with_alerts(input: &str) -> String {
        let arena = Arena::new();
        let mut options = ComrakOptions::default();
        options.extension.alerts = true;
        let root = parse_document(&arena, input, &options);
        let format_options = Options::default();
        serialize(root, &format_options)
    }

    #[test]
    fn test_serialize_github_note_alert() {
        let input = "> [!NOTE]\n> This is a note.";
        let result = parse_and_serialize_with_alerts(input);
        assert!(result.contains("> [!NOTE]"));
        assert!(result.contains("> This is a note."));
    }

    #[test]
    fn test_serialize_github_warning_alert() {
        let input = "> [!WARNING]\n> This is a warning.";
        let result = parse_and_serialize_with_alerts(input);
        assert!(result.contains("> [!WARNING]"));
        assert!(result.contains("> This is a warning."));
    }

    #[test]
    fn test_serialize_github_caution_alert() {
        let input = "> [!CAUTION]\n> Be careful!";
        let result = parse_and_serialize_with_alerts(input);
        assert!(result.contains("> [!CAUTION]"));
        assert!(result.contains("> Be careful!"));
    }

    fn parse_and_serialize_with_footnotes(input: &str) -> String {
        let arena = Arena::new();
        let mut options = ComrakOptions::default();
        options.extension.footnotes = true;
        let root = parse_document(&arena, input, &options);
        let format_options = Options::default();
        serialize(root, &format_options)
    }

    #[test]
    fn test_serialize_footnote_reference() {
        let input = "This has a footnote[^1].\n\n[^1]: The footnote text.";
        let result = parse_and_serialize_with_footnotes(input);
        assert!(result.contains("[^1]"));
    }

    #[test]
    fn test_serialize_footnote_definition() {
        let input = "Text[^note].\n\n[^note]: A named footnote.";
        let result = parse_and_serialize_with_footnotes(input);
        assert!(result.contains("[^note]"));
    }

    #[test]
    fn test_serialize_double_space_after_period() {
        // Hong's style uses two spaces after periods
        let input = "First sentence.  Second sentence.";
        let result = parse_and_serialize(input);
        // Should preserve double spaces
        assert_eq!(result, "First sentence.  Second sentence.\n");
    }

    #[test]
    fn test_serialize_long_list_item_wrapping() {
        // Long list items should wrap with 4-space continuation indent
        let input = " -  This is a very long list item that should wrap to the next line with proper indentation to maintain readability.";
        let result = parse_and_serialize_with_width(input, 80);
        // Should contain wrapped content with proper indent
        assert!(result.contains(" -  This is a very long list item"));
        assert!(result.contains("\n    ")); // Continuation with 4 spaces
    }

    fn parse_and_serialize_with_alerts_and_width(input: &str, line_width: usize) -> String {
        let arena = Arena::new();
        let mut options = ComrakOptions::default();
        options.extension.alerts = true;
        let root = parse_document(&arena, input, &options);
        let format_options = Options { line_width };
        serialize(root, &format_options)
    }

    #[test]
    fn test_serialize_list_in_alert() {
        // Lists inside alerts should have proper prefixing
        let input = "> [!NOTE]\n>  -  First item\n>  -  Second item";
        let result = parse_and_serialize_with_alerts(input);
        assert!(result.contains("> [!NOTE]"));
        assert!(result.contains(">  -  First item"));
        assert!(result.contains(">  -  Second item"));
    }

    #[test]
    fn test_serialize_long_list_item_in_alert() {
        // Long list items in alerts should wrap with proper continuation prefix
        let input = "> [!NOTE]\n>  -  This is a very long list item that should wrap properly inside the alert block.";
        let result = parse_and_serialize_with_alerts_and_width(input, 60);
        // Should wrap with ">     " continuation (> + 4 spaces)
        assert!(result.contains(">  -  This is a very long"));
        assert!(result.contains("\n>     ")); // Continuation line with > and 4 spaces
    }

    #[test]
    fn test_serialize_external_link_as_reference() {
        // External URLs should be converted to reference links
        let input = "Visit [Rust](https://www.rust-lang.org/) for more info.";
        let result = parse_and_serialize(input);
        // Should use reference style, not inline
        assert!(result.contains("[Rust]"));
        assert!(!result.contains("](https://"));
        assert!(result.contains("[Rust]: https://www.rust-lang.org/"));
    }

    #[test]
    fn test_serialize_relative_link_stays_inline() {
        // Relative paths should stay as inline links
        let input = "See the [README](./README.md) for details.";
        let result = parse_and_serialize(input);
        // Should keep inline style for relative paths
        assert!(result.contains("[README](./README.md)"));
    }

    #[test]
    fn test_serialize_reference_links_at_section_end() {
        // Reference definitions should appear at the end of each section
        let input = r#"# Title

See [Example](https://example.com/) here.

## Section One

Visit [Rust](https://www.rust-lang.org/) and [Cargo](https://doc.rust-lang.org/cargo/).

## Section Two

Check [Python](https://python.org/) too.
"#;
        let result = parse_and_serialize(input);
        // Each section should have its references at the end
        assert!(result.contains("[Rust]: https://www.rust-lang.org/"));
        assert!(result.contains("[Cargo]: https://doc.rust-lang.org/cargo/"));
        assert!(result.contains("[Python]: https://python.org/"));
        // References should come before the next section
        let rust_def_pos = result.find("[Rust]: ").unwrap();
        let section_two_pos = result.find("Section Two").unwrap();
        assert!(rust_def_pos < section_two_pos);
    }

    #[test]
    fn test_serialize_shortcut_reference_when_text_matches_label() {
        // When link text matches a sensible label, use shortcut reference [text]
        let input = "Use [comrak](https://docs.rs/comrak) for parsing.";
        let result = parse_and_serialize(input);
        // Should use shortcut reference style
        assert!(result.contains("[comrak]"));
        assert!(result.contains("[comrak]: https://docs.rs/comrak"));
    }
}
