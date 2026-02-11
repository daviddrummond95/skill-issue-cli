use crate::finding::Severity;
use clap::Parser;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "skill-issue",
    version,
    about = "Static security analyzer for Claude skill directories — skill-issue.sh"
)]
pub struct CliArgs {
    /// Path to the skill directory to analyze
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Output format
    #[arg(short, long, default_value = "table")]
    pub format: OutputFormat,

    /// Path to configuration file
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Minimum severity to report
    #[arg(short, long, default_value = "info")]
    pub severity: Severity,

    /// Rule IDs to ignore (can be repeated)
    #[arg(long, num_args = 1..)]
    pub ignore: Vec<String>,

    /// Minimum severity that causes a non-zero exit code
    #[arg(long, default_value = "error")]
    pub error_on: Severity,

    /// Suppress all output except findings
    #[arg(short, long)]
    pub quiet: bool,

    /// Show verbose output including rule details
    #[arg(short, long)]
    pub verbose: bool,

    /// Disable colored output
    #[arg(long)]
    pub no_color: bool,

    /// Remote GitHub skill specifier (e.g. owner/repo, owner/repo@skill-name, GitHub URL)
    #[arg(long)]
    pub remote: Option<String>,

    /// GitHub API token for authenticated requests (or set GITHUB_TOKEN env var)
    #[arg(long, env = "GITHUB_TOKEN")]
    pub github_token: Option<String>,
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
    Sarif,
}

#[derive(Debug, Deserialize, Default)]
pub struct ConfigFile {
    #[serde(default)]
    pub settings: ConfigSettings,
    #[serde(default)]
    pub rules: HashMap<String, RuleOverride>,
    #[serde(default)]
    pub allowlist: Vec<AllowlistEntry>,
}

#[derive(Debug, Deserialize, Default)]
#[allow(dead_code)]
pub struct ConfigSettings {
    pub severity: Option<String>,
    pub format: Option<String>,
    pub error_on: Option<String>,
    #[serde(default)]
    pub ignore: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct RuleOverride {
    pub severity: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct AllowlistEntry {
    pub rule: String,
    pub file: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Config {
    pub path: PathBuf,
    pub format: OutputFormat,
    pub min_severity: Severity,
    pub ignore: Vec<String>,
    pub error_on: Severity,
    pub quiet: bool,
    pub verbose: bool,
    pub no_color: bool,
    pub rule_overrides: HashMap<String, RuleOverride>,
    pub allowlist: Vec<AllowlistEntry>,
    pub remote: Option<String>,
    pub github_token: Option<String>,
}

impl Config {
    pub fn from_args_and_file(args: CliArgs, file: Option<ConfigFile>) -> Self {
        let file = file.unwrap_or_default();

        let ignore = if args.ignore.is_empty() {
            file.settings.ignore.clone()
        } else {
            args.ignore.clone()
        };

        Config {
            path: args.path,
            format: args.format,
            min_severity: args.severity,
            ignore,
            error_on: args.error_on,
            quiet: args.quiet,
            verbose: args.verbose,
            no_color: args.no_color,
            rule_overrides: file.rules,
            allowlist: file.allowlist,
            remote: args.remote,
            github_token: args.github_token,
        }
    }

    pub fn is_rule_ignored(&self, rule_id: &str) -> bool {
        self.ignore.iter().any(|id| id == rule_id)
    }

    pub fn is_allowlisted(&self, rule_id: &str, file_path: &str) -> bool {
        self.allowlist.iter().any(|entry| {
            entry.rule == rule_id
                && entry
                    .file
                    .as_ref()
                    .is_none_or(|f| file_path.contains(f.as_str()))
        })
    }

    pub fn effective_severity(&self, rule_id: &str, default: Severity) -> Severity {
        self.rule_overrides
            .get(rule_id)
            .and_then(|o| o.severity.as_ref())
            .and_then(|s| s.parse().ok())
            .unwrap_or(default)
    }

    pub fn is_rule_enabled(&self, rule_id: &str) -> bool {
        self.rule_overrides
            .get(rule_id)
            .and_then(|o| o.enabled)
            .unwrap_or(true)
    }
}
