use crate::remote::{RemoteError, RemoteTarget};
use crate::scanner::{FileType, ScannedFile};
use serde::Deserialize;
use std::path::PathBuf;

const USER_AGENT: &str = concat!("skill-issue/", env!("CARGO_PKG_VERSION"));

#[derive(Debug, Deserialize)]
struct TreeResponse {
    tree: Vec<TreeEntry>,
    truncated: bool,
}

#[derive(Debug, Deserialize, Clone)]
struct TreeEntry {
    path: String,
    #[serde(rename = "type")]
    entry_type: String,
    #[allow(dead_code)]
    sha: String,
}

#[derive(Debug, Clone)]
struct DiscoveredSkill {
    /// The directory prefix for this skill (e.g. "react-best-practices/")
    prefix: String,
    /// Display name (last path component)
    name: String,
}

/// Fetch skill files from a GitHub repository.
pub fn fetch_skill_files(
    target: &RemoteTarget,
    token: Option<&str>,
    verbose: bool,
) -> Result<Vec<ScannedFile>, RemoteError> {
    // Determine the branch — use specified or default
    let branch = match &target.branch {
        Some(b) => b.clone(),
        None => detect_default_branch(target, token, verbose)?,
    };

    if verbose {
        eprintln!("Using branch: {branch}");
    }

    // Fetch recursive tree
    let tree = fetch_tree(target, &branch, token, verbose)?;

    // Discover skills
    let skills = discover_skills(&tree, target)?;

    if verbose {
        eprintln!("Found {} skill(s)", skills.len());
        for s in &skills {
            eprintln!("  - {}", s.name);
        }
    }

    // Collect all file entries belonging to the discovered skills
    let mut files = Vec::new();
    for skill in &skills {
        let skill_entries: Vec<&TreeEntry> = tree
            .iter()
            .filter(|e| e.entry_type == "blob" && e.path.starts_with(&skill.prefix))
            .collect();

        if verbose {
            eprintln!(
                "Fetching {} files for skill '{}'",
                skill_entries.len(),
                skill.name
            );
        }

        for entry in skill_entries {
            let content = fetch_file_content(target, &branch, &entry.path, token)?;

            // Relative path within the skill directory
            let relative = entry
                .path
                .strip_prefix(&skill.prefix)
                .unwrap_or(&entry.path);
            let relative_path = PathBuf::from(relative);

            files.push(ScannedFile {
                path: PathBuf::from(&entry.path),
                relative_path: relative_path.clone(),
                file_type: FileType::from_path(&relative_path),
                content,
            });
        }
    }

    if files.is_empty() {
        return Err(RemoteError::NoSkillsFound);
    }

    Ok(files)
}

/// Detect the default branch of a repo via the GitHub API.
fn detect_default_branch(
    target: &RemoteTarget,
    token: Option<&str>,
    verbose: bool,
) -> Result<String, RemoteError> {
    let url = format!(
        "https://api.github.com/repos/{}/{}",
        target.owner, target.repo
    );

    if verbose {
        eprintln!("Fetching repo metadata: {url}");
    }

    let mut resp = make_request(&url, token)?;
    let body: serde_json::Value = resp
        .body_mut()
        .read_json()
        .map_err(|e| RemoteError::HttpError(e.to_string()))?;

    body["default_branch"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| RemoteError::HttpError("could not determine default branch".to_string()))
}

/// Fetch the recursive tree for a branch.
fn fetch_tree(
    target: &RemoteTarget,
    branch: &str,
    token: Option<&str>,
    verbose: bool,
) -> Result<Vec<TreeEntry>, RemoteError> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/git/trees/{}?recursive=1",
        target.owner, target.repo, branch
    );

    if verbose {
        eprintln!("Fetching tree: {url}");
    }

    let mut resp = make_request(&url, token)?;
    let tree_resp: TreeResponse = resp
        .body_mut()
        .read_json()
        .map_err(|e| RemoteError::HttpError(format!("failed to parse tree response: {e}")))?;

    if tree_resp.truncated {
        return Err(RemoteError::TreeTruncated);
    }

    Ok(tree_resp.tree)
}

/// Discover skills by finding SKILL.md files in the tree.
fn discover_skills(
    tree: &[TreeEntry],
    target: &RemoteTarget,
) -> Result<Vec<DiscoveredSkill>, RemoteError> {
    let skill_files: Vec<&TreeEntry> = tree
        .iter()
        .filter(|e| {
            e.entry_type == "blob"
                && e.path
                    .rsplit('/')
                    .next()
                    .is_some_and(|name| name == "SKILL.md")
        })
        .collect();

    if skill_files.is_empty() {
        return Err(RemoteError::NoSkillsFound);
    }

    let skills: Vec<DiscoveredSkill> = skill_files
        .iter()
        .map(|entry| {
            // "react-best-practices/SKILL.md" → prefix "react-best-practices/", name "react-best-practices"
            // "SKILL.md" at root → prefix "", name is the repo name
            let prefix = match entry.path.rfind('/') {
                Some(idx) => &entry.path[..=idx], // includes trailing /
                None => "",                       // root SKILL.md
            };

            let name = if prefix.is_empty() {
                target.repo.clone()
            } else {
                prefix
                    .trim_end_matches('/')
                    .rsplit('/')
                    .next()
                    .unwrap_or(&target.repo)
                    .to_string()
            };

            DiscoveredSkill {
                prefix: prefix.to_string(),
                name,
            }
        })
        .collect();

    // Filter to specific skill if requested
    if let Some(ref skill_name) = target.skill_name {
        let matched: Vec<DiscoveredSkill> = skills
            .into_iter()
            .filter(|s| s.name == *skill_name)
            .collect();

        if matched.is_empty() {
            return Err(RemoteError::SkillNotFound(skill_name.clone()));
        }
        return Ok(matched);
    }

    Ok(skills)
}

