Hongdown
========

[![crates.io][crates.io badge]][crates.io]
[![npm][npm badge]][npm]
[![GitHub Actions][GitHub Actions badge]][GitHub Actions]

Hongdown is a Markdown formatter that enforces [Hong Minhee's Markdown
style](./STYLE.md) conventions.  The formatter is implemented in Rust using
the [Comrak] library for parsing.  It produces consistently formatted Markdown
output following a distinctive style used across multiple projects including
[Fedify], [LogTape], and [Optique].

[crates.io badge]: https://img.shields.io/crates/v/hongdown?logo=rust
[crates.io]: https://crates.io/crates/hongdown
[npm badge]: https://img.shields.io/npm/v/hongdown?logo=npm
[npm]: https://www.npmjs.com/package/hongdown
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

# Check if files are formatted (exit 1 if not)
hongdown --check input.md
hongdown -c input.md

# Show diff of formatting changes
hongdown --diff input.md
hongdown -d input.md

# Read from stdin
echo "# Hello" | hongdown
hongdown --stdin < input.md

# Custom line width
hongdown --line-width 100 input.md
~~~~

### Disable directives

Hongdown supports special HTML comments to disable formatting for specific
sections of your document:

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
line_width = 80           # Maximum line width (default: 80)

[heading]
setext_h1 = true          # Use === underline for h1 (default: true)
setext_h2 = true          # Use --- underline for h2 (default: true)

[list]
unordered_marker = "-"    # "-", "*", or "+" (default: "-")
leading_spaces = 1        # Spaces before marker (default: 1)
trailing_spaces = 2       # Spaces after marker (default: 2)
indent_width = 4          # Indentation for nested items (default: 4)

[ordered_list]
odd_level_marker = "."    # "." or ")" at odd nesting levels (default: ".")
even_level_marker = ")"   # "." or ")" at even nesting levels (default: ")")
pad = "start"             # "start" or "end" for number alignment (default: "start")
indent_width = 4          # Indentation for nested items (default: 4)

[code_block]
fence_char = "~"          # "~" or "`" (default: "~")
min_fence_length = 4      # Minimum fence length (default: 4)
space_after_fence = true  # Space between fence and language (default: true)
default_language = ""     # Default language for code blocks (default: "")

[thematic_break]
style = "- - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -"
leading_spaces = 3        # Leading spaces (0-3, default: 3)
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

~~~~ markdown
Document Title
==============

Section
-------

### Subsection
~~~~

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


Library usage
-------------

Hongdown can also be used as a Rust library:

~~~~ rust
use hongdown::{format, Options};

let input = "# Hello World\nThis is a paragraph.";
let options = Options::default();
let output = format(input, &options).unwrap();
println!("{}", output);
~~~~


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
