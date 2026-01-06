//! Serializer for converting comrak AST to formatted Markdown.

use comrak::nodes::{AstNode, ListType, NodeValue};

use crate::Options;

/// Serializes a comrak AST node to a formatted Markdown string.
pub fn serialize<'a>(node: &'a AstNode<'a>, options: &Options) -> String {
    let mut serializer = Serializer::new(options);
    serializer.serialize_node(node);
    serializer.output
}

struct Serializer<'a> {
    output: String,
    #[allow(dead_code)]
    options: &'a Options,
    /// Current list item index (1-based) for ordered lists
    list_item_index: usize,
    /// Current list type
    list_type: Option<ListType>,
    /// Whether we're inside a block quote
    in_block_quote: bool,
}

impl<'a> Serializer<'a> {
    fn new(options: &'a Options) -> Self {
        Self {
            output: String::new(),
            options,
            list_item_index: 0,
            list_type: None,
            in_block_quote: false,
        }
    }

    fn serialize_node<'b>(&mut self, node: &'b AstNode<'b>) {
        match &node.data.borrow().value {
            NodeValue::Document => {
                self.serialize_children(node);
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
            NodeValue::Item(_) => {
                self.serialize_list_item(node);
            }
            NodeValue::Paragraph => {
                if self.list_type.is_some() {
                    // Inside a list item, don't add trailing newline
                    self.serialize_children(node);
                } else if self.in_block_quote {
                    // Inside a block quote, add > prefix
                    self.output.push_str("> ");
                    self.serialize_children(node);
                    self.output.push('\n');
                } else {
                    self.serialize_children(node);
                    self.output.push('\n');
                }
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
                self.output.push('[');
                self.serialize_children(node);
                self.output.push_str("](");
                self.output.push_str(&link.url);
                if !link.title.is_empty() {
                    self.output.push_str(" \"");
                    self.output.push_str(&link.title);
                    self.output.push('"');
                }
                self.output.push(')');
            }
            _ => {
                // For now, just recurse into children for unhandled nodes
                self.serialize_children(node);
            }
        }
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

        self.serialize_children(node);

        self.list_type = old_list_type;
        self.list_item_index = old_index;
    }

    fn serialize_list_item<'b>(&mut self, node: &'b AstNode<'b>) {
        self.list_item_index += 1;

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

        self.serialize_children(node);
        self.output.push('\n');
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
    fn test_serialize_inline_link() {
        let result = parse_and_serialize("Visit [Rust](https://www.rust-lang.org/).");
        assert_eq!(result, "Visit [Rust](https://www.rust-lang.org/).\n");
    }

    #[test]
    fn test_serialize_inline_link_with_title() {
        let result =
            parse_and_serialize("Visit [Rust](https://www.rust-lang.org/ \"The Rust Language\").");
        assert_eq!(
            result,
            "Visit [Rust](https://www.rust-lang.org/ \"The Rust Language\").\n"
        );
    }
}
