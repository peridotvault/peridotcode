//! Core data models for PeridotCode
//!
//! This module defines the fundamental data structures representing
//! user intents, project configurations, templates, generation results,
//! and provider/model configuration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Unique identifier for a template
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TemplateId(pub String);

impl TemplateId {
    /// Create a new template ID
    pub fn new<S: Into<String>>(id: S) -> Self {
        TemplateId(id.into())
    }

    /// Get the template ID as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for TemplateId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Represents the user's intent parsed from a natural language prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameIntent {
    /// User wants to create a new game project
    NewGame {
        /// The game genre (e.g., "platformer", "rpg", "puzzle")
        genre: String,
        /// List of requested features
        features: Vec<String>,
        /// Optional game description
        description: Option<String>,
    },

    /// User wants to add a feature to an existing project
    AddFeature {
        /// The feature to add
        feature: String,
        /// Additional context or requirements
        context: Option<String>,
    },

    /// User wants to modify an existing project
    ModifyProject {
        /// Description of the desired modification
        modification: String,
    },

    /// Intent could not be determined
    Unknown {
        /// The raw prompt for debugging
        raw_prompt: String,
    },
}

impl GameIntent {
    /// Check if this intent is for creating a new game
    pub fn is_new_game(&self) -> bool {
        matches!(self, GameIntent::NewGame { .. })
    }

    /// Get a display name for this intent type
    pub fn display_name(&self) -> &'static str {
        match self {
            GameIntent::NewGame { .. } => "New Game",
            GameIntent::AddFeature { .. } => "Add Feature",
            GameIntent::ModifyProject { .. } => "Modify Project",
            GameIntent::Unknown { .. } => "Unknown",
        }
    }
}

/// Configuration for a PeridotCode project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Project name
    pub name: String,
    /// Project root directory
    pub path: PathBuf,
    /// Template used to create this project
    pub template_id: TemplateId,
    /// Game metadata
    pub metadata: GameMetadata,
    /// Additional configuration values
    pub settings: HashMap<String, String>,
}

impl ProjectConfig {
    /// Create a new project configuration
    pub fn new<S: Into<String>>(name: S, path: PathBuf, template_id: TemplateId) -> Self {
        ProjectConfig {
            name: name.into(),
            path,
            template_id,
            metadata: GameMetadata::default(),
            settings: HashMap::new(),
        }
    }
}

/// Metadata about a game project
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GameMetadata {
    /// Game title
    pub title: Option<String>,
    /// Short description
    pub description: Option<String>,
    /// Game version
    pub version: Option<String>,
    /// Author name
    pub author: Option<String>,
}

/// Template manifest structure (loaded from template.toml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateManifest {
    /// Template identifier
    pub id: TemplateId,
    /// Human-readable name
    pub name: String,
    /// Template description
    pub description: String,
    /// Target game engine/framework
    pub stack: GameStack,
    /// List of files included in the template
    pub files: Vec<String>,
    /// Template placeholders that need to be filled
    #[serde(default)]
    pub placeholders: Vec<String>,
}

/// Supported game engine stacks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GameStack {
    /// Phaser HTML5 game framework
    Phaser,
    /// Godot game engine
    Godot,
    /// Other/custom stack
    Other,
}

impl Default for GameStack {
    fn default() -> Self {
        GameStack::Phaser
    }
}

/// Result of a scaffold generation operation
#[derive(Debug, Clone)]
pub struct ScaffoldResult {
    /// List of created files with their paths
    pub created_files: Vec<PathBuf>,
    /// Template used for generation
    pub template_id: TemplateId,
    /// Instructions for running the project
    pub run_instructions: Vec<String>,
}

impl ScaffoldResult {
    /// Create a new scaffold result
    pub fn new(template_id: TemplateId) -> Self {
        ScaffoldResult {
            created_files: Vec::new(),
            template_id,
            run_instructions: Vec::new(),
        }
    }

    /// Add a created file to the result
    pub fn add_file(&mut self, path: PathBuf) {
        self.created_files.push(path);
    }
}

/// User input captured from the prompt
#[derive(Debug, Clone)]
pub struct PromptInput {
    /// The raw text input
    pub text: String,
    /// Timestamp of the input
    pub timestamp: std::time::SystemTime,
}

impl PromptInput {
    /// Create a new prompt input
    pub fn new<S: Into<String>>(text: S) -> Self {
        PromptInput {
            text: text.into(),
            timestamp: std::time::SystemTime::now(),
        }
    }
}

/// Provider identifier (mirrors model_gateway::provider::ProviderId)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProviderId(pub String);

impl ProviderId {
    /// Create a new provider ID
    pub fn new<S: Into<String>>(id: S) -> Self {
        ProviderId(id.into())
    }

    /// Get the provider ID as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for ProviderId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ProviderId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Model identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModelId(pub String);

impl ModelId {
    /// Create a new model ID
    pub fn new<S: Into<String>>(id: S) -> Self {
        ModelId(id.into())
    }

    /// Get the model ID as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for ModelId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ModelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Configuration summary for display purposes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSummary {
    /// Whether a default provider is configured
    pub has_provider: bool,
    /// Default provider name (if any)
    pub provider_name: Option<String>,
    /// Default model name (if any)
    pub model_name: Option<String>,
}

impl ConfigSummary {
    /// Create a summary indicating no configuration
    pub fn empty() -> Self {
        Self {
            has_provider: false,
            provider_name: None,
            model_name: None,
        }
    }

    /// Check if the configuration is complete enough to use
    pub fn is_ready(&self) -> bool {
        self.has_provider
    }
}
