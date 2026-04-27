//! Agent Tools System
//!
//! Provides executable tools for the AI agent to interact with the codebase.
//! Tools are the primary mechanism for the agent to read files, modify code,
//! and perform other actions.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
//! │   AgentLoop     │────▶│  ToolDispatcher │────▶│   Tool Trait    │
//! └─────────────────┘     └─────────────────┘     └─────────────────┘
//!                                                          │
//!                    ┌──────────────┬──────────────────────┼───────────────────┐
//!                    ▼              ▼                      ▼                   ▼
//!            ┌──────────┐   ┌──────────┐          ┌──────────┐       ┌──────────┐
//!            │ReadFile  │   │ModifyCode│          │ListFiles │       │  Write   │
//!            └──────────┘   └──────────┘          └──────────┘       └──────────┘
//! ```
//!
//! # Available Tools
//!
//! - **read_file**: Read file contents with safety checks
//! - **modify_code**: Modify existing code via LLM assistance
//! - **list_files**: List files in a directory
//! - **write_file**: Create new files

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub mod dispatcher;
pub mod modify_code;
pub mod read_file;

pub use dispatcher::ToolDispatcher;
pub use modify_code::{modify_code, ModifyCodeParams, ModifyCodeResult};
pub use read_file::{read_file_tool, ReadFileParams, ReadFileResult};

/// Context provided to tools during execution
///
/// Contains information about the project and execution environment
#[derive(Debug, Clone)]
pub struct ToolContext {
    /// Path to the project root
    pub project_path: PathBuf,
    /// Working directory for relative paths
    pub working_dir: PathBuf,
    /// Additional context data
    pub metadata: HashMap<String, String>,
}

impl ToolContext {
    /// Create a new tool context
    pub fn new(project_path: impl AsRef<Path>) -> Self {
        let project_path = project_path.as_ref().to_path_buf();
        Self {
            working_dir: project_path.clone(),
            project_path,
            metadata: HashMap::new(),
        }
    }

    /// Create a context with explicit working directory
    pub fn with_working_dir(
        project_path: impl AsRef<Path>,
        working_dir: impl AsRef<Path>,
    ) -> Self {
        Self {
            project_path: project_path.as_ref().to_path_buf(),
            working_dir: working_dir.as_ref().to_path_buf(),
            metadata: HashMap::new(),
        }
    }

    /// Resolve a potentially relative path to an absolute path
    pub fn resolve_path(&self, path: impl AsRef<Path>) -> PathBuf {
        let path = path.as_ref();
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.working_dir.join(path)
        }
    }

    /// Check if a path is within the project boundaries
    pub fn is_within_project(&self, path: impl AsRef<Path>) -> bool {
        let path = self.resolve_path(path);
        path.starts_with(&self.project_path)
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Trait that all tools must implement
///
/// Tools are the primary mechanism for the agent to interact with the
/// codebase. Each tool has a unique ID, parameter schema, and execution
/// logic.
#[async_trait::async_trait]
pub trait Tool: Send + Sync + std::fmt::Debug {
    /// Unique tool identifier
    fn id(&self) -> &str;

    /// Human-readable name
    fn name(&self) -> &str;

    /// Tool description for LLM
    fn description(&self) -> &str;

    /// Parameter schema as JSON schema
    fn parameter_schema(&self) -> serde_json::Value;

    /// Execute the tool with given parameters
    async fn execute(
        &self,
        params: serde_json::Value,
        context: &ToolContext,
    ) -> ToolResult;
}

/// Result of a tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Whether the tool execution succeeded
    pub success: bool,
    /// Result message
    pub message: String,
    /// Optional structured data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    /// Error details if failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl ToolResult {
    /// Create a successful result
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
            data: None,
            error: None,
        }
    }

    /// Create a successful result with data
    pub fn success_with_data(
        message: impl Into<String>,
        data: impl Serialize,
    ) -> Result<Self, serde_json::Error> {
        Ok(Self {
            success: true,
            message: message.into(),
            data: Some(serde_json::to_value(data)?),
            error: None,
        })
    }

    /// Create a failed result
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            data: None,
            error: None,
        }
    }

    /// Create a failed result with error details
    pub fn error_with_details(message: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            data: None,
            error: Some(error.into()),
        }
    }

    /// Check if result is successful
    pub fn is_success(&self) -> bool {
        self.success
    }
}

/// Tool input parameters for common operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileToolParams {
    /// Path to the file (relative to project root)
    pub file_path: String,
}

/// Registry of available tools
#[derive(Debug, Default)]
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Create a registry with default tools
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register(Box::new(read_file::ReadFileTool));
        registry.register(Box::new(modify_code::ModifyCodeTool));
        registry
    }

    /// Register a tool
    pub fn register(&mut self, tool: Box<dyn Tool>) {
        let id = tool.id().to_string();
        self.tools.insert(id, tool);
    }

    /// Get a tool by ID
    pub fn get(&self, id: &str) -> Option<&dyn Tool> {
        self.tools.get(id).map(|t| t.as_ref())
    }

    /// Check if a tool exists
    pub fn has(&self, id: &str) -> bool {
        self.tools.contains_key(id)
    }

    /// List all registered tool IDs
    pub fn list_tools(&self) -> Vec<&str> {
        self.tools.keys().map(|k| k.as_str()).collect()
    }

    /// Get tool definitions for LLM
    pub fn to_tool_definitions(&self) -> Vec<serde_json::Value> {
        self.tools
            .values()
            .map(|t| {
                serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": t.id(),
                        "description": t.description(),
                        "parameters": t.parameter_schema()
                    }
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_context() {
        let ctx = ToolContext::new("/project");
        assert_eq!(ctx.project_path, PathBuf::from("/project"));

        let resolved = ctx.resolve_path("src/main.js");
        assert_eq!(resolved, PathBuf::from("/project/src/main.js"));
    }

    #[test]
    fn test_tool_context_absolute_path() {
        let ctx = ToolContext::new("/project");
        let resolved = ctx.resolve_path("/absolute/path.js");
        assert_eq!(resolved, PathBuf::from("/absolute/path.js"));
    }

    #[test]
    fn test_tool_result() {
        let result = ToolResult::success("File read successfully");
        assert!(result.is_success());
        assert_eq!(result.message, "File read successfully");

        let result = ToolResult::error("File not found");
        assert!(!result.is_success());
    }
}