/// Fetch a single file's raw content from GitHub.
fn fetch_file_content(
    target: &RemoteTarget,
    branch: &str,
    path: &str,
    token: Option<&str>,
) -> Result<String, RemoteError> {
    let url = format!(
        "https://raw.githubusercontent.com/{}/{}/{}/{}",
        target.owner, target.repo, branch, path
    );

    let mut resp = make_request(&url, token)?;
    resp.body_mut()
        .read_to_string()
        .map_err(|e| RemoteError::HttpError(format!("failed to read file {path}: {e}")))
}

/// Make an HTTP GET request with optional auth and standard headers.
fn make_request(
    url: &str,
    token: Option<&str>,
) -> Result<ureq::http::Response<ureq::Body>, RemoteError> {
    let mut req = ureq::get(url).header("User-Agent", USER_AGENT);

    if let Some(token) = token {
        req = req.header("Authorization", &format!("Bearer {token}"));
    }

    // For API endpoints, request JSON
    if url.contains("api.github.com") {
        req = req.header("Accept", "application/vnd.github+json");
    }

    let resp = req.call().map_err(|e| {
        let err_string = e.to_string();
        if err_string.contains("404") {
            RemoteError::RepoNotFound(url.to_string())
        } else if err_string.contains("403") {
            RemoteError::RateLimited {
                reset_timestamp: None,
            }
        } else {
            RemoteError::HttpError(err_string)
        }
    })?;

    Ok(resp)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tree_entry(path: &str, entry_type: &str) -> TreeEntry {
        TreeEntry {
            path: path.to_string(),
            entry_type: entry_type.to_string(),
            sha: "abc123".to_string(),
        }
    }

    #[test]
    fn test_discover_skills_single() {
        let tree = vec![
            make_tree_entry("react-best-practices/SKILL.md", "blob"),
            make_tree_entry("react-best-practices/README.md", "blob"),
            make_tree_entry("react-best-practices", "tree"),
        ];
        let target = RemoteTarget {
            owner: "vercel-labs".to_string(),
            repo: "agent-skills".to_string(),
            branch: None,
            skill_name: None,
        };

        let skills = discover_skills(&tree, &target).unwrap();
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "react-best-practices");
        assert_eq!(skills[0].prefix, "react-best-practices/");
    }

    #[test]
    fn test_discover_skills_multiple() {
        let tree = vec![
            make_tree_entry("skill-a/SKILL.md", "blob"),
            make_tree_entry("skill-a/index.md", "blob"),
            make_tree_entry("skill-b/SKILL.md", "blob"),
            make_tree_entry("skill-b/index.md", "blob"),
        ];
        let target = RemoteTarget {
            owner: "owner".to_string(),
            repo: "repo".to_string(),
            branch: None,
            skill_name: None,
        };

        let skills = discover_skills(&tree, &target).unwrap();
        assert_eq!(skills.len(), 2);
    }

    #[test]
    fn test_discover_skills_with_filter() {
        let tree = vec![
            make_tree_entry("skill-a/SKILL.md", "blob"),
            make_tree_entry("skill-b/SKILL.md", "blob"),
        ];
        let target = RemoteTarget {
            owner: "owner".to_string(),
            repo: "repo".to_string(),
            branch: None,
            skill_name: Some("skill-b".to_string()),
        };

        let skills = discover_skills(&tree, &target).unwrap();
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "skill-b");
    }

    #[test]
    fn test_discover_skills_filter_not_found() {
        let tree = vec![make_tree_entry("skill-a/SKILL.md", "blob")];
        let target = RemoteTarget {
            owner: "owner".to_string(),
            repo: "repo".to_string(),
            branch: None,
            skill_name: Some("nonexistent".to_string()),
        };

        let err = discover_skills(&tree, &target).unwrap_err();
        assert!(matches!(err, RemoteError::SkillNotFound(_)));
    }

    #[test]
    fn test_discover_skills_none_found() {
        let tree = vec![
            make_tree_entry("README.md", "blob"),
            make_tree_entry("src/main.rs", "blob"),
        ];
        let target = RemoteTarget {
            owner: "owner".to_string(),
            repo: "repo".to_string(),
            branch: None,
            skill_name: None,
        };

        let err = discover_skills(&tree, &target).unwrap_err();
        assert!(matches!(err, RemoteError::NoSkillsFound));
    }

    #[test]
    fn test_discover_skills_root_skill_md() {
        let tree = vec![
            make_tree_entry("SKILL.md", "blob"),
            make_tree_entry("README.md", "blob"),
        ];
        let target = RemoteTarget {
            owner: "owner".to_string(),
            repo: "my-skill".to_string(),
            branch: None,
            skill_name: None,
        };

        let skills = discover_skills(&tree, &target).unwrap();
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "my-skill");
        assert_eq!(skills[0].prefix, "");
    }

    #[test]
    fn test_discover_skills_nested_path() {
        let tree = vec![
            make_tree_entry("skills/react-best-practices/SKILL.md", "blob"),
            make_tree_entry("skills/react-best-practices/README.md", "blob"),
        ];
        let target = RemoteTarget {
            owner: "owner".to_string(),
            repo: "repo".to_string(),
            branch: None,
            skill_name: None,
        };

        let skills = discover_skills(&tree, &target).unwrap();
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "react-best-practices");
        assert_eq!(skills[0].prefix, "skills/react-best-practices/");
    }
}
