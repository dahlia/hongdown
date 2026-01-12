Hongdown changelog
==================

Version 0.2.0
-------------

To be released.

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

[#2]: https://github.com/dahlia/hongdown/issues/2
[#3]: https://github.com/dahlia/hongdown/pull/3
[#5]: https://github.com/dahlia/hongdown/pull/5


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
