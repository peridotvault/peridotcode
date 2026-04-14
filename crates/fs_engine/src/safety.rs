//! Path safety validation
//!
//! Prevents directory traversal attacks by ensuring all file operations
//! stay within the project directory.

use peridot_shared::{PeridotError, PeridotResult};
use std::path::{Path, PathBuf};

/// Validate that a path is safe to use (within project boundaries)
///
/// A path is safe if:
/// 1. It does not contain `..` components that escape the project root
/// 2. It is an absolute path within the project root
/// 3. It does not point to system directories
///
/// This function handles both existing paths and new paths (where parent
/// directories may not exist yet).
pub fn is_path_safe(project_root: impl AsRef<Path>, target_path: impl AsRef<Path>) -> bool {
    let project_root = match project_root.as_ref().canonicalize() {
        Ok(p) => p,
        Err(_) => return false,
    };

    let target_path = target_path.as_ref();

    // Reject absolute paths outside project
    if target_path.is_absolute() && !target_path.starts_with(&project_root) {
        return false;
    }

    // Resolve the full path
    let full_path = if target_path.is_absolute() {
        target_path.to_path_buf()
    } else {
        project_root.join(target_path)
    };

    // First, normalize the path by removing redundant components
    // This resolves . and .. without requiring the path to exist
    let normalized = normalize_path(&full_path);

    // Check if normalized path is within project root
    if !normalized.starts_with(&project_root) {
        return false;
    }

    // Verify the path doesn't escape via symlinks by checking if any
    // existing parent is outside the project
    let mut current = Some(normalized.as_path());
    while let Some(path) = current {
        if let Ok(canonical) = path.canonicalize() {
            return canonical.starts_with(&project_root);
        }
        current = path.parent();
    }

    // All parent directories are new - safe to create within project
    true
}

/// Normalize a path by resolving . and .. components without requiring
/// the path to exist (unlike canonicalize).
fn normalize_path(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();

    for component in path.components() {
        match component {
            std::path::Component::Prefix(_) | std::path::Component::RootDir => {
                result.push(component.as_os_str());
            }
            std::path::Component::CurDir => {
                // Skip . components
            }
            std::path::Component::ParentDir => {
                // Pop the last component for ..
                if !result.as_os_str().is_empty() {
                    result.pop();
                }
            }
            std::path::Component::Normal(name) => {
                result.push(name);
            }
        }
    }

    // If empty, return .
    if result.as_os_str().is_empty() {
        result.push(".");
    }

    result
}

/// Validate a project path and return the canonicalized version
///
/// # Errors
/// Returns an error if the path is outside the project root
pub fn validate_project_path(
    project_root: impl AsRef<Path>,
    relative_path: impl AsRef<Path>,
) -> PeridotResult<PathBuf> {
    let project_root = project_root.as_ref().canonicalize()?;
    let relative_path = relative_path.as_ref();

    if !is_path_safe(&project_root, relative_path) {
        return Err(PeridotError::SafetyViolation(format!(
            "Path '{}' is outside project directory",
            relative_path.display()
        )));
    }

    Ok(project_root.join(relative_path))
}

/// Check if a path component is suspicious (contains dangerous patterns)
pub fn is_suspicious_component(component: &str) -> bool {
    // Check for null bytes
    if component.contains('\0') {
        return true;
    }

    // Check for path traversal attempts beyond simple ".."
    if component.contains("..") && component != ".." {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_is_path_safe_within_project() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create directory structure for paths we want to check
        fs::create_dir_all(project_root.join("src")).unwrap();
        fs::create_dir_all(project_root.join("nested/deep")).unwrap();

        assert!(is_path_safe(project_root, "src/main.rs"));
        assert!(is_path_safe(project_root, "nested/deep/file.txt"));

        // Also test with existing files
        fs::write(project_root.join("src/main.rs"), "test").unwrap();
        assert!(is_path_safe(project_root, "src/main.rs"));
    }

    #[test]
    fn test_is_path_safe_traversal_attempt() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Attempt to escape project directory
        assert!(!is_path_safe(project_root, "../outside.txt"));
        assert!(!is_path_safe(project_root, "foo/../../outside.txt"));
    }
}
