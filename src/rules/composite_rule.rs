use crate::finding::{Finding, Location, Severity};
use crate::rules::Rule;
use crate::scanner::{FileType, ScannedFile};

/// Detects when a skill's stated description doesn't match its actual behavior.
/// Looks for mismatches like "calculator" description but network access code.
pub struct DescriptionMismatchRule;

const BENIGN_KEYWORDS: &[&str] = &[
    "calculator",
    "math",
    "text",
    "format",
    "convert",
    "simple",
    "basic",
    "helper",
    "utility",
];

const SUSPICIOUS_PATTERNS: &[(&str, &str)] = &[
    ("curl ", "network access via curl"),
    ("wget ", "network access via wget"),
    ("fetch(", "network access via fetch"),
    ("http.get", "network access via HTTP"),
    ("requests.get", "network access via requests"),
    ("subprocess", "process execution via subprocess"),
    ("child_process", "process execution via child_process"),
    ("exec(", "dynamic code execution via exec"),
    ("eval(", "dynamic code execution via eval"),
    ("/etc/passwd", "system file access"),
    ("~/.ssh", "SSH key access"),
    ("/etc/shadow", "shadow password file access"),
];

impl Rule for DescriptionMismatchRule {
    fn id(&self) -> &str {
        "SL-META-006"
    }

    fn name(&self) -> &str {
        "Description/Content Mismatch"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn applies_to(&self) -> &[FileType] {
        &[FileType::Markdown]
    }

    fn check(&self, file: &ScannedFile) -> Vec<Finding> {
        let mut findings = Vec::new();
        let content_lower = file.content.to_lowercase();

        // Check if file has a benign-sounding description
        let has_benign_desc = BENIGN_KEYWORDS
            .iter()
            .any(|kw| content_lower[..content_lower.len().min(500)].contains(kw));

        if !has_benign_desc {
            return findings;
        }

        // Look for suspicious patterns in the rest of the content
        for (pattern, desc) in SUSPICIOUS_PATTERNS {
            if let Some(pos) = content_lower.find(&pattern.to_lowercase()) {
                let line = content_lower[..pos].matches('\n').count() + 1;
                findings.push(Finding {
                    rule_id: self.id().to_string(),
                    rule_name: self.name().to_string(),
                    severity: self.default_severity(),
                    message: format!("Skill has benign description but contains {desc}"),
                    location: Location {
                        file: file.relative_path.clone(),
                        line,
                        column: 1,
                    },
                    matched_text: pattern.to_string(),
                });
            }
        }

        findings
    }
}
