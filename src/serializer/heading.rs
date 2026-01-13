// SPDX-FileCopyrightText: 2025 Hong Minhee <https://hongminhee.org/>
// SPDX-License-Identifier: GPL-3.0-or-later
//! Heading sentence case conversion.

// Include generated proper nouns constants
include!(concat!(env!("OUT_DIR"), "/proper_nouns_generated.rs"));

/// Convert heading text to sentence case.
///
/// This function applies intelligent heuristics to convert heading text:
/// - Capitalizes only the first word
/// - Preserves code spans (backticks)
/// - Preserves acronyms (2+ consecutive uppercase letters)
/// - Preserves proper nouns (built-in + user-configured, minus common_nouns)
/// - Handles compound words (hyphenated)
/// - Handles quoted text based on original capitalization
/// - Preserves non-Latin scripts (CJK, etc.)
pub fn to_sentence_case(
    text: &str,
    user_proper_nouns: &[String],
    common_nouns: &[String],
) -> String {
    if text.is_empty() {
        return String::new();
    }

    // Parse tokens FIRST (to identify code spans before normalizing quotes)
    // Then normalize quotes only in non-code-span parts
    let tokens = tokenize_with_code_spans(text);

    // Process each token
    let mut result = String::new();
    let mut is_first_word = true;

    for token in tokens {
        match token {
            Token::CodeSpan(content) => {
                result.push_str(&content);
            }
            Token::Quote(content, is_double) => {
                let processed =
                    process_quoted_text(&content, is_double, user_proper_nouns, common_nouns);
                result.push_str(&processed);
            }
            Token::Text(content) => {
                let processed = process_text(
                    &content,
                    &mut is_first_word,
                    user_proper_nouns,
                    common_nouns,
                );
                result.push_str(&processed);
            }
        }
    }

    result
}

/// Token types for parsing heading text.
#[derive(Debug, PartialEq)]
enum Token {
    /// Code span with backticks (preserved as-is)
    CodeSpan(String),
    /// Quoted text (content, is_double_quote)
    Quote(String, bool),
    /// Regular text
    Text(String),
}

/// Normalize straight quotes to curly quotes, following SmartyPants logic.
/// Apostrophes are kept as straight quotes.
/// This function applies transformations in the same order as SmartyPants:
/// 1. Special cases (decade abbreviations, etc.)
/// 2. Opening quotes
/// 3. Closing quotes (but keep apostrophes as straight quotes)
/// 4. Remaining quotes become opening quotes
fn normalize_quotes(text: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();

    // Track if we have an unmatched opening single quote
    let mut open_single_quote = false;

    let mut i = 0;
    while i < len {
        let ch = chars[i];

        if ch == '\'' {
            // Check for decade abbreviations: '80s, '90s
            if i + 3 < len
                && chars[i + 1].is_ascii_digit()
                && chars[i + 2].is_ascii_digit()
                && chars[i + 3] == 's'
            {
                // Check if at word boundary (start or after non-alphanumeric)
                let at_boundary = i == 0 || !chars[i - 1].is_alphanumeric();
                if at_boundary {
                    // Keep as straight quote (apostrophe)
                    result.push('\'');
                    i += 1;
                    continue;
                }
            }

            // Check for contractions: letter + ' + letter (let's, don't, it's)
            if i > 0 && chars[i - 1].is_alphabetic() {
                let next_is_letter = i + 1 < len && chars[i + 1].is_alphabetic();

                // Check for 's specifically (possessive: letter + 's + word boundary)
                let next_is_s_boundary = i + 1 < len
                    && chars[i + 1] == 's'
                    && (i + 2 == len || !chars[i + 2].is_alphanumeric());

                if next_is_letter || next_is_s_boundary {
                    // Keep as straight quote (apostrophe)
                    result.push('\'');
                    i += 1;
                    continue;
                }

                // Check for word-final apostrophe (goin', diggin')
                // Only if there's no unmatched opening quote
                let next_is_space = i + 1 < len && chars[i + 1].is_whitespace();
                if next_is_space && !open_single_quote {
                    // Keep as straight quote (apostrophe)
                    result.push('\'');
                    i += 1;
                    continue;
                }
            }

            // Check for opening quote: whitespace + ' + word character
            if i > 0
                && (chars[i - 1].is_whitespace() || chars[i - 1] == '(' || chars[i - 1] == '[')
                && i + 1 < len
                && chars[i + 1].is_alphanumeric()
            {
                result.push('\u{2018}');
                open_single_quote = true;
                i += 1;
                continue;
            }

            // Check for opening quote at start: start + ' + word character
            if i == 0 && i + 1 < len && chars[i + 1].is_alphanumeric() {
                result.push('\u{2018}');
                open_single_quote = true;
                i += 1;
                continue;
            }

            // Otherwise, it's a closing quote
            result.push('\u{2019}');
            open_single_quote = false;
        } else if ch == '"' {
            // Check for opening quote: whitespace + " + word character
            if i > 0
                && (chars[i - 1].is_whitespace() || chars[i - 1] == '(' || chars[i - 1] == '[')
                && i + 1 < len
                && chars[i + 1].is_alphanumeric()
            {
                result.push('\u{201C}');
                i += 1;
                continue;
            }

            // Check for opening quote at start
            if i == 0 && i + 1 < len && chars[i + 1].is_alphanumeric() {
                result.push('\u{201C}');
                i += 1;
                continue;
            }

            // Otherwise, it's a closing quote
            result.push('\u{201D}');
        } else {
            result.push(ch);
        }

        i += 1;
    }

    result
}

