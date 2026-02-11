pub mod composite_rule;
pub mod metadata_rule;
pub mod regex_rule;
pub mod unicode_rule;

use crate::finding::{Finding, Severity};
use crate::scanner::{FileType, ScannedFile};

pub trait Rule: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn default_severity(&self) -> Severity;
    fn applies_to(&self) -> &[FileType];
    fn check(&self, file: &ScannedFile) -> Vec<Finding>;
}

pub struct RuleRegistry {
    rules: Vec<Box<dyn Rule>>,
}

impl RuleRegistry {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn register(&mut self, rule: Box<dyn Rule>) {
        self.rules.push(rule);
    }

    pub fn rules_for_file(&self, file_type: FileType) -> Vec<&dyn Rule> {
        self.rules
            .iter()
            .filter(|r| {
                let applies = r.applies_to();
                applies.is_empty() || applies.contains(&file_type)
            })
            .map(|r| r.as_ref())
            .collect()
    }

    pub fn all_rules(&self) -> &[Box<dyn Rule>] {
        &self.rules
    }

    pub fn load_defaults(&mut self) {
        self.load_pattern_file(include_str!("../../patterns/hidden.toml"));
        self.load_pattern_file(include_str!("../../patterns/secrets.toml"));
        self.load_pattern_file(include_str!("../../patterns/network.toml"));
        self.load_pattern_file(include_str!("../../patterns/filesystem.toml"));
        self.load_pattern_file(include_str!("../../patterns/execution.toml"));
        self.load_pattern_file(include_str!("../../patterns/injection.toml"));
        self.load_pattern_file(include_str!("../../patterns/social.toml"));
        self.load_pattern_file(include_str!("../../patterns/metadata.toml"));

        // Register specialized rules
        self.register(Box::new(unicode_rule::UnicodeRule));
        self.register(Box::new(metadata_rule::MetadataValidationRule));
        self.register(Box::new(composite_rule::DescriptionMismatchRule));
    }

    fn load_pattern_file(&mut self, toml_str: &str) {
        let file: regex_rule::PatternFile = match toml::from_str(toml_str) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("warning: failed to parse pattern file: {e}");
                return;
            }
        };

        for def in file.rules {
            match regex_rule::RegexRule::from_definition(def) {
                Ok(rule) => self.register(Box::new(rule)),
                Err(e) => eprintln!("warning: failed to compile rule: {e}"),
            }
        }
    }
}
