Hongdown
========

[![crates.io][crates.io badge]][crates.io]
[![npm][npm badge]][npm]
[![@hongdown/wasm][@hongdown/wasm badge]][@hongdown/wasm]
[![GitHub Actions][GitHub Actions badge]][GitHub Actions]

Hongdown is a Markdown formatter that enforces [Hong Minhee's Markdown
style conventions](./STYLE.md).  The formatter is implemented in Rust using
the [Comrak] library for parsing.  It produces consistently formatted Markdown
output following a distinctive style used across multiple projects including
[Fedify], [LogTape], and [Optique].

[crates.io badge]: https://img.shields.io/crates/v/hongdown?logo=rust
[crates.io]: https://crates.io/crates/hongdown
[npm badge]: https://img.shields.io/npm/v/hongdown?logo=npm
[npm]: https://www.npmjs.com/package/hongdown
[@hongdown/wasm badge]: https://img.shields.io/npm/v/@hongdown/wasm?logo=webassembly&label=%40hongdown%2Fwasm
[@hongdown/wasm]: https://www.npmjs.com/package/@hongdown/wasm
[GitHub Actions badge]: https://github.com/dahlia/hongdown/actions/workflows/main.yaml/badge.svg
[GitHub Actions]: https://github.com/dahlia/hongdown/actions/workflows/main.yaml
[Comrak]: https://comrak.ee/
[Fedify]: https://fedify.dev/
[LogTape]: https://logtape.org/
[Optique]: https://optique.dev/


Installation
------------

### npm

~~~~ bash
npm install -g hongdown
~~~~

### mise

~~~~ bash
mise use -g github:dahlia/hongdown
~~~~

### Cargo

~~~~ bash
cargo install hongdown
~~~~

### Nix

~~~~ bash
nix run github:dahlia/hongdown
~~~~

### Pre-built binaries

Pre-built binaries for Linux, macOS, and Windows are available on the
[GitHub Releases] page.

[GitHub Releases]: https://github.com/dahlia/hongdown/releases


Usage
-----

### Basic usage

~~~~ bash
# Format a file and print to stdout
hongdown input.md

# Format a file in place
hongdown --write input.md
hongdown -w input.md

# Format multiple files
hongdown -w *.md

# Format all Markdown files in a directory (recursive)
hongdown -w .
hongdown -w docs/

# Check if files are formatted (exit 1 if not)
hongdown --check input.md
hongdown -c input.md

# Show diff of formatting changes
hongdown --diff input.md
hongdown -d input.md

# Read from stdin (use --stdin flag or - as filename)
echo "# Hello" | hongdown --stdin
echo "# Hello" | hongdown -
hongdown --stdin < input.md
hongdown - < input.md

# Custom line width
hongdown --line-width 100 input.md
~~~~

### HTML comment directives

Hongdown supports special HTML comment directives to control formatting
behavior within documents.

#### Disable formatting

You can disable formatting for specific sections:

~~~~ markdown
<!-- hongdown-disable-file -->
This entire file will not be formatted.
~~~~

~~~~ markdown
<!-- hongdown-disable-next-line -->
This   line   preserves   its   spacing.

The next line will be formatted normally.
~~~~

~~~~ markdown
<!-- hongdown-disable-next-section -->
Everything here is preserved as-is
until the next heading (h1 or h2).

Next heading
------------

This section will be formatted normally.
~~~~

~~~~ markdown
<!-- hongdown-disable -->
This section is not formatted.
<!-- hongdown-enable -->
This section is formatted again.
~~~~

#### Sentence case customization

When sentence case is enabled, you can define document-specific proper nouns
and common nouns:

~~~~ markdown
<!-- hongdown-proper-nouns: Swift, Go -->
<!-- hongdown-common-nouns: Python -->

# Using Swift And Go For Python Development

This heading will be formatted as: "Using Swift and Go for python development"
~~~~

 -  `<!-- hongdown-proper-nouns: Word1, Word2 -->` – Defines proper nouns to
    preserve (case-sensitive)
 -  `<!-- hongdown-common-nouns: Word1, Word2 -->` – Overrides built-in proper
    nouns by treating them as common nouns

