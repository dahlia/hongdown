Guidance for LLM-based code agents
==================================

This file provides guidance to LLM-based code agents (e.g., Claude Code,
OpenCode) when working with code in this repository.


Project overview
----------------

Hongdown is a Markdown formatter that enforces Hong Minhee's Markdown style
conventions.  The formatter is implemented in Rust (Edition 2024) using the
comrak library for parsing.  It produces consistently formatted Markdown
output following a distinctive style used across multiple projects including
Fedify, Hollo, and Vertana.

### Project architecture

~~~~ text
hongdown/
├── src/
│   ├── main.rs           # CLI entry point (clap-based argument parsing)
│   ├── lib.rs            # Library entry point (Options, format functions)
│   ├── config.rs         # Configuration file handling (.hongdown.toml)
│   └── serializer/       # Core formatting logic
│       ├── mod.rs        # Main serializer module
│       ├── state.rs      # Serializer state and types
│       ├── block.rs      # Block quotes and GitHub alerts
│       ├── code.rs       # Code blocks (fenced and indented)
│       ├── document.rs   # Document-level handling
│       ├── escape.rs     # Text escaping utilities
│       ├── inline.rs     # Inline elements (emphasis, code spans)
│       ├── link.rs       # Links and images
│       ├── list.rs       # Ordered and unordered lists
│       ├── table.rs      # Table formatting
│       ├── wrap.rs       # Text wrapping utilities
│       └── tests.rs      # Unit tests for serializer
├── tests/
│   └── integration.rs    # Integration tests
└── npm/                  # npm package distribution
~~~~


Development commands
--------------------

This is a Rust project using Cargo as the build system.  We use [mise] for
task management to streamline common development workflows.

[mise]: https://mise.jdx.dev/

### Initial setup

After cloning the repository, set up the Git pre-commit hook to automatically
run quality checks before each commit:

~~~~ bash
mise generate git-pre-commit --task=check --write
~~~~

### Building

~~~~ bash
cargo build            # Debug build
cargo build --release  # Release build
~~~~

### Testing

~~~~ bash
cargo test                        # Run all tests
cargo test <name>                 # Run tests matching a name pattern
cargo test serialize_table        # e.g., runs all table serialization tests
cargo test -- --nocapture         # Show println! output
cargo test -- --test-threads=1    # Run tests sequentially
~~~~

Tests are located in two places:

 -  *Unit tests*: *src/serializer/tests.rs* contains serializer tests
 -  *Integration tests*: *tests/integration.rs* contains CLI and full
    document tests

Test helper functions in *src/serializer/tests.rs*:

 -  `parse_and_serialize(input)` - Format with default options
 -  `parse_and_serialize_with_options(input, options)` - Format with custom
    options
 -  `parse_and_serialize_with_source(input)` - Format with source context
    for directive support
 -  `parse_and_serialize_with_warnings(input)` - Format and capture warnings

### Quality checks

The recommended way to run quality checks is using mise tasks:

~~~~ bash
mise run check         # Run all quality checks (recommended)
~~~~

This runs all checks in parallel:

 -  `mise run check:clippy` - Run clippy linter with warnings as errors
 -  `mise run check:fmt` - Check code formatting
 -  `mise run check:type` - Run Rust type checking
 -  `mise run check:markdown` - Check Markdown formatting

You can also run individual Cargo commands directly:

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

 -  *Before committing*: Always run the full quality check suite.  This is
    automatically enforced by the Git pre-commit hook:

    ~~~~ bash
    mise run check     # Runs all quality checks
    ~~~~

    The pre-commit hook runs `mise run check` automatically before each
    commit.  If any check fails, the commit will be aborted.

### Performance considerations

 -  *Parallel processing*: The CLI uses `rayon` for parallel file
    processing in `--write` and `--check` modes.  Keep this in mind
    when modifying file processing logic.

 -  *Avoid unnecessary allocations*: Prefer borrowing over cloning when
    possible.  Use `&str` instead of `String` for read-only string data.


Code style
----------

This project uses default `rustfmt` formatting (no *rustfmt.toml*) and default
clippy lints (no *clippy.toml*).  Run `cargo fmt` to format code.

### Imports

Organize imports in this order, with blank lines between groups:

 1.  Standard library (`use std::...`)
 2.  External crates (`use clap::...`, `use comrak::...`)
 3.  Internal modules (`use super::...`, `use crate::...`)

~~~~ rust
use std::fs;
use std::path::PathBuf;

use clap::Parser;
use comrak::{Arena, Options as ComrakOptions};

use crate::config::Config;
~~~~

### Naming conventions

 -  *Modules and files*: `snake_case` (e.g., *code\_block.rs*)
 -  *Types and traits*: `PascalCase` (e.g., `Serializer`, `FormatError`)
 -  *Functions and methods*: `snake_case` (e.g., `serialize_node`)
 -  *Constants*: `SCREAMING_SNAKE_CASE` (e.g., `CONFIG_FILE_NAME`)
 -  *Method prefixes*: Use `serialize_*` for serialization, `is_*` for
    boolean checks, `with_*` for builder patterns

### Type safety

 -  All code must be type-safe.  Avoid using `unsafe` blocks unless
    absolutely necessary.
 -  Prefer immutable data structures unless there is a specific reason to
    use mutable ones.

### Error handling

 -  Define custom error types as `enum` with descriptive variants.
 -  Manually implement `std::fmt::Display` and `std::error::Error` traits.
 -  Include context in error messages (e.g., file paths, pattern names).
 -  Prefer `Result` over `panic!` for recoverable errors.
 -  End error messages with a period.

Example error type pattern:

~~~~ rust
#[derive(Debug)]
pub enum ConfigError {
    Io(PathBuf, std::io::Error),
    Parse(PathBuf, toml::de::Error),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Io(path, err) => {
                write!(f, "failed to read {}: {}", path.display(), err)
            }
            ConfigError::Parse(path, err) => {
                write!(f, "failed to parse {}: {}", path.display(), err)
            }
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigError::Io(_, err) => Some(err),
            ConfigError::Parse(_, err) => Some(err),
        }
    }
}
~~~~

### Struct patterns

 -  Use `#[derive(...)]` extensively for common traits.
 -  Common derives: `Debug`, `Clone`, `PartialEq`.
 -  For serde: `#[derive(Deserialize)]`, `#[serde(default)]`,
    `#[serde(rename_all = "lowercase")]`.
 -  Implement `Default` for configuration structs.

### API documentation

 -  All public APIs must have doc comments describing their purpose,
    parameters, and return values.
 -  Use `///` for item documentation and `//!` for module documentation.


Markdown style guide
--------------------

This project follows [Hong Minhee's Markdown style convention], documented
in *[STYLE.md]*.  That document is the authoritative specification for all
formatting rules enforced by Hongdown.

When modifying Hongdown's formatting behavior, you must also update
*STYLE.md* to reflect the changes.  The specification should always match
the actual behavior of the formatter.

[Hong Minhee's Markdown style convention]: ./STYLE.md
[STYLE.md]: ./STYLE.md
