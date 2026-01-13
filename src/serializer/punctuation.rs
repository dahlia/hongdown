//! Punctuation transformation utilities (SmartyPants-style).
//!
//! This module provides functions to transform straight quotes, apostrophes,
//! and other punctuation into their typographic equivalents.

use crate::{DashSetting, Options};

// Unicode constants for punctuation characters
// Using escape sequences because Claude normalizes curly quotes to straight quotes

/// Straight double quote (U+0022)
pub const STRAIGHT_DOUBLE_QUOTE: char = '\u{0022}';
/// Left double quotation mark (U+201C)
pub const LEFT_DOUBLE_QUOTE: char = '\u{201C}';
/// Right double quotation mark (U+201D)
pub const RIGHT_DOUBLE_QUOTE: char = '\u{201D}';

/// Straight single quote / apostrophe (U+0027)
pub const STRAIGHT_SINGLE_QUOTE: char = '\u{0027}';
/// Left single quotation mark (U+2018)
pub const LEFT_SINGLE_QUOTE: char = '\u{2018}';
/// Right single quotation mark (U+2019) - also used as curly apostrophe
pub const RIGHT_SINGLE_QUOTE: char = '\u{2019}';

/// Horizontal ellipsis (U+2026)
pub const ELLIPSIS: char = '\u{2026}';
/// En dash (U+2013)
pub const EN_DASH: char = '\u{2013}';
/// Em dash (U+2014)
pub const EM_DASH: char = '\u{2014}';

/// Errors that can occur during punctuation configuration validation.
#[derive(Debug, Clone, PartialEq)]
pub enum PunctuationError {
    /// The en_dash and em_dash patterns conflict (are the same).
    ConflictingDashPatterns(String),
}

impl std::fmt::Display for PunctuationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PunctuationError::ConflictingDashPatterns(pattern) => {
                write!(
                    f,
                    "conflicting dash patterns: en_dash and em_dash both use {:?}",
                    pattern
                )
            }
        }
    }
}

impl std::error::Error for PunctuationError {}

/// Validate that en_dash and em_dash patterns don't conflict.
pub fn validate_dash_settings(options: &Options) -> Result<(), PunctuationError> {
    if let (DashSetting::Pattern(en), DashSetting::Pattern(em)) =
        (&options.en_dash, &options.em_dash)
        && en == em
    {
        return Err(PunctuationError::ConflictingDashPatterns(en.clone()));
    }
    Ok(())
}

/// Transform punctuation in text according to options.
///
/// This applies SmartyPants-style transformations:
/// - Straight double quotes to curly double quotes
/// - Straight single quotes to curly single quotes
/// - Straight apostrophes to curly apostrophes (if enabled)
/// - Three dots to ellipsis character
/// - Dash patterns to en-dash or em-dash
pub fn transform_punctuation(text: &str, options: &Options) -> String {
    let mut result = text.to_string();

    // Apply transformations in order (longer patterns first to avoid conflicts)

    // 1. Em-dash (process longer pattern first if applicable)
    if let DashSetting::Pattern(pattern) = &options.em_dash {
        result = transform_dashes(&result, pattern, EM_DASH);
    }

    // 2. En-dash
    if let DashSetting::Pattern(pattern) = &options.en_dash {
        result = transform_dashes(&result, pattern, EN_DASH);
    }

    // 3. Ellipsis
    if options.ellipsis {
        result = transform_ellipsis(&result);
    }

    // 4. Double quotes
    if options.curly_double_quotes {
        result = transform_double_quotes(&result);
    }

    // 5. Single quotes (before apostrophes since they share the same character)
    if options.curly_single_quotes {
        result = transform_single_quotes(&result);
    }

    // 6. Apostrophes (only if enabled, processed after single quotes)
    if options.curly_apostrophes {
        result = transform_apostrophes(&result);
    }

    result
}