These directives are merged with configuration file settings.

### Configuration file

Hongdown looks for a *.hongdown.toml* file in the current directory and
parent directories.  You can also specify a configuration file explicitly
with the `--config` option.

Below is an example configuration with all available options and their
default values:

~~~~ toml
# File patterns (glob syntax)
include = []              # Files to format (default: none, specify on CLI)
exclude = []              # Files to skip (default: none)

# Formatting options
line_width = 80           # Maximum line width (min: 8, default: 80)

[heading]
setext_h1 = true          # Use === underline for h1 (default: true)
setext_h2 = true          # Use --- underline for h2 (default: true)
sentence_case = false     # Convert headings to sentence case (default: false)
proper_nouns = []         # Additional proper nouns to preserve (default: [])
common_nouns = []         # Exclude built-in proper nouns (default: [])

[unordered_list]
unordered_marker = "-"    # "-", "*", or "+" (default: "-")
leading_spaces = 1        # Spaces before marker (0–3, default: 1)
trailing_spaces = 2       # Spaces after marker (1–2, default: 2)
indent_width = 4          # Indentation for nested items (min: 1, default: 4)

[ordered_list]
odd_level_marker = "."    # "." or ")" at odd nesting levels (default: ".")
even_level_marker = ")"   # "." or ")" at even nesting levels (default: ")")
pad = "start"             # "start" or "end" for number alignment (default: "start")
indent_width = 4          # Indentation for nested items (min: 1, default: 4)

[code_block]
fence_char = "~"          # "~" or "`" (default: "~")
min_fence_length = 4      # Minimum fence length (min: 3, default: 4)
space_after_fence = true  # Space between fence and language (default: true)
default_language = ""     # Default language for code blocks (default: "")

# External code formatters (see "External code formatters" section)
[code_block.formatters]
# javascript = ["deno", "fmt", "--ext=js", "-"]

[thematic_break]
# Must be valid CommonMark: at least 3 of *, -, or _ (with optional spaces)
style = "- - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -"
leading_spaces = 3        # Leading spaces (0–3, default: 3)

[punctuation]
curly_double_quotes = true   # "text" to "text" (default: true)
curly_single_quotes = true   # 'text' to 'text' (default: true)
curly_apostrophes = false    # it's to it's (default: false)
ellipsis = true              # ... to ... (default: true)
en_dash = false              # Disabled by default (use "--" to enable)
em_dash = "--"               # -- to --- (default: "--", use false to disable)
~~~~

Configuration values are validated at parse time.  Invalid values will produce
descriptive error messages:

~~~~ bash
$ hongdown --config invalid.toml input.md
Error: failed to parse configuration file: line_width must be at least 8, got 5.
~~~~

When `include` patterns are configured, you can run Hongdown without
specifying files:

~~~~ bash
# Format all files matching include patterns
hongdown --write

# Check all files matching include patterns
hongdown --check
~~~~

CLI options override configuration file settings:

~~~~ bash
# Use config file but override line width
hongdown --line-width 100 input.md

# Use specific config file
hongdown --config /path/to/.hongdown.toml input.md
~~~~


Style rules
-----------

Hongdown enforces the following conventions:

### Headings

 -  Level 1 and 2 use Setext-style (underlined with `=` or `-`)
 -  Level 3+ use ATX-style (`###`, `####`, etc.)
 -  Optional sentence case conversion (disabled by default)

~~~~ markdown
Document Title
==============

Section
-------

### Subsection
~~~~

#### Sentence case (optional)

When `sentence_case = true` is set in the configuration, Hongdown automatically
converts headings to sentence case using intelligent heuristics:

~~~~ markdown
# Development Commands      → Development commands
# Working With JSON APIs    → Working with JSON APIs
# Using `MyClass` In Code   → Using `MyClass` in code
~~~~

The converter:

 -  Capitalizes only the first word
 -  Preserves code spans (text in backticks)
 -  Preserves acronyms (2+ consecutive uppercase letters, e.g., `API`, `HTTP`)
 -  Preserves proper nouns (built-in list + user-configured)
 -  Handles hyphenated words (e.g., `JSON-RPC`)
 -  Respects quoted text capitalization
 -  Preserves non-Latin scripts (CJK, etc.)

