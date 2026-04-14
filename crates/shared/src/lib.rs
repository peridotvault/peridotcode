//! PeridotCode Shared Types
//!
//! This crate contains common data structures, types, and utilities used across
//! all PeridotCode crates. It is designed to have minimal external dependencies
//! to avoid circular dependency issues.
//!
//! Key modules:
//! - `models`: Core data structures for projects, templates, and intents
//! - `constants`: System-wide constants and configuration values
//! - `errors`: Common error types used throughout the workspace

#![warn(missing_docs)]

pub mod constants;
pub mod errors;
pub mod models;

// Re-export commonly used types for convenience
pub use errors::{PeridotError, PeridotResult};
pub use models::{
    ConfigSummary, GameIntent, GameMetadata, GameStack, ModelId, ProjectConfig, PromptInput,
    ProviderId, ScaffoldResult, TemplateId, TemplateManifest,
};
