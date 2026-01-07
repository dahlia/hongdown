Hongdown
========

Hongdown is a Markdown formatter that enforces Hong Minhee's Markdown style
conventions.  The formatter is implemented in Rust using the [comrak] library
for parsing.  It produces consistently formatted Markdown output following
a distinctive style used across multiple projects including [Fedify], [Hollo],
and [Vertana].

The name *Hongdown* is a portmanteau of *Hong* (from Hong Minhee, the author)
and *Markdown*.  It also sounds like the Korean word *hongdapda* (홍답다),
meaning "befitting of Hong" or "Hong-like."

[comrak]: https://docs.rs/comrak
[Fedify]: https://github.com/dahlia/fedify
[Hollo]: https://github.com/dahlia/hollo
[Vertana]: https://github.com/dahlia/vertana


Installation
------------

### From source

~~~~ bash
cargo install hongdown
~~~~


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

Hongdown looks for a `.hongdown.toml` file in the current directory and
parent directories.  You can also specify a configuration file explicitly
with the `--config` option.

~~~~ toml
# File patterns (glob syntax)
include = ["*.md", "docs/**/*.md"]  # Files to format
exclude = ["node_modules/**"]        # Files to skip

# Formatting options
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
min_fence_length = 4      # Minimum fence length
space_after_fence = true  # Space between fence and language
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

See [SPEC.md] for the complete style specification.

[SPEC.md]: SPEC.md


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


TODO
----

### Phase 1: Core formatting

 -  [x] Basic CLI with file input/output
 -  [x] Front matter preservation (YAML/TOML)
 -  [x] Headings (setext and ATX)
 -  [x] Paragraphs with line wrapping
 -  [x] Lists (ordered and unordered)
 -  [x] Code blocks (fenced)
 -  [x] Block quotes
 -  [x] Inline formatting (emphasis, strong, code, links)
 -  [x] Basic test suite

### Phase 2: Extended syntax

 -  [x] Tables with alignment
 -  [x] Definition lists
 -  [x] GitHub alerts/admonitions
 -  [x] Footnotes
 -  [x] Reference link collection and placement

### Phase 3: Polish

 -  [x] Configuration file support
    -  [x] Config file parsing and discovery (`.hongdown.toml`)
    -  [x] `line_width` option
    -  [x] `[heading]` section
        -  [x] `setext_h1` option
        -  [x] `setext_h2` option
    -  [x] `[list]` section
        -  [x] `unordered_marker` option
        -  [x] `leading_spaces` option
        -  [x] `trailing_spaces` option
        -  [x] `indent_width` option
    -  [x] `[ordered_list]` section
        -  [x] `odd_level_marker` option
        -  [x] `even_level_marker` option
    -  [x] `[code_block]` section
        -  [x] `fence_char` option
        -  [x] `min_fence_length` option
        -  [x] `space_after_fence` option
 -  [x] Check mode for CI integration
 -  [x] Diff output mode
 -  [x] Disable directives
 -  [x] Edge case handling
 -  [ ] Performance optimization


License
-------

Distributed under the [GPL-3.0-or-later].  See [LICENSE] for more information.

[GPL-3.0-or-later]: https://www.gnu.org/licenses/gpl-3.0.html
[LICENSE]: LICENSE
