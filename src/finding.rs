use serde::Serialize;
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Warning,
    Error,
}

impl Severity {
    pub fn rank(self) -> u8 {
        match self {
            Severity::Info => 0,
            Severity::Warning => 1,
            Severity::Error => 2,
        }
    }
}

impl Ord for Severity {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.rank().cmp(&other.rank())
    }
}

impl PartialOrd for Severity {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Info => write!(f, "info"),
            Severity::Warning => write!(f, "warning"),
            Severity::Error => write!(f, "error"),
        }
    }
}

impl std::str::FromStr for Severity {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "info" => Ok(Severity::Info),
            "warning" => Ok(Severity::Warning),
            "error" => Ok(Severity::Error),
            _ => Err(format!("unknown severity: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Location {
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    pub rule_id: String,
    pub rule_name: String,
    pub severity: Severity,
    pub message: String,
    pub location: Location,
    pub matched_text: String,
}

impl Finding {
    pub fn sort_key(&self) -> (std::cmp::Reverse<Severity>, PathBuf, usize, usize) {
        (
            std::cmp::Reverse(self.severity),
            self.location.file.clone(),
            self.location.line,
            self.location.column,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Error > Severity::Warning);
        assert!(Severity::Warning > Severity::Info);
        assert!(Severity::Error > Severity::Info);
    }

    #[test]
    fn test_severity_parse() {
        assert_eq!("error".parse::<Severity>().unwrap(), Severity::Error);
        assert_eq!("WARNING".parse::<Severity>().unwrap(), Severity::Warning);
        assert_eq!("Info".parse::<Severity>().unwrap(), Severity::Info);
        assert!("unknown".parse::<Severity>().is_err());
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(Severity::Error.to_string(), "error");
        assert_eq!(Severity::Warning.to_string(), "warning");
        assert_eq!(Severity::Info.to_string(), "info");
    }

    #[test]
    fn test_finding_sort_key() {
        let f1 = Finding {
            rule_id: "R1".into(),
            rule_name: "Rule 1".into(),
            severity: Severity::Error,
            message: "msg".into(),
            location: Location {
                file: "a.md".into(),
                line: 1,
                column: 1,
            },
            matched_text: "m".into(),
        };
        let f2 = Finding {
            rule_id: "R2".into(),
            rule_name: "Rule 2".into(),
            severity: Severity::Warning,
            message: "msg".into(),
            location: Location {
                file: "a.md".into(),
                line: 1,
                column: 1,
            },
            matched_text: "m".into(),
        };
        // Error should sort before Warning (Reverse ordering)
        assert!(f1.sort_key() < f2.sort_key());
    }
}