/// Tokenize text: first extract code spans, then normalize quotes and parse quotes in remaining text.
fn tokenize_with_code_spans(text: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '`' {
            // Save any accumulated text (normalize quotes in it)
            if !current.is_empty() {
                let normalized = normalize_quotes(&current);
                tokens.extend(tokenize_quotes(&normalized));
                current.clear();
            }

            // Collect code span (preserve as-is, no quote normalization)
            let mut code_span = String::from('`');
            for ch in chars.by_ref() {
                code_span.push(ch);
                if ch == '`' {
                    break;
                }
            }
            tokens.push(Token::CodeSpan(code_span));
        } else {
            current.push(ch);
        }
    }

    // Save remaining text (normalize quotes in it)
    if !current.is_empty() {
        let normalized = normalize_quotes(&current);
        tokens.extend(tokenize_quotes(&normalized));
    }

    tokens
}

/// Tokenize text that has already been quote-normalized, extracting quoted parts.
fn tokenize_quotes(text: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\u{201C}' {
            // Left double curly quote
            if !current.is_empty() {
                tokens.push(Token::Text(current.clone()));
                current.clear();
            }

            // Collect quoted content
            let mut quoted_content = String::new();
            let mut found_closing = false;
            for ch in chars.by_ref() {
                if ch == '\u{201D}' {
                    found_closing = true;
                    break;
                }
                quoted_content.push(ch);
            }

            if found_closing {
                tokens.push(Token::Quote(quoted_content, true));
            } else {
                // Unclosed quote - treat as regular text
                current.push('\u{201C}');
                current.push_str(&quoted_content);
            }
        } else if ch == '\u{2018}' {
            // Left single curly quote
            if !current.is_empty() {
                tokens.push(Token::Text(current.clone()));
                current.clear();
            }

            // Collect quoted content
            let mut quoted_content = String::new();
            let mut found_closing = false;
            for ch in chars.by_ref() {
                if ch == '\u{2019}' {
                    found_closing = true;
                    break;
                }
                quoted_content.push(ch);
            }

            if found_closing {
                tokens.push(Token::Quote(quoted_content, false));
            } else {
                // Unclosed quote - treat as regular text
                current.push('\u{2018}');
                current.push_str(&quoted_content);
            }
        } else {
            current.push(ch);
        }
    }

    // Save remaining text
    if !current.is_empty() {
        tokens.push(Token::Text(current));
    }

    tokens
}

/// Tokenize already-normalized text (for use inside quoted sections).
fn tokenize(text: &str) -> Vec<Token> {
    tokenize_quotes(text)
}

/// Process quoted text based on the capitalization of the first character.
fn process_quoted_text(
    content: &str,
    is_double: bool,
    user_proper_nouns: &[String],
    common_nouns: &[String],
) -> String {
    let opening = if is_double { "\u{201C}" } else { "\u{2018}" };
    let closing = if is_double { "\u{201D}" } else { "\u{2019}" };

    if content.is_empty() {
        return format!("{}{}", opening, closing);
    }

    // Check if first alphabetic character is uppercase
    let first_alpha = content.chars().find(|c| c.is_alphabetic());

    let processed = if let Some(first) = first_alpha {
        if first.is_uppercase() {
            // Apply sentence case inside quotes (including nested quotes)
            let tokens = tokenize(content);
            let mut result = String::new();
            let mut is_first_word = true;

            for token in tokens {
                match token {
                    Token::CodeSpan(c) => result.push_str(&c),
                    Token::Quote(c, is_dbl) => {
                        let processed =
                            process_quoted_text(&c, is_dbl, user_proper_nouns, common_nouns);
                        result.push_str(&processed);
                        is_first_word = false;
                    }
                    Token::Text(c) => {
                        let processed =
                            process_text(&c, &mut is_first_word, user_proper_nouns, common_nouns);
                        result.push_str(&processed);
                    }
                }
            }
            result
        } else {
            // Preserve as-is
            content.to_string()
        }
    } else {
        // No alphabetic characters
        content.to_string()
    };

    format!("{}{}{}", opening, processed, closing)
}

/// Collect all multi-word proper nouns (2+ words) from built-in and user lists.
/// Returns Vec of (canonical_form, lowercase_search_key).
/// Excludes any that appear in common_nouns.
fn collect_multiword_proper_nouns(
    user_proper_nouns: &[String],
    common_nouns: &[String],
) -> Vec<(String, String)> {
    let mut multiword_nouns = Vec::new();
    let common_nouns_lower: Vec<String> = common_nouns.iter().map(|s| s.to_lowercase()).collect();

    // Collect from built-in proper nouns
    for (canonical, _key) in PROPER_NOUNS {
        if canonical.contains(' ') {
            let lowercase_key = canonical.to_lowercase();
            if !common_nouns_lower.contains(&lowercase_key) {
                multiword_nouns.push((canonical.to_string(), lowercase_key));
            }
        }
    }

    // Collect from user proper nouns
    for noun in user_proper_nouns {
        if noun.contains(' ') {
            let lowercase_key = noun.to_lowercase();
            if !common_nouns_lower.contains(&lowercase_key) {
                multiword_nouns.push((noun.clone(), lowercase_key));
            }
        }
    }

    // Sort by length (longest first) to handle overlapping matches correctly
    multiword_nouns.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

    multiword_nouns
}

/// Normalize apostrophes for matching purposes.
/// Converts both straight (') and curly (') apostrophes to a canonical form.
fn normalize_apostrophes_for_matching(text: &str) -> String {
    text.replace('\u{2019}', "'")
}

/// Check if two strings match ignoring case and apostrophe style.
fn matches_ignoring_apostrophes(text: &str, pattern: &str) -> bool {
    let text_normalized = normalize_apostrophes_for_matching(&text.to_lowercase());
    let pattern_normalized = normalize_apostrophes_for_matching(&pattern.to_lowercase());
    text_normalized == pattern_normalized
}

