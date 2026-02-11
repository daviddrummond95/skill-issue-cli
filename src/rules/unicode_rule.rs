use crate::finding::{Finding, Location, Severity};
use crate::rules::Rule;
use crate::scanner::{FileType, ScannedFile};

pub struct UnicodeRule;

const SUSPICIOUS_RANGES: &[(char, char, &str)] = &[
    ('\u{200B}', '\u{200F}', "zero-width/directional character"),
    ('\u{202A}', '\u{202E}', "bidirectional override character"),
    ('\u{2066}', '\u{2069}', "bidirectional isolate character"),
    ('\u{FEFF}', '\u{FEFF}', "byte order mark (not at start)"),
    ('\u{00AD}', '\u{00AD}', "soft hyphen"),
    ('\u{034F}', '\u{034F}', "combining grapheme joiner"),
    ('\u{2060}', '\u{2064}', "invisible formatting character"),
    ('\u{FE00}', '\u{FE0F}', "variation selector"),
    ('\u{E0100}', '\u{E01EF}', "variation selector supplement"),
];

impl Rule for UnicodeRule {
    fn id(&self) -> &str {
        "SL-HID-001"
    }

    fn name(&self) -> &str {
        "Suspicious Unicode Characters"
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    fn applies_to(&self) -> &[FileType] {
        &[] // all file types
    }

    fn check(&self, file: &ScannedFile) -> Vec<Finding> {
        let mut findings = Vec::new();

        for (line_num, line) in file.content.lines().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                // Skip BOM at very start of file
                if line_num == 0 && col == 0 && ch == '\u{FEFF}' {
                    continue;
                }

                for &(start, end, desc) in SUSPICIOUS_RANGES {
                    if ch >= start && ch <= end {
                        findings.push(Finding {
                            rule_id: self.id().to_string(),
                            rule_name: self.name().to_string(),
                            severity: self.default_severity(),
                            message: format!(
                                "Found {} (U+{:04X}) in file content",
                                desc, ch as u32
                            ),
                            location: Location {
                                file: file.relative_path.clone(),
                                line: line_num + 1,
                                column: col + 1,
                            },
                            matched_text: format!("U+{:04X}", ch as u32),
                        });
                        break;
                    }
                }
            }
        }

        findings
    }
}
