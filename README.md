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
cargo install hongdown --features cli
~~~~


Usage
-----

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


Style rules
-----------

Hongdown enforces the following conventions:

 -  *Headings*: Setext-style for levels 1-2, ATX-style for levels 3+
 -  *Lists*: ` -  ` format with 4-space indentation
 -  *Code blocks*: Fenced with `~~~~` (tildes)
 -  *Line wrapping*: At approximately 80 display columns
 -  *Tables*: Aligned pipes accounting for East Asian wide characters

See [SPEC.md] for the complete style specification.

[SPEC.md]: SPEC.md


TODO
----

### Phase 1: Core formatting

 -  [ ] Basic CLI with file input/output
 -  [ ] Front matter preservation (YAML/TOML)
 -  [x] Headings (setext and ATX)
 -  [ ] Paragraphs with line wrapping
 -  [x] Lists (ordered and unordered)
 -  [x] Code blocks (fenced)
 -  [x] Block quotes
 -  [ ] Inline formatting (emphasis, strong, code, links)
 -  [ ] Basic test suite

### Phase 2: Extended syntax

 -  [ ] Tables with alignment
 -  [ ] Definition lists
 -  [ ] GitHub alerts/admonitions
 -  [ ] Footnotes
 -  [ ] Reference link collection and placement

### Phase 3: Polish

 -  [ ] Configuration file support
 -  [ ] Diff output mode
 -  [ ] Check mode for CI integration
 -  [ ] Edge case handling
 -  [ ] Performance optimization
 -  [ ] Comprehensive test coverage


License
-------

Distributed under the [GPL-3.0-or-later].  See [LICENSE] for more information.

[GPL-3.0-or-later]: https://www.gnu.org/licenses/gpl-3.0.html
[LICENSE]: LICENSE
