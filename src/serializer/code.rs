//! Code block serialization logic.

use comrak::nodes::NodeCodeBlock;

use super::Serializer;

impl<'a> Serializer<'a> {
    /// Serialize a code block with indent for description list details.
    pub(super) fn serialize_code_block_with_indent(&mut self, code: &NodeCodeBlock, indent: &str) {
        let fence_char = self.options.fence_char;
        let base_fence: String = std::iter::repeat_n(fence_char, 4).collect();
        let long_fence: String = std::iter::repeat_n(fence_char, 5).collect();
        let fence = if code.literal.contains(&base_fence) {
            &long_fence
        } else {
            &base_fence
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

    pub(super) fn serialize_code_block(&mut self, info: &str, literal: &str) {
        // Determine the minimum fence length (at least 4)
        let min_fence_length = 4;
        let fence_char = self.options.fence_char;

        // Find the longest sequence of fence characters in the content
        let max_fence_in_content = literal
            .lines()
            .filter_map(|line| {
                let trimmed = line.trim_start();
                if trimmed.starts_with(fence_char) {
                    Some(trimmed.chars().take_while(|&c| c == fence_char).count())
                } else {
                    None
                }
            })
            .max()
            .unwrap_or(0);

        // Fence length must be greater than any fence sequence in content
        let fence_length = std::cmp::max(min_fence_length, max_fence_in_content + 1);
        let fence: String = std::iter::repeat_n(fence_char, fence_length).collect();

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
    pub(super) fn serialize_code_block_indented(
        &mut self,
        info: &str,
        literal: &str,
        indent: &str,
    ) {
        // Determine the minimum fence length (at least 4)
        let min_fence_length = 4;
        let fence_char = self.options.fence_char;

        // Find the longest sequence of fence characters in the content
        let max_fence_in_content = literal
            .lines()
            .filter_map(|line| {
                let trimmed = line.trim_start();
                if trimmed.starts_with(fence_char) {
                    Some(trimmed.chars().take_while(|&c| c == fence_char).count())
                } else {
                    None
                }
            })
            .max()
            .unwrap_or(0);

        // Fence length must be greater than any fence sequence in content
        let fence_length = std::cmp::max(min_fence_length, max_fence_in_content + 1);
        let fence: String = std::iter::repeat_n(fence_char, fence_length).collect();

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
}
