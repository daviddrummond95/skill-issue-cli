pub mod github;
pub mod parse;

pub use parse::RemoteTarget;

use crate::scanner::ScannedFile;
use std::fmt;

#[derive(Debug)]
pub enum RemoteError {
    ParseError(String),
    HttpError(String),
    RateLimited { reset_timestamp: Option<u64> },
    RepoNotFound(String),
    NoSkillsFound,
    SkillNotFound(String),
    TreeTruncated,
}

impl fmt::Display for RemoteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RemoteError::ParseError(msg) => write!(f, "invalid remote specifier: {msg}"),
            RemoteError::HttpError(msg) => write!(f, "HTTP error: {msg}"),
            RemoteError::RateLimited {
                reset_timestamp: Some(ts),
            } => write!(f, "GitHub API rate limit exceeded (resets at {ts})"),
            RemoteError::RateLimited {
                reset_timestamp: None,
            } => write!(f, "GitHub API rate limit exceeded"),
            RemoteError::RepoNotFound(spec) => {
                write!(f, "repository not found: {spec}")
            }
            RemoteError::NoSkillsFound => {
                write!(f, "no skills found (no SKILL.md files in repository)")
            }
            RemoteError::SkillNotFound(name) => {
                write!(f, "skill '{name}' not found in repository")
            }
            RemoteError::TreeTruncated => write!(
                f,
                "repository tree is too large (truncated by GitHub API); try specifying a skill name with @"
            ),
        }
    }
}

/// Fetch files for a remote skill from GitHub.
///
/// Parses the target specifier, fetches the repo tree via GitHub API,
/// discovers skills, and returns ScannedFile structs compatible with the
/// existing engine pipeline.
pub fn fetch_remote_skill(
    spec: &str,
    token: Option<&str>,
    verbose: bool,
) -> Result<Vec<ScannedFile>, RemoteError> {
    let target = RemoteTarget::parse(spec).map_err(RemoteError::ParseError)?;

    if verbose {
        eprintln!("Remote target: {target}");
    }

    github::fetch_skill_files(&target, token, verbose)
}
