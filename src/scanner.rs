use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Markdown,
    Script,
    Yaml,
    Toml,
    Json,
    Unknown,
}

impl FileType {
    pub fn from_path(path: &Path) -> Self {
        match path.extension().and_then(|e| e.to_str()) {
            Some("md" | "mdx") => FileType::Markdown,
            Some("sh" | "bash" | "zsh" | "py" | "rb" | "js" | "ts") => FileType::Script,
            Some("yml" | "yaml") => FileType::Yaml,
            Some("toml") => FileType::Toml,
            Some("json") => FileType::Json,
            _ => FileType::Unknown,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScannedFile {
    #[allow(dead_code)]
    pub path: PathBuf,
    pub relative_path: PathBuf,
    pub file_type: FileType,
    pub content: String,
}

const SKIP_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    ".skill-issue-cache",
    "__pycache__",
    ".venv",
];

pub fn scan_directory(root: &Path) -> Result<Vec<ScannedFile>, String> {
    if !root.exists() {
        return Err(format!("path does not exist: {}", root.display()));
    }
    if !root.is_dir() {
        return Err(format!("path is not a directory: {}", root.display()));
    }

    let mut files = Vec::new();

    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_str().unwrap_or("");
            !SKIP_DIRS.contains(&name)
        })
    {
        let entry = entry.map_err(|e| format!("walk error: {e}"))?;
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path().to_path_buf();
        let relative_path = path.strip_prefix(root).unwrap_or(&path).to_path_buf();
        let file_type = FileType::from_path(&path);

        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue, // skip binary files
        };

        files.push(ScannedFile {
            path,
            relative_path,
            file_type,
            content,
        });
    }

    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_file_type_detection() {
        assert_eq!(FileType::from_path(Path::new("foo.md")), FileType::Markdown);
        assert_eq!(
            FileType::from_path(Path::new("foo.mdx")),
            FileType::Markdown
        );
        assert_eq!(FileType::from_path(Path::new("foo.py")), FileType::Script);
        assert_eq!(FileType::from_path(Path::new("foo.sh")), FileType::Script);
        assert_eq!(FileType::from_path(Path::new("foo.js")), FileType::Script);
        assert_eq!(FileType::from_path(Path::new("foo.yml")), FileType::Yaml);
        assert_eq!(FileType::from_path(Path::new("foo.yaml")), FileType::Yaml);
        assert_eq!(FileType::from_path(Path::new("foo.toml")), FileType::Toml);
        assert_eq!(FileType::from_path(Path::new("foo.json")), FileType::Json);
        assert_eq!(FileType::from_path(Path::new("foo.txt")), FileType::Unknown);
    }

    #[test]
    fn test_scan_directory() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("test.md"), "# Hello").unwrap();
        fs::write(dir.path().join("test.py"), "print('hi')").unwrap();

        let files = scan_directory(dir.path()).unwrap();
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_scan_skips_git() {
        let dir = TempDir::new().unwrap();
        let git_dir = dir.path().join(".git");
        fs::create_dir(&git_dir).unwrap();
        fs::write(git_dir.join("config"), "data").unwrap();
        fs::write(dir.path().join("test.md"), "# Hello").unwrap();

        let files = scan_directory(dir.path()).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].relative_path, PathBuf::from("test.md"));
    }

    #[test]
    fn test_scan_nonexistent() {
        let result = scan_directory(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }
}
