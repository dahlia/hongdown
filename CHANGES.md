Hongdown changelog
==================

Version 0.5.4
-------------

To be released.


Version 0.5.3
-------------

Released on July 28, 2026.

 -  Fixed a bug where decoded tabs in angle-bracketed reference destinations
    were written as literal control characters, so reparsing the formatted
    Markdown could lose the destination.  Tabs are now written as `&#9;`,
    preserving the destination and making formatting idempotent.
    [[#39], [#40]]

[#39]: https://github.com/dahlia/hongdown/issues/39
[#40]: https://github.com/dahlia/hongdown/pull/40


Version 0.5.2
-------------

Released on July 28, 2026.

 -  Fixed a bug where a reference definition shared by a body link and a link
    inside a footnote could sink to the later footnote's section, leaving the
    body link's section without its definition.  Shared definitions now stay at
    the end of the first section that uses them, while remaining below footnote
    definitions when both occur in the same section.  [[#30], [#38]]

 -  Fixed a bug where reference definitions lost angle brackets required by an
    empty destination, ASCII whitespace, or other targets that cannot be
    serialized safely in bare form.  Those lines no longer represented the
    original CommonMark reference definitions, and a second formatting pass
    treated them as ordinary text.  These destinations now stay
    angle-bracketed, preserving their link targets and making formatting
    idempotent.  [[#25], [#37]]

 -  Fixed a bug where an external inline link whose text contained consecutive
    whitespace could require two formatting passes to settle.  Runs of
    CommonMark ASCII whitespace in ordinary link text are now collapsed on the
    first pass, so `[a  b](https://example.com/)` immediately becomes `[a b]`
    with a matching definition.  [[#28], [#36]]

 -  Fixed a bug where non-ASCII whitespace at the edge of an inline link's text
    could leave its generated shortcut reference with no matching definition.
    Reference labels now follow the parser's edge-whitespace rules, preserving
    distinct labels and their definitions across formatting runs.
    [[#31], [#35]]

 -  Fixed a bug where formatting could silently change a link's destination.
    Reference labels are generated from the link text, so two links with the
    same text but different destinations were given the same label, and the
    later definition overwrote the earlier one.  A label is now shared only
    when the complete target matches, meaning both the URL and the title;
    otherwise the colliding link gets a numbered label and full reference
    syntax, as in `[guide][guide 2]`.  [[#26], [#29]]

 -  Fixed a bug where a numbered label assigned to a colliding link could match
    unresolved reference syntax elsewhere in the document, silently turning
    ordinary bracketed text into a link.  Numbered label allocation now skips
    labels already used by unresolved references.  [[#33], [#34]]

 -  Fixed a bug where a reference definition needed by a later section was
    dropped entirely when an earlier section had already defined the same
    label with a different destination, leaving the later link pointing at
    the earlier section's URL.  [[#26], [#29]]

 -  Fixed a bug where reference labels differing only in letter case, such as
    `[Guide]` and `[guide]`, were emitted as two separate definitions.  Since
    CommonMark matches reference labels case-insensitively, parsers ignored
    the second definition and both links resolved to the first URL.  Labels
    are now compared with runs of whitespace collapsed and Unicode case
    folding applied, the same way the underlying parser resolves them.
    [[#26], [#29]]

 -  Fixed a bug where a reference definition placed inside a
    `<!-- hongdown-disable -->` region was dropped, leaving the region's links
    pointing at nothing.  Since the loss was stable across runs, `--check`
    never reported the damaged file afterwards.  The region between the two
    directives is now copied from the source in one piece instead of being
    rebuilt block by block, so its reference definitions survive, as do its
    footnote definitions, interior blank lines, and indentation.  (The blank
    line separating the region from the directives around it is still
    normalized to exactly one.)  [[#27], [#32]]

 -  Fixed a bug where a reference definition was dropped when the only links
    using it sat in a region that skips formatting.  Those links are copied
    from the source and never registered the definition they depend on, which
    silently broke reference-style badges under `<!-- hongdown-disable -->`,
    `<!-- hongdown-disable-next-line -->`,
    `<!-- hongdown-disable-next-section -->`, and
    `<!-- hongdown-disable-file -->`.  [[#27], [#32]]

 -  Fixed a bug where a link inside a region that skips formatting could have
    its destination changed by a link outside it.  A copied link keeps the
    label the source gave it, so when both wanted the same label for different
    destinations the copied one silently lost its target.  The formatted link,
    which can be given a derived label such as `[guide][guide 2]`, now yields
    instead.  [[#27], [#32]]

 -  Fixed a bug where a directive comment gained an extra blank line above it
    when it directly followed front matter, and lost the blank line entirely
    when the only thing above it was a reference definition.  [[#27], [#32]]

 -  Fixed a bug where a block whose formatting was skipped by
    `<!-- hongdown-disable-next-line -->` or
    `<!-- hongdown-disable-next-section -->` was copied from where its content
    begins rather than from the start of its lines, losing the indentation and
    the list or blockquote markers before it.  An indented code block came out
    as a paragraph, and a reference definition the parser had taken out of a
    list item was dropped altogether.  Such a block also gained a blank line on
    every run, so `--check` reported a file `--write` had just written.
    [[#32]]

 -  Fixed a bug where a reference label holding a bracket escaped with a
    backslash, as in `[a\]b]`, was cut short at that bracket.  The link was
    given a label naming no definition, and the definition itself was written
    out cut short too, so the next run no longer read it as a definition at
    all.  [[#32]]

[#25]: https://github.com/dahlia/hongdown/issues/25
[#26]: https://github.com/dahlia/hongdown/issues/26
[#27]: https://github.com/dahlia/hongdown/issues/27
[#28]: https://github.com/dahlia/hongdown/issues/28
[#29]: https://github.com/dahlia/hongdown/pull/29
[#30]: https://github.com/dahlia/hongdown/issues/30
[#31]: https://github.com/dahlia/hongdown/issues/31
[#32]: https://github.com/dahlia/hongdown/pull/32
[#33]: https://github.com/dahlia/hongdown/issues/33
[#34]: https://github.com/dahlia/hongdown/pull/34
[#35]: https://github.com/dahlia/hongdown/pull/35
[#36]: https://github.com/dahlia/hongdown/pull/36
[#37]: https://github.com/dahlia/hongdown/pull/37
[#38]: https://github.com/dahlia/hongdown/pull/38


Version 0.5.1
-------------

Released on July 2, 2026.

 -  Fixed a bug where formatting was not idempotent for fenced code blocks
    inside list items nested in blockquotes.  Blank lines between the blocks
    of a blockquoted list item lost their `>` marker, so running Hongdown
    a second time reinterpreted the fenced code blocks as indented code and
    wrapped them in another fence.  [[#24]]

[#24]: https://github.com/dahlia/hongdown/issues/24


Version 0.5.0
-------------

Released on June 21, 2026.

 -  Added MDX support.  MDX documents embed JavaScript and JSX that Comrak (the
    underlying CommonMark parser) does not understand, so Hongdown used to
    mangle them—turning straight quotes into curly quotes, backslash-escaping
    Markdown punctuation, and collapsing indentation.  When MDX mode is
    enabled, Hongdown now detects those constructs and preserves them verbatim
    while still formatting the surrounding Markdown prose.

    Within Markdown prose, the protected constructs are ESM `import`/`export`
    statements, JSX elements and fragments, and `{…}` expressions (including JSX
    comments like `{/* … */}`).  The embedded JavaScript/JSX is preserved as-is,
    not reformatted; constructs that the underlying parser already handles
    correctly—such as a valid inline-HTML JSX tag or a `{#id}` heading
    anchor—are left to it.

    MDX mode is enabled automatically for files with the *.mdx* extension, which
    are now also discovered when a directory is passed on the command line.  It
    can be turned on explicitly for stdin or *.md* input with the `--mdx`
    command-line flag or the `mdx` option in *.hongdown.toml* (Rust API:
    `Options.mdx`, type `bool`; WASM/JavaScript: `mdx`, type `boolean`).  All
    default to off, so non-MDX documents are unaffected.  [[#22], [#23]]

 -  Fixed a bug where a document whose only content was one or more HTML blocks
    (for example a single HTML comment) gained two spurious leading blank lines.
    [[#23]]

 -  Fixed a bug where two consecutive trailing HTML blocks that were separated
    by a blank line in the source were emitted on adjacent lines, dropping the
    blank line between them.  [[#23]]

 -  Added an [official Visual Studio Code extension].  The extension formats
    Markdown and MDX documents with a bundled WebAssembly build by default, can
    be configured to run a system Hongdown CLI instead, and is packaged as a
    *.vsix* artifact by CI.  Tag releases publish the same *.vsix* to GitHub
    Releases, Visual Studio Marketplace, and Open VSX Registry.

 -  Added `loadConfigFromToml()` and `headingAnchorAlign` to the
    WASM/JavaScript API so editor integrations can reuse *.hongdown.toml*
    settings without shelling out to the CLI.  External code block formatters in
    `code_block.formatters` are reported as warnings because the WASM runtime
    cannot execute external commands.

[official Visual Studio Code extension]: https://marketplace.visualstudio.com/items?itemName=hongminhee.hongdown
[#22]: https://github.com/dahlia/hongdown/issues/22
[#23]: https://github.com/dahlia/hongdown/pull/23


Version 0.4.4
-------------

Released on July 2, 2026.

 -  Fixed a bug where formatting was not idempotent for fenced code blocks
    inside list items nested in blockquotes.  Blank lines between the blocks
    of a blockquoted list item lost their `>` marker, so running Hongdown
    a second time reinterpreted the fenced code blocks as indented code and
    wrapped them in another fence.  [[#24]]


Version 0.4.3
-------------

Released on June 1, 2026.

 -  Release archives now use a flat layout.  Extracting a *.zip* or
    *.tar.bz2* archive places *hongdown* (or *hongdown.exe*) and the bundled
    documentation files directly in the extraction directory instead of under
    an extra target-named top-level directory.


Version 0.4.2
-------------

Released on June 1, 2026.

 -  Windows release executables now statically link the MSVC runtime, so
    *hongdown.exe* no longer requires separate Visual C++ runtime DLLs such as
    *vcruntime140.dll* to be installed on the target system.


Version 0.4.1
-------------

Released on May 30, 2026.

 -  Fixed a bug where a table cell containing inline TeX math (for example,
    `$\frac{1}{2}$`) was padded two columns too narrow, throwing off the
    table's pipe alignment.  Introduced in version 0.4.0 together with math
    support.


Version 0.4.0
-------------

Released on May 30, 2026.

 -  Added TeX/LaTeX math support.  Inline (`$…$`) and display (`$$…$$`) math is
    now recognized and preserved verbatim — never escaped or
    punctuation-transformed — fixing a bug where backslashes inside a formula
    (for example, `$O(\text{x})$`) were doubled to `$O(\\text{x})$`.

    This is controlled by a new `math` configuration option (Rust API:
    `Options.math`, type `bool`; WASM/JavaScript: `math`, type `boolean`),
    enabled by default.  Set `math = false` in *.hongdown.toml* (or pass
    `math: false` to the library) to treat every `$` as literal text.

    Dollar-math parsing follows GitHub's heuristics, so a lone `$`, a shell
    prompt like `$ command`, or a price like `$5` is not treated as math.
    Inline math is kept on a single line when wrapping, and multi-line display
    math is preserved verbatim, including inside lists and block quotes.

 -  Added `heading.anchor_align` configuration option (Rust API:
    `Options.heading_anchor_align`, type `i32`) to control the placement of
    explicit anchor identifiers (`{#name}`) in headings.

    A positive value sets the number of spaces between the heading body and
    the anchor (capped at 10,000).  Zero or a negative value right-aligns
    the anchor so that the full heading line spans `line_width + anchor_align`
    columns; ATX-style headings include the `# ` prefix in that calculation.
    Falls back to a single space when `line_width` is `false` or the heading
    body is already too wide to fit the anchor at the target column.

    The default is `0`, which right-aligns anchors to the configured line
    width (80 columns by default).  To restore the previous single-space
    behaviour, set `anchor_align = 1`.

 -  Word wrapping can now be disabled entirely.  Set `line_width = false` in
    the configuration file, pass `--no-line-width` on the command line, or set
    `Options.line_width = None` in the Rust API.  The `--no-line-width` flag and
    `--line-width` are mutually exclusive.

    The `Options.line_width` field type has changed from `LineWidth` to
    `Option<LineWidth>` (`None` means “no wrapping”).  This is a breaking
    change for Rust API users; update call sites by wrapping existing values
    with `Some(…)` and using `None` to disable wrapping.

    The WASM/JavaScript API similarly accepts `{ lineWidth: false }` to
    disable wrapping.  The TypeScript type for `lineWidth` is now
    `number | false`.


Version 0.3.11
--------------

Released on April 30, 2026.

 -  Fixed a bug where loose list items inside a blockquote were separated by a
    blank line without a `>` marker, causing one blockquote to be formatted as
    two separate blockquotes.  Hongdown now keeps the blank separator inside the
    blockquote.


Version 0.3.10
--------------

Released on April 26, 2026.

 -  Fixed a bug where HTML entities in fenced code block info strings were
    decoded during formatting (for example, `&quot;` became `"`).  Hongdown now
    preserves the original entity spelling in the emitted info string.


Version 0.3.9
-------------

Released on April 21, 2026.

 -  Fixed a bug where explicit anchor names at the end of headings (for
    example, `{#myAPI}`) were lowercased when `heading.sentence_case` was
    enabled.  Sentence-case conversion now applies only to the visible heading
    text and preserves the anchor name exactly as written.

 -  Fixed a bug where the first line of wrapped list items could exceed the
    configured `line_width` because Hongdown did not count already emitted
    prefixes such as list markers, blockquote markers, and definition-list
    markers.  Hongdown now includes those visible prefixes when measuring the
    first line, so wrapped list content stays within the configured width.


Version 0.3.8
-------------

Released on March 10, 2026.

 -  Fixed a bug where continuation lines in nested ordered lists inside
    definition-list details were indented one space too little.  This made
    already formatted documents appear dirty in `--check` and `--diff` mode;
    Hongdown now keeps those list items idempotent.


Version 0.3.7
-------------

Released on March 1, 2026.

 -  Fixed a regression where square brackets in unresolved reference-style
    link text were escaped during source-preserving serialization when the
    document also contained abbreviation definitions (e.g., `[RabbitMQ]`
    became `\[RabbitMQ]`).  Hongdown now preserves literal square brackets in
    this path so valid reference-style links remain intact.  [[#21]]

[#21]: https://github.com/dahlia/hongdown/issues/21


Version 0.3.6
-------------

Released on March 1, 2026.

 -  Fixed a bug where escaped square brackets at the beginning of a text node
    could lose the opening backslash during source-preserving serialization
    (e.g., `\[foo\]` became `[foo\]`).  Hongdown now applies context-aware
    escaping in that path, so escaped brackets remain symmetric and formatting
    stays idempotent.  [[#21]]


Version 0.3.5
-------------

Released on March 1, 2026.

 -  Fixed a bug where backslash-escaped ASCII punctuation in plain text was
    double-escaped during formatting (e.g., `\[identifier\]` became
    `\\[identifier\\]`).  This made formatting non-idempotent and changed
    rendered output by showing literal backslashes.  Hongdown now preserves
    explicit backslash escapes for ASCII punctuation as written.  [[#20]]

[#20]: https://github.com/dahlia/hongdown/issues/20


Version 0.3.4
-------------

Released on February 27, 2026.

 -  Fixed a bug where fenced code blocks inside blockquotes that were nested
    inside list items were not formatted idempotently.  The code fence lines
    (opening, content, and closing) were missing the list item's continuation
    indentation, causing the code block to appear outside the blockquote
    context on subsequent formatting passes.


Version 0.3.3
-------------

Released on February 27, 2026.

 -  Fixed a bug where tables inside list items were emitted without a separating
    blank line and without continuation indentation on every table row.  Tables
    in list items are now formatted as proper nested block content and remain
    stable across repeated formatting passes.


Version 0.3.2
-------------

Released on January 26, 2026.

 -  Fixed a bug where footnote definitions with simple inline text were
    always wrapped at 80 characters, ignoring the `line_width` configuration
    option.  Footnotes now correctly respect the configured line width when
    wrapping their content.  This fix applies to footnotes containing only
    inline text; footnotes with block elements (paragraphs, lists, code
    blocks) were already honoring the line width setting.

 -  Improved footnote formatting to properly handle multiple paragraphs and
    complex block structures.  The formatter now correctly:

     -  Preserves blank lines between multiple paragraphs within footnotes
     -  Maintains line breaks within blockquotes inside footnotes
     -  Applies proper continuation indentation to all block elements

    Previously, multiple paragraphs in footnotes would be concatenated without
    blank lines, and multiline blockquotes would collapse into single lines.


Version 0.3.1
-------------

Released on January 26, 2026.

 -  Fixed a bug where code blocks inside footnotes were being silently dropped
    during formatting.  The formatter now properly preserves block elements
    (code blocks, block quotes, lists, tables) within footnote definitions,
    outputting them with correct indentation.


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
