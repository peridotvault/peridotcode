//! File System Operations
//!
//! Defines operation types and results for file system operations.
//! Provides a structured way to track file changes and report results.
//!
//! # Operation Types
//!
//! - `Create` - Create a new file
//! - `Modify` - Modify an existing file
//! - `Delete` - Delete a file
//! - `CreateDir` - Create a directory
//!
//! # Batch Operations
//!
//! The [`FsOperations`] type allows queuing multiple operations for execution.
//! Note that batch operations are not transactional - partial failures leave
//! some operations completed.
//!
//! # Result Types
//!
//! - [`FsOperationResult`] - Result of a single operation with path, change type, and bytes written
//! - [`FsOperations`] - Batch operation tracker with success/failure counts

use crate::summary::{ChangeType, FileChange};
use std::path::PathBuf;

/// Result of a file system operation
#[derive(Debug, Clone)]
pub struct FsOperationResult {
    /// Full path to the file
    pub path: PathBuf,
    /// Type of change that occurred
    pub change: FileChange,
    /// Number of bytes written
    pub bytes_written: usize,
}

impl FsOperationResult {
    /// Create a new operation result
    pub fn new(path: PathBuf, change: FileChange, bytes_written: usize) -> Self {
        FsOperationResult {
            path,
            change,
            bytes_written,
        }
    }

    /// Check if this operation created a new file
    pub fn is_created(&self) -> bool {
        self.change.change_type == ChangeType::Created
    }

    /// Check if this operation modified an existing file
    pub fn is_modified(&self) -> bool {
        self.change.change_type == ChangeType::Modified
    }

    /// Get the relative path (filename only for display)
    pub fn file_name(&self) -> String {
        self.path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| self.path.to_string_lossy().to_string())
    }
}

/// Types of file system operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsOperation {
    /// Create a new file
    Create,
    /// Modify an existing file
    Modify,
    /// Delete a file
    Delete,
    /// Create a directory
    CreateDir,
}

impl FsOperation {
    /// Get a human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            FsOperation::Create => "create",
            FsOperation::Modify => "modify",
            FsOperation::Delete => "delete",
            FsOperation::CreateDir => "create directory",
        }
    }
}

/// Batch file operations with results
#[derive(Debug, Default)]
pub struct FsOperations {
    /// Operations to perform
    operations: Vec<FsOperationItem>,
    /// Results of completed operations
    results: Vec<FsOperationResult>,
    /// Errors encountered
    errors: Vec<(PathBuf, String)>,
}

/// A single file operation item
#[derive(Debug, Clone)]
pub struct FsOperationItem {
    /// Operation type
    pub op: FsOperation,
    /// Relative path
    pub path: PathBuf,
    /// Content (for create/modify)
    pub content: Option<String>,
}

impl FsOperations {
    /// Create a new batch of operations
    pub fn new() -> Self {
        FsOperations {
            operations: Vec::new(),
            results: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Add a file creation operation
    pub fn create_file(&mut self, path: impl Into<PathBuf>, content: impl Into<String>) {
        self.operations.push(FsOperationItem {
            op: FsOperation::Create,
            path: path.into(),
            content: Some(content.into()),
        });
    }

    /// Add a directory creation operation
    pub fn create_dir(&mut self, path: impl Into<PathBuf>) {
        self.operations.push(FsOperationItem {
            op: FsOperation::CreateDir,
            path: path.into(),
            content: None,
        });
    }

    /// Get all operations
    pub fn operations(&self) -> &[FsOperationItem] {
        &self.operations
    }

    /// Get successful results
    pub fn results(&self) -> &[FsOperationResult] {
        &self.results
    }

    /// Get errors
    pub fn errors(&self) -> &[(PathBuf, String)] {
        &self.errors
    }

    /// Check if all operations succeeded
    pub fn all_succeeded(&self) -> bool {
        self.errors.is_empty()
    }

    /// Get count of operations
    pub fn count(&self) -> usize {
        self.operations.len()
    }

    /// Get count of successful operations
    pub fn success_count(&self) -> usize {
        self.results.len()
    }

    /// Get count of failed operations
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }
}
