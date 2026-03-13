//! # souprune-lint
//!
//! RON file linter for SoupRune projects.
//! Validates `.view.ron` and `.sdf.ron` files against the SoupRune schema.
//!
//! SoupRune 项目的 RON 文件 Linter。
//! 根据 SoupRune Schema 校验 `.view.ron` 和 `.sdf.ron` 文件。
//!
//! ## Usage
//!
//! ```bash
//! # Check a single file
//! souprune-lint check path/to/file.view.ron
//!
//! # Check a directory recursively
//! souprune-lint check projects/
//!
//! # JetBrains File Watcher format
//! souprune-lint check --format jetbrains path/to/file.view.ron
//! ```

mod lint;

use clap::{Parser, ValueEnum};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    name = "souprune-lint",
    about = "RON file linter for SoupRune projects"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Validate RON files against SoupRune schemas.
    Check {
        /// Files or directories to check.
        #[arg(required = true)]
        paths: Vec<PathBuf>,

        /// Output format.
        #[arg(long, default_value = "pretty")]
        format: OutputFormat,
    },
}

#[derive(ValueEnum, Clone, Copy, Debug)]
enum OutputFormat {
    /// Colorful diagnostic output (ariadne).
    Pretty,
    /// `file:line:col: level: message` — compatible with JetBrains File Watcher.
    Jetbrains,
    /// Machine-readable JSON.
    Json,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check { paths, format } => {
            let results = lint::check_paths(&paths);
            let has_errors = results.iter().any(|r| !r.diagnostics.is_empty());

            for result in &results {
                match format {
                    OutputFormat::Pretty => lint::output::print_pretty(result),
                    OutputFormat::Jetbrains => lint::output::print_jetbrains(result),
                    OutputFormat::Json => lint::output::print_json(result),
                }
            }

            if has_errors {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
    }
}
