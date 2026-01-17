Hongdown changelog
==================

Version 0.4.0
-------------

To be released.


Version 0.3.0
-------------

Released on January 17, 2026.

 -  Added support for cascading configuration files from multiple locations.
    Hongdown now loads and merges configuration files in the following order
    (lowest to highest priority): [[#15]]

    1)  System-wide: */etc/hongdown/config.toml* (Linux/Unix only)
    2)  User (legacy): *~/.hongdown.toml* (all platforms)
    3)  User (platform-specific):
         -  Linux: *$XDG\_CONFIG\_HOME/hongdown/config.toml* or
            *~/.config/hongdown/config.toml*
         -  macOS: *~/Library/Application Support/hongdown/config.toml*
         -  Windows: *%APPDATA%\\hongdown\\config.toml*
    4)  Project: *.hongdown.toml* in current or parent directories

    Settings from higher-priority configurations override those from
    lower-priority ones, allowing you to set global defaults at the user or
    system level while overriding them for specific projects.

    For example, you can set your preferred `line_width = 72` in
    *~/.hongdown.toml*, and most projects will inherit this setting.  Projects
    that need a different width can override it in their *.hongdown.toml*.

    To prevent a project from inheriting system or user configurations, add
    `no_inherit = true` to the project's *.hongdown.toml*:

    ~~~~ toml
    no_inherit = true
    line_width = 100
    ~~~~

    This ensures the project uses only its own settings plus Hongdown's
    defaults, regardless of system or user preferences.  This is useful for
    projects with strict formatting requirements.

    The `--config` flag continues to work as before, bypassing the cascading
    system and using only the specified file.

 -  Added `git_aware` configuration option (default: `true`).  When enabled,
    Hongdown respects _.gitignore_ files and automatically skips the `.git`
    directory during file collection.  This is particularly useful when using
    `include` patterns like `**/*.md` to avoid formatting files that are
    intentionally ignored by Git.  Set `git_aware = false` in your
    _.hongdown.toml_ to disable this behavior and traverse all files.

 -  Dramatically improved file collection performance when using `include` and
    `exclude` patterns in configuration files.  The implementation now uses the
    [`ignore`] crate instead of [`glob`], which efficiently skips excluded
    directories before traversing them rather than filtering after collection.

    In large projects with many files in `node_modules` or similar directories,
    this provides 100–200× speedup.  For example, in a project with ~130,000
    files where only 81 need formatting, `hongdown --check` now completes in
    0.06 seconds instead of 16 seconds (265× faster).

    Configuration files require no changes—all existing `include` and `exclude`
    patterns remain fully compatible.

 -  Breaking changes: All configuration options now use type-safe newtypes
    and enums instead of primitive types, preventing invalid configurations at
    parse time rather than runtime.  This implements the “Make Invalid States
    Unrepresentable” pattern.  [[#14], [#16]]

    The following types have been added:

     -  `UnorderedMarker` enum: Validates unordered list markers (hyphen,
        asterisk, plus)
     -  `OrderedMarker` enum: Validates ordered list markers (dot, parenthesis)
     -  `FenceChar` enum: Validates code block fence characters (tilde,
        backtick)
     -  `MinFenceLength` newtype: Ensures minimum fence length is at least 3
     -  `LeadingSpaces` newtype: Validates leading spaces are between 0–3
        (CommonMark requirement)
     -  `TrailingSpaces` newtype: Validates trailing spaces are between 0–3
     -  `IndentWidth` newtype: Ensures indentation width is at least 1
     -  `LineWidth` newtype: Ensures line width is at least 8 (warns if below
        40 for readability)
     -  `ThematicBreakStyle` newtype: Validates thematic break patterns follow
        CommonMark spec (at least 3 of the same character: `*`, `-`, or `_`)
     -  `DashPattern` newtype: Ensures dash transformation patterns are
        non-empty and contain only printable ASCII characters

    Configuration files with invalid values will now fail to parse with
    descriptive error messages.  For example, setting `line_width = 5` will
    produce: `"line_width must be at least 8, got 5."`

    If you're using the Rust API directly, you'll need to update your code to
    use the new types.  For example:

    ~~~~ rust
    use hongdown::{Options, LineWidth, UnorderedMarker};

    let options = Options {
        line_width: LineWidth::new(80).unwrap(),
        unordered_marker: UnorderedMarker::Hyphen,
        ..Options::default()
    };
    ~~~~

    WASM and CLI users are unaffected as the types are automatically converted
    from configuration values.

 -  Fixed non-idempotent backslash escaping in emphasis text.  The formatter
    was producing different output on each pass when processing backslashes
    inside italic or bold text (e.g., `*C:\Users\Alice\Documents*`).  Each
    formatting pass would add more backslashes, causing `--check` to
    perpetually report files as needing formatting even immediately after
    `--write`.  [[#18]]

[`ignore`]: https://crates.io/crates/ignore
[`glob`]: https://crates.io/crates/glob
[#14]: https://github.com/dahlia/hongdown/issues/14
[#15]: https://github.com/dahlia/hongdown/issues/15
[#16]: https://github.com/dahlia/hongdown/pull/16
[#18]: https://github.com/dahlia/hongdown/issues/18


Version 0.2.6
-------------

Released on January 14, 2026.

 -  Fixed a bug where possessive apostrophes after digits (e.g., `1.2.3's`,
    `2024's`) were incorrectly converted to curly apostrophes even when
    `punctuation.curly_apostrophes` was set to `false` (the default).


Version 0.2.5
-------------

Released on January 14, 2026.

 -  Fixed a bug where list item continuation lines inside definition lists
    were indented one extra space.  The formatter now correctly aligns
    continuation lines with the first line of content.  For example:

    ~~~~ markdown
    Pros
    :    -  The actor URI is more predictable and human-readable,
            which makes debugging easier.
    ~~~~

    Previously, the continuation line was indented with 9 spaces instead of 8,
    causing misalignment.


Version 0.2.4
-------------

Released on January 14, 2026.

 -  Fixed a bug where definition lists with a list (ordered or unordered) as
    the first block element would produce non-idempotent output.  The formatter
    previously output `:` followed by a newline and indented list, which caused
    the list to break out of the definition on subsequent formatting passes.
    The list is now kept on the same line as the colon (`:    -  item`),
    ensuring stable and idempotent formatting.


Version 0.2.3
-------------

Released on January 13, 2026.

 -  Fixed a bug where proper nouns inside parentheses (e.g., “(Deno only)”) were
    incorrectly lowercased when `heading.sentence_case` was enabled.  The proper
    noun matching now correctly strips leading punctuation like opening
    parentheses and brackets before matching against the proper nouns list.


Version 0.2.2
-------------

Released on January 13, 2026.

 -  Fixed a bug where possessive apostrophes after link references (e.g.,
    `[Fedify]'s`) were incorrectly converted to curly apostrophes even when
    `punctuation.curly_apostrophes` was set to `false` (the default).

 -  Fixed a bug where footnote definitions and link reference definitions
    placed before `<!-- hongdown-disable -->` (or other disable directives)
    were incorrectly moved below the directive.  The definitions now correctly
    stay above the directive where they were originally placed.

 -  Fixed a bug where headings starting with a code span (e.g.,
    `` # `Foo` object ``) would incorrectly capitalize the word following
    the code span when `heading.sentence_case` was enabled.  Now the code span
    counts as the first word, so subsequent words are correctly lowercased.

 -  Fixed a bug where the English first-person pronoun “I” was incorrectly
    lowercased when `heading.sentence_case` was enabled.  The pronoun “I” and
    its contractions (I'm, I've, I'll, I'd) are now always capitalized
    regardless of their position in the heading.

 -  Fixed en dash (–) handling in `heading.sentence_case` mode.  En dash is
    now treated as a word delimiter like em dash (—), colon, and semicolon.


Version 0.2.1
-------------

Released on January 13, 2026.

 -  Fixed an issue where `heading.proper_nouns` entries containing slashes
    or hyphens (e.g., `@foo/javascript`, `my-custom-lib`) were not recognized
    as proper nouns because the word was split before matching.  Now the
    entire word is checked against user proper nouns before splitting.


Version 0.2.0
-------------

Released on January 13, 2026.

 -  Added [`@hongdown/wasm`] package, a WebAssembly-based JavaScript/TypeScript
    library.  This allows using Hongdown as a library in Node.js, Bun, Deno,
    and web browsers.  [[#7]]

 -  Added heading sentence case conversion.  The formatter can now
    automatically convert headings to sentence case (capitalizing only the
    first word) while preserving proper nouns, acronyms, and code spans.
    Configurable via the `[heading]` section in _.hongdown.toml_:  [[#8]]

     -  `sentence_case`: Enable sentence case conversion (default: `false`)
     -  `proper_nouns`: List of user-defined proper nouns to preserve
     -  `common_nouns`: List of words to exclude from built-in proper nouns

    The formatter includes ~450 built-in proper nouns (programming languages,
    frameworks, cloud providers, countries, natural languages, etc.) and
    supports multi-word proper nouns like “GitHub Actions” and “United States
    of America”.  It applies intelligent heuristics:

     -  Preserves acronyms (2+ consecutive uppercase letters: API, HTTP)
     -  Preserves acronyms with periods (U.S.A., Ph.D., R.O.K.)
     -  Preserves proper nouns (case-insensitive matching)
     -  Preserves code spans (backticks)
     -  Handles quoted text based on original capitalization
     -  Handles hyphenated words independently
     -  Preserves all-caps words (intentional emphasis: IMPORTANT)
     -  Preserves non-Latin scripts (CJK, etc.)

    Document-level directives allow per-document customization:

     -  `<!-- hongdown-proper-nouns: Swift, Go -->` – Define proper nouns to
        preserve within the document
     -  `<!-- hongdown-common-nouns: Python -->` – Override built-in proper
        nouns by treating them as common nouns

    These directives are merged with configuration file settings, enabling
    fine-tuned control over capitalization for specific documents.

 -  Added SmartyPants-style punctuation transformation.  The formatter can now
    convert ASCII punctuation to typographically correct Unicode equivalents.
    Configurable via the `[punctuation]` section in _.hongdown.toml_:

     -  `curly_double_quotes`: Convert `"text"` to `“text”` (default: `true`)
     -  `curly_single_quotes`: Convert `'text'` to `‘text’` (default: `true`)
     -  `curly_apostrophes`: Convert `it's` to `it’s` (default: `false`)
     -  `ellipsis`: Convert `...` to `…` (default: `true`)
     -  `en_dash`: Convert pattern to `–` (default: `false`)
     -  `em_dash`: Convert `--` to `—` (default: `"--"`)

    Code spans and fenced code blocks are never transformed.

 -  Fixed Setext-style heading underlines to match the display width of the
    heading text.  East Asian wide characters are now correctly counted as
    2 columns.  [[#5] by Lee Dogeon]

 -  Fixed text wrapping to use Unicode display width instead of byte length.
    East Asian wide characters (Korean, Japanese, Chinese) are now correctly
    counted as 2 columns, so text wraps at the correct visual position.
    [[#3] by Lee Dogeon]

 -  Added support for directory arguments.  When a directory is passed as an
    argument, Hongdown now recursively finds all Markdown files (_\*.md_ and
    _\*.markdown_) within it.  [[#2]]

 -  Added external code formatter support for code blocks.  You can now
    configure language-specific formatters in _.hongdown.toml_ to automatically
    format code inside fenced code blocks.  [[#9]]

    ~~~~ toml
    [code_block.formatters]
    javascript = ["deno", "fmt", "-"]
    typescript = ["deno", "fmt", "-"]

    [code_block.formatters.python]
    command = ["black", "-"]
    timeout = 10
    ~~~~

    Code is passed to the formatter via stdin, and the formatted output is read
    from stdout.  If the formatter fails (non-zero exit, timeout, etc.), the
    original code is preserved and a warning is emitted.

    To skip formatting for a specific code block, add `hongdown-no-format` after
    the language identifier:

    ~~~~~ markdown
    ~~~~ python hongdown-no-format
    def hello(): print("Hello, World!")
    ~~~~
    ~~~~~

    For WASM builds, use the `formatWithCodeFormatter` function with a callback:

    ~~~~ typescript
    import { formatWithCodeFormatter } from "@hongdown/wasm";

    const { output } = await formatWithCodeFormatter(markdown, {
      codeFormatter: (language, code) => {
        if (language === "javascript") {
          return prettier.format(code, { parser: "babel" });
        }
        return null;
      },
    });
    ~~~~

[`@hongdown/wasm`]: https://www.npmjs.com/package/@hongdown/wasm
[#2]: https://github.com/dahlia/hongdown/issues/2
[#3]: https://github.com/dahlia/hongdown/pull/3
[#5]: https://github.com/dahlia/hongdown/pull/5
[#7]: https://github.com/dahlia/hongdown/issues/7
[#8]: https://github.com/dahlia/hongdown/issues/8
[#9]: https://github.com/dahlia/hongdown/issues/9


Version 0.1.1
-------------

Released on January 12, 2026.

 -  Fixed a bug where an extra blank line was added between a nested list and
    a following paragraph within the same list item.


Version 0.1.0
-------------

Released on January 10, 2026. Initial release with the following features:

 -  Markdown formatting following Hong Minhee's style conventions:

     -  Setext-style headings for H1 and H2, ATX-style for H3+
     -  Four-tilde code fences instead of backticks
     -  Reference-style links
     -  Sentence-case headings
     -  Proper list formatting with ` -  ` prefix
     -  GitHub-flavored Markdown alert blocks

 -  CLI with multiple modes:

     -  Default: output formatted Markdown to stdout
     -  `--write` (`-w`): format files in place
     -  `--check` (`-c`): verify files are properly formatted
     -  `--diff` (`-d`): show formatting changes

 -  Configuration via `.hongdown.toml`:

     -  `include`: glob patterns for files to format
     -  `exclude`: glob patterns for files to skip
     -  `line_width`: maximum line width (default: 80)
     -  `list_marker`: list marker style (default: `-`)

 -  Cross-platform support: Linux (glibc/musl), macOS, Windows

 -  Distribution via:

     -  [crates.io]
     -  [npm] (via `@hongdown/*` packages)
     -  Pre-built binaries on [GitHub Releases]

[crates.io]: https://crates.io/crates/hongdown
[npm]: https://www.npmjs.com/package/hongdown
[GitHub Releases]: https://github.com/dahlia/hongdown/releases
