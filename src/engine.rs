use crate::config::Config;
use crate::finding::{Finding, Severity};
use crate::rules::RuleRegistry;
use crate::scanner::ScannedFile;

pub struct Engine<'a> {
    config: &'a Config,
    registry: &'a RuleRegistry,
}

impl<'a> Engine<'a> {
    pub fn new(config: &'a Config, registry: &'a RuleRegistry) -> Self {
        Self { config, registry }
    }

    pub fn run(&self, files: &[ScannedFile]) -> Vec<Finding> {
        let mut findings = Vec::new();

        for file in files {
            let rules = self.registry.rules_for_file(file.file_type);
            for rule in rules {
                if !self.config.is_rule_enabled(rule.id()) {
                    continue;
                }
                if self.config.is_rule_ignored(rule.id()) {
                    continue;
                }

                let file_path_str = file.relative_path.to_string_lossy();
                if self.config.is_allowlisted(rule.id(), &file_path_str) {
                    continue;
                }

                let mut rule_findings = rule.check(file);

                // Apply severity overrides
                for f in &mut rule_findings {
                    f.severity = self.config.effective_severity(&f.rule_id, f.severity);
                }

                findings.extend(rule_findings);
            }
        }

        // Filter by minimum severity
        findings.retain(|f| f.severity >= self.config.min_severity);

        // Sort: severity desc, then file, then line
        findings.sort_by_key(|a| a.sort_key());

        findings
    }

    pub fn max_severity(findings: &[Finding]) -> Option<Severity> {
        findings.iter().map(|f| f.severity).max()
    }

    pub fn exit_code(findings: &[Finding], error_on: Severity) -> i32 {
        match Self::max_severity(findings) {
            None => 0,
            Some(max) if max >= error_on => 2,
            Some(Severity::Warning) => 1,
            Some(Severity::Info) => 0,
            Some(_) => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finding::Location;

    fn make_finding(severity: Severity) -> Finding {
        Finding {
            rule_id: "TEST-001".into(),
            rule_name: "Test Rule".into(),
            severity,
            message: "test".into(),
            location: Location {
                file: "test.md".into(),
                line: 1,
                column: 1,
            },
            matched_text: "test".into(),
        }
    }

    #[test]
    fn test_exit_code_no_findings() {
        assert_eq!(Engine::exit_code(&[], Severity::Error), 0);
    }

    #[test]
    fn test_exit_code_errors() {
        let findings = vec![make_finding(Severity::Error)];
        assert_eq!(Engine::exit_code(&findings, Severity::Error), 2);
    }

    #[test]
    fn test_exit_code_warnings_only() {
        let findings = vec![make_finding(Severity::Warning)];
        assert_eq!(Engine::exit_code(&findings, Severity::Error), 1);
    }

    #[test]
    fn test_exit_code_info_only() {
        let findings = vec![make_finding(Severity::Info)];
        assert_eq!(Engine::exit_code(&findings, Severity::Error), 0);
    }

    #[test]
    fn test_exit_code_error_on_warning() {
        let findings = vec![make_finding(Severity::Warning)];
        assert_eq!(Engine::exit_code(&findings, Severity::Warning), 2);
    }

    #[test]
    fn test_max_severity() {
        assert_eq!(Engine::max_severity(&[]), None);
        let findings = vec![
            make_finding(Severity::Info),
            make_finding(Severity::Error),
            make_finding(Severity::Warning),
        ];
        assert_eq!(Engine::max_severity(&findings), Some(Severity::Error));
    }
}
