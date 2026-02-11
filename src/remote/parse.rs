/// Parsing of remote GitHub skill specifiers.
///
/// Supported formats:
/// - `owner/repo`
/// - `owner/repo@skill-name`
/// - `owner/repo:branch`
/// - `owner/repo:branch@skill-name`
/// - `https://github.com/owner/repo`
/// - `https://github.com/owner/repo/tree/branch/path/to/skill`

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteTarget {
    pub owner: String,
    pub repo: String,
    pub branch: Option<String>,
    pub skill_name: Option<String>,
}

impl RemoteTarget {
    pub fn parse(input: &str) -> Result<Self, String> {
        let input = input.trim();

        if input.starts_with("https://") || input.starts_with("http://") {
            return Self::parse_url(input);
        }

        Self::parse_shorthand(input)
    }

    fn parse_url(url: &str) -> Result<Self, String> {
        // Parse: https://github.com/owner/repo[/tree/branch[/path/to/skill]]
        let url = url
            .trim_end_matches('/')
            .strip_prefix("https://github.com/")
            .or_else(|| url.strip_prefix("http://github.com/"))
            .ok_or_else(|| format!("unsupported URL host (only github.com): {url}"))?;

        let parts: Vec<&str> = url.splitn(4, '/').collect();

        if parts.len() < 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err("invalid GitHub URL: must contain owner/repo".to_string());
        }

        let owner = parts[0].to_string();
        let repo = parts[1].trim_end_matches(".git").to_string();

        if parts.len() == 2 {
            return Ok(RemoteTarget {
                owner,
                repo,
                branch: None,
                skill_name: None,
            });
        }

        // parts[2] should be "tree"
        if parts[2] != "tree" {
            return Err(format!(
                "unsupported GitHub URL path segment '{}' (expected 'tree')",
                parts[2]
            ));
        }

        if parts.len() < 4 || parts[3].is_empty() {
            return Err("GitHub URL with /tree/ must include a branch name".to_string());
        }

        // parts[3] = "branch/path/to/skill" or just "branch"
        let rest = parts[3];
        let (branch, skill_path) = match rest.find('/') {
            Some(idx) => (&rest[..idx], Some(&rest[idx + 1..])),
            None => (rest, None),
        };

        let skill_name = skill_path.map(|p| {
            // Use the last path component as the skill name
            p.rsplit('/').next().unwrap_or(p).to_string()
        });

        Ok(RemoteTarget {
            owner,
            repo,
            branch: Some(branch.to_string()),
            skill_name,
        })
    }

    fn parse_shorthand(input: &str) -> Result<Self, String> {
        // Format: owner/repo[:branch][@skill-name]
        let slash_idx = input
            .find('/')
            .ok_or_else(|| format!("invalid remote specifier '{input}': must contain '/'"))?;

        let owner = &input[..slash_idx];
        if owner.is_empty() {
            return Err("owner cannot be empty".to_string());
        }

        let rest = &input[slash_idx + 1..];
        if rest.is_empty() {
            return Err("repo cannot be empty".to_string());
        }

        // Split off @skill-name first (from the right to handle edge cases)
        let (repo_branch, skill_name) = match rest.rfind('@') {
            Some(idx) => {
                let skill = &rest[idx + 1..];
                if skill.is_empty() {
                    return Err("skill name after '@' cannot be empty".to_string());
                }
                (&rest[..idx], Some(skill.to_string()))
            }
            None => (rest, None),
        };

        // Split off :branch
        let (repo, branch) = match repo_branch.find(':') {
            Some(idx) => {
                let branch = &repo_branch[idx + 1..];
                if branch.is_empty() {
                    return Err("branch after ':' cannot be empty".to_string());
                }
                (&repo_branch[..idx], Some(branch.to_string()))
            }
            None => (repo_branch, None),
        };

        if repo.is_empty() {
            return Err("repo cannot be empty".to_string());
        }

        Ok(RemoteTarget {
            owner: owner.to_string(),
            repo: repo.to_string(),
            branch,
            skill_name,
        })
    }

    /// Display string for use in output (e.g., "owner/repo@skill")
    pub fn display(&self) -> String {
        let mut s = format!("{}/{}", self.owner, self.repo);
        if let Some(ref branch) = self.branch {
            s.push(':');
            s.push_str(branch);
        }
        if let Some(ref skill) = self.skill_name {
            s.push('@');
            s.push_str(skill);
        }
        s
    }
}

