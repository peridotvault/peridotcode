//! File System Engine
//!
//! Provides safe file system operations for PeridotCode with the following guarantees:
//! - All paths are validated to be within the project directory (no traversal attacks)
//! - File operations are atomic where possible
//! - Clear error messages for all failure modes
//! - Change tracking and reporting
//!
//! This is the only crate that should perform direct file system mutations.
//!
//! # Architecture
//!
//! The fs_engine is organized into focused modules:
//!
//! - `safety`: Path validation to prevent directory traversal attacks
//! - `read`: Safe file reading with size limits and encoding detection
//! - `write`: File writing with automatic directory creation
//! - `summary`: Change tracking and reporting (created/modified/deleted/unchanged)
//! - `operations`: Operation types and batch processing utilities
//!
//! # Usage
//!
//! ```rust,no_run
//! use peridot_fs_engine::FsEngine;
//!
//! // Create engine for a project directory
//! let mut engine = FsEngine::new("./my-project")?;
//!
//! // Write files safely (validates path is within project)
//! engine.write_file("src/main.js", "console.log('hello');")?;
//! engine.write_file("README.md", "# My Game")?;
//!
//! // Get change summary
//! let summary = engine.take_change_summary();
//! println!("{}", summary.format_report());
//! # Ok::<(), peridot_shared::PeridotError>(())
//! ```
//!
//! # Safety Guarantees
//!
//! ## Path Validation
//!
//! All write operations validate that the target path is within the project root:
//!
//! - Rejects absolute paths outside the project
//! - Resolves and validates `..` components
//! - Canonicalizes paths before validation
//! - Checks parent directories for non-existent paths
//!
//! ## Examples of Blocked Paths
//!
//! ```rust,no_run
//! use peridot_fs_engine::safety::is_path_safe;
//! use std::path::Path;
//!
//! # let project = Path::new("/project");
//! assert!(!is_path_safe(project, "../outside.txt"));      // Traversal attack
//! assert!(!is_path_safe(project, "/etc/passwd"));          // Absolute outside
//! assert!(!is_path_safe(project, "foo/../../etc/shadow")); // Escapes project
//! ```
//!
//! # Change Tracking
//!
//! Every file operation is tracked with its change type:
//!
//! - `Created`: New file written
//! - `Modified`: Existing file with different content
//! - `Deleted`: File removed
//! - `Unchanged`: Existing file with identical content (not reported)
//!
//! The [`ChangeSummary`](summary::ChangeSummary) provides formatted reports:
//!
//! ```text
//! File Changes:
//! ============
//!
//! Created (3):
//!   + src/main.js
//!   + package.json
//!   + README.md
//!
//! Total: 3 created, 0 modified, 0 deleted
//! ```
//!
//! # Assumptions and Limitations
//!
//! ## Current Assumptions
//!
//! 1. **Project Root is Canonical**: The project root is canonicalized once at engine
//!    creation. If the directory is moved during operations, behavior is undefined.
//!
//! 2. **Single-threaded Use**: The `FsEngine` is not `Sync`. For concurrent operations,
//!    create multiple engines or wrap in appropriate synchronization primitives.
//!
//! 3. **UTF-8 Text Files**: While binary files are supported, content comparison for
//!    change detection assumes UTF-8 text. Binary files are compared as byte sequences.
//!
//! 4. **No Concurrent External Modification**: Change detection compares against the
//!    state at write time. External modifications during operation may result in
//!    incorrect change classification.
//!
//! 5. **Platform Path Conventions**: Relies on standard Rust path handling. On Windows,
//!    UNC paths and drive letters are handled but not thoroughly tested.
//!
//! ## Known Limitations
//!
//! 1. **No True Diff**: Change detection is binary (same/different), not a line-by-line
//!    diff. Use external tools like `git diff` for detailed change analysis.
//!
//! 2. **No Rollback**: Failed batch operations leave partial changes. Implement
//!    transactions manually if atomic batches are required.
//!
//! 3. **Limited Binary Support**: Binary files are written correctly but placeholder
//!    substitution in the template engine may corrupt binary content.
//!
//! 4. **No File Locking**: No advisory locks are used. Concurrent writes from other
//!    processes may corrupt files.
//!
//! 5. **Size Limits**: File reading enforces a 10MB limit. Large files must be
//!    handled with standard `std::fs` operations.
//!
//! # Deferred Improvements
//!
//! The following improvements are planned but not yet implemented:
//!
//! ## Short Term
//!
//! - [ ] Add more comprehensive unit tests for edge cases (symlinks, permissions)
//! - [ ] Implement file locking for critical operations
//! - [ ] Add configurable size limits per operation type
//! - [ ] Support for `.gitignore`-style exclusion patterns
//!
//! ## Medium Term
//!
//! - [ ] True diff generation for modified files (line-by-line)
//! - [ ] Transaction support with rollback capability
//! - [ ] Async I/O support for better performance
//! - [ ] Directory watching integration for external change detection
//!
//! ## Long Term
//!
//! - [ ] Virtual file system layer for testing without real disk operations
//! - [ ] Content-addressable storage for deduplication
//! - [ ] Integration with PeridotVault for cloud sync
//! - [ ] Cross-platform permission preservation

