pub mod json;
pub mod sarif;
pub mod table;

use crate::finding::Finding;
use std::path::Path;

pub fn format_findings(
    format: &crate::config::OutputFormat,
    findings: &[Finding],
    skill_path: &Path,
) -> String {
    match format {
        crate::config::OutputFormat::Table => table::format_table(findings),
        crate::config::OutputFormat::Json => json::format_json(findings, skill_path),
        crate::config::OutputFormat::Sarif => sarif::format_sarif(findings, skill_path),
    }
}
