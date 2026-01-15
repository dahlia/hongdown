//! Hongdown CLI - A Markdown formatter for Hong Minhee's style conventions.

use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};

use clap::Parser;
use hongdown::config::Config;
use hongdown::{CodeFormatter, LineWidth, Options, format_with_warnings, validate_dash_settings};
use rayon::prelude::*;
use similar::{ChangeTag, TextDiff};
use walkdir::WalkDir;

/// A Markdown formatter that enforces Hong Minhee's Markdown style conventions.
#[derive(Parser, Debug)]
#[command(name = "hongdown")]
#[command(version, about, long_about = None)]
struct Args {
    /// Input file(s) to format. Use `-` to read from stdin.
    #[arg(value_name = "FILE")]
    files: Vec<PathBuf>,

    /// Write formatted output back to the input file(s).
    #[arg(short, long, conflicts_with_all = ["check", "diff"])]
    write: bool,

    /// Check if files are already formatted (exit 1 if not).
    #[arg(short, long, conflicts_with_all = ["write", "diff"])]
    check: bool,

    /// Show a diff of formatting changes.
    #[arg(short, long, conflicts_with_all = ["write", "check"])]
    diff: bool,

    /// Read input from stdin (alternative to using `-` as filename).
    #[arg(long)]
    stdin: bool,

    /// Line width for wrapping (overrides config file).
    #[arg(long)]
    line_width: Option<usize>,

    /// Path to configuration file.
    #[arg(long, value_name = "FILE")]
    config: Option<PathBuf>,
}

fn main() -> ExitCode {
    let args = Args::parse();

    // Load configuration
    let (config, config_dir) = load_config(&args);

    // Build options, with CLI args overriding config file
    let options = Options {
        line_width: args
            .line_width
            .map(|w| LineWidth::new(w).expect("Invalid line width"))
            .unwrap_or(config.line_width),
        setext_h1: config.heading.setext_h1,
        setext_h2: config.heading.setext_h2,
        heading_sentence_case: config.heading.sentence_case,
        heading_proper_nouns: config.heading.proper_nouns.clone(),
        heading_common_nouns: config.heading.common_nouns.clone(),
        unordered_marker: config.unordered_list.unordered_marker,
        leading_spaces: config.unordered_list.leading_spaces,
        trailing_spaces: config.unordered_list.trailing_spaces,
        indent_width: config.unordered_list.indent_width,
        odd_level_marker: config.ordered_list.odd_level_marker,
        even_level_marker: config.ordered_list.even_level_marker,
        ordered_list_pad: config.ordered_list.pad,
        ordered_list_indent_width: config.ordered_list.indent_width,
        fence_char: config.code_block.fence_char,
        min_fence_length: config.code_block.min_fence_length,
        space_after_fence: config.code_block.space_after_fence,
        default_language: config.code_block.default_language.clone(),
        thematic_break_style: config.thematic_break.style.clone(),
        thematic_break_leading_spaces: config.thematic_break.leading_spaces,
        curly_double_quotes: config.punctuation.curly_double_quotes,
        curly_single_quotes: config.punctuation.curly_single_quotes,
        curly_apostrophes: config.punctuation.curly_apostrophes,
        ellipsis: config.punctuation.ellipsis,
        en_dash: config.punctuation.en_dash.clone(),
        em_dash: config.punctuation.em_dash.clone(),
        code_formatters: config
            .code_block
            .formatters
            .iter()
            .map(|(lang, cfg)| {
                (
                    lang.clone(),
                    CodeFormatter {
                        command: cfg.command().to_vec(),
                        timeout_secs: cfg.timeout(),
                    },
                )
            })
            .collect(),
    };

    // Validate formatter configurations
    for (lang, cfg) in &config.code_block.formatters {
        if let Err(msg) = cfg.validate() {
            eprintln!("Error: formatter for '{}': {}", lang, msg);
            return ExitCode::FAILURE;
        }
    }

    // Validate punctuation settings
    if let Err(e) = validate_dash_settings(&options) {
        eprintln!("Error: {}", e);
        return ExitCode::FAILURE;
    }

    // Check if stdin is explicitly requested via --stdin or `-` as filename
    let stdin_requested = args.stdin || args.files.iter().any(|f| f.to_str() == Some("-"));

    // Determine files to process
    let files: Vec<PathBuf> = if args.files.is_empty() && !stdin_requested {
        // No files specified and stdin not requested, try to use include patterns from config
        if !config.include.is_empty() {
            match config.collect_files(&config_dir) {
                Ok(collected) => collected,
                Err(e) => {
                    eprintln!("Error collecting files: {}", e);
                    return ExitCode::FAILURE;
                }
            }
        } else {
            // No files, no stdin, no include patterns - error
            eprintln!("Error: no input files specified.");
            eprintln!("Use `hongdown --stdin` or `hongdown -` to read from stdin,");
            eprintln!("or specify file paths as arguments.");
            return ExitCode::FAILURE;
        }
    } else if stdin_requested {
        // Filter out `-` from files list since we'll handle stdin separately
        let filtered: Vec<PathBuf> = args
            .files
            .iter()
            .filter(|f| f.to_str() != Some("-"))
            .cloned()
            .collect();
        expand_paths(&filtered)
    } else {
        expand_paths(&args.files)
    };

    if stdin_requested {
        // Read from stdin
        let mut input = String::new();
        if let Err(e) = io::stdin().read_to_string(&mut input) {
            eprintln!("Error reading stdin: {}", e);
            return ExitCode::FAILURE;
        }

        match format_with_warnings(&input, &options) {
            Ok(result) => {
                // Print warnings to stderr
                for warning in &result.warnings {
                    eprintln!("<stdin>:{}: warning: {}", warning.line, warning.message);
                }
                if args.diff {
                    print_diff("<stdin>", &input, &result.output);
                } else {
                    print!("{}", result.output);
                }
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("Error formatting: {}", e);
                ExitCode::FAILURE
            }
        }
    } else if args.write || args.check {
        // Parallel processing for --write and --check modes
        process_files_parallel(&files, &options, args.write, args.check)
    } else if args.diff {
        // Diff mode for files
        process_files_diff(&files, &options)
    } else {
        // Sequential processing for stdout mode (order matters)
        process_files_sequential(&files, &options)
    }
}

