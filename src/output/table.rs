use crate::finding::{Finding, Severity};
use colored::Colorize;
use comfy_table::{
    modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Cell, Color as TableColor,
    ContentArrangement, Table,
};

pub fn format_table(findings: &[Finding]) -> String {
    if findings.is_empty() {
        return format!("{}", "No issues found.".green());
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Severity", "Rule", "File", "Line", "Message"]);

    for finding in findings {
        let severity_cell = match finding.severity {
            Severity::Error => Cell::new("ERROR").fg(TableColor::Red),
            Severity::Warning => Cell::new("WARN").fg(TableColor::Yellow),
            Severity::Info => Cell::new("INFO").fg(TableColor::Cyan),
        };

        table.add_row(vec![
            severity_cell,
            Cell::new(&finding.rule_id),
            Cell::new(finding.location.file.display().to_string()),
            Cell::new(format!(
                "{}:{}",
                finding.location.line, finding.location.column
            )),
            Cell::new(&finding.message),
        ]);
    }

    let error_count = findings
        .iter()
        .filter(|f| f.severity == Severity::Error)
        .count();
    let warn_count = findings
        .iter()
        .filter(|f| f.severity == Severity::Warning)
        .count();
    let info_count = findings
        .iter()
        .filter(|f| f.severity == Severity::Info)
        .count();

    let summary = format!(
        "\nFound {} issue(s): {} error(s), {} warning(s), {} info(s)",
        findings.len(),
        error_count,
        warn_count,
        info_count
    );

    let colored_summary = if error_count > 0 {
        summary.red().bold().to_string()
    } else if warn_count > 0 {
        summary.yellow().bold().to_string()
    } else {
        summary.cyan().to_string()
    };

    format!("{table}\n{colored_summary}")
}
