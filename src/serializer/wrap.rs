//! Text wrapping utilities for Markdown serialization.

/// Wrap text at the specified line width.
///
/// This function handles soft break markers (`\x00`) which represent where
/// the original document had line breaks. Short lines are preserved as-is,
/// while long lines are merged and rewrapped.
pub fn wrap_text(text: &str, prefix: &str, line_width: usize) -> String {
    // Split by soft break markers (original line breaks)
    // \x00 represents where the original document had line breaks
    let original_lines: Vec<&str> = text.split('\x00').collect();

    if original_lines.len() == 1 {
        // No original line breaks, just wrap normally
        return wrap_single_segment(text, prefix, prefix, line_width);
    }

    // Process lines: keep short lines as-is until we hit a long line,
    // then merge everything from that point onward and rewrap
    let mut result = String::new();
    let mut i = 0;

    while i < original_lines.len() {
        let line = original_lines[i].trim();
        let line_with_prefix_len = prefix.len() + line.len();

        if line_with_prefix_len <= line_width {
            // Line fits within limit, keep it as-is
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str(prefix);
            result.push_str(line);
            i += 1;
        } else {
            // Line exceeds limit, merge ALL remaining lines and rewrap
            let mut merged = String::from(line);
            i += 1;
            while i < original_lines.len() {
                let next_line = original_lines[i].trim();
                merged.push(' ');
                merged.push_str(next_line);
                i += 1;
            }

            // Wrap the merged content
            let wrapped = wrap_single_segment(&merged, prefix, prefix, line_width);

            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str(&wrapped);
        }
    }

    result
}

/// Wrap text where the first line has a different prefix than continuation lines.
///
/// This is used for list items where the marker is already output and continuation
/// lines need indentation.
pub fn wrap_text_first_line(
    text: &str,
    first_prefix: &str,
    continuation_prefix: &str,
    line_width: usize,
) -> String {
    // Split by soft break markers (original line breaks)
    let original_lines: Vec<&str> = text.split('\x00').collect();

    if original_lines.len() == 1 {
        // No original line breaks, just wrap normally
        return wrap_single_segment(text, first_prefix, continuation_prefix, line_width);
    }

    // Process lines: keep short lines as-is until we hit a long line,
    // then merge everything from that point onward and rewrap
    let mut result = String::new();
    let mut i = 0;
    let mut is_first_line = true;

    while i < original_lines.len() {
        let line = original_lines[i].trim();
        let current_prefix = if is_first_line {
            first_prefix
        } else {
            continuation_prefix
        };
        let line_with_prefix_len = current_prefix.len() + line.len();

        if line_with_prefix_len <= line_width {
            // Line fits within limit, keep it as-is
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str(current_prefix);
            result.push_str(line);
            is_first_line = false;
            i += 1;
        } else {
            // Line exceeds limit, merge ALL remaining lines and rewrap
            let mut merged = String::from(line);
            i += 1;
            while i < original_lines.len() {
                let next_line = original_lines[i].trim();
                merged.push(' ');
                merged.push_str(next_line);
                i += 1;
            }

            // Wrap the merged content
            let wrapped =
                wrap_single_segment(&merged, current_prefix, continuation_prefix, line_width);

            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str(&wrapped);
            is_first_line = false;
        }
    }

    result
}

/// Wrap a single segment of text (no original line break markers).
///
/// Handles special tokens like backtick-delimited code spans and bracketed
/// content (links) as unbreakable units.
pub fn wrap_single_segment(
    text: &str,
    first_prefix: &str,
    prefix: &str,
    line_width: usize,
) -> String {
    let mut result = String::new();
    let mut current_line = String::new();
    let mut is_first_line = true;
    let first_prefix_len = first_prefix.len();

    // Add prefix to first line
    current_line.push_str(first_prefix);

    // Split into "tokens" where each token is either:
    // - A word (non-space characters) followed by optional spaces
    // - Content inside backticks (treated as a single unbreakable unit)
    // - Content inside brackets (treated as a single unbreakable unit for links)
    // We preserve double spaces after periods.
    let chars = text.chars();
    let mut current_token = String::new();
    let mut trailing_spaces = String::new();
    let mut in_backticks = false;
    let mut bracket_depth = 0;

    for ch in chars {
        if ch == '`' && bracket_depth == 0 {
            if in_backticks {
                // End of backtick region
                current_token.push(ch);
                in_backticks = false;
            } else {
                // Start of backtick region - include any accumulated content first
                if !current_token.is_empty() && !trailing_spaces.is_empty() {
                    // We have a previous word, output it
                    add_token_to_line_with_prefix(
                        &mut result,
                        &mut current_line,
                        &current_token,
                        &trailing_spaces,
                        first_prefix_len,
                        prefix,
                        line_width,
                        &mut is_first_line,
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
        } else if ch == '[' {
            // Start of bracket region
            if bracket_depth == 0 && !current_token.is_empty() && !trailing_spaces.is_empty() {
                // Output previous token before starting bracket
                add_token_to_line_with_prefix(
                    &mut result,
                    &mut current_line,
                    &current_token,
                    &trailing_spaces,
                    first_prefix_len,
                    prefix,
                    line_width,
                    &mut is_first_line,
                );
                current_token.clear();
                trailing_spaces.clear();
            }
            current_token.push(ch);
            bracket_depth += 1;
        } else if ch == ']' && bracket_depth > 0 {
            // End of bracket region (or nested bracket)
            current_token.push(ch);
            bracket_depth -= 1;
        } else if bracket_depth > 0 {
            // Inside brackets, everything is part of the token
            current_token.push(ch);
        } else if ch == ' ' {
            trailing_spaces.push(ch);
        } else {
            // Regular character outside backticks and brackets
            if !current_token.is_empty() && !trailing_spaces.is_empty() {
                // We have a previous word with trailing spaces, output it
                add_token_to_line_with_prefix(
                    &mut result,
                    &mut current_line,
                    &current_token,
                    &trailing_spaces,
                    first_prefix_len,
                    prefix,
                    line_width,
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
        add_token_to_line_with_prefix(
            &mut result,
            &mut current_line,
            &current_token,
            "",
            first_prefix_len,
            prefix,
            line_width,
            &mut is_first_line,
        );
    }

    // Add the last line (trim trailing spaces)
    let final_line = current_line.trim_end();
    if !final_line.is_empty() {
        result.push_str(final_line);
    }

    result
}

#[allow(clippy::too_many_arguments)]
fn add_token_to_line_with_prefix(
    result: &mut String,
    current_line: &mut String,
    token: &str,
    trailing_spaces: &str,
    first_prefix_len: usize,
    prefix: &str,
    line_width: usize,
    is_first_line: &mut bool,
) {
    let token_len = token.len();
    let spaces_len = trailing_spaces.len();
    let current_prefix_len = if *is_first_line {
        first_prefix_len
    } else {
        prefix.len()
    };

    if current_line.len() == current_prefix_len {
        // First word on this line (prefix already added)
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
        *is_first_line = false;
    }
}
