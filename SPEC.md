Specification: Hongdown
=======================

This document describes a planned Markdown formatter that enforces
Hong Minhee's Markdown style conventions.  The formatter is implemented
in Rust using the comrak library for parsing.


Background
----------

Hong Minhee maintains a distinctive Markdown style that combines several
uncommon conventions.  This style is used across multiple projects
including [Fedify], [Hollo], [Vertana], and others.

Existing tools like Prettier and markdownlint cannot fully enforce this
style due to limitations in their configuration options and parser
support for extended syntax.  A dedicated formatter provides complete
control over output formatting while supporting all required Markdown
extensions.

This specification covers:

 -  Style rules to be enforced
 -  Architecture and implementation approach
 -  Project structure and build configuration

[Fedify]: https://github.com/dahlia/fedify
[Hollo]: https://github.com/dahlia/hollo
[Vertana]: https://github.com/dahlia/vertana


Style rules
-----------

The formatter enforces the following conventions.  A canonical reference
is available in [Vertana's AGENTS.md] file.

[Vertana's AGENTS.md]: https://github.com/dahlia/vertana/blob/main/AGENTS.md

### Front matter

 -  YAML front matter (delimited by `---`) is preserved as-is
 -  TOML front matter (delimited by `+++`) is preserved as-is
 -  Front matter content is not formatted; only passed through verbatim
 -  Front matter must appear at the very beginning of the document

### Headings

 -  Setext-style headings for document title (level 1, underlined with
    `=`) and sections (level 2, underlined with `-`)
 -  ATX-style headings (`###`, `####`, etc.) for level 3 and below
 -  Setext underlines match the display width of the heading text,
    accounting for East Asian wide characters (wcwidth)
 -  Two blank lines before level 2 headings; one blank line before
    other headings
 -  One blank line after all headings

### Lists

 -  Unordered lists use ` -  ` format: one leading space, hyphen, two
    trailing spaces
 -  Nested items indented by 4 spaces (aligning content with parent)
 -  Continuation lines aligned with item content (4 spaces from marker)
 -  Ordered lists use ` 1.  ` format with same spacing principles
 -  Nested ordered lists alternate marker style: `1.` at odd nesting
    levels, `1)` at even nesting levels
 -  One blank line before and after lists (unless nested)

### Code blocks

 -  Fenced code blocks use tildes, minimum four (`~~~~`)
 -  When nesting code blocks (e.g., in documentation about Markdown),
    use more tildes than the inner fence (e.g., `~~~~~` to wrap `~~~~`)
 -  One space between fence and language identifier (e.g., `~~~~ python`)
 -  Language identifier always specified (use `text` if none applicable)
 -  One blank line before and after code blocks

### Inline formatting

 -  Emphasis uses single asterisks (`*text*`)
 -  Strong emphasis uses double asterisks (`**text**`)
 -  Inline code uses single backticks

### Links

 -  Reference-style links preferred for repeated URLs or long URLs
 -  Reference definitions placed at end of containing section
 -  Inline links acceptable for single-use short URLs

### Block quotes

 -  GitHub-style alerts preserved (`> [!NOTE]`, `> [!WARNING]`, etc.)
 -  Continuation lines include `>` prefix

### Definition lists

 -  Term on its own line
 -  Definition begins with ` :    ` (colon followed by three spaces)
 -  Multiple definitions for same term each start with ` :    `

### Tables

 -  Pipes aligned vertically, accounting for East Asian wide characters
    (wcwidth) to ensure proper display in monospace fonts
 -  Header separator uses hyphens with colons for alignment
 -  One space padding inside cells

### Line wrapping

 -  Lines wrapped at approximately 80 display columns (not bytes or
    codepoints), accounting for East Asian wide characters (wcwidth)
 -  Wrapping occurs at word boundaries
 -  URLs and inline code not broken across lines
 -  No trailing whitespace


Architecture
------------

### Overview

The formatter follows a simple pipeline:

~~~~ text
Input Markdown → Parser (comrak) → AST → Serializer → Output Markdown
~~~~

The parser is provided by comrak, a mature Rust library that supports
CommonMark and GitHub Flavored Markdown extensions.  The serializer is
implemented from scratch to provide complete control over output format.

### Why comrak

Comrak was selected for the following reasons:

 -  **Complete GFM support**: Tables, task lists, strikethrough,
    autolinks, and footnotes
 -  **Extended syntax**: Description lists (definition lists) and
    GitHub-style alerts/admonitions (as of v0.34.0)
 -  **AST-based**: Provides a traversable tree structure rather than
    a stream of events
 -  **Active maintenance**: Regular updates and bug fixes
 -  **Rust native**: No FFI overhead, single binary distribution

### AST node types

Comrak provides the following node types relevant to formatting:

**Block nodes**:

 -  `Document` — root node
 -  `FrontMatter` — YAML or TOML front matter (preserved verbatim)
 -  `Heading` — includes level and setext flag
 -  `Paragraph` — text content container
 -  `List` — ordered or unordered, tight or loose
 -  `Item` — list item
 -  `CodeBlock` — fenced or indented, with info string
 -  `BlockQuote` — quoted content
 -  `ThematicBreak` — horizontal rule
 -  `Table`, `TableRow`, `TableCell` — table structure
 -  `DescriptionList`, `DescriptionItem`, `DescriptionTerm`,
    `DescriptionDetails` — definition lists
 -  `Alert` — GitHub-style admonitions

**Inline nodes**:

 -  `Text` — literal text
 -  `Code` — inline code
 -  `Emph` — emphasis
 -  `Strong` — strong emphasis
 -  `Link` — inline or reference link
 -  `Image` — inline or reference image
 -  `SoftBreak` — soft line break (becomes space or newline)
 -  `HardBreak` — hard line break
 -  `FootnoteReference` — footnote marker

### Serializer design

The serializer traverses the AST and produces formatted output.  Key
responsibilities include:

**Block-level formatting**:

 -  Choosing setext vs ATX heading style based on level
 -  Calculating heading underline length
 -  Managing blank lines between blocks
 -  Indenting nested content appropriately
 -  Formatting list markers with correct spacing

**Inline formatting**:

 -  Preserving emphasis and strong markers
 -  Handling nested inline elements
 -  Managing line wrapping within paragraphs

**Line wrapping**:

 -  Tracking current line length
 -  Breaking at word boundaries near 80 characters
 -  Preserving non-breakable elements (URLs, code spans)
 -  Handling indentation for continuation lines

**Reference link management**:

 -  Collecting reference definitions during traversal
 -  Outputting definitions at section boundaries
 -  Generating reference labels when needed


Project structure
-----------------

~~~~ text
hongdown/
├── src/
│   ├── main.rs               # CLI entry point
│   ├── lib.rs                # Library entry point
│   ├── config.rs             # Configuration handling
│   ├── formatter.rs          # Main formatter orchestration
│   ├── serializer/
│   │   ├── mod.rs            # Serializer module
│   │   ├── block.rs          # Block-level serialization
│   │   ├── inline.rs         # Inline serialization
│   │   ├── table.rs          # Table formatting
│   │   └── wrap.rs           # Line wrapping logic
│   └── tests/
│       ├── mod.rs            # Test module
│       ├── headings.rs       # Heading format tests
│       ├── lists.rs          # List format tests
│       ├── code.rs           # Code block tests
│       └── fixtures/         # Test input/output files
├── Cargo.toml
├── LICENSE
├── README.md
└── CHANGELOG.md
~~~~


Command-line interface
----------------------

### Basic usage

~~~~ bash
# Format a file and print to stdout
hongdown input.md

# Format a file in place
hongdown --write input.md
hongdown -w input.md

# Format multiple files
hongdown -w *.md

# Format and show diff
hongdown --diff input.md

# Check if files are formatted (exit 1 if not)
hongdown --check input.md
~~~~

### Options

`-w, --write`
:   Write formatted output back to the input file(s) instead of stdout.

`-c, --check`
:   Check if files are already formatted.  Exit with code 0 if all files
    are formatted, code 1 otherwise.  No output is written.

`-d, --diff`
:   Show a diff of formatting changes instead of the formatted output.

`--stdin`
:   Read input from stdin instead of a file.  Output goes to stdout.

`--line-width <N>`
:   Set line width for wrapping.  Default: 80.

`--config <FILE>`
:   Path to configuration file.

`-h, --help`
:   Show help message.

`-V, --version`
:   Show version information.

### Configuration file

A `.hongdown.toml` file in the current or parent directories can provide
default options:

~~~~ toml
line_width = 80

[heading]
setext_h1 = true          # Use === underline for h1
setext_h2 = true          # Use --- underline for h2

[list]
unordered_marker = "-"    # "-", "*", or "+"
leading_spaces = 1        # Spaces before marker
trailing_spaces = 2       # Spaces after marker
indent_width = 4          # Indentation for nested items

[ordered_list]
odd_level_marker = "."    # "1." at odd nesting levels
even_level_marker = ")"   # "1)" at even nesting levels

[code_block]
fence_char = "~"          # "~" or "`"
min_fence_length = 4      # Minimum length, increases for nesting
space_after_fence = true  # Space between fence and language
~~~~


Implementation plan
-------------------

### Phase 1: Core formatting

 -  Basic CLI with file input/output
 -  Serializer for core Markdown elements:
    -  Headings (setext and ATX)
    -  Paragraphs with line wrapping
    -  Lists (ordered and unordered)
    -  Code blocks (fenced)
    -  Block quotes
    -  Inline formatting (emphasis, strong, code, links)
 -  Basic test suite

### Phase 2: Extended syntax

 -  Tables with alignment
 -  Definition lists
 -  GitHub alerts/admonitions
 -  Footnotes
 -  Reference link collection and placement

### Phase 3: Polish

 -  Configuration file support
 -  Diff output mode
 -  Check mode for CI integration
 -  Edge case handling
 -  Performance optimization
 -  Comprehensive test coverage


Dependencies
------------

~~~~ toml
[dependencies]
comrak = "0.43"          # Markdown parsing
thiserror = "2.0"        # Error handling
unicode-width = "0.2"    # East Asian width calculation (wcwidth)

# CLI-only dependencies
clap = { version = "4.5", features = ["derive"], optional = true }
toml = { version = "0.9", optional = true }
similar = { version = "2.7", optional = true }

[features]
default = []
cli = ["dep:clap", "dep:toml", "dep:similar"]

[[bin]]
name = "hongdown"
required-features = ["cli"]

[dev-dependencies]
pretty_assertions = "1"  # Better test output
~~~~

This structure allows lightweight library usage with just `hongdown = "x.y"`,
while the CLI binary requires `cargo install hongdown --features cli`.


Build and distribution
----------------------

### Building

~~~~ bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run with example
cargo run -- example.md
~~~~

### Distribution

The formatter will be distributed as:

 -  **Pre-built binaries**: Linux (x86_64, aarch64), macOS (x86_64,
    aarch64), Windows (x86_64, aarch64) via GitHub Releases
 -  **Cargo**: `cargo install hongdown --features cli`
 -  **Homebrew**: Formula for macOS/Linux (optional)
 -  **Scoop**: Manifest for Windows (optional)
 -  **winget**: Package for Windows (optional)
 -  **npm**: Wrapper package that downloads and invokes the appropriate
    pre-built binary for the user's platform

### Binary size

Expected binary size is approximately 3–5 MB for a statically linked
release build with LTO enabled.

~~~~ toml
# Cargo.toml profile for small binaries
[profile.release]
lto = true
codegen-units = 1
strip = true
~~~~


Testing strategy
----------------

### Reference repositories

The following repositories contain Markdown documents that serve as the
canonical examples of properly formatted content.  These documents are
already formatted according to Hong Minhee's style and should remain
unchanged when processed by Hongdown:

 -  [dahlia/optique] — Optique documentation
 -  [dahlia/logtape] — LogTape documentation

The test suite includes these documents as fixtures.  A formatting pass
over them must produce identical output (no diff).

[dahlia/optique]: https://github.com/dahlia/optique
[dahlia/logtape]: https://github.com/dahlia/logtape

### Unit tests

Each serializer component has unit tests covering:

 -  Basic formatting for each node type
 -  Edge cases (empty content, deeply nested structures)
 -  Interaction between adjacent elements

### Integration tests

Full document tests using fixture files:

 -  Input file in `tests/fixtures/<n>.input.md`
 -  Expected output in `tests/fixtures/<n>.output.md`
 -  Test runner compares formatter output against expected

### Roundtrip tests

Verify that formatting is idempotent:

 -  Format a document
 -  Format the result again
 -  Assert output is identical

### Compatibility tests

Verify that formatted output parses to equivalent AST:

 -  Parse original document to AST
 -  Format document
 -  Parse formatted output to AST
 -  Compare ASTs for semantic equivalence


Future considerations
---------------------

### Editor integration

 -  VS Code extension using the formatter binary
 -  Neovim plugin via null-ls or conform.nvim
 -  Format-on-save support

### Pre-commit hook

~~~~ yaml
# .pre-commit-config.yaml
repos:
  - repo: https://github.com/dahlia/hongdown
    rev: v0.1.0
    hooks:
      - id: hongdown
~~~~

### GitHub Action

~~~~ yaml
# .github/workflows/lint.yml
- name: Check Markdown formatting
  uses: dahlia/hongdown-action@v1
~~~~

### Library usage

The formatter can be used as a Rust library.  When used as a library,
only the core dependencies (comrak, thiserror, unicode-width) are
included—CLI-specific dependencies like clap and similar are not pulled in.

~~~~ toml
# Cargo.toml
[dependencies]
hongdown = "0.1"
~~~~

~~~~ rust
use hongdown::{format, Options};

let input = "# Hello\nWorld";
let options = Options::default();
let output = format(input, &options)?;
~~~~


References
----------

 -  [comrak documentation]
 -  [CommonMark specification]
 -  [GitHub Flavored Markdown specification]

[comrak documentation]: https://docs.rs/comrak
[CommonMark specification]: https://spec.commonmark.org/
[GitHub Flavored Markdown specification]: https://github.github.com/gfm/


Naming
------

The name *Hongdown* is a portmanteau of *Hong* (from Hong Minhee, the
author) and *Markdown*.  It also sounds like the Korean word 홍답다
(*hongdapda*), meaning "befitting of Hong" or "Hong-like"—a playful
way to describe Markdown formatted in Hong Minhee's style.
