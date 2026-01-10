//! Text wrapping utilities for Markdown serialization.

use unicode_width::UnicodeWidthStr;

/// Wrap text at the specified line width.
///
/// This function handles soft break markers (`\x00`) which represent where
/// the original document had line breaks. Short lines are preserved as-is,
/// while long lines are merged and rewrapped.
///
/// Hard line breaks (`\n`) are preserved with two trailing spaces before the
/// newline, and the prefix is added to the continuation line.
pub fn wrap_text(text: &str, prefix: &str, line_width: usize) -> String {
    // First, split by hard line breaks (actual newlines)
    // These must be preserved with two trailing spaces
    let hard_break_segments: Vec<&str> = text.split('\n').collect();

    if hard_break_segments.len() == 1 {
        // No hard line breaks, process normally with soft breaks
        return wrap_text_segment(text, prefix, line_width);
    }

    // Process each segment separated by hard line breaks
    let mut result = String::new();
    for (idx, segment) in hard_break_segments.iter().enumerate() {
        if idx > 0 {
            // Add two trailing spaces for hard line break, then newline and prefix
            result.push_str("  \n");
        }
        // First segment uses the normal prefix, subsequent segments also need prefix
        // (wrap_text_segment handles adding the prefix to the first line)
        let wrapped = wrap_text_segment(segment, prefix, line_width);
        result.push_str(&wrapped);
    }

    result
}

/// Wrap a single segment of text (between hard line breaks).
fn wrap_text_segment(text: &str, prefix: &str, line_width: usize) -> String {
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
        let line_with_prefix_len = prefix.width() + line.width();

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
///
/// Hard line breaks (`\n`) are preserved with two trailing spaces before the
/// newline, and the continuation prefix is added to the continuation line.
pub fn wrap_text_first_line(
    text: &str,
    first_prefix: &str,
    continuation_prefix: &str,
    line_width: usize,
) -> String {
    // First, split by hard line breaks (actual newlines)
    // These must be preserved with two trailing spaces
    let hard_break_segments: Vec<&str> = text.split('\n').collect();

    if hard_break_segments.len() == 1 {
        // No hard line breaks, process normally with soft breaks
        return wrap_text_first_line_segment(text, first_prefix, continuation_prefix, line_width);
    }

    // Process each segment separated by hard line breaks
    let mut result = String::new();
    let mut is_first_segment = true;
    for segment in hard_break_segments {
        if !is_first_segment {
            // Add two trailing spaces for hard line break, then newline
            result.push_str("  \n");
            result.push_str(continuation_prefix);
        }
        let (current_first, current_cont) = if is_first_segment {
            (first_prefix, continuation_prefix)
        } else {
            ("", continuation_prefix)
        };
        let wrapped =
            wrap_text_first_line_segment(segment, current_first, current_cont, line_width);
        result.push_str(&wrapped);
        is_first_segment = false;
    }

    result
}

/// Wrap a single segment of text (between hard line breaks) with first line prefix.
fn wrap_text_first_line_segment(
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
        let line_with_prefix_len = current_prefix.width() + line.width();

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
    let first_prefix_width = first_prefix.width();

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
                        first_prefix_width,
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
                    first_prefix_width,
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
                    first_prefix_width,
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
            first_prefix_width,
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
    first_prefix_width: usize,
    prefix: &str,
    line_width: usize,
    is_first_line: &mut bool,
) {
    let token_width = token.width();
    let spaces_len = trailing_spaces.len();
    let current_prefix_width = if *is_first_line {
        first_prefix_width
    } else {
        prefix.width()
    };

    if current_line.width() == current_prefix_width {
        // First word on this line (prefix already added)
        current_line.push_str(token);
        current_line.push_str(trailing_spaces);
    } else if current_line.width() + token_width + spaces_len <= line_width {
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
