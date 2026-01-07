//! Hongdown CLI - A Markdown formatter for Hong Minhee's style conventions.

use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use hongdown::config::Config;
use hongdown::{Options, format_with_warnings};

/// A Markdown formatter that enforces Hong Minhee's Markdown style conventions.
#[derive(Parser, Debug)]
#[command(name = "hongdown")]
#[command(version, about, long_about = None)]
struct Args {
    /// Input file(s) to format. Use - for stdin.
    #[arg(value_name = "FILE")]
    files: Vec<PathBuf>,

    /// Write formatted output back to the input file(s).
    #[arg(short, long)]
    write: bool,

    /// Check if files are already formatted (exit 1 if not).
    #[arg(short, long)]
    check: bool,

    /// Read input from stdin.
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
    let config = load_config(&args);

    // Build options, with CLI args overriding config file
    let options = Options {
        line_width: args.line_width.unwrap_or(config.line_width),
        setext_h1: config.heading.setext_h1,
    };

    if args.stdin || args.files.is_empty() {
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
                print!("{}", result.output);
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("Error formatting: {}", e);
                ExitCode::FAILURE
            }
        }
    } else {
        let mut all_formatted = true;

        for file in &args.files {
            let input = match fs::read_to_string(file) {
                Ok(content) => content,
                Err(e) => {
                    eprintln!("Error reading {}: {}", file.display(), e);
                    return ExitCode::FAILURE;
                }
            };

            match format_with_warnings(&input, &options) {
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

                    if args.check {
                        if input != result.output {
                            eprintln!("{}: not formatted", file.display());
                            all_formatted = false;
                        }
                    } else if args.write {
                        if input != result.output
                            && let Err(e) = fs::write(file, &result.output)
                        {
                            eprintln!("Error writing {}: {}", file.display(), e);
                            return ExitCode::FAILURE;
                        }
                    } else {
                        print!("{}", result.output);
                    }
                }
                Err(e) => {
                    eprintln!("Error formatting {}: {}", file.display(), e);
                    return ExitCode::FAILURE;
                }
            }
        }

        if args.check && !all_formatted {
            ExitCode::FAILURE
        } else {
            ExitCode::SUCCESS
        }
    }
}

/// Load configuration from file or use defaults.
///
/// Priority:
/// 1. Explicit `--config` path
/// 2. Auto-discovered `.hongdown.toml` in current or parent directories
/// 3. Default configuration
fn load_config(args: &Args) -> Config {
    // If explicit config path is provided, use it
    if let Some(config_path) = &args.config {
        match Config::from_file(config_path) {
            Ok(config) => return config,
            Err(e) => {
                eprintln!("Warning: {}", e);
                return Config::default();
            }
        }
    }

    // Try to auto-discover config file from current directory
    let start_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    match Config::discover(&start_dir) {
        Ok(Some((_path, config))) => config,
        Ok(None) => Config::default(),
        Err(e) => {
            eprintln!("Warning: {}", e);
            Config::default()
        }
    }
}
