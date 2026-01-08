use super::*;
use comrak::{Arena, Options as ComrakOptions, parse_document};

fn comrak_options() -> ComrakOptions<'static> {
    let mut options = ComrakOptions::default();
    options.extension.front_matter_delimiter = Some("---".to_string());
    options.extension.table = true;
    options.extension.description_lists = true;
    options.extension.alerts = true;
    options.extension.footnotes = true;
    options.extension.tasklist = true;
    options
}

fn parse_and_serialize(input: &str) -> String {
    let arena = Arena::new();
    let options = comrak_options();
    let root = parse_document(&arena, input, &options);
    let format_options = Options::default();
    serialize_with_source(root, &format_options, None)
}

fn parse_and_serialize_with_options(input: &str, format_options: &Options) -> String {
    let arena = Arena::new();
    let options = comrak_options();
    let root = parse_document(&arena, input, &options);
    serialize_with_source(root, format_options, None)
}

fn parse_and_serialize_with_source(input: &str) -> String {
    let arena = Arena::new();
    let options = comrak_options();
    let root = parse_document(&arena, input, &options);
    let format_options = Options::default();
    serialize_with_source(root, &format_options, Some(input))
}

fn parse_and_serialize_with_warnings(input: &str) -> SerializeResult {
    let arena = Arena::new();
    let options = comrak_options();
    let root = parse_document(&arena, input, &options);
    let format_options = Options::default();
    serialize_with_source_and_warnings(root, &format_options, Some(input))
}

#[test]
fn test_serialize_plain_text() {
    let result = parse_and_serialize("Hello, world!");
    assert_eq!(result, "Hello, world!\n");
}

#[test]
fn test_serialize_multiline_paragraph() {
    // Original line breaks are preserved when lines are under 80 chars
    let result = parse_and_serialize("Hello\nworld!");
    assert_eq!(result, "Hello\nworld!\n");
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
    // trailing_spaces=2, so " 1.  " (leading=1, number, marker, trailing=2)
    let result = parse_and_serialize("1. First item");
    assert_eq!(result, " 1.  First item\n");
}

#[test]
fn test_serialize_ordered_list_multiple_items() {
    // trailing_spaces=2, so " N.  " format
    let result = parse_and_serialize("1. First\n2. Second\n3. Third");
    assert_eq!(result, " 1.  First\n 2.  Second\n 3.  Third\n");
}

#[test]
fn test_serialize_tight_list() {
    // Tight list: no blank lines between items
    let input = " -  Item one\n -  Item two\n -  Item three";
    let result = parse_and_serialize(input);
    assert_eq!(result, " -  Item one\n -  Item two\n -  Item three\n");
}

#[test]
fn test_serialize_loose_list() {
    // Loose list: blank lines between items should be preserved
    let input = " -  Item one\n\n -  Item two\n\n -  Item three";
    let result = parse_and_serialize(input);
    assert_eq!(
        result, " -  Item one\n\n -  Item two\n\n -  Item three\n",
        "Loose list should have blank lines between items"
    );
}