You can add custom proper nouns to preserve:

~~~~ toml
[heading]
sentence_case = true
proper_nouns = ["MyCompany", "MyProduct", "MyAPI"]
~~~~

You can also exclude built-in proper nouns by treating them as common nouns.
This is useful for words like “Go” which can be either a programming language
or a common verb:

~~~~ toml
[heading]
sentence_case = true
common_nouns = ["Go", "Swift"]  # Treat these as common nouns, not proper nouns
~~~~

Built-in proper nouns include ~450 entries: programming languages (JavaScript,
TypeScript, Python, Rust, Go), technologies (GitHub, Docker, React, Node.js),
databases (PostgreSQL, MySQL, MongoDB), countries (United States, Republic of
Korea), natural languages (English, Korean, Japanese), and more.

You can also use HTML comment directives to define document-specific proper
nouns and common nouns.  See the “HTML comment directives” section for details.

### Lists

 -  Unordered lists use ` -  ` (space-hyphen-two spaces)
 -  Ordered lists use `1.` format
 -  4-space indentation for nested items

~~~~ markdown
 -  First item
 -  Second item
     -  Nested item
~~~~

### Code blocks

 -  Fenced with four tildes (`~~~~`)
 -  Language identifier on the opening fence

~~~~~ text
~~~~ rust
fn main() {
    println!("Hello, world!");
}
~~~~
~~~~~

### External code formatters

You can configure external formatters for code blocks in your *.hongdown.toml*.
This allows automatic formatting of embedded code using language-specific tools.

~~~~ toml
[code_block.formatters]
# Simple format: command as array
javascript = ["deno", "fmt", "--ext=js", "-"]
typescript = ["deno", "fmt", "--ext=ts", "-"]
rust = ["rustfmt"]

# With custom timeout (default is 5 seconds)
[code_block.formatters.python]
command = ["black", "-"]
timeout = 10
~~~~

Behavior:

 -  Language matching is exact only (`javascript` matches `javascript`, not `js`)
 -  Code is passed to the formatter via stdin, formatted output read from stdout
 -  If the formatter fails (non-zero exit, timeout, etc.), the original code is
    preserved and a warning is emitted
 -  External formatters are only available in CLI mode (not in WASM)

To skip formatting for a specific code block, add `hongdown-no-format` after the
language identifier:

~~~~~ markdown
~~~~ python hongdown-no-format
def hello(): print("Hello, World!")
~~~~
~~~~~

The `hongdown-no-format` keyword is preserved in the output, ensuring the code
block remains unformatted on subsequent runs.

For WASM builds, use the `formatWithCodeFormatter` function with a callback:

~~~~ typescript
import { formatWithCodeFormatter } from "@hongdown/wasm";
import * as prettier from "prettier";

const { output, warnings } = await formatWithCodeFormatter(markdown, {
  codeFormatter: (language, code) => {
    if (language === "javascript" || language === "typescript") {
      return prettier.format(code, { parser: "babel" });
    }
    return null; // Keep original for other languages
  },
});
~~~~

### Line wrapping

 -  Lines wrap at approximately 80 display columns
 -  East Asian wide characters are counted as 2 columns
 -  Long words that cannot be broken are preserved

### Links

 -  External URLs are converted to reference-style links
 -  References are placed at the end of each section
 -  Relative/local URLs remain inline

~~~~ markdown
See the [documentation] for more details.

[documentation]: https://example.com/docs
~~~~

### Tables

 -  Pipes are aligned accounting for East Asian wide characters
 -  Minimum column width is maintained

See *[STYLE.md](./STYLE.md)* for the complete style specification, including
the philosophy behind these conventions and detailed formatting rules.


Editor integrations
-------------------

### Zed

Add the following to your Zed settings to use Hongdown as the Markdown
formatter (contributed by [Lee Dogeon][moreal zed]):