/// Process files in parallel (for --write and --check modes).
fn process_files_parallel(
    files: &[PathBuf],
    options: &Options,
    write: bool,
    check: bool,
) -> ExitCode {
    let has_error = AtomicBool::new(false);
    let all_formatted = AtomicBool::new(true);

    files.par_iter().for_each(|file| {
        let input = match fs::read_to_string(file) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Error reading {}: {}", file.display(), e);
                has_error.store(true, Ordering::Relaxed);
                return;
            }
        };

        match format_with_warnings(&input, options) {
            Ok(result) => {
                // Print warnings to stderr
                for warning in &result.warnings {
                    eprintln!(
                        "{}:{}: warning: {}",
                        file.display(),
                        warning.line,
                        warning.message
                    );
                }

                if check {
                    if input != result.output {
                        eprintln!("{}: not formatted", file.display());
                        all_formatted.store(false, Ordering::Relaxed);
                    }
                } else if write && input != result.output {
                    match fs::write(file, &result.output) {
                        Ok(()) => {
                            println!("{}", file.display());
                        }
                        Err(e) => {
                            eprintln!("Error writing {}: {}", file.display(), e);
                            has_error.store(true, Ordering::Relaxed);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Error formatting {}: {}", file.display(), e);
                has_error.store(true, Ordering::Relaxed);
            }
        }
    });

    if has_error.load(Ordering::Relaxed) || (check && !all_formatted.load(Ordering::Relaxed)) {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

/// Process files sequentially (for stdout mode where order matters).
fn process_files_sequential(files: &[PathBuf], options: &Options) -> ExitCode {
    for file in files {
        let input = match fs::read_to_string(file) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Error reading {}: {}", file.display(), e);
                return ExitCode::FAILURE;
            }
        };

        match format_with_warnings(&input, options) {
            Ok(result) => {
                // Print warnings to stderr
                for warning in &result.warnings {
                    eprintln!(
                        "{}:{}: warning: {}",
                        file.display(),
                        warning.line,
                        warning.message
                    );
                }
                print!("{}", result.output);
            }
            Err(e) => {
                eprintln!("Error formatting {}: {}", file.display(), e);
                return ExitCode::FAILURE;
            }
        }
    }

    ExitCode::SUCCESS
}

/// Process files in diff mode.
fn process_files_diff(files: &[PathBuf], options: &Options) -> ExitCode {
    for file in files {
        let input = match fs::read_to_string(file) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Error reading {}: {}", file.display(), e);
                return ExitCode::FAILURE;
            }
        };

        match format_with_warnings(&input, options) {
            Ok(result) => {
                // Print warnings to stderr
                for warning in &result.warnings {
                    eprintln!(
                        "{}:{}: warning: {}",
                        file.display(),
                        warning.line,
                        warning.message
                    );
                }
                print_diff(&file.display().to_string(), &input, &result.output);
            }
            Err(e) => {
                eprintln!("Error formatting {}: {}", file.display(), e);
                return ExitCode::FAILURE;
            }
        }
    }

    ExitCode::SUCCESS
}