/// Replace multi-word proper nouns with placeholders.
/// Returns (modified_text, replacements) where replacements is Vec of (placeholder, canonical_form).
fn replace_multiword_with_placeholders(
    text: &str,
    multiword_nouns: &[(String, String)],
) -> (String, Vec<(String, String)>) {
    let mut result = text.to_string();
    let mut replacements = Vec::new();
    let mut placeholder_counter = 0;

    for (canonical, search_key) in multiword_nouns {
        // Case-insensitive search with apostrophe-aware matching
        let mut search_from = 0;
        let search_key_len = search_key.chars().count();

        loop {
            let remaining = &result[search_from..];
            let mut found = false;
            let mut match_end_byte = search_from;

            // Scan through the text to find a match
            for (byte_pos, _) in remaining.char_indices() {
                let substring_start = search_from + byte_pos;

                // Try to extract a substring of the same character length as search_key
                let chars_from_here: Vec<char> = result[substring_start..]
                    .chars()
                    .take(search_key_len)
                    .collect();
                if chars_from_here.len() != search_key_len {
                    break;
                }

                let substring: String = chars_from_here.iter().collect();
                if matches_ignoring_apostrophes(&substring, search_key) {
                    let actual_pos = substring_start;
                    let mut end_pos = substring_start + substring.len();
                    match_end_byte = end_pos;

                    // Check word boundaries
                    let is_word_start = actual_pos == 0
                        || result[..actual_pos]
                            .chars()
                            .last()
                            .map(|c| !c.is_alphanumeric())
                            .unwrap_or(true);
                    let is_word_end = end_pos >= result.len()
                        || result[end_pos..]
                            .chars()
                            .next()
                            .map(|c| !c.is_alphanumeric())
                            .unwrap_or(true);

                    if is_word_start && is_word_end {
                        // Check if followed by possessive marker ('s or 's)
                        // If so, include it in the replacement to preserve proper noun
                        let remaining = &result[end_pos..];
                        if remaining.starts_with("'s") {
                            end_pos += 2;
                        } else if remaining.starts_with("\u{2019}s") {
                            end_pos += "\u{2019}s".len();
                        }

                        // Replace with placeholder (using only symbols to avoid case conversion)
                        let placeholder = format!("\u{FFFD}{}\u{FFFD}", placeholder_counter);

                        // Store the canonical form with possessive if present
                        let full_canonical = if end_pos > substring_start + substring.len() {
                            let possessive_suffix =
                                &result[substring_start + substring.len()..end_pos];
                            format!("{}{}", canonical, possessive_suffix)
                        } else {
                            canonical.clone()
                        };

                        replacements.push((placeholder.clone(), full_canonical));
                        placeholder_counter += 1;

                        result.replace_range(actual_pos..end_pos, &placeholder);
                        search_from = actual_pos + placeholder.len();
                        found = true;
                        break;
                    }
                }
            }

            if !found {
                if match_end_byte > search_from {
                    search_from = match_end_byte;
                } else {
                    break;
                }
            }
        }
    }

    (result, replacements)
}

/// Restore placeholders back to their original proper nouns.
fn restore_placeholders(text: &str, replacements: &[(String, String)]) -> String {
    let mut result = text.to_string();
    for (placeholder, canonical) in replacements {
        result = result.replace(placeholder, canonical);
    }
    result
}

/// Process regular text with sentence case rules.
fn process_text(
    text: &str,
    is_first_word: &mut bool,
    user_proper_nouns: &[String],
    common_nouns: &[String],
) -> String {
    // Step 1: Collect multi-word proper nouns
    let multiword_nouns = collect_multiword_proper_nouns(user_proper_nouns, common_nouns);

    // Step 2: Replace multi-word proper nouns with placeholders
    let (text_with_placeholders, replacements) =
        replace_multiword_with_placeholders(text, &multiword_nouns);

    // Step 3: Process text word-by-word (placeholders won't match any rules)
    let mut result = String::new();
    let mut current_word = String::new();
    let mut word_count = 0;
    let mut after_delimiter = false; // Track if we're after :, ;, or —

    for ch in text_with_placeholders.chars() {
        if ch.is_whitespace() {
            if !current_word.is_empty() {
                // Check if the current word starts with uppercase in original text
                let original_is_uppercase = current_word
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false);

                // Only the very first word (word_count == 0) gets first-word treatment
                // Words after delimiters preserve their original capitalization
                let should_capitalize = if word_count == 0 && *is_first_word {
                    true
                } else if after_delimiter {
                    original_is_uppercase
                } else {
                    false
                };

                let processed = process_word(
                    &current_word,
                    should_capitalize,
                    user_proper_nouns,
                    common_nouns,
                );
                result.push_str(&processed);
                current_word.clear();
                word_count += 1;
                *is_first_word = false;
                after_delimiter = false;
            }
            result.push(ch);
        } else {
            // Check if this is a delimiter that should trigger capitalization check
            if ch == ':' || ch == ';' || ch == '—' {
                if !current_word.is_empty() {
                    let processed = process_word(
                        &current_word,
                        word_count == 0 && *is_first_word,
                        user_proper_nouns,
                        common_nouns,
                    );
                    result.push_str(&processed);
                    current_word.clear();
                    word_count += 1;
                    *is_first_word = false;
                }
                result.push(ch);
                after_delimiter = true;
            } else {
                current_word.push(ch);
            }
        }
    }

    // Process remaining word
    if !current_word.is_empty() {
        let original_is_uppercase = current_word
            .chars()
            .next()
            .map(|c| c.is_uppercase())
            .unwrap_or(false);

        let should_capitalize = if word_count == 0 && *is_first_word {
            true
        } else if after_delimiter {
            original_is_uppercase
        } else {
            false
        };

        let processed = process_word(
            &current_word,
            should_capitalize,
            user_proper_nouns,
            common_nouns,
        );
        result.push_str(&processed);
        *is_first_word = false;
    }

    // Step 4: Restore placeholders back to original proper nouns
    restore_placeholders(&result, &replacements)
}