impl std::fmt::Display for RemoteTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_owner_repo() {
        let t = RemoteTarget::parse("vercel-labs/agent-skills").unwrap();
        assert_eq!(t.owner, "vercel-labs");
        assert_eq!(t.repo, "agent-skills");
        assert_eq!(t.branch, None);
        assert_eq!(t.skill_name, None);
    }

    #[test]
    fn test_parse_owner_repo_at_skill() {
        let t = RemoteTarget::parse("vercel-labs/agent-skills@react-best-practices").unwrap();
        assert_eq!(t.owner, "vercel-labs");
        assert_eq!(t.repo, "agent-skills");
        assert_eq!(t.branch, None);
        assert_eq!(t.skill_name, Some("react-best-practices".to_string()));
    }

    #[test]
    fn test_parse_owner_repo_branch() {
        let t = RemoteTarget::parse("vercel-labs/agent-skills:main").unwrap();
        assert_eq!(t.owner, "vercel-labs");
        assert_eq!(t.repo, "agent-skills");
        assert_eq!(t.branch, Some("main".to_string()));
        assert_eq!(t.skill_name, None);
    }

    #[test]
    fn test_parse_owner_repo_branch_skill() {
        let t = RemoteTarget::parse("vercel-labs/agent-skills:main@react-best-practices").unwrap();
        assert_eq!(t.owner, "vercel-labs");
        assert_eq!(t.repo, "agent-skills");
        assert_eq!(t.branch, Some("main".to_string()));
        assert_eq!(t.skill_name, Some("react-best-practices".to_string()));
    }

    #[test]
    fn test_parse_github_url_simple() {
        let t = RemoteTarget::parse("https://github.com/vercel-labs/agent-skills").unwrap();
        assert_eq!(t.owner, "vercel-labs");
        assert_eq!(t.repo, "agent-skills");
        assert_eq!(t.branch, None);
        assert_eq!(t.skill_name, None);
    }

    #[test]
    fn test_parse_github_url_trailing_slash() {
        let t = RemoteTarget::parse("https://github.com/vercel-labs/agent-skills/").unwrap();
        assert_eq!(t.owner, "vercel-labs");
        assert_eq!(t.repo, "agent-skills");
    }

    #[test]
    fn test_parse_github_url_dot_git() {
        let t = RemoteTarget::parse("https://github.com/vercel-labs/agent-skills.git").unwrap();
        assert_eq!(t.repo, "agent-skills");
    }

    #[test]
    fn test_parse_github_url_tree_branch() {
        let t = RemoteTarget::parse(
            "https://github.com/vercel-labs/agent-skills/tree/main/react-best-practices",
        )
        .unwrap();
        assert_eq!(t.owner, "vercel-labs");
        assert_eq!(t.repo, "agent-skills");
        assert_eq!(t.branch, Some("main".to_string()));
        assert_eq!(t.skill_name, Some("react-best-practices".to_string()));
    }

    #[test]
    fn test_parse_github_url_tree_nested_path() {
        let t = RemoteTarget::parse(
            "https://github.com/owner/repo/tree/main/skills/react-best-practices",
        )
        .unwrap();
        assert_eq!(t.branch, Some("main".to_string()));
        assert_eq!(t.skill_name, Some("react-best-practices".to_string()));
    }

    #[test]
    fn test_parse_github_url_tree_branch_only() {
        let t =
            RemoteTarget::parse("https://github.com/vercel-labs/agent-skills/tree/main").unwrap();
        assert_eq!(t.branch, Some("main".to_string()));
        assert_eq!(t.skill_name, None);
    }

    #[test]
    fn test_parse_invalid_no_slash() {
        assert!(RemoteTarget::parse("just-a-name").is_err());
    }

    #[test]
    fn test_parse_invalid_empty_owner() {
        assert!(RemoteTarget::parse("/repo").is_err());
    }

    #[test]
    fn test_parse_invalid_empty_repo() {
        assert!(RemoteTarget::parse("owner/").is_err());
    }

    #[test]
    fn test_parse_invalid_empty_skill() {
        assert!(RemoteTarget::parse("owner/repo@").is_err());
    }

    #[test]
    fn test_parse_invalid_empty_branch() {
        assert!(RemoteTarget::parse("owner/repo:").is_err());
    }

    #[test]
    fn test_parse_invalid_url_host() {
        assert!(RemoteTarget::parse("https://gitlab.com/owner/repo").is_err());
    }

    #[test]
    fn test_display() {
        let t = RemoteTarget {
            owner: "owner".to_string(),
            repo: "repo".to_string(),
            branch: Some("main".to_string()),
            skill_name: Some("skill".to_string()),
        };
        assert_eq!(t.display(), "owner/repo:main@skill");
    }

    #[test]
    fn test_display_simple() {
        let t = RemoteTarget {
            owner: "owner".to_string(),
            repo: "repo".to_string(),
            branch: None,
            skill_name: None,
        };
        assert_eq!(t.display(), "owner/repo");
    }
}
