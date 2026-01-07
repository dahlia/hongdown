//! Table serialization logic.

use comrak::nodes::{AstNode, NodeTable, TableAlignment};

use super::Serializer;
use super::escape;

impl<'a> Serializer<'a> {
    pub(super) fn serialize_table<'b>(&mut self, node: &'b AstNode<'b>, table: &NodeTable) {
        let alignments = &table.alignments;
        let expected_cols = alignments.len();
        // Collect all rows and cells first to calculate column widths
        let rows: Vec<_> = node.children().collect();
        if rows.is_empty() {
            return;
        }

        // Get table source line for warnings
        let table_line = node.data.borrow().sourcepos.start.line;

        // Check source lines for column count mismatches
        // comrak normalizes column counts, so we need to check the source directly
        self.check_table_source_columns(node, expected_cols, table_line);

        // Collect cell contents (with full inline formatting) and calculate max widths
        let mut all_cells: Vec<Vec<String>> = Vec::new();
        let mut col_widths: Vec<usize> = vec![0; alignments.len()];

        for row in &rows {
            let mut row_cells: Vec<String> = Vec::new();

            for (i, cell) in row.children().enumerate() {
                // Use collect_inline_content to preserve links and formatting
                let mut content = String::new();
                self.collect_inline_content(cell, &mut content);
                // Escape pipe characters in table cells to prevent cell boundary confusion
                let content = escape::escape_table_cell(&content);
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
            if self.in_block_quote {
                self.output.push_str("> ");
            }
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
        if self.in_block_quote {
            self.output.push_str("> ");
        }
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
            if self.in_block_quote {
                self.output.push_str("> ");
            }
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

    pub(super) fn serialize_table_row<'b>(&mut self, _node: &'b AstNode<'b>, _is_header: bool) {
        // Table rows are handled by serialize_table
    }

    /// Check source lines for table column count mismatches.
    /// This detects issues that comrak normalizes away, like unescaped pipes in cells.
    fn check_table_source_columns<'b>(
        &mut self,
        node: &'b AstNode<'b>,
        expected_cols: usize,
        table_start_line: usize,
    ) {
        let sourcepos = node.data.borrow().sourcepos;
        let start_line = sourcepos.start.line;
        let end_line = sourcepos.end.line;

        if start_line == 0 || end_line == 0 {
            return;
        }

        for line_num in start_line..=end_line {
            let line_idx = line_num - 1;
            if line_idx >= self.source_lines.len() {
                continue;
            }
            let line = self.source_lines[line_idx];

            // Skip separator row (contains only |, -, :, and spaces)
            if line
                .chars()
                .all(|c| c == '|' || c == '-' || c == ':' || c == ' ')
            {
                continue;
            }

            // Count unescaped pipe characters (column separators)
            let pipe_count = count_unescaped_pipes(line);

            // A row with N columns has N+1 pipe characters (including leading and trailing)
            // But some tables may omit leading/trailing pipes
            // Expected: expected_cols + 1 pipes for a proper table row
            // Allow expected_cols pipes if leading or trailing is omitted
            let expected_pipes_full = expected_cols + 1;
            let expected_pipes_min = expected_cols;

            if pipe_count > expected_pipes_full {
                self.add_warning(
                    line_num,
                    format!(
                        "table row has {} pipe characters, expected {} for {} columns; \
                         unescaped `|` in cell content? (table starts at line {})",
                        pipe_count, expected_pipes_full, expected_cols, table_start_line
                    ),
                );
            } else if pipe_count < expected_pipes_min {
                self.add_warning(
                    line_num,
                    format!(
                        "table row has {} pipe characters, expected at least {} for {} columns \
                         (table starts at line {})",
                        pipe_count, expected_pipes_min, expected_cols, table_start_line
                    ),
                );
            }
        }
    }
}

/// Count unescaped pipe characters in a line.
/// Pipes inside backticks should still be counted as they're not escaped for GFM tables.
fn count_unescaped_pipes(line: &str) -> usize {
    let mut count = 0;
    let mut chars = line.chars().peekable();
    let mut prev_char = None;

    while let Some(c) = chars.next() {
        if c == '|' && prev_char != Some('\\') {
            count += 1;
        }
        prev_char = Some(c);
    }

    count
}