/// Transform three consecutive dots to ellipsis character.
fn transform_ellipsis(text: &str) -> String {
    // Replace ... with ellipsis, but handle .... (4 dots) as ellipsis + period
    let mut result = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if i + 2 < chars.len() && chars[i] == '.' && chars[i + 1] == '.' && chars[i + 2] == '.' {
            // Check if there's a 4th dot
            if i + 3 < chars.len() && chars[i + 3] == '.' {
                // Four dots: ellipsis + period
                result.push(ELLIPSIS);
                result.push('.');
                i += 4;
            } else {
                // Three dots: just ellipsis
                result.push(ELLIPSIS);
                i += 3;
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    result
}

/// Transform straight double quotes to curly double quotes.
fn transform_double_quotes(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let mut expecting_open = true;

    for (i, &ch) in chars.iter().enumerate() {
        if ch == STRAIGHT_DOUBLE_QUOTE {
            // Determine if this should be an opening or closing quote
            let is_opening = should_be_opening_quote(&chars, i, expecting_open);

            if is_opening {
                result.push(LEFT_DOUBLE_QUOTE);
                expecting_open = false;
            } else {
                result.push(RIGHT_DOUBLE_QUOTE);
                expecting_open = true;
            }
        } else {
            result.push(ch);
        }
    }

    result
}

/// Transform straight single quotes to curly single quotes.
/// This only handles quotes that appear to be used as quotation marks,
/// not apostrophes within words.
#[allow(clippy::collapsible_if)]
fn transform_single_quotes(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let mut expecting_open = true;

    for (i, &ch) in chars.iter().enumerate() {
        if ch == STRAIGHT_SINGLE_QUOTE {
            // Check if this looks like an apostrophe (within a word)
            let prev_char = if i > 0 { Some(chars[i - 1]) } else { None };
            let next_char = if i + 1 < chars.len() {
                Some(chars[i + 1])
            } else {
                None
            };

            let prev_is_letter = prev_char.is_some_and(|c| c.is_alphabetic());
            let next_is_letter = next_char.is_some_and(|c| c.is_alphabetic());

            // If surrounded by letters, it's an apostrophe - leave for apostrophe transform
            if prev_is_letter && next_is_letter {
                result.push(ch);
                continue;
            }

            // Word-final apostrophes like goin', diggin' - leave for apostrophe transform
            // BUT only if we're not expecting a closing quote (no unmatched opening quote)
            // (letter followed by apostrophe followed by space, punctuation, or end of text)
            if expecting_open
                && prev_is_letter
                && (next_char.is_none()
                    || next_char.is_some_and(|c| c.is_whitespace() || c.is_ascii_punctuation()))
            {
                result.push(ch);
                continue;
            }

            // Possessive after closing brackets like [name]'s, (term)'s, {item}'s
            // These are apostrophes, not quotation marks - leave for apostrophe transform
            if prev_char.is_some_and(|c| c == ']' || c == ')' || c == '}')
                && next_char.is_some_and(|c| c == 's' || c == 'S')
            {
                // Check if it's actually possessive (s followed by non-letter)
                let after_s = chars.get(i + 2);
                if after_s.is_none() || after_s.is_some_and(|c| !c.is_alphabetic()) {
                    result.push(ch);
                    continue;
                }
            }

            // Leading possessive 's at text start (e.g., after a link in parsed Markdown)
            // This is an apostrophe, not a quotation mark - leave for apostrophe transform
            if prev_char.is_none() && next_char.is_some_and(|c| c == 's' || c == 'S') {
                let after_s = chars.get(i + 2);
                if after_s.is_none() || after_s.is_some_and(|c| !c.is_alphabetic()) {
                    result.push(ch);
                    continue;
                }
            }

            // Decade abbreviations like '80s - always closing/apostrophe style
            if next_char.is_some_and(|c| c.is_ascii_digit()) {
                result.push(RIGHT_SINGLE_QUOTE);
                continue;
            }

            // Leading contractions like 'twas - always closing/apostrophe style
            if prev_char.is_none() || prev_char.is_some_and(|c| c.is_whitespace()) {
                if next_is_letter {
                    // Could be opening quote or leading contraction
                    // Check if there's a closing quote later in a reasonable distance
                    let has_closing = chars[i + 1..]
                        .iter()
                        .take(50)
                        .any(|&c| c == STRAIGHT_SINGLE_QUOTE);
                    if has_closing {
                        result.push(LEFT_SINGLE_QUOTE);
                        expecting_open = false;
                        continue;
                    } else {
                        // Likely a leading contraction like 'twas
                        result.push(RIGHT_SINGLE_QUOTE);
                        continue;
                    }
                }
            }

            // Determine if this should be an opening or closing quote
            let is_opening = should_be_opening_quote(&chars, i, expecting_open);

            if is_opening {
                result.push(LEFT_SINGLE_QUOTE);
                expecting_open = false;
            } else {
                result.push(RIGHT_SINGLE_QUOTE);
                expecting_open = true;
            }
        } else {
            result.push(ch);
        }
    }

    result
}

/// Transform straight apostrophes within words to curly apostrophes.
/// This function is conservative to avoid converting single quotes used as quotation marks.
fn transform_apostrophes(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();

    // Track unmatched opening single quotes to detect quotation marks
    let mut open_single_quote_count = 0;

    for (i, &ch) in chars.iter().enumerate() {
        if ch == STRAIGHT_SINGLE_QUOTE {
            let prev_char = if i > 0 { Some(chars[i - 1]) } else { None };
            let next_char = if i + 1 < chars.len() {
                Some(chars[i + 1])
            } else {
                None
            };

            let prev_is_letter_or_digit = prev_char.is_some_and(|c| c.is_alphanumeric());
            let next_is_letter_or_digit = next_char.is_some_and(|c| c.is_alphanumeric());
            let prev_is_space_or_start =
                prev_char.is_none() || prev_char.is_some_and(|c| c.is_whitespace());
            let next_is_space_or_end =
                next_char.is_none() || next_char.is_some_and(|c| c.is_whitespace());

            // Mid-word apostrophe (surrounded by letters/digits) - always convert
            // e.g., "it's", "don't", "rock'n'roll"
            if prev_is_letter_or_digit && next_is_letter_or_digit {
                result.push(RIGHT_SINGLE_QUOTE);
                continue;
            }

            // Possessive with 's - convert if followed by 's' then word boundary
            // e.g., "GitHub's", "John's"
            if prev_is_letter_or_digit && next_char.is_some_and(|c| c == 's' || c == 'S') {
                let after_s = if i + 2 < chars.len() {
                    Some(chars[i + 2])
                } else {
                    None
                };
                // Make sure it's actually possessive (s followed by non-letter)
                if after_s.is_none() || after_s.is_some_and(|c| !c.is_alphabetic()) {
                    result.push(RIGHT_SINGLE_QUOTE);
                    continue;
                }
            }

            // Possessive after closing brackets like [name]'s, (term)'s, {item}'s
            if prev_char.is_some_and(|c| c == ']' || c == ')' || c == '}')
                && next_char.is_some_and(|c| c == 's' || c == 'S')
            {
                let after_s = chars.get(i + 2);
                if after_s.is_none() || after_s.is_some_and(|c| !c.is_alphabetic()) {
                    result.push(RIGHT_SINGLE_QUOTE);
                    continue;
                }
            }

            // Leading possessive 's at text start (e.g., after a link in parsed Markdown)
            if prev_char.is_none() && next_char.is_some_and(|c| c == 's' || c == 'S') {
                let after_s = chars.get(i + 2);
                if after_s.is_none() || after_s.is_some_and(|c| !c.is_alphabetic()) {
                    result.push(RIGHT_SINGLE_QUOTE);
                    continue;
                }
            }

            // Potential opening single quote (space/start followed by quote followed by letter)
            if prev_is_space_or_start && next_is_letter_or_digit {
                // This could be an opening quote - track it
                open_single_quote_count += 1;
                result.push(ch);
                continue;
            }

            // Potential closing single quote (letter followed by quote followed by space/end)
            if prev_is_letter_or_digit && next_is_space_or_end {
                if open_single_quote_count > 0 {
                    // There's an unmatched opening quote, so this is likely a closing quote
                    open_single_quote_count -= 1;
                    result.push(ch);
                } else {
                    // No unmatched opening quote - this is a word-final apostrophe
                    // e.g., "goin'", "diggin'"
                    result.push(RIGHT_SINGLE_QUOTE);
                }
                continue;
            }

            // Default: leave as-is
            result.push(ch);
        } else {
            result.push(ch);
        }
    }

    result
}

/// Transform dash patterns to the specified dash character.
/// For single-character patterns, only transform when surrounded by whitespace.
fn transform_dashes(text: &str, pattern: &str, replacement: char) -> String {
    if pattern.is_empty() {
        return text.to_string();
    }

    let pattern_chars: Vec<char> = pattern.chars().collect();
    let is_single_char = pattern_chars.len() == 1;

    let mut result = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Check if we have the pattern at this position
        if i + pattern_chars.len() <= chars.len() {
            let slice: Vec<char> = chars[i..i + pattern_chars.len()].to_vec();
            if slice == pattern_chars {
                // For single-character patterns, require surrounding whitespace
                if is_single_char {
                    let prev_is_space = i == 0 || chars[i - 1].is_whitespace();
                    let next_is_space = i + pattern_chars.len() >= chars.len()
                        || chars[i + pattern_chars.len()].is_whitespace();

                    if prev_is_space && next_is_space {
                        result.push(replacement);
                        i += pattern_chars.len();
                        continue;
                    }
                } else {
                    // Multi-character patterns: always replace
                    result.push(replacement);
                    i += pattern_chars.len();
                    continue;
                }
            }
        }
        result.push(chars[i]);
        i += 1;
    }

    result
}

/// Determine if a quote at position `i` should be an opening quote.
fn should_be_opening_quote(chars: &[char], i: usize, expecting_open: bool) -> bool {
    let prev_char = if i > 0 { Some(chars[i - 1]) } else { None };
    let next_char = if i + 1 < chars.len() {
        Some(chars[i + 1])
    } else {
        None
    };

    // Opening quote indicators:
    // - At start of text
    // - After whitespace
    // - After opening brackets/parens
    // - After other opening quotes
    let prev_suggests_open = prev_char.is_none()
        || prev_char.is_some_and(|c| {
            c.is_whitespace()
                || c == '('
                || c == '['
                || c == '{'
                || c == LEFT_DOUBLE_QUOTE
                || c == LEFT_SINGLE_QUOTE
        });

    // Closing quote indicators:
    // - Before whitespace
    // - Before punctuation
    // - At end of text
    let next_suggests_close = next_char.is_none()
        || next_char.is_some_and(|c| {
            c.is_whitespace()
                || c == '.'
                || c == ','
                || c == ';'
                || c == ':'
                || c == '!'
                || c == '?'
                || c == ')'
                || c == ']'
                || c == '}'
        });

    if prev_suggests_open && !next_suggests_close {
        true
    } else if next_suggests_close && !prev_suggests_open {
        false
    } else {
        // Ambiguous case: use expectation based on alternation
        expecting_open
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create default options
    fn default_options() -> Options {
        Options::default()
    }

    // Helper to create options with specific settings
    fn options_with(
        curly_double_quotes: bool,
        curly_single_quotes: bool,
        curly_apostrophes: bool,
        ellipsis: bool,
        en_dash: DashSetting,
        em_dash: DashSetting,
    ) -> Options {
        let mut options = Options::default();
        options.curly_double_quotes = curly_double_quotes;
        options.curly_single_quotes = curly_single_quotes;
        options.curly_apostrophes = curly_apostrophes;
        options.ellipsis = ellipsis;
        options.en_dash = en_dash;
        options.em_dash = em_dash;
        options
    }

    // ========== Ellipsis tests ==========

    #[test]
    fn test_ellipsis_basic() {
        let options = default_options();
        let result = transform_punctuation("Wait...", &options);
        assert_eq!(result, format!("Wait{}", ELLIPSIS));
    }

    #[test]
    fn test_ellipsis_four_dots() {
        let options = default_options();
        let result = transform_punctuation("Wait....", &options);
        // Four dots become ellipsis + period
        assert_eq!(result, format!("Wait{}.", ELLIPSIS));
    }

    #[test]
    fn test_ellipsis_multiple() {
        let options = default_options();
        let result = transform_punctuation("Wait... and... more...", &options);
        assert_eq!(
            result,
            format!("Wait{} and{} more{}", ELLIPSIS, ELLIPSIS, ELLIPSIS)
        );
    }

    #[test]
    fn test_ellipsis_disabled() {
        let options = options_with(
            true,
            true,
            false,
            false,
            DashSetting::Disabled,
            DashSetting::Pattern("--".to_string()),
        );
        let result = transform_punctuation("Wait...", &options);
        assert_eq!(result, "Wait...");
    }

    #[test]
    fn test_ellipsis_preserve_existing() {
        let options = default_options();
        let input = format!("Already has{}", ELLIPSIS);
        let result = transform_punctuation(&input, &options);
        assert_eq!(result, input);
    }

    // ========== Double quotes tests ==========

    #[test]
    fn test_curly_double_quotes_basic() {
        let options = default_options();
        let result = transform_punctuation("He said \"hello\" to her.", &options);
        assert_eq!(
            result,
            format!(
                "He said {}hello{} to her.",
                LEFT_DOUBLE_QUOTE, RIGHT_DOUBLE_QUOTE
            )
        );
    }

    #[test]
    fn test_curly_double_quotes_multiple_pairs() {
        let options = default_options();
        let result = transform_punctuation("\"Hello\" and \"world\"", &options);
        assert_eq!(
            result,
            format!(
                "{}Hello{} and {}world{}",
                LEFT_DOUBLE_QUOTE, RIGHT_DOUBLE_QUOTE, LEFT_DOUBLE_QUOTE, RIGHT_DOUBLE_QUOTE
            )
        );
    }

    #[test]
    fn test_curly_double_quotes_at_start() {
        let options = default_options();
        let result = transform_punctuation("\"Hello world\"", &options);
        assert_eq!(
            result,
            format!("{}Hello world{}", LEFT_DOUBLE_QUOTE, RIGHT_DOUBLE_QUOTE)
        );
    }

    #[test]
    fn test_curly_double_quotes_after_paren() {
        let options = default_options();
        let result = transform_punctuation("(\"quoted\")", &options);
        assert_eq!(
            result,
            format!("({}quoted{})", LEFT_DOUBLE_QUOTE, RIGHT_DOUBLE_QUOTE)
        );
    }

    #[test]
    fn test_curly_double_quotes_disabled() {
        let options = options_with(
            false,
            true,
            false,
            true,
            DashSetting::Disabled,
            DashSetting::Pattern("--".to_string()),
        );
        let result = transform_punctuation("He said \"hello\" to her.", &options);
        assert_eq!(result, "He said \"hello\" to her.");
    }

    #[test]
    fn test_curly_double_quotes_preserve_existing() {
        let options = default_options();
        let input = format!(
            "Already has {}curly{}",
            LEFT_DOUBLE_QUOTE, RIGHT_DOUBLE_QUOTE
        );
        let result = transform_punctuation(&input, &options);
        assert_eq!(result, input);
    }

    // ========== Single quotes tests ==========

    #[test]
    fn test_curly_single_quotes_basic() {
        let options = default_options();
        let result = transform_punctuation("She said 'hello' to him.", &options);
        assert_eq!(
            result,
            format!(
                "She said {}hello{} to him.",
                LEFT_SINGLE_QUOTE, RIGHT_SINGLE_QUOTE
            )
        );
    }

    #[test]
    fn test_curly_single_quotes_disabled() {
        let options = options_with(
            true,
            false,
            false,
            true,
            DashSetting::Disabled,
            DashSetting::Pattern("--".to_string()),
        );
        let result = transform_punctuation("She said 'hello' to him.", &options);
        assert_eq!(result, "She said 'hello' to him.");
    }

    #[test]
    fn test_curly_single_quotes_preserve_existing() {
        let options = default_options();
        let input = format!(
            "Already has {}curly{}",
            LEFT_SINGLE_QUOTE, RIGHT_SINGLE_QUOTE
        );
        let result = transform_punctuation(&input, &options);
        assert_eq!(result, input);
    }

    // ========== Apostrophe tests ==========

    #[test]
    fn test_curly_apostrophes_disabled_by_default() {
        let options = default_options();
        // Apostrophes within words are not transformed by default
        let result = transform_punctuation("it's a test", &options);
        assert_eq!(result, "it's a test");
    }

    #[test]
    fn test_leading_apostrophe_s_stays_straight() {
        // When text starts with 's (like after a link in parsed Markdown),
        // it should stay straight when curly_apostrophes is disabled
        let options = default_options();
        assert!(!options.curly_apostrophes);

        let result = transform_punctuation("'s API", &options);
        assert_eq!(result, "'s API");
    }

    #[test]
    fn test_leading_apostrophe_s_curly_when_enabled() {
        // When curly_apostrophes is enabled, leading 's should become curly
        let options = options_with(
            true,
            true,
            true, // curly_apostrophes enabled
            true,
            DashSetting::Disabled,
            DashSetting::Pattern("--".to_string()),
        );

        let result = transform_punctuation("'s API", &options);
        assert_eq!(result, format!("{}s API", RIGHT_SINGLE_QUOTE));
    }

    #[test]
    fn test_curly_apostrophes_enabled() {
        let options = options_with(
            true,
            true,
            true,
            true,
            DashSetting::Disabled,
            DashSetting::Pattern("--".to_string()),
        );
        let result = transform_punctuation("it's a test", &options);
        assert_eq!(result, format!("it{}s a test", RIGHT_SINGLE_QUOTE));
    }

    #[test]
    fn test_apostrophe_in_contraction() {
        let options = options_with(
            true,
            true,
            true,
            true,
            DashSetting::Disabled,
            DashSetting::Pattern("--".to_string()),
        );
        let result = transform_punctuation("don't do it", &options);
        assert_eq!(result, format!("don{}t do it", RIGHT_SINGLE_QUOTE));
    }

    #[test]
    fn test_apostrophe_decade_abbreviation() {
        let options = default_options();
        let result = transform_punctuation("the '80s were great", &options);
        // Decade abbreviation should use right single quote
        assert_eq!(result, format!("the {}80s were great", RIGHT_SINGLE_QUOTE));
    }

    #[test]
    fn test_word_final_apostrophe_disabled_by_default() {
        // Word-final apostrophes like goin', diggin' should remain straight
        // when curly_apostrophes is disabled (default)
        let options = default_options();
        let result = transform_punctuation("we're goin' to the store", &options);
        assert_eq!(result, "we're goin' to the store");
    }

    #[test]
    fn test_word_final_apostrophe_enabled() {
        // Word-final apostrophes should become curly when curly_apostrophes is enabled
        let options = options_with(
            true,
            true,
            true,
            true,
            DashSetting::Disabled,
            DashSetting::Pattern("--".to_string()),
        );
        let result = transform_punctuation("we're goin' to the store", &options);
        assert_eq!(
            result,
            format!(
                "we{}re goin{} to the store",
                RIGHT_SINGLE_QUOTE, RIGHT_SINGLE_QUOTE
            )
        );
    }

    #[test]
    fn test_possessive_apostrophe_disabled_by_default() {
        // Possessive apostrophes like GitHub's should remain straight by default
        let options = default_options();
        let result = transform_punctuation("GitHub's API is great", &options);
        assert_eq!(result, "GitHub's API is great");
    }

    #[test]
    fn test_possessive_apostrophe_enabled() {
        // Possessive apostrophes should become curly when curly_apostrophes is enabled
        let options = options_with(
            true,
            true,
            true,
            true,
            DashSetting::Disabled,
            DashSetting::Pattern("--".to_string()),
        );
        let result = transform_punctuation("GitHub's API is great", &options);
        assert_eq!(
            result,
            format!("GitHub{}s API is great", RIGHT_SINGLE_QUOTE)
        );
    }

    #[test]
    fn test_single_quotes_and_apostrophe_mixed() {
        // Test both single quotes as quotation marks and apostrophes in same text
        let options = default_options();
        let result = transform_punctuation("She said 'it's fine' to him", &options);
        assert_eq!(
            result,
            format!(
                "She said {}it's fine{} to him",
                LEFT_SINGLE_QUOTE, RIGHT_SINGLE_QUOTE
            )
        );
    }

    #[test]
    fn test_single_quotes_and_apostrophe_mixed_all_curly() {
        // Test with both curly_single_quotes and curly_apostrophes enabled
        let options = options_with(
            true,
            true,
            true,
            true,
            DashSetting::Disabled,
            DashSetting::Pattern("--".to_string()),
        );
        let result = transform_punctuation("She said 'it's fine' to him", &options);
        assert_eq!(
            result,
            format!(
                "She said {}it{}s fine{} to him",
                LEFT_SINGLE_QUOTE, RIGHT_SINGLE_QUOTE, RIGHT_SINGLE_QUOTE
            )
        );
    }

    #[test]
    fn test_curly_apostrophes_only_no_single_quotes() {
        // curly_apostrophes enabled but curly_single_quotes disabled
        let options = options_with(
            true,
            false,
            true,
            true,
            DashSetting::Disabled,
            DashSetting::Pattern("--".to_string()),
        );
        let result = transform_punctuation("She said 'it's fine' to him", &options);
        // Single quotes stay straight, but apostrophe becomes curly
        assert_eq!(
            result,
            format!("She said 'it{}s fine' to him", RIGHT_SINGLE_QUOTE)
        );
    }

    #[test]
    fn test_curly_single_quotes_only_no_apostrophes() {
        // curly_single_quotes enabled but curly_apostrophes disabled
        let options = options_with(
            true,
            true,
            false,
            true,
            DashSetting::Disabled,
            DashSetting::Pattern("--".to_string()),
        );
        let result = transform_punctuation("She said 'it's fine' to him", &options);
        // Single quotes become curly, but apostrophe stays straight
        assert_eq!(
            result,
            format!(
                "She said {}it's fine{} to him",
                LEFT_SINGLE_QUOTE, RIGHT_SINGLE_QUOTE
            )
        );
    }

    #[test]
    fn test_multiple_word_final_apostrophes() {
        // Multiple word-final apostrophes in same sentence
        let options = options_with(
            true,
            true,
            true,
            true,
            DashSetting::Disabled,
            DashSetting::Pattern("--".to_string()),
        );
        let result = transform_punctuation("We're goin' and they're comin' too", &options);
        assert_eq!(
            result,
            format!(
                "We{}re goin{} and they{}re comin{} too",
                RIGHT_SINGLE_QUOTE, RIGHT_SINGLE_QUOTE, RIGHT_SINGLE_QUOTE, RIGHT_SINGLE_QUOTE
            )
        );
    }

    #[test]
    fn test_apostrophe_at_end_of_sentence() {
        // Apostrophe at end of sentence (possessive at end)
        let options = options_with(
            true,
            true,
            true,
            true,
            DashSetting::Disabled,
            DashSetting::Pattern("--".to_string()),
        );
        let result = transform_punctuation("That's John's.", &options);
        assert_eq!(
            result,
            format!("That{}s John{}s.", RIGHT_SINGLE_QUOTE, RIGHT_SINGLE_QUOTE)
        );
    }

    #[test]
    fn test_decade_with_curly_single_quotes_disabled() {
        // Decade abbreviation with curly_single_quotes disabled
        // Should still convert to curly because it's handled specially
        let options = options_with(
            true,
            false,
            false,
            true,
            DashSetting::Disabled,
            DashSetting::Pattern("--".to_string()),
        );
        let result = transform_punctuation("the '80s and '90s", &options);
        // Even with single quotes disabled, decade abbreviations get curly
        assert_eq!(result, "the '80s and '90s");
    }

    #[test]
    fn test_decade_with_curly_single_quotes_enabled() {
        // Decade abbreviation with curly_single_quotes enabled
        let options = default_options();
        let result = transform_punctuation("music from the '80s", &options);
        assert_eq!(result, format!("music from the {}80s", RIGHT_SINGLE_QUOTE));
    }

    #[test]
    fn test_bracket_possessive_apostrophe_stays_straight() {
        // Possessive apostrophe after closing bracket should stay straight
        // when curly_apostrophes is disabled (default)
        let options = default_options();
        assert!(!options.curly_apostrophes); // Ensure default is false

        // Square bracket
        let result = transform_punctuation("[Fedify]'s API", &options);
        assert_eq!(result, "[Fedify]'s API");

        // Parenthesis
        let result = transform_punctuation("(something)'s value", &options);
        assert_eq!(result, "(something)'s value");

        // Curly brace
        let result = transform_punctuation("{item}'s property", &options);
        assert_eq!(result, "{item}'s property");
    }

    #[test]
    fn test_bracket_possessive_apostrophe_curly_when_enabled() {
        // Possessive apostrophe after closing bracket should become curly
        // when curly_apostrophes is enabled
        let options = options_with(
            true,
            true,
            true, // curly_apostrophes enabled
            true,
            DashSetting::Disabled,
            DashSetting::Pattern("--".to_string()),
        );

        let result = transform_punctuation("[Fedify]'s API", &options);
        assert_eq!(result, format!("[Fedify]{}s API", RIGHT_SINGLE_QUOTE));
    }

    // ========== Em-dash tests ==========

    #[test]
    fn test_em_dash_default_double_hyphen() {
        let options = default_options();
        let result = transform_punctuation("Hello--world", &options);
        assert_eq!(result, format!("Hello{}world", EM_DASH));
    }

    #[test]
    fn test_em_dash_triple_hyphen() {
        let options = options_with(
            true,
            true,
            false,
            true,
            DashSetting::Disabled,
            DashSetting::Pattern("---".to_string()),
        );
        let result = transform_punctuation("Hello---world", &options);
        assert_eq!(result, format!("Hello{}world", EM_DASH));
    }

    #[test]
    fn test_em_dash_single_hyphen_with_spaces() {
        let options = options_with(
            true,
            true,
            false,
            true,
            DashSetting::Disabled,
            DashSetting::Pattern("-".to_string()),
        );
        let result = transform_punctuation("word - word", &options);
        assert_eq!(result, format!("word {} word", EM_DASH));
    }

    #[test]
    fn test_em_dash_single_hyphen_without_spaces_no_change() {
        let options = options_with(
            true,
            true,
            false,
            true,
            DashSetting::Disabled,
            DashSetting::Pattern("-".to_string()),
        );
        let result = transform_punctuation("word-word", &options);
        // Single hyphen without surrounding spaces should not be transformed
        assert_eq!(result, "word-word");
    }

    #[test]
    fn test_em_dash_disabled() {
        let options = options_with(
            true,
            true,
            false,
            true,
            DashSetting::Disabled,
            DashSetting::Disabled,
        );
        let result = transform_punctuation("Hello--world", &options);
        assert_eq!(result, "Hello--world");
    }

    // ========== En-dash tests ==========

    #[test]
    fn test_en_dash_disabled_by_default() {
        let options = default_options();
        let result = transform_punctuation("pages 10-20", &options);
        // En-dash is disabled by default
        assert_eq!(result, "pages 10-20");
    }

    #[test]
    fn test_en_dash_enabled() {
        let options = options_with(
            true,
            true,
            false,
            true,
            DashSetting::Pattern("--".to_string()),
            DashSetting::Pattern("---".to_string()),
        );
        let result = transform_punctuation("pages 10--20", &options);
        assert_eq!(result, format!("pages 10{}20", EN_DASH));
    }

    #[test]
    fn test_en_and_em_dash_different_patterns() {
        let options = options_with(
            true,
            true,
            false,
            true,
            DashSetting::Pattern("--".to_string()),
            DashSetting::Pattern("---".to_string()),
        );
        let result = transform_punctuation("em---dash and en--dash", &options);
        assert_eq!(result, format!("em{}dash and en{}dash", EM_DASH, EN_DASH));
    }

    // ========== Validation tests ==========

    #[test]
    fn test_validate_conflicting_dash_patterns() {
        let options = options_with(
            true,
            true,
            false,
            true,
            DashSetting::Pattern("--".to_string()),
            DashSetting::Pattern("--".to_string()),
        );
        let result = validate_dash_settings(&options);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            PunctuationError::ConflictingDashPatterns("--".to_string())
        );
    }

    #[test]
    fn test_validate_non_conflicting_dash_patterns() {
        let options = options_with(
            true,
            true,
            false,
            true,
            DashSetting::Pattern("--".to_string()),
            DashSetting::Pattern("---".to_string()),
        );
        let result = validate_dash_settings(&options);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_disabled_dash_settings() {
        let options = options_with(
            true,
            true,
            false,
            true,
            DashSetting::Disabled,
            DashSetting::Disabled,
        );
        let result = validate_dash_settings(&options);
        assert!(result.is_ok());
    }

    // ========== Integration tests ==========

    #[test]
    fn test_all_punctuation_transforms_enabled() {
        let options = options_with(
            true,
            true,
            true,
            true,
            DashSetting::Pattern("--".to_string()),
            DashSetting::Pattern("---".to_string()),
        );
        let input = "He said \"It's... amazing---isn't it?\" she replied 'yes'";
        let result = transform_punctuation(input, &options);
        // Check that all transformations were applied
        assert!(result.contains(LEFT_DOUBLE_QUOTE));
        assert!(result.contains(RIGHT_DOUBLE_QUOTE));
        assert!(result.contains(LEFT_SINGLE_QUOTE));
        assert!(result.contains(RIGHT_SINGLE_QUOTE));
        assert!(result.contains(ELLIPSIS));
        assert!(result.contains(EM_DASH));
    }

    #[test]
    fn test_all_punctuation_transforms_disabled() {
        let options = options_with(
            false,
            false,
            false,
            false,
            DashSetting::Disabled,
            DashSetting::Disabled,
        );
        let input = "He said \"It's... amazing--isn't it?\" she replied 'yes'";
        let result = transform_punctuation(input, &options);
        // Nothing should be transformed
        assert_eq!(result, input);
    }
}
