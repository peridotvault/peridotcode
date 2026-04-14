//! Common error types for PeridotCode
//!
//! This module defines the error hierarchy used across all crates.
//! Each crate may extend these with its own specific error variants.

use thiserror::Error;

/// The main result type used throughout PeridotCode
pub type PeridotResult<T> = Result<T, PeridotError>;

/// The main error type for PeridotCode operations
#[derive(Error, Debug)]
pub enum PeridotError {
    /// Error when a template is not found
    #[error("Template not found: {0}")]
    TemplateNotFound(String),

    /// Error during template rendering
    #[error("Template rendering failed: {0}")]
    TemplateRenderError(String),

    /// Error during file system operations
    #[error("File system error: {0}")]
    FsError(String),

    /// Safety violation (attempted to write outside project directory)
    #[error("Safety violation: {0}")]
    SafetyViolation(String),

    /// Error parsing user intent
    #[error("Failed to parse intent: {0}")]
    IntentParseError(String),

    /// Error executing a command
    #[error("Command execution failed: {0}")]
    CommandError(String),

    /// Project already exists
    #[error("Project already exists at path: {0}")]
    ProjectExists(String),

    /// Invalid project structure
    #[error("Invalid project structure: {0}")]
    InvalidProject(String),

    /// IO error wrapper
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// General error with message
    #[error("{0}")]
    General(String),
}

impl PeridotError {
    /// Create a new general error from a string message
    pub fn new<S: Into<String>>(msg: S) -> Self {
        PeridotError::General(msg.into())
    }
}
