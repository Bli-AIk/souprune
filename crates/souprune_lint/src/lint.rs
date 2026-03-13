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
    // Accept ALL .ron files, not just known extensions
    RonFileKind::is_ron_file(&s)
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
        Some(RonFileKind::Performance) => {
            validate::<souprune_schema::danmaku::DanmakuPerformance>(&source)
        }
        Some(RonFileKind::Sequence) => {
            validate::<souprune_schema::sequence::SequenceAsset>(&source)
        }
        Some(RonFileKind::Enemy) => validate::<souprune_schema::enemy::EnemyDef>(&source),
        Some(RonFileKind::Items) => validate::<souprune_schema::item::ItemListAsset>(&source),
        Some(RonFileKind::BattlePlayer) => {
            validate::<souprune_schema::battle::BattlePlayerConfig>(&source)
        }
        Some(RonFileKind::Fre) => validate::<souprune_schema::fre::FreAsset>(&source),
        Some(RonFileKind::Input) => validate::<souprune_schema::config::InputConfig>(&source),
        Some(RonFileKind::States) => validate::<souprune_schema::config::StateConfig>(&source),
        Some(RonFileKind::TouchLayout) => {
            validate::<souprune_schema::config::TouchLayoutDef>(&source)
        }
        Some(RonFileKind::AmConfig) => validate::<souprune_schema::config::AmBattleConfig>(&source),
        Some(RonFileKind::Character) => validate_character(&source),
        Some(RonFileKind::PlayerBehavior) => {
            validate::<souprune_schema::overworld::PlayerBehaviorFile>(&source)
        }
        Some(RonFileKind::ChaseConfig) => {
            validate::<souprune_schema::overworld::ChaseConfig>(&source)
        }
        // Unknown .ron file — fall back to ron::Value syntax check
        None => validate_ron_syntax(&source),
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
        Err(spanned) => spanned_to_diagnostics(source, &spanned),
    }
}

/// Validate `.character.ron` — try CharacterAsset first, then AnimationConfigAsset.
fn validate_character(source: &str) -> Vec<Diagnostic> {
    if ron::from_str::<souprune_schema::character::CharacterAsset>(source).is_ok() {
        return Vec::new();
    }
    if ron::from_str::<souprune_schema::character::AnimationConfigAsset>(source).is_ok() {
        return Vec::new();
    }
    // Both failed — report the CharacterAsset error as primary
    match ron::from_str::<souprune_schema::character::CharacterAsset>(source) {
        Ok(_) => Vec::new(),
        Err(spanned) => spanned_to_diagnostics(source, &spanned),
    }
}

/// Fallback: validate unknown `.ron` files with `ron::Value` syntax check.
fn validate_ron_syntax(source: &str) -> Vec<Diagnostic> {
    match ron::from_str::<ron::Value>(source) {
        Ok(_) => vec![Diagnostic {
            severity: Severity::Warning,
            line: 0,
            column: 0,
            message: "unknown RON file type — syntax OK but no schema validation".to_string(),
            offset: 0,
            len: 0,
        }],
        Err(spanned) => spanned_to_diagnostics(source, &spanned),
    }
}

/// Convert a `SpannedError` to diagnostics.
fn spanned_to_diagnostics(source: &str, spanned: &ron::error::SpannedError) -> Vec<Diagnostic> {
    let start = &spanned.span.start;
    let offset = line_col_to_offset(source, start.line, start.col);

    vec![Diagnostic {
        severity: Severity::Error,
        line: start.line,
        column: start.col,
        message: format!("{}", spanned.code),
        offset,
        len: {
            let end = &spanned.span.end;
            let end_offset = line_col_to_offset(source, end.line, end.col);
            if end_offset > offset {
                end_offset - offset
            } else {
                1
            }
        },
    }]
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