/// Print a unified diff between original and formatted content.
fn print_diff(filename: &str, original: &str, formatted: &str) {
    if original == formatted {
        return;
    }

    let diff = TextDiff::from_lines(original, formatted);

    println!("--- {}", filename);
    println!("+++ {}", filename);

    for hunk in diff.unified_diff().iter_hunks() {
        println!("{}", hunk.header());
        for change in hunk.iter_changes() {
            let sign = match change.tag() {
                ChangeTag::Delete => '-',
                ChangeTag::Insert => '+',
                ChangeTag::Equal => ' ',
            };
            print!("{}{}", sign, change.value());
            if !change.value().ends_with('\n') {
                println!();
            }
        }
    }
}

/// Expand paths, converting directories to their contained `.md` files.
///
/// If a path is a directory, recursively finds all `.md` files within it.
/// If a path is a file, it is included as-is.
fn expand_paths(paths: &[PathBuf]) -> Vec<PathBuf> {
    let mut result = Vec::new();
    for path in paths {
        if path.is_dir() {
            result.extend(collect_md_files(path));
        } else {
            result.push(path.clone());
        }
    }
    result
}

/// Recursively collect all Markdown files (`.md` and `.markdown`) from a directory.
fn collect_md_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in WalkDir::new(dir).follow_links(true) {
        let Ok(entry) = entry else { continue };
        let path = entry.path();
        if path.is_file() {
            let is_markdown = path.extension().is_some_and(|ext| {
                ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("markdown")
            });
            if is_markdown {
                files.push(path.to_path_buf());
            }
        }
    }
    files.sort();
    files
}

/// Load configuration from file or use defaults.
///
/// Returns the configuration and the base directory for glob patterns.
///
/// Priority:
/// 1. Explicit `--config` path (bypasses cascading)
/// 2. Cascading configuration:
///    - System config: `/etc/hongdown/config.toml`
///    - User legacy config: `~/.hongdown.toml`
///    - User XDG config: `$XDG_CONFIG_HOME/hongdown/config.toml`
///    - Project config: `.hongdown.toml` in current or parent directories
/// 3. Default configuration
fn load_config(args: &Args) -> (Config, PathBuf) {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    // If explicit config path is provided, use it without cascading
    if let Some(config_path) = &args.config {
        match Config::from_file(config_path) {
            Ok(config) => {
                let config_dir = config_path
                    .parent()
                    .map(|p| p.to_path_buf())
                    .unwrap_or_else(|| cwd.clone());
                return (config, config_dir);
            }
            Err(e) => {
                eprintln!("Warning: {}", e);
                return (Config::default(), cwd);
            }
        }
    }

    // Use cascading config loading from all sources
    match Config::load_cascading(&cwd) {
        Ok((config, maybe_project_path)) => {
            let config_dir = maybe_project_path
                .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                .unwrap_or_else(|| cwd.clone());
            (config, config_dir)
        }
        Err(e) => {
            eprintln!("Warning: {}", e);
            (Config::default(), cwd)
        }
    }
}
