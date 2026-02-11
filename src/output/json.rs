use crate::finding::{Finding, Severity};
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct JsonOutput<'a> {
    version: &'static str,
    skill_path: String,
    findings: &'a [Finding],
    summary: JsonSummary,
}

#[derive(Serialize)]
struct JsonSummary {
    total: usize,
    errors: usize,
    warnings: usize,
    info: usize,
}

pub fn format_json(findings: &[Finding], skill_path: &Path) -> String {
    let output = JsonOutput {
        version: env!("CARGO_PKG_VERSION"),
        skill_path: skill_path.display().to_string(),
        findings,
        summary: JsonSummary {
            total: findings.len(),
            errors: findings
                .iter()
                .filter(|f| f.severity == Severity::Error)
                .count(),
            warnings: findings
                .iter()
                .filter(|f| f.severity == Severity::Warning)
                .count(),
            info: findings
                .iter()
                .filter(|f| f.severity == Severity::Info)
                .count(),
        },
    };

    serde_json::to_string_pretty(&output).unwrap_or_else(|e| format!("{{\"error\": \"{e}\"}}"))
}
