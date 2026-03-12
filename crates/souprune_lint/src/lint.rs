//! Core linting logic.

pub mod output;

use souprune_schema::RonFileKind;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// A single diagnostic message.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub line: usize,
    pub column: usize,
    pub message: String,
    /// Byte offset into the source for ariadne spans.
    pub offset: usize,
    /// Length of the span (0 if unknown).
    pub len: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

/// Result of checking a single file.
#[derive(Debug)]
pub struct CheckResult {
    pub path: PathBuf,
    pub source: String,
    pub diagnostics: Vec<Diagnostic>,
}

/// Check all paths (files or directories), returning results for each RON file.
pub fn check_paths(paths: &[PathBuf]) -> Vec<CheckResult> {
    let mut results = Vec::new();

    for path in paths {
        if path.is_dir() {
            let ron_files = WalkDir::new(path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file() && is_supported_ron(e.path()));
            results.extend(ron_files.map(|e| check_file(e.path())));
        } else if path.is_file() {
            results.push(check_file(path));
        } else {
            results.push(not_found_result(path));
        }
    }

    results
}

fn is_supported_ron(path: &Path) -> bool {
    let s = path.to_string_lossy();
    RonFileKind::all_extensions()
        .iter()
        .any(|ext| s.ends_with(ext))
}

fn not_found_result(path: &Path) -> CheckResult {
    CheckResult {
        path: path.to_path_buf(),
        source: String::new(),
        diagnostics: vec![Diagnostic {
            severity: Severity::Error,
            line: 0,
            column: 0,
            message: format!("path not found: {}", path.display()),
            offset: 0,
            len: 0,
        }],
    }
}

/// Check a single RON file.
fn check_file(path: &Path) -> CheckResult {
    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            return CheckResult {
                path: path.to_path_buf(),
                source: String::new(),
                diagnostics: vec![Diagnostic {
                    severity: Severity::Error,
                    line: 0,
                    column: 0,
                    message: format!("failed to read file: {e}"),
                    offset: 0,
                    len: 0,
                }],
            };
        }
    };

    let path_str = path.to_string_lossy();
    let kind = RonFileKind::from_path(&path_str);

    let diagnostics = match kind {
        Some(RonFileKind::View) => validate::<souprune_schema::view::ViewLayout>(&source),
        Some(RonFileKind::SdfStructure) => validate::<souprune_schema::view::SdfStructure>(&source),
        None => vec![Diagnostic {
            severity: Severity::Warning,
            line: 0,
            column: 0,
            message: format!("unrecognized file type: {path_str}"),
            offset: 0,
            len: 0,
        }],
    };

    CheckResult {
        path: path.to_path_buf(),
        source,
        diagnostics,
    }
}

/// Attempt to deserialize a RON string and convert errors to diagnostics.
fn validate<T: serde::de::DeserializeOwned>(source: &str) -> Vec<Diagnostic> {
    match ron::from_str::<T>(source) {
        Ok(_) => Vec::new(),
        Err(spanned) => {
            let start_pos = &spanned.span.start;
            let end_pos = &spanned.span.end;
            let offset = line_col_to_offset(source, start_pos.line, start_pos.col);
            let end_offset = line_col_to_offset(source, end_pos.line, end_pos.col);

            vec![Diagnostic {
                severity: Severity::Error,
                line: start_pos.line,
                column: start_pos.col,
                message: format!("{}", spanned.code),
                offset,
                len: if end_offset > offset {
                    end_offset - offset
                } else {
                    1
                },
            }]
        }
    }
}

/// Convert 1-based line/column to byte offset.
fn line_col_to_offset(source: &str, line: usize, col: usize) -> usize {
    let mut current_line = 1;
    let mut current_col = 1;

    for (i, ch) in source.char_indices() {
        if current_line == line && current_col == col {
            return i;
        }
        if ch == '\n' {
            current_line += 1;
            current_col = 1;
        } else {
            current_col += 1;
        }
    }

    source.len()
}
