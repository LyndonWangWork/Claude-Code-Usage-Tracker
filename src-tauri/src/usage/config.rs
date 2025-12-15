//! Configuration and data directory discovery

use std::path::PathBuf;
use std::env;

/// Get the Claude data directory path
/// Priority: 1. Custom path from config, 2. CLAUDE_CONFIG_DIR env var, 3. Default ~/.claude
pub fn get_claude_data_dir(custom_path: Option<&str>) -> PathBuf {
    // 1. Custom path takes highest priority
    if let Some(path) = custom_path {
        return PathBuf::from(path);
    }

    // 2. Check CLAUDE_CONFIG_DIR environment variable
    if let Ok(env_path) = env::var("CLAUDE_CONFIG_DIR") {
        return PathBuf::from(env_path);
    }

    // 3. Default to ~/.claude
    if let Some(home) = dirs::home_dir() {
        return home.join(".claude");
    }

    // Fallback for edge cases
    PathBuf::from(".claude")
}

/// Get the projects directory within the Claude data directory
pub fn get_projects_dir(custom_path: Option<&str>) -> PathBuf {
    get_claude_data_dir(custom_path).join("projects")
}

/// Decode an encoded project path (Claude Code custom encoding)
/// Claude Code encodes paths: `--` represents `:\` and `-` represents `\`
pub fn decode_project_path(encoded: &str) -> String {
    // First replace `--` with `:\` (drive letter separator on Windows)
    let result = encoded.replace("--", ":\\");
    // Then replace remaining `-` with `\` (path separator)
    result.replace("-", "\\")
}

/// Extract a display-friendly name from a project path
pub fn get_display_name(project_path: &str) -> String {
    // Get the last component of the path as display name
    let path = PathBuf::from(project_path);
    path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(project_path)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_project_path() {
        // Test Windows path: D--code-project becomes D:\code\project
        let encoded = "D--code-project";
        let decoded = decode_project_path(encoded);
        assert_eq!(decoded, "D:\\code\\project");
    }

    #[test]
    fn test_decode_project_path_nested() {
        // Test nested path: D--code-work-YueShan-react
        let encoded = "D--code-work-YueShan-react";
        let decoded = decode_project_path(encoded);
        assert_eq!(decoded, "D:\\code\\work\\YueShan\\react");
    }

    #[test]
    fn test_get_display_name() {
        let path = "D:\\code\\my-project";
        assert_eq!(get_display_name(path), "my-project");
    }
}