#![warn(missing_docs)]

pub mod operations;
pub mod read;
pub mod safety;
pub mod summary;
pub mod write;

pub use operations::{FsOperation, FsOperationResult, FsOperations};
pub use read::{list_project_files, read_file, read_project_config};
pub use safety::{is_path_safe, validate_project_path};
pub use summary::{ChangeSummary, ChangeType, FileChange};
pub use write::{create_directory, write_file, write_project_file};

use peridot_shared::PeridotResult;
use std::path::{Path, PathBuf};

/// Main file system engine that coordinates read and write operations
#[derive(Debug, Clone)]
pub struct FsEngine {
    /// The root project path (used for safety validation)
    project_root: PathBuf,
    /// Track changes for reporting
    change_summary: ChangeSummary,
}

impl FsEngine {
    /// Create a new FsEngine for the given project directory
    ///
    /// # Errors
    /// Returns an error if the project path cannot be canonicalized
    pub fn new(project_root: impl AsRef<Path>) -> PeridotResult<Self> {
        let project_root = project_root.as_ref().canonicalize()?;
        Ok(FsEngine {
            project_root,
            change_summary: ChangeSummary::new(),
        })
    }

    /// Get the project root path
    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    /// Initialize a new FsEngine pointing to the current directory
    pub fn current_dir() -> PeridotResult<Self> {
        Self::new(std::env::current_dir()?)
    }

    /// Write a file to the project directory with safety checks
    ///
    /// # Arguments
    /// * `relative_path` - Path relative to project root
    /// * `content` - File content to write
    ///
    /// # Returns
    /// Result containing the full path and change information
    pub fn write_file(
        &mut self,
        relative_path: impl AsRef<Path>,
        content: &str,
    ) -> PeridotResult<FsOperationResult> {
        let relative_path = relative_path.as_ref();
        let full_path = safety::validate_project_path(&self.project_root, relative_path)?;

        // Check if file already exists
        let change_type = if full_path.exists() {
            // Check if content is different
            match std::fs::read_to_string(&full_path) {
                Ok(existing) if existing == content => ChangeType::Unchanged,
                Ok(_) => ChangeType::Modified,
                Err(_) => ChangeType::Modified, // Couldn't read, assume modified
            }
        } else {
            ChangeType::Created
        };

        // Perform the write
        write::write_project_file(&self.project_root, relative_path, content)?;

        // Record the change
        let change = FileChange::new(relative_path.to_path_buf(), change_type);
        self.change_summary.add_change(change.clone());

        Ok(FsOperationResult {
            path: full_path,
            change,
            bytes_written: content.len(),
        })
    }

    /// Write multiple files to the project directory
    ///
    /// # Arguments
    /// * `files` - Map of relative paths to content
    ///
    /// # Returns
    /// Results for each file operation
    pub fn write_files(
        &mut self,
        files: Vec<(PathBuf, String)>,
    ) -> Vec<PeridotResult<FsOperationResult>> {
        let mut results = Vec::new();

        for (relative_path, content) in files {
            results.push(self.write_file(relative_path, &content));
        }

        results
    }

    /// Create a directory within the project
    pub fn create_dir(&mut self, relative_path: impl AsRef<Path>) -> PeridotResult<PathBuf> {
        let relative_path = relative_path.as_ref();
        let full_path = safety::validate_project_path(&self.project_root, relative_path)?;

        std::fs::create_dir_all(&full_path)?;

        // Record directory creation
        let change = FileChange::new(relative_path.to_path_buf(), ChangeType::Created);
        self.change_summary.add_change(change);

        Ok(full_path)
    }

    /// Get the current change summary
    pub fn change_summary(&self) -> &ChangeSummary {
        &self.change_summary
    }

    /// Take the change summary (resets internal tracking)
    pub fn take_change_summary(&mut self) -> ChangeSummary {
        std::mem::take(&mut self.change_summary)
    }

    /// Clear the change summary
    pub fn clear_changes(&mut self) {
        self.change_summary.clear();
    }

    /// Check if any changes were made
    pub fn has_changes(&self) -> bool {
        !self.change_summary.is_empty()
    }

    /// Get count of changes by type
    pub fn change_counts(&self) -> (usize, usize, usize) {
        self.change_summary.counts()
    }
}

/// Check if a file or directory exists within the project
pub fn exists(path: impl AsRef<Path>) -> bool {
    path.as_ref().exists()
}

/// Check if a path is within the project directory
pub fn is_within_project(project_root: impl AsRef<Path>, path: impl AsRef<Path>) -> bool {
    safety::is_path_safe(project_root, path)
}

/// Remove a file (with safety checks)
pub fn remove_file(
    project_root: impl AsRef<Path>,
    relative_path: impl AsRef<Path>,
) -> PeridotResult<()> {
    let full_path = safety::validate_project_path(project_root, relative_path)?;
    std::fs::remove_file(full_path)?;
    Ok(())
}
