//! File reading operations
//!
//! Provides safe file reading capabilities with size limits and
//! encoding detection.

use peridot_shared::{constants, PeridotError, PeridotResult, ProjectConfig};
use std::fs;
use std::path::{Path, PathBuf};

/// Read a text file with size validation
///
/// # Errors
/// Returns an error if:
/// - The file does not exist
/// - The file exceeds `MAX_FILE_SIZE`
/// - The file contains invalid UTF-8
pub fn read_file(path: impl AsRef<Path>) -> PeridotResult<String> {
    let path = path.as_ref();

    if !path.exists() {
        return Err(PeridotError::FsError(format!(
            "File not found: {}",
            path.display()
        )));
    }

    let metadata = fs::metadata(path)?;
    let size = metadata.len() as usize;

    if size > constants::MAX_FILE_SIZE {
        return Err(PeridotError::FsError(format!(
            "File too large: {} bytes (max: {})",
            size,
            constants::MAX_FILE_SIZE
        )));
    }

    fs::read_to_string(path).map_err(|e| PeridotError::FsError(format!("Failed to read file: {e}")))
}

/// Read the project configuration file if it exists
///
/// # Returns
/// Returns `Ok(Some(config))` if config exists and is valid,
/// `Ok(None)` if no config exists, or an error if config is invalid
pub fn read_project_config(project_path: impl AsRef<Path>) -> PeridotResult<Option<ProjectConfig>> {
    let config_path = project_path.as_ref().join("peridot.toml");

    if !config_path.exists() {
        return Ok(None);
    }

    // TODO: Implement config file parsing
    // This will parse peridot.toml and return ProjectConfig
    tracing::debug!("Found project config at {:?}", config_path);

    Err(PeridotError::General(
        "Project config parsing not yet implemented".to_string(),
    ))
}

/// List all files in a project directory recursively
///
/// Returns a list of relative paths from the project root
pub fn list_project_files(project_path: impl AsRef<Path>) -> PeridotResult<Vec<PathBuf>> {
    let project_path = project_path.as_ref();
    let mut files = Vec::new();

    if !project_path.exists() {
        return Ok(files);
    }

    list_files_recursive(project_path, project_path, &mut files)?;

    Ok(files)
}

fn list_files_recursive(
    root: &Path,
    current: &Path,
    files: &mut Vec<PathBuf>,
) -> PeridotResult<()> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Skip hidden directories and common non-project directories
            if let Some(name) = path.file_name() {
                let name = name.to_string_lossy();
                if name.starts_with('.') || name == "node_modules" || name == "target" {
                    continue;
                }
            }
            list_files_recursive(root, &path, files)?;
        } else {
            // Store relative path
            if let Ok(rel_path) = path.strip_prefix(root) {
                files.push(rel_path.to_path_buf());
            }
        }
    }

    Ok(())
}

/// Check if a project exists at the given path
///
/// A project exists if it has a peridot.toml file or recognizable structure
pub fn is_project(project_path: impl AsRef<Path>) -> bool {
    let path = project_path.as_ref();
    path.join("peridot.toml").exists()
}
