//! Output formatters for lint diagnostics.

use super::{CheckResult, Severity};
use ariadne::{Color, Label, Report, ReportKind, Source};

/// Pretty output using ariadne.
pub fn print_pretty(result: &CheckResult) {
    if result.diagnostics.is_empty() {
        return;
    }

    let path_str = result.path.to_string_lossy().to_string();
    let path: &str = &path_str;

    for diag in &result.diagnostics {
        // If we have no source content, fall back to simple text output.
        if result.source.is_empty() {
            let severity = match diag.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
            };
            eprintln!("{severity}: {}: {}", path_str, diag.message);
            continue;
        }

        let kind = match diag.severity {
            Severity::Error => ReportKind::Error,
            Severity::Warning => ReportKind::Warning,
        };

        let color = match diag.severity {
            Severity::Error => Color::Red,
            Severity::Warning => Color::Yellow,
        };

        let end = (diag.offset + diag.len).min(result.source.len());
        let start = diag.offset.min(end);

        Report::build(kind, (path, start..end))
            .with_message(&diag.message)
            .with_label(
                Label::new((path, start..end))
                    .with_message(&diag.message)
                    .with_color(color),
            )
            .finish()
            .eprint((path, Source::from(&result.source)))
            .ok();
    }
}

/// JetBrains File Watcher compatible output:
/// `file:line:col: severity: message`
pub fn print_jetbrains(result: &CheckResult) {
    let path = result.path.display();
    for diag in &result.diagnostics {
        let severity = match diag.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        println!(
            "{path}:{line}:{col}: {severity}: {msg}",
            line = diag.line,
            col = diag.column,
            msg = diag.message,
        );
    }
}

/// JSON output for tool integration.
pub fn print_json(result: &CheckResult) {
    if result.diagnostics.is_empty() {
        return;
    }

    let diagnostics: Vec<JsonDiagnostic> = result
        .diagnostics
        .iter()
        .map(|d| JsonDiagnostic {
            severity: match d.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
            },
            line: d.line,
            column: d.column,
            message: &d.message,
        })
        .collect();

    let output = JsonOutput {
        file: &result.path.to_string_lossy(),
        diagnostics,
    };

    // Use serde_json-like manual serialization to avoid extra dependency.
    println!("{{");
    println!("  \"file\": {:?},", output.file);
    println!("  \"diagnostics\": [");
    for (i, d) in output.diagnostics.iter().enumerate() {
        let comma = if i + 1 < output.diagnostics.len() {
            ","
        } else {
            ""
        };
        println!(
            "    {{ \"severity\": {:?}, \"line\": {}, \"column\": {}, \"message\": {:?} }}{comma}",
            d.severity, d.line, d.column, d.message,
        );
    }
    println!("  ]");
    println!("}}");
}

struct JsonOutput<'a> {
    file: &'a str,
    diagnostics: Vec<JsonDiagnostic<'a>>,
}

struct JsonDiagnostic<'a> {
    severity: &'a str,
    line: usize,
    column: usize,
    message: &'a str,
}
