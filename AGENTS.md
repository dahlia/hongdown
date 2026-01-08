Guidance for LLM-based code agents
==================================

This file provides guidance to LLM-based code agents (e.g., Claude Code,
OpenCode) when working with code in this repository.


Project overview
----------------

Hongdown is a Markdown formatter that enforces Hong Minhee's Markdown style
conventions.  The formatter is implemented in Rust using the comrak library
for parsing.  It produces consistently formatted Markdown output following
a distinctive style used across multiple projects including Fedify, Hollo,
and Vertana.


Development commands
--------------------

This is a Rust project using Cargo as the build system.

### Building

~~~~ bash
cargo build            # Debug build
cargo build --release  # Release build
~~~~

### Testing

~~~~ bash
cargo test             # Run all tests
cargo test <name>      # Run specific test
~~~~

### Quality checks

~~~~ bash
cargo fmt              # Format code
cargo fmt --check      # Check formatting without modifying
cargo clippy           # Run linter
~~~~


Development practices
---------------------

### Test-driven development

This project strictly follows test-driven development (TDD) practices.
All new code must be developed using the TDD cycle:

 1.  *Red*: Write a failing test that describes the expected behavior.
    Run the test to confirm it fails.
 2.  *Green*: Write the minimum code necessary to make the test pass.
    Run the test to confirm it passes.
 3.  *Refactor*: Improve the code while keeping all tests passing.
    Run tests after each refactoring step.

Additional TDD guidelines:

 -  *Write tests first*: Before implementing new functionality, write tests
    that describe the expected behavior.  Confirm that the tests fail before
    proceeding with the implementation.
 -  *Regression tests for bugs*: When fixing bugs, first write a regression
    test that reproduces the bug.  Confirm that the test fails, then fix the
    bug and verify the test passes.
 -  *Small increments*: Implement features in small, testable increments.
    Each increment should have its own test.
 -  *Run tests frequently*: Run `cargo test` after every change to ensure
    existing functionality is not broken.

### Commit messages

 -  Do not use Conventional Commits (no `fix:`, `feat:`, etc. prefixes).
    Keep the first line under 50 characters when possible.

 -  Focus on *why* the change was made, not just *what* changed.

 -  When referencing issues or PRs, use permalink URLs instead of just
    numbers (e.g., `#123`).  This preserves context if the repository
    is moved later.

 -  When listing items after a colon, add a blank line after the colon:

    ~~~~
    This commit includes the following changes:

    - Added foo
    - Fixed bar
    ~~~~


 -  When using LLMs or coding agents, include credit via `Co-Authored-By:`.
    Include a permalink to the agent session if available.


Development tips
----------------

### Dependency management

 -  *Always use `cargo add`*: When adding dependencies, use `cargo add`
    instead of manually editing *Cargo.toml*.  This ensures you get the
    latest compatible version:

    ~~~~ bash
    cargo add serde --features derive
    cargo add tokio --features full
    ~~~~


 -  *Check before adding*: Before adding a new dependency, consider whether
    the functionality can be achieved with existing dependencies or the
    standard library.

### Configuration changes

 -  *Options struct*: When adding new configuration options, update both
    the `Config` struct in *src/config.rs* and the `Options` struct in
    *src/lib.rs*.  Also update *src/main.rs* to wire them together.

 -  *Update documentation*: When adding configuration options, update the
    Configuration file section in *README.md*.

