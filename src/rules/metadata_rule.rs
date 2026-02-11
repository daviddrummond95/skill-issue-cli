use crate::finding::{Finding, Location, Severity};
use crate::rules::Rule;
use crate::scanner::{FileType, ScannedFile};

pub struct MetadataValidationRule;

const MAX_NAME_LENGTH: usize = 100;
const MAX_DESCRIPTION_LENGTH: usize = 500;

impl Rule for MetadataValidationRule {
    fn id(&self) -> &str {
        "SL-META-001"
    }

    fn name(&self) -> &str {
        "Metadata Validation"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn applies_to(&self) -> &[FileType] {
        &[FileType::Markdown, FileType::Yaml]
    }

    fn check(&self, file: &ScannedFile) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Only check files that look like metadata/frontmatter
        let frontmatter = extract_frontmatter(&file.content);
        let Some(fm) = frontmatter else {
            return findings;
        };

        let yaml: serde_yaml::Value = match serde_yaml::from_str(&fm) {
            Ok(v) => v,
            Err(_) => return findings,
        };

        let map = match yaml.as_mapping() {
            Some(m) => m,
            None => return findings,
        };

        // Check for missing description (SL-META-002)
        if !map.contains_key(serde_yaml::Value::String("description".into())) {
            findings.push(Finding {
                rule_id: "SL-META-002".to_string(),
                rule_name: "Missing Skill Description".to_string(),
                severity: Severity::Warning,
                message: "Skill metadata missing description field".to_string(),
                location: Location {
                    file: file.relative_path.clone(),
                    line: 1,
                    column: 1,
                },
                matched_text: "---".to_string(),
            });
        }

        // Check name length
        if let Some(name) = map.get(serde_yaml::Value::String("name".into())) {
            if let Some(s) = name.as_str() {
                if s.len() > MAX_NAME_LENGTH {
                    findings.push(Finding {
                        rule_id: self.id().to_string(),
                        rule_name: self.name().to_string(),
                        severity: Severity::Warning,
                        message: format!(
                            "Skill name exceeds {} characters ({} chars)",
                            MAX_NAME_LENGTH,
                            s.len()
                        ),
                        location: Location {
                            file: file.relative_path.clone(),
                            line: 1,
                            column: 1,
                        },
                        matched_text: s.to_string(),
                    });
                }
            }
        }

        // Check description length
        if let Some(desc) = map.get(serde_yaml::Value::String("description".into())) {
            if let Some(s) = desc.as_str() {
                if s.len() > MAX_DESCRIPTION_LENGTH {
                    findings.push(Finding {
                        rule_id: self.id().to_string(),
                        rule_name: self.name().to_string(),
                        severity: Severity::Warning,
                        message: format!(
                            "Skill description exceeds {} characters ({} chars)",
                            MAX_DESCRIPTION_LENGTH,
                            s.len()
                        ),
                        location: Location {
                            file: file.relative_path.clone(),
                            line: 1,
                            column: 1,
                        },
                        matched_text: format!("{}...", &s[..50.min(s.len())]),
                    });
                }
            }
        }

        findings
    }
}

fn extract_frontmatter(content: &str) -> Option<String> {
    let content = content.trim_start();
    if !content.starts_with("---") {
        return None;
    }
    let after_first = &content[3..];
    let end = after_first.find("\n---")?;
    Some(after_first[..end].to_string())
}
