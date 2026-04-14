//! File write operations
//!
//! Provides safe file writing with atomic operations where possible
//! and automatic directory creation.

use peridot_shared::PeridotResult;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::safety::validate_project_path;

/// Write content to a file within the project directory
///
/// # Safety
/// This function validates that the target path is within the project
/// directory before writing.
///
/// # Errors
/// Returns an error if:
/// - The path is outside the project directory
/// - The parent directory cannot be created
/// - The file cannot be written
pub fn write_project_file(
    project_root: impl AsRef<Path>,
    relative_path: impl AsRef<Path>,
    content: &str,
) -> PeridotResult<PathBuf> {
    let full_path = validate_project_path(&project_root, &relative_path)?;

    // Create parent directories if they don't exist
    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Write the file
    let mut file = fs::File::create(&full_path)?;
    file.write_all(content.as_bytes())?;
    file.sync_all()?;

    tracing::info!("Wrote file: {}", full_path.display());

    Ok(full_path)
}

/// Write content to any file (without project safety checks)
///
/// # Warning
/// Only use this for paths that have already been validated.
pub fn write_file(path: impl AsRef<Path>, content: &str) -> PeridotResult<()> {
    let path = path.as_ref();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, content)?;
    Ok(())
}

/// Create a directory and all parent directories
pub fn create_directory(path: impl AsRef<Path>) -> PeridotResult<()> {
    fs::create_dir_all(path)?;
    Ok(())
}

/// Atomically write a file (write to temp file then rename)
///
/// This prevents partial writes from corrupting existing files.
pub fn write_file_atomic(path: impl AsRef<Path>, content: &str) -> PeridotResult<()> {
    let path = path.as_ref();
    let temp_path = path.with_extension("tmp");

    // Write to temp file
    fs::write(&temp_path, content)?;

    // Atomically rename
    fs::rename(&temp_path, path)?;

    Ok(())
}
