Hongdown changelog
==================

Version 0.3.0
-------------

To be released.


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
