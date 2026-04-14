//! Change Summary
//!
//! Provides tracking and reporting of file system changes.
//! Shows which files were created, modified, or remained unchanged.
//!
//! # Diff-like Reporting
//!
//! This module provides a lightweight change tracking system suitable for
//! displaying file operation results to users. It is NOT a full diff engine:
//!
//! - **What it does**: Track which files were created, modified, or deleted
//! - **What it doesn't do**: Show line-by-line diffs of content changes
//!
//! For detailed diffs, use external tools like `git diff` or the `similar`
//! crate after generation.
//!
//! # Display Format
//!
//! Changes are displayed with single-character symbols similar to `git status`:
//!
//! - `+` - Created (new file)
//! - `~` - Modified (existing file changed)
//! - `-` - Deleted (file removed)
//! - ` ` - Unchanged (no difference detected)
//!
//! Example output:
//! ```text
//! + src/main.js
//! + package.json
//! ~ README.md
//! ```
//!
//! # Integration with TUI/CLI
//!
//! The [`ChangeSummary`] can format reports suitable for display:
//!
//! ```rust,no_run
//! use peridot_fs_engine::ChangeSummary;
//!
//! # let summary = ChangeSummary::new();
//! // Full formatted report
//! println!("{}", summary.format_report());
//!
//! // Single-line summary
//! println!("{}", summary.summary_line()); // "3 created, 1 modified"
//! ```

use std::path::PathBuf;

/// Type of change made to a file
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeType {
    /// File was newly created
    Created,
    /// Existing file was modified
    Modified,
    /// File was deleted
    Deleted,
    /// File exists with identical content (no change)
    Unchanged,
}

impl ChangeType {
    /// Get a human-readable label
    pub fn label(&self) -> &'static str {
        match self {
            ChangeType::Created => "created",
            ChangeType::Modified => "modified",
            ChangeType::Deleted => "deleted",
            ChangeType::Unchanged => "unchanged",
        }
    }

    /// Get a single-character symbol
    pub fn symbol(&self) -> char {
        match self {
            ChangeType::Created => '+',
            ChangeType::Modified => '~',
            ChangeType::Deleted => '-',
            ChangeType::Unchanged => ' ',
        }
    }

    /// Check if this change type represents an actual modification
    pub fn is_modified(&self) -> bool {
        matches!(
            self,
            ChangeType::Created | ChangeType::Modified | ChangeType::Deleted
        )
    }
}

/// Record of a single file change
#[derive(Debug, Clone)]
pub struct FileChange {
    /// Relative path to the file
    pub path: PathBuf,
    /// Type of change
    pub change_type: ChangeType,
}

impl FileChange {
    /// Create a new file change record
    pub fn new(path: PathBuf, change_type: ChangeType) -> Self {
        FileChange { path, change_type }
    }

    /// Format as a display string
    pub fn format(&self) -> String {
        format!("{} {}", self.change_type.symbol(), self.path.display())
    }
}

/// Summary of all changes made during an operation
#[derive(Debug, Clone, Default)]
pub struct ChangeSummary {
    /// List of file changes
    changes: Vec<FileChange>,
}

impl ChangeSummary {
    /// Create a new empty summary
    pub fn new() -> Self {
        ChangeSummary {
            changes: Vec::new(),
        }
    }

    /// Add a file change to the summary
    pub fn add_change(&mut self, change: FileChange) {
        // If we already have a record for this path, update it
        if let Some(existing) = self.changes.iter_mut().find(|c| c.path == change.path) {
            existing.change_type = change.change_type;
        } else {
            self.changes.push(change);
        }
    }

    /// Get all changes
    pub fn changes(&self) -> &[FileChange] {
        &self.changes
    }

    /// Get only modified/created/deleted changes (excluding unchanged)
    pub fn modified_changes(&self) -> Vec<&FileChange> {
        self.changes
            .iter()
            .filter(|c| c.change_type.is_modified())
            .collect()
    }