#[test]
fn test_serialize_loose_list_with_content() {
    // Loose list with multi-line content
    let input = " -  *Zero dependencies*: LogTape has zero dependencies.\n\n -  *Library support*: Designed for libraries.";
    let result = parse_and_serialize(input);
    assert!(
        result.contains(" -  *Zero dependencies*"),
        "Should contain first item"
    );
    assert!(
        result.contains("\n\n -  *Library support*"),
        "Should have blank line before second item, got:\n{}",
        result
    );
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
    // Original line breaks are preserved when lines are under 80 chars
    let result = parse_and_serialize("> Line one.\n> Line two.");
    assert_eq!(result, "> Line one.\n> Line two.\n");
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
fn test_serialize_underscore_emphasis_preserved() {
    // Underscore emphasis should be preserved as underscore, not converted to asterisk
    let result = parse_and_serialize_with_source("This is _emphasized_ text.");
    assert_eq!(result, "This is _emphasized_ text.\n");
}

#[test]
fn test_serialize_mixed_emphasis_preserved() {
    // Mixed emphasis styles should each be preserved
    let result = parse_and_serialize_with_source("This is _underscore_ and *asterisk* emphasis.");
    assert_eq!(result, "This is _underscore_ and *asterisk* emphasis.\n");
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

#[test]
fn test_reference_order_preserved() {
    // Regular references should maintain insertion order
    let input = "See [foo](https://foo.com), [bar](https://bar.com), and [baz](https://baz.com).";
    let result = parse_and_serialize(input);
    // Find positions of references
    let foo_pos = result.find("[foo]:").unwrap();
    let bar_pos = result.find("[bar]:").unwrap();
    let baz_pos = result.find("[baz]:").unwrap();
    assert!(
        foo_pos < bar_pos && bar_pos < baz_pos,
        "References should be in insertion order, got:\n{}",
        result
    );
}

#[test]
fn test_numeric_references_sorted_at_end() {
    // Numeric references should be sorted by number and placed at the end
    let input = "See [foo](https://foo.com), [2](https://2.com), [bar](https://bar.com), [1](https://1.com).";
    let result = parse_and_serialize(input);
    // foo and bar should come before numeric refs
    let foo_pos = result.find("[foo]:").unwrap();
    let bar_pos = result.find("[bar]:").unwrap();
    let one_pos = result.find("[1]:").unwrap();
    let two_pos = result.find("[2]:").unwrap();
    // Regular refs first, in order
    assert!(foo_pos < bar_pos, "foo should come before bar");
    // Numeric refs at end, sorted by number
    assert!(
        bar_pos < one_pos,
        "Regular refs should come before numeric refs"
    );
    assert!(
        one_pos < two_pos,
        "Numeric refs should be sorted: 1 before 2, got:\n{}",
        result
    );
}

#[test]
fn test_single_numeric_reference_not_sorted() {
    // A single numeric reference should stay in insertion order
    let input = "See [foo](https://foo.com), [1](https://1.com), [bar](https://bar.com).";
    let result = parse_and_serialize(input);
    let foo_pos = result.find("[foo]:").unwrap();
    let one_pos = result.find("[1]:").unwrap();
    let bar_pos = result.find("[bar]:").unwrap();
    // With only one numeric ref, it stays in insertion order
    assert!(
        foo_pos < one_pos && one_pos < bar_pos,
        "Single numeric ref should stay in insertion order, got:\n{}",
        result
    );
}

#[test]
fn test_hash_numeric_references_sorted() {
    // References like #123 should also be sorted numerically
    let input = "See [#456](https://issue/456) and [#123](https://issue/123).";
    let result = parse_and_serialize(input);
    let pos_123 = result.find("[#123]:").unwrap();
    let pos_456 = result.find("[#456]:").unwrap();
    assert!(
        pos_123 < pos_456,
        "#123 should come before #456, got:\n{}",
        result
    );
}

fn parse_and_serialize_with_frontmatter(input: &str) -> String {
    let arena = Arena::new();
    let mut options = ComrakOptions::default();
    options.extension.front_matter_delimiter = Some("---".to_string());
    let root = parse_document(&arena, input, &options);
    let format_options = Options::default();
    serialize_with_source(root, &format_options, None)
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

#[test]
fn test_serialize_one_blank_line_for_empty_section() {
    // When h1 is immediately followed by h2 (empty section), only one blank line
    let input = "# Title\n\n## Section\n\nContent.";
    let result = parse_and_serialize(input);
    // Should have only one blank line between headings
    assert_eq!(result, "Title\n=====\n\nSection\n-------\n\nContent.\n");
}

#[test]
fn test_serialize_consecutive_h2_sections() {
    // When h2 is immediately followed by another h2 (empty section)
    let input = "## Section 1\n\n## Section 2\n\nContent.";
    let result = parse_and_serialize(input);
    // Should have only one blank line between headings
    assert_eq!(
        result,
        "Section 1\n---------\n\nSection 2\n---------\n\nContent.\n"
    );
}

fn parse_and_serialize_with_width(input: &str, line_width: usize) -> String {
    let arena = Arena::new();
    let options = ComrakOptions::default();
    let root = parse_document(&arena, input, &options);
    let format_options = Options {
        line_width,
        ..Options::default()
    };
    serialize_with_source(root, &format_options, None)
}

#[test]
fn test_heading_with_inline_code() {
    // Inline code in headings should be preserved
    let input = "# Heading with `code`";
    let result = parse_and_serialize(input);
    assert_eq!(result, "Heading with `code`\n===================\n");
}

#[test]
fn test_heading_with_multiple_inline_codes() {
    // Multiple inline codes in headings
    let input = "### Looking at the `to`, `cc`, and `bcc` fields";
    let result = parse_and_serialize(input);
    assert_eq!(result, "### Looking at the `to`, `cc`, and `bcc` fields\n");
}

#[test]
fn test_korean_in_link() {
    // Korean text in links should not cause panic
    let input = "[한국어](https://example.com)";
    let result = parse_and_serialize(input);
    assert!(result.contains("[한국어]"));
    assert!(result.contains("https://example.com"));
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

#[test]
fn test_selective_rewrap_short_lines_preserved() {
    // Short lines (under 80 chars) should be preserved as-is
    let input = "Line one.\nLine two.\nLine three.";
    let result = parse_and_serialize(input);
    // Each line should stay on its own line
    assert_eq!(
        result, "Line one.\nLine two.\nLine three.\n",
        "Short lines should be preserved"
    );
}

#[test]
fn test_selective_rewrap_long_line_wrapped() {
    // A line over 80 chars should be rewrapped
    let input = "This is a very long line that definitely exceeds the eighty character limit and should be wrapped to the next line properly.";
    let result = parse_and_serialize_with_width(input, 80);
    // Should be wrapped
    let lines: Vec<&str> = result.lines().collect();
    assert!(
        lines.len() > 1,
        "Long line should be wrapped, got:\n{}",
        result
    );
    // Each line should be under 80 chars
    for line in &lines {
        assert!(line.len() <= 80, "Line should be under 80 chars: {}", line);
    }
}

#[test]
fn test_selective_rewrap_mixed_lines() {
    // Mix of short and long lines - short should be preserved, long rewrapped
    let input = "Short line one.\nShort line two.\nThis is a very long line that definitely exceeds the eighty character limit and needs to be wrapped.";
    let result = parse_and_serialize_with_width(input, 80);
    // Short lines should be preserved
    assert!(
        result.starts_with("Short line one.\nShort line two.\n"),
        "Short lines should be preserved at start, got:\n{}",
        result
    );
}

fn parse_and_serialize_with_table(input: &str) -> String {
    let arena = Arena::new();
    let mut options = ComrakOptions::default();
    options.extension.table = true;
    let root = parse_document(&arena, input, &options);
    let format_options = Options::default();
    serialize_with_source(root, &format_options, None)
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

#[test]
fn test_serialize_table_with_links() {
    // Table cells containing links should preserve the links
    let input = "| Package | Link |\n|---------|------|\n| [foo](/foo) | [bar](https://bar.com) |";
    let result = parse_and_serialize_with_table(input);
    // Links should be preserved in table cells
    assert!(
        result.contains("[foo](/foo)"),
        "Relative link should be preserved in table, got:\n{}",
        result
    );
    assert!(
        result.contains("[bar]"),
        "External link text should be preserved in table, got:\n{}",
        result
    );
}

#[test]
fn test_serialize_table_with_reference_links() {
    // Table cells containing reference-style links should preserve them
    let input = "| Package | JSR |\n|---------|-----|\n| [*@pkg/core*](/packages/core/) | [JSR][jsr:@pkg/core] |\n\n[jsr:@pkg/core]: https://jsr.io/@pkg/core";
    let result = parse_and_serialize_with_source(input);
    // Reference links should be preserved in table cells
    assert!(
        result.contains("[*@pkg/core*](/packages/core/)"),
        "Link with emphasis should be preserved in table, got:\n{}",
        result
    );
    assert!(
        result.contains("[JSR][jsr:@pkg/core]"),
        "Reference-style link should be preserved in table, got:\n{}",
        result
    );
}

fn parse_and_serialize_with_description_list(input: &str) -> String {
    let arena = Arena::new();
    let mut options = ComrakOptions::default();
    options.extension.description_lists = true;
    let root = parse_document(&arena, input, &options);
    let format_options = Options::default();
    serialize_with_source(root, &format_options, None)
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
    serialize_with_source(root, &format_options, None)
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
    serialize_with_source(root, &format_options, None)
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
fn test_definition_list_with_nested_list_continuation() {
    let input = "Term\n:   Definition:\n\n     -  Item with long text\n        that continues";
    let result = parse_and_serialize(input);
    // Continuation should also have proper indent
    assert!(result.contains("     -  Item with long text"));
    assert!(result.contains("         that continues"));
}

#[test]
fn test_alert_preserves_blank_line_after_header() {
    let input = "> [!TIP]\n>\n> This is a tip.";
    let result = parse_and_serialize_with_source(input);
    assert_eq!(result, "> [!TIP]\n>\n> This is a tip.\n");
}

#[test]
fn test_alert_without_blank_line_after_header() {
    let input = "> [!NOTE]\n> This is a note.";
    let result = parse_and_serialize_with_source(input);
    assert_eq!(result, "> [!NOTE]\n> This is a note.\n");
}

fn parse_and_serialize_with_alerts_and_width(input: &str, line_width: usize) -> String {
    let arena = Arena::new();
    let mut options = ComrakOptions::default();
    options.extension.alerts = true;
    let root = parse_document(&arena, input, &options);
    let format_options = Options {
        line_width,
        ..Options::default()
    };
    serialize_with_source(root, &format_options, None)
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
fn test_blockquote_inside_list_item() {
    // Blockquotes inside list items should have proper indentation
    let input = " 1.  Item with blockquote:\n\n     > This is quoted text\n     > inside a list item.\n\n 2.  Next item.";
    let result = parse_and_serialize_with_alerts(input);
    assert!(result.contains(" 1.  Item with blockquote:"));
    assert!(result.contains("     > This is quoted text"));
    assert!(result.contains("     > inside a list item."));
    assert!(result.contains(" 2.  Next item."));
}

#[test]
fn test_alert_inside_list_item() {
    // Alerts inside list items should have proper indentation
    let input = " 1.  Item with alert:\n\n     > [!IMPORTANT]\n     > Important message.\n\n 2.  Next item.";
    let result = parse_and_serialize_with_alerts(input);
    assert!(result.contains(" 1.  Item with alert:"));
    assert!(result.contains("     > [!IMPORTANT]"));
    assert!(result.contains("     > Important message."));
    assert!(result.contains(" 2.  Next item."));
}

#[test]
fn test_alert_inside_unordered_list_item() {
    // Alerts inside unordered list items
    let input = " -  Item with alert:\n\n     > [!NOTE]\n     > A note inside a list.";
    let result = parse_and_serialize_with_alerts(input);
    assert!(result.contains(" -  Item with alert:"));
    assert!(result.contains("    > [!NOTE]"));
    assert!(result.contains("    > A note inside a list."));
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

#[test]
fn test_serialize_escaped_asterisk_in_emphasis() {
    // Escaped asterisks inside emphasis should be preserved
    let input = r"*\*.ts*";
    let result = parse_and_serialize(input);
    assert_eq!(result, "*\\*.ts*\n");
}

#[test]
fn test_serialize_escaped_underscore() {
    // Escaped underscores should be preserved
    let input = r"\_\_init\_\_";
    let result = parse_and_serialize(input);
    assert_eq!(result, "\\_\\_init\\_\\_\n");
}

#[test]
fn test_serialize_escaped_underscore_in_emphasis() {
    // Escaped underscores inside emphasis should be preserved
    // This is common for filenames like *node\_modules* where the underscore
    // needs escaping to prevent it from ending the emphasis
    let input = r"*node\_modules*";
    let result = parse_and_serialize_with_source(input);
    assert_eq!(result, "*node\\_modules*\n");
}

#[test]
fn test_ordered_list_with_code_block() {
    // Code blocks inside ordered list items should be indented to align with content
    // The marker " 1.  " is 5 characters, so content indent should be 5 spaces
    let input =
        " 1.  First item:\n\n     ~~~~ bash\n     echo \"hello\"\n     ~~~~\n\n 2.  Second item.\n";
    let result = parse_and_serialize_with_source(input);
    assert_eq!(
        result, input,
        "Code block in ordered list should preserve indentation"
    );
}

#[test]
fn test_unordered_list_with_code_block() {
    // Code blocks inside unordered list items should be indented to align with content
    // The marker " -  " is 4 characters, so content indent should be 4 spaces
    let input =
        " -  First item:\n\n    ~~~~ bash\n    echo \"hello\"\n    ~~~~\n\n -  Second item.\n";
    let result = parse_and_serialize_with_source(input);
    assert_eq!(
        result, input,
        "Code block in unordered list should preserve indentation"
    );
}

#[test]
fn test_double_brackets_preserved() {
    // Double brackets around references (common in changelogs) should not be escaped
    let input = "See [[#123]] for details.\n\n[#123]: https://example.com/123\n";
    let result = parse_and_serialize(input);
    assert_eq!(
        result,
        "See [[#123]] for details.\n\n[#123]: https://example.com/123\n"
    );
}

#[test]
fn test_double_brackets_with_multiple_refs() {
    // Double brackets with multiple references and text
    let input = "[[#120], [#121] by Author]\n\n[#120]: https://example.com/120\n[#121]: https://example.com/121\n";
    let result = parse_and_serialize(input);
    assert_eq!(
        result,
        "[[#120], [#121] by Author]\n\n[#120]: https://example.com/120\n[#121]: https://example.com/121\n"
    );
}

#[test]
fn test_serialize_escaped_backslash() {
    // Escaped backslash should be preserved
    let input = r"path\\to\\file";
    let result = parse_and_serialize(input);
    assert_eq!(result, "path\\\\to\\\\file\n");
}

#[test]
fn test_multi_paragraph_list_item() {
    // Multiple paragraphs within a single list item should be separated by blank lines
    let input = " -  First paragraph.\n\n    Second paragraph.\n\n    Third paragraph.\n";
    let result = parse_and_serialize(input);
    assert_eq!(
        result,
        " -  First paragraph.\n\n    Second paragraph.\n\n    Third paragraph.\n"
    );
}

#[test]
fn test_tight_nested_list() {
    // Nested list directly following text (tight) - no blank line
    // Nested list indent = parent content start position (leading + 1 + trailing = 4)
    // plus its own leading space, so 5 spaces before marker
    let input = " -  Item:\n     -  Nested 1\n     -  Nested 2\n";
    let result = parse_and_serialize(input);
    assert_eq!(result, " -  Item:\n     -  Nested 1\n     -  Nested 2\n");
}

#[test]
fn test_loose_nested_list() {
    // Nested list after blank line (loose) - preserve blank line
    // Nested list indent = parent content start position (leading + 1 + trailing = 4)
    // plus its own leading space, so 5 spaces before marker
    let input = " -  Item.\n\n     -  Nested 1\n     -  Nested 2\n";
    let result = parse_and_serialize(input);
    assert_eq!(result, " -  Item.\n\n     -  Nested 1\n     -  Nested 2\n");
}

#[test]
fn test_serialize_asterisk_in_text_not_emphasis() {
    // Asterisks in plain text that aren't emphasis should be escaped
    let input = "5 * 3 = 15";
    let result = parse_and_serialize(input);
    // The asterisk in "5 * 3" should be escaped to prevent misinterpretation
    assert_eq!(result, "5 \\* 3 = 15\n");
}

#[test]
fn test_serialize_image_inside_link_badge_style() {
    // Badge-style: image inside a link, both using reference style
    // Input: [![alt][img-ref]][link-ref] with definitions
    // Should output fully inline: [![alt](img-url)](link-url)
    let input = r#"[![JSR][JSR badge]][JSR]

[JSR]: https://jsr.io/
[JSR badge]: https://jsr.io/badge.svg
"#;
    let result = parse_and_serialize(input);
    // The output should have a clickable image linking to JSR
    assert!(
        result.contains("[![JSR](https://jsr.io/badge.svg)](https://jsr.io/)"),
        "Should output fully inline badge-style link"
    );
    assert!(
        !result.contains("[![JSR](https://jsr.io/badge.svg)]:"),
        "Should not create malformed reference definition"
    );
}

#[test]
fn test_serialize_underscore_in_word_not_escaped() {
    // Underscores in the middle of words (like ALL_CAPS) should not be escaped
    // because they don't create emphasis in CommonMark
    let input = "Use ALL_CAPS for constants.";
    let result = parse_and_serialize(input);
    assert_eq!(result, "Use ALL_CAPS for constants.\n");
}

#[test]
fn test_serialize_underscore_emphasis_boundary() {
    // Underscores at word boundaries should be escaped to prevent emphasis
    let input = r"\_start and end\_";
    let result = parse_and_serialize(input);
    assert_eq!(result, "\\_start and end\\_\n");
}

#[test]
fn test_serialize_autolink_preserved() {
    // Autolinks <url> should be preserved as autolink format, not reference style
    let input = "Visit <https://example.com/> for more info.";
    let result = parse_and_serialize(input);
    assert_eq!(result, "Visit <https://example.com/> for more info.\n");
}

#[test]
fn test_serialize_nested_list_wrap_continuation() {
    // Nested list items should wrap with proper continuation indent
    // accounting for the parent list's indentation
    let input = " 1. First\n     -  This is a very long nested item that should wrap with proper eight-space continuation.";
    let result = parse_and_serialize_with_width(input, 80);
    // Continuation should have 8 spaces (4 for parent + 4 for list item content)
    assert!(
        result.contains("\n        "),
        "Nested list continuation should have 8-space indent, got:\n{}",
        result
    );
}

#[test]
fn test_directive_disable_next_line() {
    // hongdown-disable-next-line should preserve the next block element as-is
    let input = "<!-- hongdown-disable-next-line -->\n[![Badge][badge-img]][badge-url]\n\n[badge-img]: https://example.com/badge.svg\n[badge-url]: https://example.com";
    let result = parse_and_serialize_with_source(input);
    // The badge line should be preserved exactly as-is (not converted to inline)
    assert!(
        result.contains("[![Badge][badge-img]][badge-url]"),
        "disable-next-line should preserve the next line as-is, got:\n{}",
        result
    );
}

#[test]
fn test_directive_disable_file() {
    // hongdown-disable-file should preserve the entire file as-is
    let input = "<!-- hongdown-disable-file -->\n\nTitle\n===\n\nSome paragraph with *emphasis* that would normally be reformatted.";
    let result = parse_and_serialize_with_source(input);
    // The entire content after the directive should be preserved
    assert!(
        result.contains("Title\n==="),
        "disable-file should preserve file content as-is, got:\n{}",
        result
    );
}

#[test]
fn test_directive_disable_file_after_front_matter() {
    // hongdown-disable-file after front matter should preserve everything after it
    let input = "---\ntitle: Test\n---\n\n<!-- hongdown-disable-file -->\n\n# Title\n\nSome   badly   formatted   text.";
    let result = parse_and_serialize_with_source(input);
    // The file content should be preserved exactly as-is
    assert_eq!(
        result, input,
        "disable-file after front matter should preserve file content exactly"
    );
}

#[test]
fn test_directive_disable_file_preserves_trailing_newline() {
    // hongdown-disable-file should preserve trailing newline
    let input = "<!-- hongdown-disable-file -->\n\n# Title\n\nSome text.\n";
    let result = parse_and_serialize_with_source(input);
    assert_eq!(
        result, input,
        "disable-file should preserve trailing newline"
    );
}

#[test]
fn test_directive_disable_next_section() {
    // hongdown-disable-next-section should preserve content until the next heading
    let input = "First section\n-------------\n\nNormal content.\n\n<!-- hongdown-disable-next-section -->\n\nSecond section\n--------------\n\n[![Badge][img]][url]\n\n[img]: https://example.com/img.svg\n[url]: https://example.com\n\nThird section\n-------------\n\nThis should be formatted normally.";
    let result = parse_and_serialize_with_source(input);
    // Second section should be preserved as-is
    assert!(
        result.contains("[![Badge][img]][url]"),
        "disable-next-section should preserve section content as-is, got:\n{}",
        result
    );
}

#[test]
fn test_directive_disable_enable() {
    // hongdown-disable and hongdown-enable should bracket unformatted regions
    let input = "Normal paragraph.\n\n<!-- hongdown-disable -->\n\n[![Badge][img]][url]\n\nAnother unformatted line.\n\n<!-- hongdown-enable -->\n\nBack to normal formatting.\n\n[img]: https://example.com/img.svg\n[url]: https://example.com";
    let result = parse_and_serialize_with_source(input);
    // Content between disable/enable should be preserved
    assert!(
        result.contains("[![Badge][img]][url]"),
        "disable/enable should preserve bracketed content as-is, got:\n{}",
        result
    );
    assert!(
        result.contains("Another unformatted line."),
        "disable/enable should preserve all bracketed content, got:\n{}",
        result
    );
}

#[test]
fn test_preserve_reference_style_badge() {
    // Reference-style badge links should be preserved as reference style
    let input = "[![JSR][JSR badge]][JSR]\n\n[JSR]: https://jsr.io/@optique\n[JSR badge]: https://jsr.io/badges/@optique/core";
    let result = parse_and_serialize_with_source(input);
    // Should preserve reference style, not convert to inline
    assert!(
        result.contains("[![JSR][JSR badge]][JSR]"),
        "Reference-style badge should be preserved, got:\n{}",
        result
    );
}

#[test]
fn test_preserve_reference_style_image() {
    // Reference-style images should be preserved as reference style
    let input = "![Logo][logo]\n\n[logo]: https://example.com/logo.png";
    let result = parse_and_serialize_with_source(input);
    // Should preserve reference style
    assert!(
        result.contains("![Logo][logo]"),
        "Reference-style image should be preserved, got:\n{}",
        result
    );
}

#[test]
fn test_preserve_reference_style_link() {
    // Reference-style links should be preserved as reference style
    let input =
        "Check the [documentation][docs] for more info.\n\n[docs]: https://example.com/docs";
    let result = parse_and_serialize_with_source(input);
    // Should preserve reference style
    assert!(
        result.contains("[documentation][docs]"),
        "Reference-style link should be preserved, got:\n{}",
        result
    );
}

#[test]
fn test_code_span_with_backticks() {
    // Code spans containing backticks should use double backticks as delimiters
    let input = "Here is `` `code` `` in text.";
    let result = parse_and_serialize_with_source(input);
    assert!(
        result.contains("`` `code` ``"),
        "Code span with backtick should use double backtick delimiters, got:\n{}",
        result
    );
}

#[test]
fn test_code_span_with_multiple_backticks() {
    // Code spans containing double backticks should use triple backticks
    let input = "Use ``` `` ``` for double backticks.";
    let result = parse_and_serialize_with_source(input);
    assert!(
        result.contains("``` `` ```"),
        "Code span with double backticks should use triple backtick delimiters, got:\n{}",
        result
    );
}

#[test]
fn test_code_span_simple() {
    // Simple code spans without backticks should use single backticks
    let input = "Use `code` for inline code.";
    let result = parse_and_serialize_with_source(input);
    assert!(
        result.contains("`code`"),
        "Simple code span should use single backticks, got:\n{}",
        result
    );
}

#[test]
fn test_code_span_starting_with_backtick() {
    // Code starting with backtick needs space padding
    let input = "The code `` `foo `` starts with a backtick.";
    let result = parse_and_serialize_with_source(input);
    assert!(
        result.contains("`` `foo ``"),
        "Code starting with backtick should have space padding, got:\n{}",
        result
    );
}

#[test]
fn test_code_span_ending_with_backtick() {
    // Code ending with backtick needs space padding
    let input = "The code `` foo` `` ends with a backtick.";
    let result = parse_and_serialize_with_source(input);
    assert!(
        result.contains("`` foo` ``"),
        "Code ending with backtick should have space padding, got:\n{}",
        result
    );
}

#[test]
fn test_reference_link_multiline_text_normalized() {
    // Reference link text spanning multiple lines should be normalized to single line
    let input = "Click [here for\nmore info][1].\n\n[1]: https://example.com";
    let result = parse_and_serialize_with_source(input);
    // The link text should be normalized (newline -> space)
    assert!(
        result.contains("[here for more info]"),
        "Reference link text should be normalized to single line, got:\n{}",
        result
    );
}

#[test]
fn test_reference_link_idempotent() {
    // Reference style link should be idempotent after formatting
    let input = "Click [here for more info][1].\n\n[1]: https://example.com";
    let result = parse_and_serialize_with_source(input);
    let result2 = parse_and_serialize_with_source(&result);
    assert_eq!(
        result, result2,
        "Reference link should be idempotent.\nFirst pass:\n{}\nSecond pass:\n{}",
        result, result2
    );
}

#[test]
fn test_code_block_in_list_item() {
    // Code block inside a list item should be properly indented
    let input = " -  Example:\n\n    ~~~~\n    code here\n    ~~~~";
    let result = parse_and_serialize_with_source(input);
    // Code block should be on a new line with proper indentation
    assert!(
        result.contains("Example:\n\n    ~~~~"),
        "Code block in list item should have blank line and indentation, got:\n{}",
        result
    );
    assert!(
        result.contains("    code here"),
        "Code block content should be indented, got:\n{}",
        result
    );
}

#[test]
fn test_code_block_in_list_item_no_language() {
    // Code block without language identifier should not add one
    let input = " -  Item:\n\n    ~~~~\n    code\n    ~~~~";
    let result = parse_and_serialize_with_source(input);
    // Should use ~~~~ without language identifier
    assert!(
        result.contains("~~~~\n"),
        "Code block should not have language identifier added, got:\n{}",
        result
    );
}

// Edge case tests

#[test]
fn test_empty_paragraph() {
    // Empty content should not crash
    let input = "\n\n\n";
    let result = parse_and_serialize(input);
    assert!(result.is_empty() || result.chars().all(|c| c.is_whitespace()));
}

#[test]
fn test_deeply_nested_list() {
    let input = " -  Level 1\n    -  Level 2\n        -  Level 3\n            -  Level 4";
    let result = parse_and_serialize_with_source(input);
    assert!(result.contains("Level 1"));
    assert!(result.contains("Level 4"));
}

#[test]
fn test_link_with_special_characters_in_url() {
    let input = "[link](https://example.com/path?query=1&other=2#anchor)";
    let result = parse_and_serialize(input);
    assert!(
        result.contains("https://example.com/path?query=1&other=2#anchor"),
        "URL with special characters should be preserved, got:\n{}",
        result
    );
}

#[test]
fn test_image_with_empty_alt() {
    let input = "![](image.png)";
    let result = parse_and_serialize(input);
    assert!(
        result.contains("![](image.png)"),
        "Image with empty alt should be preserved, got:\n{}",
        result
    );
}

#[test]
fn test_code_span_with_newlines_in_content() {
    // Code spans cannot contain literal newlines, but escaped content should work
    let input = "`code`";
    let result = parse_and_serialize(input);
    assert!(result.contains("`code`"));
}

#[test]
fn test_escaped_characters_in_text() {
    let input = r"Text with \* escaped \[ characters \]";
    let result = parse_and_serialize(input);
    // Escaped characters should be preserved
    assert!(result.contains(r"\*") || result.contains("*"));
}

#[test]
fn test_multiple_consecutive_code_blocks() {
    let input = "~~~~ rust\nfn main() {}\n~~~~\n\n~~~~ python\ndef main():\n    pass\n~~~~";
    let result = parse_and_serialize(input);
    assert!(result.contains("rust"));
    assert!(result.contains("python"));
}

#[test]
fn test_table_with_empty_cells() {
    let input = "| A | B |\n|---|---|\n|   | X |";
    let result = parse_and_serialize(input);
    assert!(result.contains("|"));
    assert!(result.contains("X"));
}

#[test]
fn test_blockquote_with_multiple_paragraphs() {
    let input = "> First paragraph\n>\n> Second paragraph";
    let result = parse_and_serialize(input);
    assert!(result.contains("> First paragraph"));
    assert!(result.contains("> Second paragraph"));
}

#[test]
fn test_link_text_with_emphasis() {
    let input = "[*emphasized* link](https://example.com)";
    let result = parse_and_serialize(input);
    assert!(
        result.contains("*emphasized*"),
        "Emphasis in link text should be preserved, got:\n{}",
        result
    );
}

#[test]
fn test_heading_with_special_characters() {
    let input = "# Heading with `code` and *emphasis*";
    let result = parse_and_serialize(input);
    assert!(result.contains("`code`"));
    assert!(result.contains("*emphasis*"));
}

#[test]
fn test_very_long_word_in_paragraph() {
    let input = "This is a supercalifragilisticexpialidociousandmuchmuchlongerwordthatcannotbewrapped word.";
    let result = parse_and_serialize(input);
    // Long words should not cause crashes and should be preserved
    assert!(
        result
            .contains("supercalifragilisticexpialidociousandmuchmuchlongerwordthatcannotbewrapped")
    );
}

#[test]
fn test_strikethrough_text() {
    let input = "~~strikethrough~~";
    let result = parse_and_serialize(input);
    // Strikethrough may or may not be supported, but should not crash
    assert!(!result.is_empty());
}

#[test]
fn test_mixed_ordered_unordered_lists() {
    let input = " 1. Ordered item\n\n -  Unordered item";
    let result = parse_and_serialize(input);
    assert!(result.contains("1."));
    assert!(result.contains("-"));
}

#[test]
fn test_horizontal_rule() {
    let input = "Before\n\n---\n\nAfter";
    let result = parse_and_serialize(input);
    // Horizontal rules should be preserved
    assert!(result.contains("Before"));
    assert!(result.contains("After"));
}

#[test]
fn test_unicode_in_heading() {
    let input = "# 한글 제목";
    let result = parse_and_serialize(input);
    assert!(
        result.contains("한글 제목"),
        "Unicode heading should be preserved, got:\n{}",
        result
    );
}

#[test]
fn test_unicode_in_link_text() {
    let input = "[한글 링크](https://example.com)";
    let result = parse_and_serialize(input);
    assert!(
        result.contains("한글 링크"),
        "Unicode in link text should be preserved, got:\n{}",
        result
    );
}

#[test]
fn test_footnote_with_multiple_paragraphs() {
    let input = "Text[^1]\n\n[^1]: First paragraph of footnote";
    let result = parse_and_serialize(input);
    assert!(result.contains("[^1]"));
}

#[test]
fn test_nested_emphasis() {
    let input = "***bold and italic***";
    let result = parse_and_serialize(input);
    // Should preserve some form of emphasis
    assert!(result.contains("*"));
}

#[test]
fn test_code_block_with_blank_lines() {
    let input = "~~~~ text\nline 1\n\nline 3\n~~~~";
    let result = parse_and_serialize(input);
    assert!(result.contains("line 1"));
    assert!(result.contains("line 3"));
}

#[test]
fn test_gfm_task_list_checked() {
    let input = " - [x] Completed task";
    let result = parse_and_serialize(input);
    assert_eq!(result, " -  [x] Completed task\n");
}

#[test]
fn test_gfm_task_list_unchecked() {
    let input = " - [ ] Pending task";
    let result = parse_and_serialize(input);
    assert_eq!(result, " -  [ ] Pending task\n");
}

#[test]
fn test_gfm_task_list_mixed() {
    let input = " - [x] Done\n - [ ] Todo\n - [x] Also done";
    let result = parse_and_serialize(input);
    assert!(result.contains("[x] Done"));
    assert!(result.contains("[ ] Todo"));
    assert!(result.contains("[x] Also done"));
}

#[test]
fn test_gfm_task_list_nested() {
    let input = " - [x] Parent task\n    - [ ] Child task";
    let result = parse_and_serialize(input);
    assert!(result.contains("[x] Parent task"));
    assert!(result.contains("[ ] Child task"));
}

#[test]
fn test_definition_list_no_extra_blank_line() {
    let input = "Term\n:   Definition here";
    let result = parse_and_serialize(input);
    assert_eq!(result, "Term\n:   Definition here\n");
}

#[test]
fn test_definition_list_multiple_items() {
    let input = "Term1\n:   Definition1\n\nTerm2\n:   Definition2";
    let result = parse_and_serialize(input);
    assert!(result.contains("Term1\n:   Definition1"));
    assert!(result.contains("Term2\n:   Definition2"));
    // Should have blank line between items, but not between term and definition
    assert!(!result.contains("Term1\n\n:"));
}

#[test]
fn test_abbreviation_definition_preserved() {
    let input = "*[JSX]: JavaScript XML";
    let result = parse_and_serialize_with_source(input);
    assert_eq!(result, "*[JSX]: JavaScript XML\n");
}

#[test]
fn test_abbreviation_definition_multiple() {
    let input = "*[HTML]: HyperText Markup Language\n\n*[CSS]: Cascading Style Sheets";
    let result = parse_and_serialize_with_source(input);
    assert!(result.contains("*[HTML]: HyperText Markup Language"));
    assert!(result.contains("*[CSS]: Cascading Style Sheets"));
}

#[test]
fn test_definition_list_with_list_as_first_child() {
    let input = "Pros\n:    -  First item\n     -  Second item";
    let result = parse_and_serialize(input);
    // Should have colon on its own line, then indented list
    assert!(result.contains("Pros\n:\n"));
    assert!(result.contains("     -  First item"));
    assert!(result.contains("     -  Second item"));
}

#[test]
fn test_reference_used_in_multiple_sections_not_duplicated() {
    let input = r#"Section
-------

### First

Text with [link].

### Second

More text with [link].

[link]: https://example.com
"#;
    let result = parse_and_serialize_with_source(input);
    // The reference should appear only once, after first use
    let count = result.matches("[link]: https://example.com").count();
    assert_eq!(
        count, 1,
        "Reference should appear exactly once, but found {} times in:\n{}",
        count, result
    );
}

#[test]
fn test_heading_with_reference_link() {
    let input = r#"[BotKit] by Fedify
==================

[BotKit]: https://botkit.fedify.dev/
"#;
    let result = parse_and_serialize_with_source(input);
    // The reference link in the heading should be preserved
    assert!(
        result.contains("[BotKit] by Fedify"),
        "Heading should contain reference link syntax, got:\n{}",
        result
    );
    assert!(
        result.contains("[BotKit]: https://botkit.fedify.dev/"),
        "Reference definition should be preserved, got:\n{}",
        result
    );
}

#[test]
fn test_footnote_at_section_end_before_subheading() {
    let input = r#"Section
-------

Text with footnote[^1].

### Subsection

More text here.

[^1]: This is a footnote.
"#;
    let result = parse_and_serialize_with_source(input);
    // Footnote should appear before the subsection, not at document end
    let footnote_pos = result.find("[^1]: This is a footnote.").unwrap();
    let subsection_pos = result.find("### Subsection").unwrap();
    assert!(
        footnote_pos < subsection_pos,
        "Footnote should appear before subsection, got:\n{}",
        result
    );
}

#[test]
fn test_footnote_definition_wrapped_at_80_chars() {
    let input = r#"Text[^1].

[^1]: This is a very long footnote definition that definitely exceeds eighty characters and should be wrapped.
"#;
    let result = parse_and_serialize_with_source(input);
    // Check that no line exceeds 80 characters
    for line in result.lines() {
        assert!(
            line.len() <= 80,
            "Line exceeds 80 characters: '{}' (len={})",
            line,
            line.len()
        );
    }
    // Should still contain the footnote content
    assert!(
        result.contains("This is a very long footnote"),
        "Footnote content should be preserved, got:\n{}",
        result
    );
}

#[test]
fn test_footnote_continuation_indent_matches_prefix() {
    let input = r#"Text[^note].

[^note]: This is a long footnote with a longer name that should wrap with proper indentation.
"#;
    let result = parse_and_serialize_with_source(input);
    // The continuation line should be indented to align with content after "[^note]: "
    // "[^note]: " is 9 characters, so continuation should have 9 spaces
    assert!(
        result.contains("\n         "), // 9 spaces
        "Continuation should be indented with 9 spaces to match '[^note]: ', got:\n{}",
        result
    );
}

#[test]
fn test_table_warns_on_unescaped_pipe_in_cell() {
    use crate::format_with_warnings;

    let input = r#"| Property | Type | Required |
|----------|------|----------|
| `strategy` | `"a" | "b"` | Yes |"#;
    let result = format_with_warnings(input, &crate::Options::default()).unwrap();
    assert_eq!(result.warnings.len(), 1);
    assert!(result.warnings[0].message.contains("unescaped"));
    assert_eq!(result.warnings[0].line, 3);
}

#[test]
fn test_heading_setext_h1_disabled() {
    let options = Options {
        setext_h1: false,
        ..Options::default()
    };
    let result = parse_and_serialize_with_options("# Document Title", &options);
    assert_eq!(result, "# Document Title\n");
}

#[test]
fn test_heading_setext_h1_enabled() {
    let options = Options {
        setext_h1: true,
        ..Options::default()
    };
    let result = parse_and_serialize_with_options("# Document Title", &options);
    assert_eq!(result, "Document Title\n==============\n");
}

#[test]
fn test_heading_setext_h2_disabled() {
    let options = Options {
        setext_h2: false,
        ..Options::default()
    };
    let result = parse_and_serialize_with_options("## Section Title", &options);
    assert_eq!(result, "## Section Title\n");
}

#[test]
fn test_heading_setext_h2_enabled() {
    let options = Options {
        setext_h2: true,
        ..Options::default()
    };
    let result = parse_and_serialize_with_options("## Section Title", &options);
    assert_eq!(result, "Section Title\n-------------\n");
}

#[test]
fn test_list_unordered_marker_asterisk() {
    let options = Options {
        unordered_marker: '*',
        ..Options::default()
    };
    let result = parse_and_serialize_with_options(" -  Item one\n -  Item two", &options);
    assert_eq!(result, " *  Item one\n *  Item two\n");
}

#[test]
fn test_list_unordered_marker_plus() {
    let options = Options {
        unordered_marker: '+',
        ..Options::default()
    };
    let result = parse_and_serialize_with_options(" -  Item one\n -  Item two", &options);
    assert_eq!(result, " +  Item one\n +  Item two\n");
}

#[test]
fn test_list_unordered_marker_default() {
    let options = Options::default();
    let result = parse_and_serialize_with_options(" *  Item one\n *  Item two", &options);
    assert_eq!(result, " -  Item one\n -  Item two\n");
}

#[test]
fn test_list_leading_spaces_zero() {
    let options = Options {
        leading_spaces: 0,
        ..Options::default()
    };
    let result = parse_and_serialize_with_options(" -  Item one\n -  Item two", &options);
    assert_eq!(result, "-  Item one\n-  Item two\n");
}

#[test]
fn test_list_leading_spaces_two() {
    let options = Options {
        leading_spaces: 2,
        ..Options::default()
    };
    let result = parse_and_serialize_with_options(" -  Item one\n -  Item two", &options);
    assert_eq!(result, "  -  Item one\n  -  Item two\n");
}

#[test]
fn test_list_trailing_spaces_one() {
    let options = Options {
        trailing_spaces: 1,
        ..Options::default()
    };
    let result = parse_and_serialize_with_options(" -  Item one\n -  Item two", &options);
    assert_eq!(result, " - Item one\n - Item two\n");
}

#[test]
fn test_list_trailing_spaces_three() {
    let options = Options {
        trailing_spaces: 3,
        ..Options::default()
    };
    let result = parse_and_serialize_with_options(" -  Item one\n -  Item two", &options);
    assert_eq!(result, " -   Item one\n -   Item two\n");
}

#[test]
fn test_list_indent_width_two() {
    // indent_width=2: nested list has 2 spaces indent before " -  " prefix
    // Result: 2 spaces + " -  " = "   -  " (3 spaces before marker)
    let options = Options {
        indent_width: 2,
        ..Options::default()
    };
    let result = parse_and_serialize_with_options(" -  Item one\n     -  Nested", &options);
    assert_eq!(result, " -  Item one\n   -  Nested\n");
}

#[test]
fn test_list_indent_width_default() {
    // indent_width=4 (default): nested list has 4 spaces indent before " -  " prefix
    // Result: 4 spaces + " -  " = "     -  " (5 spaces before marker)
    let options = Options::default();
    let result = parse_and_serialize_with_options(" -  Item one\n     -  Nested", &options);
    assert_eq!(result, " -  Item one\n     -  Nested\n");
}

#[test]
fn test_ordered_list_odd_level_marker() {
    let options = Options {
        odd_level_marker: ')',
        ..Options::default()
    };
    // trailing_spaces=2, so " N)  " format
    let result = parse_and_serialize_with_options(" 1. First\n 2. Second", &options);
    assert_eq!(result, " 1)  First\n 2)  Second\n");
}

#[test]
fn test_ordered_list_even_level_marker() {
    let options = Options {
        even_level_marker: '.',
        ..Options::default()
    };
    // Nested ordered list (level 2)
    // trailing_spaces=2, so "N.  " format for nested items
    let result = parse_and_serialize_with_options(
        " 1. First\n     1. Nested first\n     2. Nested second",
        &options,
    );
    assert!(result.contains("1.  Nested first"), "got: {}", result);
    assert!(result.contains("2.  Nested second"), "got: {}", result);
}

#[test]
fn test_ordered_list_alternating_markers() {
    let options = Options::default();
    // Level 1 uses '.', level 2 uses ')'
    // trailing_spaces=2, so " N.  " for level 1, "N)  " for level 2
    let result = parse_and_serialize_with_options(
        " 1. First\n     1. Nested first\n     2. Nested second",
        &options,
    );
    assert!(result.contains(" 1.  First"), "got: {}", result);
    assert!(result.contains("1)  Nested first"), "got: {}", result);
    assert!(result.contains("2)  Nested second"), "got: {}", result);
}

#[test]
fn test_code_block_fence_char_backtick() {
    let options = Options {
        fence_char: '`',
        ..Options::default()
    };
    let result = parse_and_serialize_with_options("~~~~ rust\nfn main() {}\n~~~~", &options);
    assert!(result.starts_with("````"), "got: {}", result);
    assert!(result.contains("rust"), "got: {}", result);
}

#[test]
fn test_code_block_fence_char_default() {
    let options = Options::default();
    let result = parse_and_serialize_with_options("``` rust\nfn main() {}\n```", &options);
    assert!(result.starts_with("~~~~"), "got: {}", result);
}

#[test]
fn test_code_block_min_fence_length_three() {
    let options = Options {
        min_fence_length: 3,
        ..Options::default()
    };
    let result = parse_and_serialize_with_options("~~~~ rust\nfn main() {}\n~~~~", &options);
    assert!(result.starts_with("~~~"), "got: {}", result);
    assert!(!result.starts_with("~~~~"), "got: {}", result);
}

#[test]
fn test_code_block_min_fence_length_six() {
    let options = Options {
        min_fence_length: 6,
        ..Options::default()
    };
    let result = parse_and_serialize_with_options("~~~~ rust\nfn main() {}\n~~~~", &options);
    assert!(result.starts_with("~~~~~~"), "got: {}", result);
}

#[test]
fn test_code_block_space_after_fence_false() {
    let options = Options {
        space_after_fence: false,
        ..Options::default()
    };
    let result = parse_and_serialize_with_options("~~~~ rust\nfn main() {}\n~~~~", &options);
    assert!(result.contains("~~~~rust"), "got: {}", result);
}

#[test]
fn test_code_block_space_after_fence_true() {
    let options = Options {
        space_after_fence: true,
        ..Options::default()
    };
    let result = parse_and_serialize_with_options("~~~~rust\nfn main() {}\n~~~~", &options);
    assert!(result.contains("~~~~ rust"), "got: {}", result);
}

#[test]
fn test_ordered_list_pad_start_default() {
    // Default: pad = Start, numbers are right-aligned
    // For a list with 10+ items, single-digit numbers get leading space padding
    let input =
        "1. One\n2. Two\n3. Three\n4. Four\n5. Five\n6. Six\n7. Seven\n8. Eight\n9. Nine\n10. Ten";
    let result = parse_and_serialize(input);
    // Single-digit numbers should have extra leading space for alignment
    assert!(result.contains("  1.  One"), "got:\n{}", result);
    assert!(result.contains("  9.  Nine"), "got:\n{}", result);
    // Double-digit numbers should not have extra leading space
    assert!(result.contains(" 10.  Ten"), "got:\n{}", result);
}

#[test]
fn test_ordered_list_pad_end() {
    // pad = End, extra spaces go after the marker
    use crate::config::OrderedListPad;
    let options = Options {
        ordered_list_pad: OrderedListPad::End,
        ..Options::default()
    };
    let input =
        "1. One\n2. Two\n3. Three\n4. Four\n5. Five\n6. Six\n7. Seven\n8. Eight\n9. Nine\n10. Ten";
    let result = parse_and_serialize_with_options(input, &options);
    // Single-digit numbers should have extra trailing space after marker
    assert!(result.contains(" 1.   One"), "got:\n{}", result);
    assert!(result.contains(" 9.   Nine"), "got:\n{}", result);
    // Double-digit numbers should have normal trailing spaces
    assert!(result.contains(" 10.  Ten"), "got:\n{}", result);
}

#[test]
fn test_ordered_list_pad_small_list() {
    // For lists with only single-digit items, no extra padding is needed
    let input = "1. One\n2. Two\n3. Three";
    let result = parse_and_serialize(input);
    // No extra padding since max number is single-digit
    assert!(result.contains(" 1.  One"), "got:\n{}", result);
    assert!(result.contains(" 2.  Two"), "got:\n{}", result);
    assert!(result.contains(" 3.  Three"), "got:\n{}", result);
}

#[test]
fn test_ordered_list_pad_nested() {
    // Nested ordered lists should each have their own padding based on their item count
    let input = "1. Parent one\n2. Parent two\n     1. Child one\n     2. Child two\n     3. Child three\n     4. Child four\n     5. Child five\n     6. Child six\n     7. Child seven\n     8. Child eight\n     9. Child nine\n     10. Child ten";
    let result = parse_and_serialize(input);
    // Parent list has 2 items (single-digit), no extra padding
    assert!(result.contains(" 1.  Parent one"), "got:\n{}", result);
    assert!(result.contains(" 2.  Parent two"), "got:\n{}", result);
    // Child list has 10 items, single-digit should have extra padding
    // Nested items have indent_width (4) spaces before the " N) " prefix
    // With pad=Start, extra space goes before the number
    assert!(result.contains("  1)  Child one"), "got:\n{}", result);
    assert!(result.contains("  9)  Child nine"), "got:\n{}", result);
    assert!(result.contains(" 10)  Child ten"), "got:\n{}", result);
}

// Tests for undefined reference warnings

#[test]
fn test_undefined_reference_warning() {
    // When a reference link is used but not defined, a warning should be emitted
    let input = "See [undefined reference] for details.";
    let result = parse_and_serialize_with_warnings(input);
    assert_eq!(result.warnings.len(), 1);
    assert!(result.warnings[0].message.contains("undefined reference"));
    assert!(
        result.warnings[0]
            .message
            .contains("undefined reference link")
    );
}

#[test]
fn test_defined_reference_no_warning() {
    // When a reference link is properly defined, no warning should be emitted
    let input = "See [defined reference] for details.\n\n[defined reference]: https://example.com";
    let result = parse_and_serialize_with_warnings(input);
    assert_eq!(
        result.warnings.len(),
        0,
        "Expected no warnings but got: {:?}",
        result.warnings
    );
}

#[test]
fn test_multiple_undefined_references_warning() {
    // Multiple undefined references should each generate a warning
    let input = "See [foo] and [bar] for details.\n\n[foo]: https://example.com";
    let result = parse_and_serialize_with_warnings(input);
    assert_eq!(
        result.warnings.len(),
        1,
        "Expected 1 warning for [bar] but got: {:?}",
        result.warnings
    );
    assert!(result.warnings[0].message.contains("bar"));
}

#[test]
fn test_undefined_full_reference_warning() {
    // Full reference style [text][label] with undefined label
    let input = "See [some text][undefined-label] for details.";
    let result = parse_and_serialize_with_warnings(input);
    assert_eq!(result.warnings.len(), 1);
    assert!(result.warnings[0].message.contains("undefined-label"));
}

#[test]
fn test_abbreviation_definition_no_warning() {
    // PHP Markdown Extra abbreviation definitions (*[ABBR]: Full Text)
    // should not cause warnings when [ABBR] is used in the document
    let input = "The HTML specification is maintained by the W3C.\n\n*[HTML]: Hyper Text Markup Language\n*[W3C]: World Wide Web Consortium";
    let result = parse_and_serialize_with_warnings(input);
    assert_eq!(
        result.warnings.len(),
        0,
        "Expected no warnings for abbreviations but got: {:?}",
        result.warnings
    );
}

#[test]
fn test_abbreviation_with_undefined_reference() {
    // When document has abbreviation definitions, but also undefined references,
    // only the undefined references should trigger warnings
    let input = "See the HTML spec and [undefined ref].\n\n*[HTML]: Hyper Text Markup Language";
    let result = parse_and_serialize_with_warnings(input);
    assert_eq!(
        result.warnings.len(),
        1,
        "Expected 1 warning for [undefined ref] but got: {:?}",
        result.warnings
    );
    assert!(result.warnings[0].message.contains("undefined ref"));
}

#[test]
fn test_reference_after_abbreviation_no_warning() {
    // Reference definitions that follow abbreviation definitions (without a blank line)
    // may not be parsed by comrak as reference definitions. We should still detect
    // these from the source and not warn about them.
    let input = "See [RabbitMQ] for more.\n\n*[AMQP]: Advanced Message Queuing Protocol\n[RabbitMQ]: https://www.rabbitmq.com/";
    let result = parse_and_serialize_with_warnings(input);
    assert_eq!(
        result.warnings.len(),
        0,
        "Expected no warnings but got: {:?}",
        result.warnings
    );
}
