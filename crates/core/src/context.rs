//! Project Context
//!
//! Manages the state and metadata of the current project.
//! Detects whether we are in an existing project or creating a new one.

use peridot_fs_engine::FsEngine;
use peridot_shared::{PeridotResult, ProjectConfig};
use std::path::{Path, PathBuf};

/// Context for the current project
#[derive(Debug)]
pub struct ProjectContext {
    /// Path to the project directory
    path: PathBuf,
    /// Whether this is a new project (not yet created)
    is_new: bool,
    /// Project configuration if it exists
    config: Option<ProjectConfig>,
    /// File system engine for this project
    fs_engine: FsEngine,
}

impl ProjectContext {
    /// Create a context for the current directory
    pub fn current() -> PeridotResult<Self> {
        let current_dir = std::env::current_dir()?;
        Self::at_path(current_dir)
    }

    /// Create a context for a specific path
    pub fn at_path(path: impl AsRef<Path>) -> PeridotResult<Self> {
        let path = path.as_ref().canonicalize()?;
        let fs_engine = FsEngine::new(&path)?;

        // Try to load existing config
        let config = peridot_fs_engine::read::read_project_config(&path)
            .ok()
            .flatten();
        let is_new = config.is_none();

        tracing::info!(
            "ProjectContext created at {:?} (new: {}, has_config: {})",
            path,
            is_new,
            config.is_some()
        );

        Ok(ProjectContext {
            path,
            is_new,
            config,
            fs_engine,
        })
    }

    /// Get the project path
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Check if this is a new project
    pub fn is_new(&self) -> bool {
        self.is_new
    }

    /// Check if this is an existing project
    pub fn is_existing(&self) -> bool {
        !self.is_new
    }

    /// Get the project configuration if it exists
    pub fn config(&self) -> Option<&ProjectConfig> {
        self.config.as_ref()
    }

    /// Get the file system engine
    pub fn fs_engine(&self) -> &FsEngine {
        &self.fs_engine
    }

    /// Get the project name (from config or directory name)
    pub fn name(&self) -> String {
        if let Some(config) = &self.config {
            config.name.clone()
        } else {
            self.path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "unnamed".to_string())
        }
    }

    /// Detect the project type based on files present
    pub fn detect_project_type(&self) -> ProjectType {
        // TODO: Implement project type detection
        // Check for:
        // - package.json (Node.js/Phaser)
        // - project.godot (Godot)
        // - Cargo.toml (Rust)

        ProjectType::Unknown
    }

    /// List all files in the project
    pub fn list_files(&self) -> PeridotResult<Vec<PathBuf>> {
        peridot_fs_engine::read::list_project_files(&self.path)
    }
}

/// Type of project detected
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectType {
    /// Phaser/HTML5 project
    Phaser,
    /// Godot project
    Godot,
    /// Unknown project type
    Unknown,
}
