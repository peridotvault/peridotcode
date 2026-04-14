//! Shared constants for PeridotCode
//!
//! This module contains system-wide constants that define behavior,
//! file paths, limits, and naming conventions.

/// Default project directory name
pub const DEFAULT_PROJECT_DIR: &str = ".";

/// Template directory name within the installation
pub const TEMPLATES_DIR: &str = "templates";

/// Template manifest filename
pub const TEMPLATE_MANIFEST: &str = "template.toml";

/// Default Phaser template identifier
pub const DEFAULT_TEMPLATE_ID: &str = "phaser-2d-starter";

/// Maximum prompt length in characters
pub const MAX_PROMPT_LENGTH: usize = 1000;

/// Maximum file size for reading (10MB)
pub const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;

/// Supported file extensions for template files
pub const TEMPLATE_EXTENSIONS: &[&str] = &[".js", ".html", ".css", ".json", ".toml", ".md"];

/// Placeholder prefix for template variables
pub const PLACEHOLDER_PREFIX: &str = "{{";

/// Placeholder suffix for template variables
pub const PLACEHOLDER_SUFFIX: &str = "}}";