/// Process a single word according to sentence case rules.
fn process_word(
    word: &str,
    is_first: bool,
    user_proper_nouns: &[String],
    common_nouns: &[String],
) -> String {
    // Check if the whole word (including hyphens/slashes) is a proper noun.
    // This allows user-defined proper nouns like "@foo/javascript" or "my-custom-lib"
    // to be preserved as-is without splitting.
    if (word.contains('-') || word.contains('/'))
        && let Some(canonical) = find_proper_noun(word, user_proper_nouns, common_nouns)
    {
        return canonical;
    }

    // Handle hyphenated words
    if word.contains('-') {
        let parts: Vec<&str> = word.split('-').collect();
        let mut is_first_part = is_first;
        let processed_parts: Vec<String> = parts
            .into_iter()
            .map(|part| {
                let result =
                    process_word_simple(part, is_first_part, user_proper_nouns, common_nouns);
                is_first_part = false;
                result
            })
            .collect();
        return processed_parts.join("-");
    }

    // Handle slash-separated words
    if word.contains('/') {
        let parts: Vec<&str> = word.split('/').collect();
        let mut is_first_part = is_first;
        let processed_parts: Vec<String> = parts
            .into_iter()
            .map(|part| {
                let result =
                    process_word_simple(part, is_first_part, user_proper_nouns, common_nouns);
                is_first_part = false;
                result
            })
            .collect();
        return processed_parts.join("/");
    }

    process_word_simple(word, is_first, user_proper_nouns, common_nouns)
}

/// Process a simple (non-hyphenated) word.
fn process_word_simple(
    word: &str,
    is_first: bool,
    user_proper_nouns: &[String],
    common_nouns: &[String],
) -> String {
    if word.is_empty() {
        return String::new();
    }

    // Check if this is a placeholder for multi-word proper noun
    // Placeholders use replacement character (U+FFFD) as markers
    if word.contains('\u{FFFD}') {
        return word.to_string();
    }

    // Check if all alphabetic characters are uppercase (intentional emphasis)
    let alphabetic_chars: Vec<char> = word.chars().filter(|c| c.is_alphabetic()).collect();
    if !alphabetic_chars.is_empty() && alphabetic_chars.iter().all(|c| c.is_uppercase()) {
        return word.to_string();
    }

    // Check if it's an acronym (2+ consecutive uppercase letters at start)
    if is_acronym(word) {
        return word.to_string();
    }

    // Check if it's a proper noun (excluding common_nouns)
    if let Some(canonical) = find_proper_noun(word, user_proper_nouns, common_nouns) {
        return canonical;
    }

    // Apply sentence case rules
    if is_first {
        // First word: capitalize first letter, lowercase the rest
        capitalize_first(word)
    } else {
        // Other words: all lowercase
        word.to_lowercase()
    }
}

/// Check if a word is an acronym (2+ consecutive uppercase letters at the start).
fn is_acronym(word: &str) -> bool {
    let chars: Vec<char> = word.chars().collect();
    if chars.len() < 2 {
        return false;
    }

    // Check if first two characters are uppercase letters
    if chars[0].is_uppercase()
        && chars[0].is_alphabetic()
        && chars[1].is_uppercase()
        && chars[1].is_alphabetic()
    {
        return true;
    }

    // Check for acronyms with periods (e.g., "Ph.D.", "U.S.", "R.O.K.")
    // Pattern: uppercase letter followed by period, repeated at least twice
    let letters: Vec<char> = word.chars().filter(|c| c.is_alphabetic()).collect();
    if letters.len() < 2 {
        return false;
    }

    // Check if it matches the pattern: letter-period pairs
    // e.g., "Ph.D." -> "P" "h" "." "D" "."
    // We check if there are at least 2 uppercase letters with periods between/after them
    let mut uppercase_count = 0;
    let mut has_period = false;
    let mut i = 0;

    while i < chars.len() {
        if chars[i].is_alphabetic() {
            if chars[i].is_uppercase() {
                uppercase_count += 1;
            }
            // Check if next char is a period
            if i + 1 < chars.len() && chars[i + 1] == '.' {
                has_period = true;
                i += 2; // Skip the period
                continue;
            }
        }
        i += 1;
    }

    // If we have at least 2 letters (including at least 1 uppercase) and periods, treat as acronym
    has_period && letters.len() >= 2 && uppercase_count >= 1
}

/// Find a proper noun match (case-insensitive search).
/// Returns None if the word is in the common_nouns list.
/// Handles words with trailing punctuation (e.g., "France," matches "France").
/// Handles possessive forms (e.g., "GitHub's" matches "GitHub").
fn find_proper_noun(
    word: &str,
    user_proper_nouns: &[String],
    common_nouns: &[String],
) -> Option<String> {
    // Check for possessive form ('s or 's) and strip it temporarily
    let (core_word, possessive_suffix) = if let Some(stripped) = word.strip_suffix("'s") {
        (stripped, "'s")
    } else if let Some(stripped) = word.strip_suffix("\u{2019}s") {
        (stripped, "\u{2019}s")
    } else {
        (word, "")
    };

    // Strip remaining trailing punctuation to find the core word
    let core_word = core_word.trim_end_matches(|c: char| !c.is_alphanumeric());
    let trailing_punct = &word[core_word.len()..word.len() - possessive_suffix.len()];
    let core_word_lower = core_word.to_lowercase();

    // If no alphabetic characters remain, return None
    if core_word.is_empty() || !core_word.chars().any(|c| c.is_alphabetic()) {
        return None;
    }

    // Check if it's in the common_nouns list (case-insensitive)
    // If so, treat it as a common noun, not a proper noun
    for common_noun in common_nouns {
        if common_noun.to_lowercase() == core_word_lower {
            return None;
        }
    }

    // Check user proper nouns first
    for proper_noun in user_proper_nouns {
        if proper_noun.to_lowercase() == core_word_lower {
            return Some(format!(
                "{}{}{}",
                proper_noun, trailing_punct, possessive_suffix
            ));
        }
    }

    // Check built-in proper nouns (excluding those in common_nouns)
    for (canonical, key) in PROPER_NOUNS {
        if *key == core_word_lower {
            return Some(format!(
                "{}{}{}",
                canonical, trailing_punct, possessive_suffix
            ));
        }
    }

    None
}