### Code organization

 -  *Serializer modules*: The serializer is split into multiple modules
    under *src/serializer/*.  Each module handles a specific type of
    Markdown element (e.g., *list.rs*, *table.rs*, *code.rs*).

 -  *Test location*: Unit tests for the serializer are in
    *src/serializer/tests.rs*.  Integration tests are in *tests/*.

### Quality checks

 -  *Run checks frequently*: Run `cargo fmt --check` and `cargo clippy`
    frequently during development, not just before committing.  This
    catches issues early and keeps the codebase clean.

 -  *Format Markdown files*: After editing any Markdown files (*.md*),
    always format them with Hongdown:

    ~~~~ bash
    cargo run -- -w README.md AGENTS.md
    ~~~~


 -  *Before committing*: Always run the full quality check suite:

    ~~~~ bash
    cargo test && cargo fmt --check && cargo clippy -- -D warnings
    ~~~~


### Performance considerations

 -  *Parallel processing*: The CLI uses `rayon` for parallel file
    processing in `--write` and `--check` modes.  Keep this in mind
    when modifying file processing logic.

 -  *Avoid unnecessary allocations*: Prefer borrowing over cloning when
    possible.  Use `&str` instead of `String` for read-only string data.


Code style
----------

### Type safety

 -  All code must be type-safe.  Avoid using `unsafe` blocks unless
    absolutely necessary.
 -  Prefer immutable data structures unless there is a specific reason to
    use mutable ones.

### Error handling

 -  Use the `thiserror` crate for defining custom error types.
 -  Prefer `Result` over `panic!` for recoverable errors.
 -  End error messages with a period.

### API documentation

 -  All public APIs must have doc comments describing their purpose,
    parameters, and return values.
 -  Use `///` for item documentation and `//!` for module documentation.


Markdown style guide
--------------------

When creating or editing Markdown documentation files in this project,
follow these style conventions to maintain consistency with existing
documentation:

### Headings

 -  *Setext-style headings*: Use underline-style for the document title
    (with `=`) and sections (with `-`):

    ~~~~
    Document Title
    ==============

    Section Name
    ------------
    ~~~~


 -  *ATX-style headings*: Use only for subsections within a section:

    ~~~~
    ### Subsection Name
    ~~~~


 -  *Heading case*: Use sentence case (capitalize only the first word and
    proper nouns) rather than Title Case:

    ~~~~
    Development commands    <- Correct
    Development Commands    <- Incorrect
    ~~~~


### Text formatting

 -  *Italics* (`*text*`): Use for emphasis and to distinguish concepts, and
    for file paths and filenames
 -  *Bold* (`**text**`): Use sparingly for strong emphasis
 -  *Inline code* (`` `code` ``): Use for code spans, function names,
    and command-line options

### Lists

 -  Use ` -  ` (space-hyphen-two spaces) for unordered list items

 -  Indent nested items with 4 spaces

 -  Align continuation text with the item content:

    ~~~~
     -  *First item*: Description text that continues
        on the next line with proper alignment
     -  *Second item*: Another item
    ~~~~


### Code blocks

 -  Use four tildes (`~~~~`) for code fences instead of backticks

 -  Always specify the language identifier:

    ~~~~~
    ~~~~ rust
    let example = "Hello, world!";
    ~~~~
    ~~~~~


 -  For shell commands, use `bash`:

    ~~~~~
    ~~~~ bash
    cargo test
    ~~~~
    ~~~~~


### Links

 -  Use reference-style links placed at the *end of each section*
    (not at document end)

 -  Format reference links with consistent spacing:

    ~~~~
    See the [comrak documentation] for parsing details.

    [comrak documentation]: https://docs.rs/comrak
    ~~~~


### GitHub alerts

Use GitHub-style alert blocks for important information:

 -  *Note*: `> [!NOTE]`
 -  *Tip*: `> [!TIP]`
 -  *Important*: `> [!IMPORTANT]`
 -  *Warning*: `> [!WARNING]`
 -  *Caution*: `> [!CAUTION]`

Continue alert content on subsequent lines with `>`:

~~~~ text
> [!CAUTION]
> This feature is experimental and may change in future versions.
~~~~

### Tables

Use pipe tables with proper alignment markers:

~~~~ text
| Column 1        | Column 2                      |
| --------------- | ----------------------------- |
| Cell 1          | Cell 2                        |
~~~~

### Spacing and line length

 -  Wrap lines at approximately 80 characters for readability
 -  Use one blank line between sections and major elements
 -  Use two blank lines before Setext-style section headings
 -  Place one blank line before and after code blocks
 -  End sections with reference links (if any) followed by a blank line
