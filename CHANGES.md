Hongdown changelog
==================

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