/// Capitalize the first letter of a word.
fn capitalize_first(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => {
            let mut result = first.to_uppercase().to_string();
            result.push_str(&chars.as_str().to_lowercase());
            result
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_sentence_case() {
        assert_eq!(to_sentence_case("Hello World", &[], &[]), "Hello world");
    }

    #[test]
    fn test_all_lowercase() {
        assert_eq!(to_sentence_case("hello world", &[], &[]), "Hello world");
    }

    #[test]
    fn test_multiple_words() {
        assert_eq!(
            to_sentence_case("Development Commands", &[], &[]),
            "Development commands"
        );
    }

    #[test]
    fn test_preserve_code_spans() {
        assert_eq!(
            to_sentence_case("Using `MyAPI` Components", &[], &[]),
            "Using `MyAPI` components"
        );
    }

    #[test]
    fn test_preserve_acronyms() {
        // Acronyms (2+ consecutive uppercase at start) should be preserved
        assert_eq!(
            to_sentence_case("Using API Endpoints", &[], &[]),
            "Using API endpoints"
        );
        assert_eq!(
            to_sentence_case("HTTP And HTTPS", &[], &[]),
            "HTTP and HTTPS"
        );
    }

    #[test]
    fn test_preserve_acronyms_with_periods() {
        // Acronyms with periods should be preserved
        assert_eq!(
            to_sentence_case("U.S.A. And U.K.", &[], &[]),
            "U.S.A. and U.K."
        );
        assert_eq!(
            to_sentence_case("R.O.K. Development", &[], &[]),
            "R.O.K. development"
        );
        assert_eq!(
            to_sentence_case("P.R.C. And R.O.C.", &[], &[]),
            "P.R.C. and R.O.C."
        );
        assert_eq!(
            to_sentence_case("Using U.S. Military", &[], &[]),
            "Using U.S. military"
        );
    }

    #[test]
    fn test_preserve_academic_titles_with_periods() {
        // Academic titles with periods should be preserved
        assert_eq!(
            to_sentence_case("Ph.D. Programs", &[], &[]),
            "Ph.D. programs"
        );
        assert_eq!(
            to_sentence_case("M.D. And Ph.D.", &[], &[]),
            "M.D. and Ph.D."
        );
        assert_eq!(to_sentence_case("B.A. Degree", &[], &[]), "B.A. degree");
    }

    #[test]
    fn test_lowercase_latin_abbreviations_preserved() {
        // Latin abbreviations like "e.g." and "i.e." should stay lowercase
        assert_eq!(
            to_sentence_case("Using e.g. And i.e. In Text", &[], &[]),
            "Using e.g. and i.e. in text"
        );
        assert_eq!(
            to_sentence_case("Examples e.g. Foo", &[], &[]),
            "Examples e.g. foo"
        );
        assert_eq!(
            to_sentence_case("That Is i.e. This", &[], &[]),
            "That is i.e. this"
        );
        // Other common Latin abbreviations
        assert_eq!(
            to_sentence_case("See cf. Section 3", &[], &[]),
            "See cf. section 3"
        );
        assert_eq!(
            to_sentence_case("And etc. At End", &[], &[]),
            "And etc. at end"
        );
    }

    #[test]
    fn test_preserve_acronyms_with_suffix() {
        assert_eq!(
            to_sentence_case("Working With APIs", &[], &[]),
            "Working with APIs"
        );
    }

    #[test]
    fn test_hyphenated_acronyms() {
        assert_eq!(
            to_sentence_case("Working With JSON-RPC", &[], &[]),
            "Working with JSON-RPC"
        );
    }

    #[test]
    fn test_user_proper_nouns() {
        assert_eq!(
            to_sentence_case("Introduction To Hongdown", &["Hongdown".to_string()], &[]),
            "Introduction to Hongdown"
        );
    }

    #[test]
    fn test_builtin_proper_nouns() {
        // JavaScript is in the built-in proper nouns list
        assert_eq!(
            to_sentence_case("Working With JavaScript", &[], &[]),
            "Working with JavaScript"
        );
    }

    #[test]
    fn test_proper_noun_case_insensitive() {
        // Should match "github actions" and convert to "GitHub Actions"
        assert_eq!(
            to_sentence_case("Using Github Actions", &[], &[]),
            "Using GitHub Actions"
        );
    }

    #[test]
    fn test_hyphenated_with_proper_noun() {
        assert_eq!(
            to_sentence_case("React-based Application", &["React".to_string()], &[]),
            "React-based application"
        );
    }

    #[test]
    fn test_quoted_text_uppercase_first() {
        // First char inside quotes is uppercase - apply sentence case inside
        assert_eq!(
            to_sentence_case("Smart Suggestion: \"Did You Mean?\"", &[], &[]),
            "Smart suggestion: \u{201C}Did you mean?\u{201D}"
        );
    }

    #[test]
    fn test_quoted_text_lowercase_first() {
        // First char inside quotes is lowercase - preserve as-is
        assert_eq!(
            to_sentence_case("Smart Suggestion: \"did you mean?\"", &[], &[]),
            "Smart suggestion: \u{201C}did you mean?\u{201D}"
        );
    }

    #[test]
    fn test_curly_quotes() {
        assert_eq!(
            to_sentence_case("Message: \u{201C}Hello World\u{201D}", &[], &[]),
            "Message: \u{201C}Hello world\u{201D}"
        );
    }

    #[test]
    fn test_single_quotes() {
        assert_eq!(
            to_sentence_case("User Says 'Hello World'", &[], &[]),
            "User says \u{2018}Hello world\u{2019}"
        );
    }

    #[test]
    fn test_single_quotes_lowercase_first() {
        assert_eq!(
            to_sentence_case("User Says 'hello world'", &[], &[]),
            "User says \u{2018}hello world\u{2019}"
        );
    }

    #[test]
    fn test_nested_quotes() {
        assert_eq!(
            to_sentence_case("Message: \"He Said 'Hello World'\"", &[], &[]),
            "Message: \u{201C}He said \u{2018}Hello world\u{2019}\u{201D}"
        );
    }

    #[test]
    fn test_non_latin_scripts() {
        assert_eq!(
            to_sentence_case("한글 제목 With English", &[], &[]),
            "한글 제목 with English"
        );
    }

    #[test]
    fn test_all_caps_word_preserved() {
        // All caps words are intentional emphasis, keep as-is
        assert_eq!(
            to_sentence_case("IMPORTANT Notice", &[], &[]),
            "IMPORTANT notice"
        );
    }

    #[test]
    fn test_all_caps_proper_noun() {
        // Even if it's a proper noun, all caps means intentional
        assert_eq!(
            to_sentence_case("Working With JAVASCRIPT", &[], &[]),
            "Working with JAVASCRIPT"
        );
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(to_sentence_case("", &[], &[]), "");
    }

    #[test]
    fn test_single_word() {
        assert_eq!(to_sentence_case("Hello", &[], &[]), "Hello");
    }

    #[test]
    fn test_code_span_with_quotes() {
        // Code span takes precedence
        assert_eq!(
            to_sentence_case("Using `\"MyAPI\"`: \"The Best API\"", &[], &[]),
            "Using `\"MyAPI\"`: \u{201C}The best API\u{201D}"
        );
    }

    #[test]
    fn test_unclosed_quotes() {
        // Unclosed quotes are treated as regular text
        assert_eq!(
            to_sentence_case("Error: \"Something Went Wrong", &[], &[]),
            "Error: \u{201C}something went wrong"
        );
    }

    #[test]
    fn test_json_rpc_protocol() {
        assert_eq!(
            to_sentence_case("JSON-RPC Protocol", &[], &[]),
            "JSON-RPC protocol"
        );
    }

    #[test]
    fn test_mixed_case_not_acronym() {
        // "Api" is not an acronym (only first letter uppercase)
        assert_eq!(
            to_sentence_case("Working With Api", &[], &[]),
            "Working with api"
        );
    }

    #[test]
    fn test_common_nouns_exclude_builtin() {
        // "Python" is in built-in proper nouns, but excluded via common_nouns
        assert_eq!(
            to_sentence_case("Using Python Now", &[], &[]),
            "Using Python now"
        );
        assert_eq!(
            to_sentence_case("Using Python Now", &[], &["Python".to_string()]),
            "Using python now"
        );
    }

    #[test]
    fn test_common_nouns_case_insensitive() {
        // common_nouns matching is case-insensitive
        assert_eq!(
            to_sentence_case(
                "Using JavaScript Programming",
                &[],
                &["javascript".to_string()]
            ),
            "Using javascript programming"
        );
    }

    #[test]
    fn test_common_nouns_with_proper_nouns() {
        // User proper_nouns should still work even if other words are in common_nouns
        assert_eq!(
            to_sentence_case(
                "Using Python With MyAPI",
                &["MyAPI".to_string()],
                &["Python".to_string()]
            ),
            "Using python with MyAPI"
        );
    }

    #[test]
    fn test_common_nouns_multiple() {
        // Multiple words in common_nouns
        assert_eq!(
            to_sentence_case(
                "Learning JavaScript And Python",
                &[],
                &["JavaScript".to_string(), "Python".to_string()]
            ),
            "Learning javascript and python"
        );
    }

    #[test]
    fn test_apostrophe_not_quote() {
        // Apostrophes should remain as straight quotes, not curly quotes
        // This includes contractions (let's, don't) and word-final apostrophes (goin', diggin')
        assert_eq!(
            to_sentence_case("Let's Code With JavaScript", &[], &[]),
            "Let's code with JavaScript"
        );
        assert_eq!(
            to_sentence_case("It's Working Now", &[], &[]),
            "It's working now"
        );
        assert_eq!(
            to_sentence_case("Don't Use This", &[], &[]),
            "Don't use this"
        );
        // Word-final apostrophes (goin', diggin') should also be apostrophes, not curly quotes
        assert_eq!(
            to_sentence_case("Let's Code In JavaScript And Diggin' It", &[], &[]),
            "Let's code in JavaScript and diggin' it"
        );
        assert_eq!(
            to_sentence_case("We're Goin' To The Store", &[], &[]),
            "We're goin' to the store"
        );
        // Decade abbreviations
        assert_eq!(
            to_sentence_case("Music From The '80s And '90s", &[], &[]),
            "Music from the '80s and '90s"
        );
    }

    #[test]
    fn test_apostrophe_vs_single_quotes() {
        // Single quotes as quotation marks vs apostrophes
        assert_eq!(
            to_sentence_case("User Says 'Hello World'", &[], &[]),
            "User says \u{2018}Hello world\u{2019}"
        );
        assert_eq!(
            to_sentence_case("It's In The 'Quick Start' Guide", &[], &[]),
            "It's in the \u{2018}Quick start\u{2019} guide"
        );
    }

    #[test]
    fn test_multiword_proper_noun_github_actions() {
        // "GitHub Actions" is a multi-word proper noun
        assert_eq!(
            to_sentence_case("Using GitHub Actions For CI/CD", &[], &[]),
            "Using GitHub Actions for CI/CD"
        );
    }

    #[test]
    fn test_multiword_proper_noun_github_pages() {
        // "GitHub Pages" is a multi-word proper noun
        assert_eq!(
            to_sentence_case("Deploying To GitHub Pages", &[], &[]),
            "Deploying to GitHub Pages"
        );
    }

    #[test]
    fn test_multiword_proper_noun_codeberg_pages() {
        // "Codeberg Pages" is a multi-word proper noun
        assert_eq!(
            to_sentence_case("Hosting On Codeberg Pages", &[], &[]),
            "Hosting on Codeberg Pages"
        );
    }

    #[test]
    fn test_multiword_proper_noun_at_beginning() {
        // Multi-word proper noun at the beginning
        assert_eq!(
            to_sentence_case("GitHub Actions For CI/CD", &[], &[]),
            "GitHub Actions for CI/CD"
        );
    }

    #[test]
    fn test_multiword_proper_noun_case_insensitive() {
        // Case-insensitive matching for multi-word proper nouns
        assert_eq!(
            to_sentence_case("Using github actions For CI", &[], &[]),
            "Using GitHub Actions for CI"
        );
    }

    #[test]
    fn test_multiword_proper_noun_with_common_nouns() {
        // Multi-word proper noun excluded via common_nouns
        // Note: "GitHub" is still a proper noun on its own, so we need to exclude it too
        assert_eq!(
            to_sentence_case(
                "Using GitHub Actions For CI",
                &[],
                &["GitHub Actions".to_string(), "GitHub".to_string()]
            ),
            "Using github actions for CI"
        );
    }

    #[test]
    fn test_multiword_proper_noun_user_defined() {
        // User-defined multi-word proper noun
        assert_eq!(
            to_sentence_case(
                "Working With My Cool Project",
                &["My Cool Project".to_string()],
                &[]
            ),
            "Working with My Cool Project"
        );
    }

    #[test]
    fn test_multiword_proper_noun_multiple_instances() {
        // Multiple instances of the same multi-word proper noun
        assert_eq!(
            to_sentence_case("GitHub Actions And GitHub Pages Integration", &[], &[]),
            "GitHub Actions and GitHub Pages integration"
        );
    }

    #[test]
    fn test_multiword_proper_noun_with_punctuation() {
        // Multi-word proper noun followed by punctuation
        // Note: Text after colon is not treated as a new sentence
        assert_eq!(
            to_sentence_case("Using GitHub Actions: A Tutorial", &[], &[]),
            "Using GitHub Actions: A tutorial"
        );
    }

    #[test]
    fn test_single_word_github_preserved() {
        // Single-word "GitHub" should still be preserved
        assert_eq!(
            to_sentence_case("Using GitHub For Development", &[], &[]),
            "Using GitHub for development"
        );
    }

    #[test]
    fn test_multiword_proper_noun_partial_match_not_replaced() {
        // Partial match should not be replaced (word boundaries)
        assert_eq!(
            to_sentence_case("MyGitHub Actions Service", &[], &[]),
            "Mygithub actions service"
        );
    }

    #[test]
    fn test_natural_language_names_preserved() {
        // Natural language names should be preserved
        assert_eq!(
            to_sentence_case("Learning Korean And Japanese", &[], &[]),
            "Learning Korean and Japanese"
        );
        assert_eq!(
            to_sentence_case("Translating From English To French", &[], &[]),
            "Translating from English to French"
        );
        assert_eq!(
            to_sentence_case("Guide To Vietnamese And Thai", &[], &[]),
            "Guide to Vietnamese and Thai"
        );
    }

    #[test]
    fn test_country_names_preserved() {
        // Country names should be preserved
        assert_eq!(
            to_sentence_case("Traveling Through Japan And Korea", &[], &[]),
            "Traveling through Japan and Korea"
        );
        assert_eq!(
            to_sentence_case("From United States To Canada", &[], &[]),
            "From United States to Canada"
        );
        // Test individual country names first
        assert_eq!(
            to_sentence_case("Visiting France", &[], &[]),
            "Visiting France"
        );
        assert_eq!(
            to_sentence_case("Visiting Germany", &[], &[]),
            "Visiting Germany"
        );
        assert_eq!(
            to_sentence_case("Visiting Italy", &[], &[]),
            "Visiting Italy"
        );
        assert_eq!(
            to_sentence_case("Visiting France, Germany, And Italy", &[], &[]),
            "Visiting France, Germany, and Italy"
        );
    }

    #[test]
    fn test_multiword_country_names() {
        // Multi-word country names should be preserved
        assert_eq!(
            to_sentence_case("New Zealand Travel Guide", &[], &[]),
            "New Zealand travel guide"
        );
        assert_eq!(
            to_sentence_case("South Korea And North Korea", &[], &[]),
            "South Korea and North Korea"
        );
        assert_eq!(
            to_sentence_case("United Kingdom History", &[], &[]),
            "United Kingdom history"
        );
    }

    #[test]
    fn test_official_country_names() {
        // Official country names should be preserved
        assert_eq!(
            to_sentence_case("Republic Of Korea Development", &[], &[]),
            "Republic of Korea development"
        );
        // Test with straight apostrophe - should be normalized to curly and matched
        let result = to_sentence_case("People's Republic Of China", &[], &[]);
        println!("Result: {}", result);
        println!("Expected: People\u{2019}s Republic of China");
        assert_eq!(result, "People\u{2019}s Republic of China");
    }

    #[test]
    fn test_special_regions() {
        // Special administrative regions should be preserved
        assert_eq!(
            to_sentence_case("Hong Kong Travel Guide", &[], &[]),
            "Hong Kong travel guide"
        );
        assert_eq!(
            to_sentence_case("Visiting Macau And Hong Kong", &[], &[]),
            "Visiting Macau and Hong Kong"
        );
        assert_eq!(
            to_sentence_case("Puerto Rico History", &[], &[]),
            "Puerto Rico history"
        );
    }

    #[test]
    fn test_full_country_names_and_abbreviations() {
        // Full country names should be preserved
        assert_eq!(
            to_sentence_case("United States Of America History", &[], &[]),
            "United States of America history"
        );
        assert_eq!(
            to_sentence_case("Federal Republic Of Germany", &[], &[]),
            "Federal Republic of Germany"
        );
        assert_eq!(
            to_sentence_case("Russian Federation Development", &[], &[]),
            "Russian Federation development"
        );
        // Abbreviations should be preserved as acronyms
        assert_eq!(
            to_sentence_case("USA And UK Relations", &[], &[]),
            "USA and UK relations"
        );
        assert_eq!(
            to_sentence_case("ROK And DPRK Summit", &[], &[]),
            "ROK and DPRK summit"
        );
    }

    #[test]
    fn test_slash_separated_words() {
        // Words separated by slashes should be handled independently
        assert_eq!(
            to_sentence_case("Coding With JavaScript/TypeScript", &[], &[]),
            "Coding with JavaScript/TypeScript"
        );
        assert_eq!(
            to_sentence_case("Using Python/Ruby/Perl", &[], &[]),
            "Using Python/Ruby/Perl"
        );
        assert_eq!(
            to_sentence_case("Learning HTML/CSS/JavaScript", &[], &[]),
            "Learning HTML/CSS/JavaScript"
        );
        // Slash-separated with custom proper nouns
        assert_eq!(
            to_sentence_case(
                "Using Swift/Go",
                &["Swift".to_string(), "Go".to_string()],
                &[]
            ),
            "Using Swift/Go"
        );
    }

    #[test]
    fn test_continents_and_regions() {
        // Continent names should be preserved
        assert_eq!(
            to_sentence_case("Traveling Through Europe And Asia", &[], &[]),
            "Traveling through Europe and Asia"
        );
        assert_eq!(
            to_sentence_case("Exploring Africa And South America", &[], &[]),
            "Exploring Africa and South America"
        );
        // Regional adjectives should be preserved
        assert_eq!(
            to_sentence_case("European And Asian Cultures", &[], &[]),
            "European and Asian cultures"
        );
        assert_eq!(
            to_sentence_case("African And American History", &[], &[]),
            "African and American history"
        );
        // Sub-regions should be preserved
        assert_eq!(
            to_sentence_case("East Asian Languages", &[], &[]),
            "East Asian languages"
        );
        assert_eq!(
            to_sentence_case("Southeast Asian Cuisine", &[], &[]),
            "Southeast Asian cuisine"
        );
        assert_eq!(
            to_sentence_case("Western European Countries", &[], &[]),
            "Western European countries"
        );
        assert_eq!(
            to_sentence_case("Middle Eastern Politics", &[], &[]),
            "Middle Eastern politics"
        );
        assert_eq!(
            to_sentence_case("Latin American Music", &[], &[]),
            "Latin American music"
        );
    }

    #[test]
    fn test_punctuation_with_capitalization() {
        // Colon: preserve original capitalization after colon
        assert_eq!(to_sentence_case("Part 1: Food", &[], &[]), "Part 1: Food");
        assert_eq!(to_sentence_case("Part 1: food", &[], &[]), "Part 1: food");
        // Colon with proper nouns should preserve them
        assert_eq!(
            to_sentence_case("Part 1: East Asia", &[], &[]),
            "Part 1: East Asia"
        );
        assert_eq!(
            to_sentence_case("Chapter 2: JavaScript Basics", &[], &[]),
            "Chapter 2: JavaScript basics"
        );
        // Semicolon should work the same way
        assert_eq!(
            to_sentence_case("Note; This Is Important", &[], &[]),
            "Note; This is important"
        );
        assert_eq!(
            to_sentence_case("Warning; Python Required", &[], &[]),
            "Warning; Python required"
        );
        // Em dash should work the same way
        assert_eq!(
            to_sentence_case("Introduction—A Brief Overview", &[], &[]),
            "Introduction—A brief overview"
        );
        assert_eq!(
            to_sentence_case("Section 1—Europe And Asia", &[], &[]),
            "Section 1—Europe and Asia"
        );
    }

    #[test]
    fn test_possessive_proper_nouns() {
        // Possessive forms of proper nouns should work correctly
        assert_eq!(
            to_sentence_case(
                "Hong Minhee's Markdown Style Convention",
                &["Hong Minhee".to_string()],
                &[]
            ),
            "Hong Minhee's Markdown style convention"
        );
        assert_eq!(
            to_sentence_case("GitHub's Documentation System", &[], &[]),
            "GitHub's documentation system"
        );
        assert_eq!(
            to_sentence_case("Python's Features And JavaScript's Benefits", &[], &[]),
            "Python's features and JavaScript's benefits"
        );
    }

    #[test]
    fn test_slash_containing_user_proper_noun() {
        // User-defined proper noun containing slash should be preserved as-is
        assert_eq!(
            to_sentence_case(
                "Using @foo/javascript Package",
                &["@foo/javascript".to_string()],
                &[]
            ),
            "Using @foo/javascript package"
        );
        // Multiple slash-containing proper nouns
        assert_eq!(
            to_sentence_case(
                "Using @foo/bar And @baz/qux",
                &["@foo/bar".to_string(), "@baz/qux".to_string()],
                &[]
            ),
            "Using @foo/bar and @baz/qux"
        );
    }

    #[test]
    fn test_hyphen_containing_user_proper_noun() {
        // User-defined proper noun containing hyphen should be preserved as-is
        assert_eq!(
            to_sentence_case(
                "Using my-custom-lib Package",
                &["my-custom-lib".to_string()],
                &[]
            ),
            "Using my-custom-lib package"
        );
    }

    #[test]
    fn test_slash_proper_noun_not_matching_splits() {
        // Without user proper noun, slash-separated words are processed independently
        // "javascript" matches built-in "JavaScript"
        assert_eq!(
            to_sentence_case("Using @foo/javascript Package", &[], &[]),
            "Using @foo/JavaScript package"
        );
    }

    #[test]
    fn test_hyphen_proper_noun_not_matching_splits() {
        // Without user proper noun, hyphen-separated words are processed independently
        // "javascript" matches built-in "JavaScript"
        assert_eq!(
            to_sentence_case("Using foo-javascript Package", &[], &[]),
            "Using foo-JavaScript package"
        );
    }
}
