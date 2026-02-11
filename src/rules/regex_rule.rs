use crate::finding::{Finding, Location, Severity};
use crate::rules::Rule;
use crate::scanner::{FileType, ScannedFile};
use regex::Regex;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct PatternFile {
    #[serde(rename = "rules")]
    pub rules: Vec<RuleDefinition>,
}

#[derive(Deserialize)]
pub struct RuleDefinition {
    pub id: String,
    pub name: String,
    pub severity: String,
    pub pattern: String,
    #[serde(default)]
    pub applies_to: Vec<String>,
    pub message_template: String,
    #[serde(default)]
    pub multiline: bool,
}

pub struct RegexRule {
    pub id: String,
    pub name: String,
    pub severity: Severity,
    pub pattern: Regex,
    pub applies_to: Vec<FileType>,
    pub message_template: String,
    pub multiline: bool,
}

fn parse_file_type(s: &str) -> Option<FileType> {
    match s.to_lowercase().as_str() {
        "markdown" | "md" => Some(FileType::Markdown),
        "script" | "sh" | "py" | "js" => Some(FileType::Script),
        "yaml" | "yml" => Some(FileType::Yaml),
        "toml" => Some(FileType::Toml),
        "json" => Some(FileType::Json),
        _ => None,
    }
}

impl RegexRule {
    pub fn from_definition(def: RuleDefinition) -> Result<Self, String> {
        let severity: Severity = def.severity.parse()?;
        let pattern = if def.multiline {
            regex::RegexBuilder::new(&def.pattern)
                .multi_line(true)
                .dot_matches_new_line(true)
                .build()
        } else {
            Regex::new(&def.pattern)
        }
        .map_err(|e| format!("rule {}: invalid regex: {e}", def.id))?;

        let applies_to: Vec<FileType> = def
            .applies_to
            .iter()
            .filter_map(|s| parse_file_type(s))
            .collect();

        Ok(RegexRule {
            id: def.id,
            name: def.name,
            severity,
            pattern,
            applies_to,
            message_template: def.message_template,
            multiline: def.multiline,
        })
    }
}

impl Rule for RegexRule {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn default_severity(&self) -> Severity {
        self.severity
    }

    fn applies_to(&self) -> &[FileType] {
        &self.applies_to
    }

    fn check(&self, file: &ScannedFile) -> Vec<Finding> {
        let mut findings = Vec::new();

        if self.multiline {
            for mat in self.pattern.find_iter(&file.content) {
                let line = file.content[..mat.start()].matches('\n').count() + 1;
                let last_newline = file.content[..mat.start()].rfind('\n').map_or(0, |p| p + 1);
                let column = mat.start() - last_newline + 1;
                let matched = mat.as_str();
                let display_match = if matched.len() > 80 {
                    format!("{}...", &matched[..77])
                } else {
                    matched.to_string()
                };

                findings.push(Finding {
                    rule_id: self.id.clone(),
                    rule_name: self.name.clone(),
                    severity: self.severity,
                    message: self.message_template.replace("{match}", &display_match),
                    location: Location {
                        file: file.relative_path.clone(),
                        line,
                        column,
                    },
                    matched_text: display_match,
                });
            }
        } else {
            for (line_num, line) in file.content.lines().enumerate() {
                for mat in self.pattern.find_iter(line) {
                    let matched = mat.as_str();
                    let display_match = if matched.len() > 80 {
                        format!("{}...", &matched[..77])
                    } else {
                        matched.to_string()
                    };

                    findings.push(Finding {
                        rule_id: self.id.clone(),
                        rule_name: self.name.clone(),
                        severity: self.severity,
                        message: self.message_template.replace("{match}", &display_match),
                        location: Location {
                            file: file.relative_path.clone(),
                            line: line_num + 1,
                            column: mat.start() + 1,
                        },
                        matched_text: display_match,
                    });
                }
            }
        }

        findings
    }
}