    /// Get changes filtered by type
    pub fn changes_of_type(&self, change_type: ChangeType) -> Vec<&FileChange> {
        self.changes
            .iter()
            .filter(|c| c.change_type == change_type)
            .collect()
    }

    /// Get count of each change type
    pub fn counts(&self) -> (usize, usize, usize) {
        let created = self.changes_of_type(ChangeType::Created).len();
        let modified = self.changes_of_type(ChangeType::Modified).len();
        let deleted = self.changes_of_type(ChangeType::Deleted).len();
        (created, modified, deleted)
    }

    /// Check if any changes were made
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    /// Get total number of changes
    pub fn len(&self) -> usize {
        self.changes.len()
    }

    /// Get number of files that were actually modified
    pub fn modified_count(&self) -> usize {
        self.modified_changes().len()
    }

    /// Clear all changes
    pub fn clear(&mut self) {
        self.changes.clear();
    }

    /// Format a summary report
    pub fn format_report(&self) -> String {
        let mut report = String::new();

        // Add header
        report.push_str("File Changes:\n");
        report.push_str("============\n");

        // Group by type for display
        let created = self.changes_of_type(ChangeType::Created);
        let modified = self.changes_of_type(ChangeType::Modified);
        let deleted = self.changes_of_type(ChangeType::Deleted);
        let unchanged = self.changes_of_type(ChangeType::Unchanged);

        // Show created files
        if !created.is_empty() {
            report.push_str(&format!("\nCreated ({}):\n", created.len()));
            for change in created {
                report.push_str(&format!("  {}\n", change.format()));
            }
        }

        // Show modified files
        if !modified.is_empty() {
            report.push_str(&format!("\nModified ({}):\n", modified.len()));
            for change in modified {
                report.push_str(&format!("  {}\n", change.format()));
            }
        }

        // Show deleted files
        if !deleted.is_empty() {
            report.push_str(&format!("\nDeleted ({}):\n", deleted.len()));
            for change in deleted {
                report.push_str(&format!("  {}\n", change.format()));
            }
        }

        // Show unchanged count briefly
        if !unchanged.is_empty() {
            report.push_str(&format!("\nUnchanged ({} files)\n", unchanged.len()));
        }

        // Summary line
        let (c, m, d) = self.counts();
        report.push_str(&format!(
            "\nTotal: {} created, {} modified, {} deleted\n",
            c, m, d
        ));

        report
    }

    /// Get a simple summary line
    pub fn summary_line(&self) -> String {
        let (created, modified, deleted) = self.counts();
        let parts: Vec<String> = [
            if created > 0 {
                Some(format!("{} created", created))
            } else {
                None
            },
            if modified > 0 {
                Some(format!("{} modified", modified))
            } else {
                None
            },
            if deleted > 0 {
                Some(format!("{} deleted", deleted))
            } else {
                None
            },
        ]
        .into_iter()
        .flatten()
        .collect();

        if parts.is_empty() {
            "No changes".to_string()
        } else {
            parts.join(", ")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_summary() {
        let mut summary = ChangeSummary::new();

        summary.add_change(FileChange::new(
            PathBuf::from("src/main.js"),
            ChangeType::Created,
        ));
        summary.add_change(FileChange::new(
            PathBuf::from("package.json"),
            ChangeType::Modified,
        ));

        assert_eq!(summary.len(), 2);
        assert_eq!(summary.counts(), (1, 1, 0));
        assert!(!summary.is_empty());
    }

    #[test]
    fn test_summary_line() {
        let mut summary = ChangeSummary::new();
        assert_eq!(summary.summary_line(), "No changes");

        summary.add_change(FileChange::new(
            PathBuf::from("file1.js"),
            ChangeType::Created,
        ));
        assert_eq!(summary.summary_line(), "1 created");

        summary.add_change(FileChange::new(
            PathBuf::from("file2.js"),
            ChangeType::Modified,
        ));
        assert_eq!(summary.summary_line(), "1 created, 1 modified");
    }
}
