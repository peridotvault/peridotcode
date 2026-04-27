//! Unified Context System
//!
//! This module provides a comprehensive context management system that combines:
//! - User input parsing and metadata extraction
//! - Short-term conversation history/memory
//! - File inputs (markdown, code files)
//! - Project context
//! - Template loading
//! - System prompts
//! - Structured output constraints
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                      UnifiedContext                              │
//! ├─────────────────────────────────────────────────────────────────┤
//! │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
//! │  │  UserInput   │  │  Conversation│  │  FileInputs  │          │
//! │  │   Parser     │  │   Memory     │  │   Loader     │          │
//! │  └──────────────┘  └──────────────┘  └──────────────┘          │
//! ├─────────────────────────────────────────────────────────────────┤
//! │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
//! │  │   Project    │  │   Template   │  │   System     │          │
//! │  │   Context    │  │    Engine    │  │   Prompts    │          │
//! │  └──────────────┘  └──────────────┘  └──────────────┘          │
//! └─────────────────────────────────────────────────────────────────┘
//!                              │
//!                              ▼
//!                    ┌──────────────────┐
//!                    │  ContextBuilder  │
//!                    │    (merger)      │
//!                    └──────────────────┘
//! ```

use peridot_shared::{GameIntent, PeridotResult, ProjectConfig};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub mod conversation;
pub mod file_loader;
pub mod input_parser;
pub mod prompt_builder;

pub use conversation::{ConversationMemory, ConversationTurn};
pub use file_loader::{FileInput, FileInputLoader, FileType};
pub use input_parser::{InputParser, ParsedInput};
pub use prompt_builder::{ContextMerger, PromptBuilder, SystemPromptTemplate};

/// Unified context that aggregates all contextual information for an agent interaction
#[derive(Debug, Clone)]
pub struct UnifiedContext {
    /// Parsed user input
    pub user_input: ParsedInput,
    /// Conversation history/memory
    pub conversation: ConversationMemory,
    /// Loaded file inputs
    pub files: Vec<FileInput>,
    /// Project context
    pub project: ProjectContextInfo,
    /// System prompt template
    pub system_prompt: Option<SystemPromptTemplate>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
    /// Timestamp of context creation
    pub created_at: SystemTime,
}

/// Project context information
#[derive(Debug, Clone)]
pub struct ProjectContextInfo {
    /// Project path
    pub path: PathBuf,
    /// Whether this is a new project
    pub is_new: bool,
    /// Project configuration if available
    pub config: Option<ProjectConfig>,
    /// Project type detected
    pub project_type: ProjectType,
    /// Files in the project
    pub files: Vec<PathBuf>,
}

/// Project type detection (re-exported from context module)
pub use crate::context::ProjectType;

impl UnifiedContext {
    /// Create a new empty unified context
    pub fn new(user_input: ParsedInput) -> Self {
        Self {
            user_input,
            conversation: ConversationMemory::new(),
            files: Vec::new(),
            project: ProjectContextInfo {
                path: PathBuf::from("."),
                is_new: true,
                config: None,
                project_type: ProjectType::Unknown,
                files: Vec::new(),
            },
            system_prompt: None,
            metadata: HashMap::new(),
            created_at: SystemTime::now(),
        }
    }

    /// Create a context with project information
    pub fn with_project(mut self, project_path: impl AsRef<Path>) -> PeridotResult<Self> {
        let path = project_path.as_ref().to_path_buf();
        
        // Detect project type
        let project_type = Self::detect_project_type(&path);
        
        // List project files
        let files = peridot_fs_engine::read::list_project_files(&path)?;
        
        // Try to load config
        let config = peridot_fs_engine::read::read_project_config(&path).ok().flatten();
        
        self.project = ProjectContextInfo {
            path,
            is_new: config.is_none(),
            config,
            project_type,
            files,
        };
        
        Ok(self)
    }

    /// Add conversation history
    pub fn with_conversation(mut self, conversation: ConversationMemory) -> Self {
        self.conversation = conversation;
        self
    }

    /// Add file inputs
    pub fn with_files(mut self, files: Vec<FileInput>) -> Self {
        self.files = files;
        self
    }

    /// Add system prompt
    pub fn with_system_prompt(mut self, prompt: SystemPromptTemplate) -> Self {
        self.system_prompt = Some(prompt);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Detect project type based on files present
    fn detect_project_type(path: &Path) -> ProjectType {
        // Check for package.json with phaser
        let package_json = path.join("package.json");
        if package_json.exists() {
            if let Ok(content) = std::fs::read_to_string(&package_json) {
                if content.contains("phaser") {
                    return ProjectType::Phaser;
                }
            }
        }

        // Check for Godot project file
        if path.join("project.godot").exists() {
            return ProjectType::Godot;
        }

        // TODO: Check for Unity project files
        // Unity has Assets/ and ProjectSettings/ directories

        ProjectType::Unknown
    }

    /// Get the detected intent from user input
    pub fn intent(&self) -> Option<&GameIntent> {
        self.user_input.intent.as_ref()
    }

    /// Get the raw user prompt
    pub fn raw_prompt(&self) -> &str {
        &self.user_input.raw_text
    }

    /// Check if this is a new game creation request
    pub fn is_new_game_request(&self) -> bool {
        matches!(self.user_input.intent, Some(GameIntent::NewGame { .. }))
    }

    /// Check if this is a feature addition request
    pub fn is_add_feature_request(&self) -> bool {
        matches!(self.user_input.intent, Some(GameIntent::AddFeature { .. }))
    }

    /// Check if this is a modification request
    pub fn is_modify_request(&self) -> bool {
        matches!(self.user_input.intent, Some(GameIntent::ModifyProject { .. }))
    }

    /// Get file input by path
    pub fn get_file(&self, path: &str) -> Option<&FileInput> {
        self.files.iter().find(|f| f.original_path == path || f.resolved_path.to_string_lossy() == path)
    }

    /// Get conversation summary
    pub fn conversation_summary(&self) -> String {
        self.conversation.summary()
    }
}

impl Default for ProjectType {
    fn default() -> Self {
        ProjectType::Unknown
    }
}

impl std::fmt::Display for ProjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectType::Phaser => write!(f, "Phaser"),
            ProjectType::Godot => write!(f, "Godot"),
            ProjectType::Unknown => write!(f, "Unknown"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_context_creation() {
        let parsed = ParsedInput::new("Create a platformer game");
        let context = UnifiedContext::new(parsed);

        assert_eq!(context.raw_prompt(), "Create a platformer game");
        assert!(context.conversation.is_empty());
        assert!(context.files.is_empty());
    }

    #[test]
    fn test_project_type_display() {
        assert_eq!(ProjectType::Phaser.to_string(), "Phaser");
        assert_eq!(ProjectType::Godot.to_string(), "Godot");
        assert_eq!(ProjectType::Unknown.to_string(), "Unknown");
    }

    #[test]
    fn test_unified_context_with_metadata() {
        let parsed = ParsedInput::new("Test prompt");
        let context = UnifiedContext::new(parsed)
            .with_metadata("session_id", "12345")
            .with_metadata("user_id", "user_abc");

        assert_eq!(context.metadata.get("session_id"), Some(&"12345".to_string()));
        assert_eq!(context.metadata.get("user_id"), Some(&"user_abc".to_string()));
    }
}