~~~~ json
{
  "languages": {
    "Markdown": {
      "formatter": {
        "external": {
          "command": "hongdown",
          "arguments": ["-"]
        }
      }
    }
  }
}
~~~~

[moreal zed]: https://hackers.pub/@moreal/019bb141-dc94-7103-ab3d-779941125430

### Neovim

If you use [none-ls.nvim] (a community-maintained fork of *null-ls.nvim*), you
can register Hongdown as a formatter (contributed by [Vladimir Rubin]):

~~~~ lua
local null_ls = require("null-ls")
local hongdown = {
    name = "hongdown",
    method = null_ls.methods.FORMATTING,
    filetypes = { "markdown" },
    generator = null_ls.generator({
        command = "hongdown",
        args = { "--stdin" },
        to_stdin = true,
        from_stderr = false,
        format = "raw",
        check_exit_code = function(code, stderr)
            local success = code <= 1
            if not success then
                print(stderr)
            end
            return success
        end,
        on_output = function(params, done)
            local output = params.output
            if not output then
                return done()
            end
            return done({ { text = output } })
        end,
    }),
}

null_ls.register(hongdown)
~~~~

[none-ls.nvim]: https://github.com/nvimtools/none-ls.nvim
[Vladimir Rubin]: https://github.com/dahlia/hongdown/issues/4

### Helix

If you use [Helix], you can register Hongdown as a formatter (contributed by
[Jean Simard]):

~~~~ toml
[[language]]
name = "markdown"
auto-format = true
formatter = { command = "hongdown", args = ["--stdin"] }
~~~~

[Helix]: https://helix-editor.com/
[Jean Simard]: https://github.com/dahlia/hongdown/pull/12


Library usage
-------------

### Rust

Hongdown can also be used as a Rust library:

~~~~ rust
use hongdown::{format, Options};

let input = "# Hello World\nThis is a paragraph.";
let options = Options::default();
let output = format(input, &options).unwrap();
println!("{}", output);
~~~~

### JavaScript/TypeScript

Hongdown is available as a WebAssembly-based library for JavaScript and
TypeScript:

~~~~ bash
npm install @hongdown/wasm
~~~~

~~~~ typescript
import { format, formatWithWarnings } from "@hongdown/wasm";

// Basic usage
const markdown = "# Hello\nWorld";
const formatted = await format(markdown);

// With options
const result = await format(markdown, {
  lineWidth: 100,
  setextH1: false,
  fenceChar: "`",
});

// Get warnings along with formatted output
const { output, warnings } = await formatWithWarnings(markdown);
if (warnings.length > 0) {
  for (const warning of warnings) {
    console.warn(`Line ${warning.line}: ${warning.message}`);
  }
}
~~~~

The library works in Node.js, Bun, Deno, and web browsers.  See the
[TypeScript type definitions] for all available options.

[TypeScript type definitions]: ./packages/wasm/src/types.ts


Development
-----------

This project uses [mise] for task management.

[mise]: https://mise.jdx.dev/

### Initial setup

After cloning the repository, set up the Git pre-commit hook to automatically
run quality checks before each commit:

~~~~ bash
mise generate git-pre-commit --task=check --write
~~~~

### Quality checks

The following tasks are available:

~~~~ bash
# Run all quality checks
mise run check

# Individual checks
mise run check:clippy     # Run clippy linter
mise run check:fmt        # Check code formatting
mise run check:type       # Run Rust type checking
mise run check:markdown   # Check Markdown formatting
~~~~

See *[AGENTS.md]* for detailed development guidelines including TDD
practices, code style conventions, and commit message guidelines.

[AGENTS.md]: ./AGENTS.md


Etymology
---------

The name *Hongdown* is a portmanteau of *Hong* (from Hong Minhee, the author)
and *Markdown*.  It also sounds like the Korean word *hongdapda* (홍답다),
meaning “befitting of Hong” or “Hong-like.”


License
-------

Distributed under the [GPL-3.0-or-later].  See *[LICENSE]* for more information.

[GPL-3.0-or-later]: https://www.gnu.org/licenses/gpl-3.0.html
[LICENSE]: ./LICENSE
