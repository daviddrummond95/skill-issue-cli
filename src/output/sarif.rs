use crate::finding::{Finding, Severity};
use crate::rules::RuleRegistry;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
struct SarifLog {
    #[serde(rename = "$schema")]
    schema: &'static str,
    version: &'static str,
    runs: Vec<SarifRun>,
}

#[derive(Serialize)]
struct SarifRun {
    tool: SarifTool,
    results: Vec<SarifResult>,
}

#[derive(Serialize)]
struct SarifTool {
    driver: SarifDriver,
}

#[derive(Serialize)]
struct SarifDriver {
    name: &'static str,
    version: String,
    rules: Vec<SarifRuleDescriptor>,
}

#[derive(Serialize)]
struct SarifRuleDescriptor {
    id: String,
    name: String,
    #[serde(rename = "shortDescription")]
    short_description: SarifMessage,
    #[serde(rename = "defaultConfiguration")]
    default_configuration: SarifDefaultConfig,
}

#[derive(Serialize)]
struct SarifDefaultConfig {
    level: String,
}

#[derive(Serialize)]
struct SarifResult {
    #[serde(rename = "ruleId")]
    rule_id: String,
    level: String,
    message: SarifMessage,
    locations: Vec<SarifLocation>,
}

#[derive(Serialize)]
struct SarifMessage {
    text: String,
}

#[derive(Serialize)]
struct SarifLocation {
    #[serde(rename = "physicalLocation")]
    physical_location: SarifPhysicalLocation,
}

#[derive(Serialize)]
struct SarifPhysicalLocation {
    #[serde(rename = "artifactLocation")]
    artifact_location: SarifArtifactLocation,
    region: SarifRegion,
}

#[derive(Serialize)]
struct SarifArtifactLocation {
    uri: String,
}

#[derive(Serialize)]
struct SarifRegion {
    #[serde(rename = "startLine")]
    start_line: usize,
    #[serde(rename = "startColumn")]
    start_column: usize,
}

fn severity_to_level(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "note",
    }
}

pub fn format_sarif(findings: &[Finding], _skill_path: &Path) -> String {
    format_sarif_with_rules(findings, _skill_path, None)
}

pub fn format_sarif_with_rules(
    findings: &[Finding],
    _skill_path: &Path,
    registry: Option<&RuleRegistry>,
) -> String {
    let rules: Vec<SarifRuleDescriptor> = if let Some(reg) = registry {
        reg.all_rules()
            .iter()
            .map(|r| SarifRuleDescriptor {
                id: r.id().to_string(),
                name: r.name().to_string(),
                short_description: SarifMessage {
                    text: r.name().to_string(),
                },
                default_configuration: SarifDefaultConfig {
                    level: severity_to_level(r.default_severity()).to_string(),
                },
            })
            .collect()
    } else {
        // Derive rules from findings
        let mut seen = std::collections::HashSet::new();
        findings
            .iter()
            .filter(|f| seen.insert(f.rule_id.clone()))
            .map(|f| SarifRuleDescriptor {
                id: f.rule_id.clone(),
                name: f.rule_name.clone(),
                short_description: SarifMessage {
                    text: f.rule_name.clone(),
                },
                default_configuration: SarifDefaultConfig {
                    level: severity_to_level(f.severity).to_string(),
                },
            })
            .collect()
    };

    let results: Vec<SarifResult> = findings
        .iter()
        .map(|f| SarifResult {
            rule_id: f.rule_id.clone(),
            level: severity_to_level(f.severity).to_string(),
            message: SarifMessage {
                text: f.message.clone(),
            },
            locations: vec![SarifLocation {
                physical_location: SarifPhysicalLocation {
                    artifact_location: SarifArtifactLocation {
                        uri: f.location.file.display().to_string(),
                    },
                    region: SarifRegion {
                        start_line: f.location.line,
                        start_column: f.location.column,
                    },
                },
            }],
        })
        .collect();

    let log = SarifLog {
        schema: "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/main/sarif-2.1/schema/sarif-schema-2.1.0.json",
        version: "2.1.0",
        runs: vec![SarifRun {
            tool: SarifTool {
                driver: SarifDriver {
                    name: "skill-issue",
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    rules,
                },
            },
            results,
        }],
    };

    serde_json::to_string_pretty(&log).unwrap_or_else(|e| format!("{{\"error\": \"{e}\"}}"))
}
