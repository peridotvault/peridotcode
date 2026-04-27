//! Tool Dispatcher
//!
//! The ToolDispatcher routes action requests to the appropriate tool implementations
//! and manages tool execution.
//!
//! # Architecture
//!
//! ```text
//! AgentRequest → ToolDispatcher → ToolRegistry → Tool::execute()
//!                                    ↓
//!                              ToolResult
//! ```

use peridot_model_gateway::{InferenceRequest, InferenceResponse};
use peridot_shared::PeridotResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::tools::{
    modify_code::{modify_code, LlmClient, LlmContext, ModifyCodeParams},
    read_file::{read_file_tool, ReadFileParams},
    ToolContext, ToolRegistry, ToolResult,
};

/// A tool call request from the agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Tool identifier
    pub tool_id: String,
    /// Parameters for the tool
    pub params: serde_json::Value,
    /// Unique call identifier (for tracking)
    pub call_id: String,
}

impl ToolCall {
    /// Create a new tool call
    pub fn new(tool_id: impl Into<String>, params: serde_json::Value) -> Self {
        Self {
            tool_id: tool_id.into(),
            params,
            call_id: format!("call_{}", uuid::Uuid::new_v4().to_string()[..8].to_string()),
        }
    }

    /// Create with explicit call ID
    pub fn with_id(tool_id: impl Into<String>, params: serde_json::Value, call_id: impl Into<String>) -> Self {
        Self {
            tool_id: tool_id.into(),
            params,
            call_id: call_id.into(),
        }
    }
}

/// Result of executing a tool call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResult {
    /// The original call ID
    pub call_id: String,
    /// The tool that was executed
    pub tool_id: String,
    /// Execution result
    pub result: ToolResult,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
}

/// Tool dispatcher that routes actions to tools
#[derive(Debug)]
pub struct ToolDispatcher {
    /// Registry of available tools
    registry: ToolRegistry,
    /// Execution statistics
    stats: ToolExecutionStats,
}

/// Statistics about tool execution
#[derive(Debug, Default, Clone, Serialize)]
pub struct ToolExecutionStats {
    /// Total number of calls executed
    pub total_calls: u64,
    /// Number of successful calls
    pub successful_calls: u64,
    /// Number of failed calls
    pub failed_calls: u64,
    /// Calls per tool
    pub calls_by_tool: HashMap<String, u64>,
}

impl ToolDispatcher {
    /// Create a new tool dispatcher with default tools
    pub fn new() -> Self {
        Self {
            registry: ToolRegistry::with_defaults(),
            stats: ToolExecutionStats::default(),
        }
    }

    /// Create with custom registry
    pub fn with_registry(registry: ToolRegistry) -> Self {
        Self {
            registry,
            stats: ToolExecutionStats::default(),
        }
    }

    /// Execute a single tool call
    ///
    /// This is the main entry point for tool execution.
    pub async fn execute(
        &mut self,
        call: ToolCall,
        context: &ToolContext,
    ) -> ToolCallResult {
        let start = std::time::Instant::now();

        let result = if let Some(tool) = self.registry.get(&call.tool_id) {
            tracing::info!("Executing tool '{}' (call: {})", call.tool_id, call.call_id);
            tool.execute(call.params, context).await
        } else {
            ToolResult::error(format!("Unknown tool: {}", call.tool_id))
        };

        let execution_time_ms = start.elapsed().as_millis() as u64;

        // Update stats
        self.stats.total_calls += 1;
        if result.is_success() {
            self.stats.successful_calls += 1;
        } else {
            self.stats.failed_calls += 1;
        }
        *self.stats.calls_by_tool.entry(call.tool_id.clone()).or_insert(0) += 1;

        tracing::info!(
            "Tool '{}' completed in {}ms (success: {})",
            call.tool_id,
            execution_time_ms,
            result.is_success()
        );

        ToolCallResult {
            call_id: call.call_id,
            tool_id: call.tool_id,
            result,
            execution_time_ms,
        }
    }

    /// Execute a tool call with LLM support (for tools that need AI)
    ///
    /// This version provides access to the model gateway for tools like modify_code.
    pub async fn execute_with_llm<C: LlmClient>(
        &mut self,
        call: ToolCall,
        context: &ToolContext,
        llm_client: &C,
        model: &str,
    ) -> ToolCallResult {
        let start = std::time::Instant::now();

        let result = match call.tool_id.as_str() {
            "read_file" => {
                // Read file doesn't need LLM
                match serde_json::from_value::<ReadFileParams>(call.params.clone()) {
                    Ok(params) => match read_file_tool(&params, context).await {
                        Ok(data) => match ToolResult::success_with_data("File read successfully", data) {
                            Ok(r) => r,
                            Err(e) => ToolResult::error(format!("Serialization error: {}", e)),
                        },
                        Err(e) => ToolResult::error(format!("Failed to read file: {}", e)),
                    },
                    Err(e) => ToolResult::error(format!("Invalid parameters: {}", e)),
                }
            }
            "modify_code" => {
                // Modify code needs LLM
                match serde_json::from_value::<ModifyCodeParams>(call.params.clone()) {
                    Ok(params) => {
                        let llm_context = LlmContext {
                            client: llm_client,
                            model: model.to_string(),
                        };
                        match modify_code(&params, context, &llm_context).await {
                            Ok(data) => match ToolResult::success_with_data("Code modified successfully", data) {
                                Ok(r) => r,
                                Err(e) => ToolResult::error(format!("Serialization error: {}", e)),
                            },
                            Err(e) => ToolResult::error(format!("Failed to modify code: {}", e)),
                        }
                    }
                    Err(e) => ToolResult::error(format!("Invalid parameters: {}", e)),
                }
            }
            _ => ToolResult::error(format!("Unknown tool: {}", call.tool_id)),
        };

        let execution_time_ms = start.elapsed().as_millis() as u64;

        // Update stats
        self.stats.total_calls += 1;
        if result.is_success() {
            self.stats.successful_calls += 1;
        } else {
            self.stats.failed_calls += 1;
        }
        *self.stats.calls_by_tool.entry(call.tool_id.clone()).or_insert(0) += 1;

        ToolCallResult {
            call_id: call.call_id,
            tool_id: call.tool_id,
            result,
            execution_time_ms,
        }
    }

    /// Execute multiple tool calls in sequence
    pub async fn execute_batch(
        &mut self,
        calls: Vec<ToolCall>,
        context: &ToolContext,
    ) -> Vec<ToolCallResult> {
        let mut results = Vec::new();
        for call in calls {
            let result = self.execute(call, context).await;
            results.push(result);
        }
        results
    }

    /// Check if a tool exists
    pub fn has_tool(&self, tool_id: &str) -> bool {
        self.registry.has(tool_id)
    }

    /// Get list of available tools
    pub fn list_tools(&self) -> Vec<&str> {
        self.registry.list_tools()
    }

    /// Get tool definitions for LLM function calling
    pub fn get_tool_definitions(&self) -> Vec<serde_json::Value> {
        self.registry.to_tool_definitions()
    }

    /// Get execution statistics
    pub fn stats(&self) -> &ToolExecutionStats {
        &self.stats
    }

    /// Get a reference to the registry
    pub fn registry(&self) -> &ToolRegistry {
        &self.registry
    }

    /// Get mutable reference to registry
    pub fn registry_mut(&mut self) -> &mut ToolRegistry {
        &mut self.registry
    }
}

impl Default for ToolDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple LLM client wrapper for model gateway
pub struct ModelGatewayClient<'a> {
    provider: &'a dyn peridot_model_gateway::Provider,
}

impl<'a> ModelGatewayClient<'a> {
    /// Create a new client from a provider
    pub fn new(provider: &'a dyn peridot_model_gateway::Provider) -> Self {
        Self { provider }
    }
}

#[async_trait::async_trait]
impl<'a> LlmClient for ModelGatewayClient<'a> {
    async fn infer(&self, request: InferenceRequest) -> PeridotResult<InferenceResponse> {
        self.provider.infer(request).await.map_err(|e| {
            peridot_shared::PeridotError::General(format!("Gateway error: {}", e))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::read_file::ReadFileTool;

    // Mock LLM client for testing
    struct MockLlmClient;

    #[async_trait::async_trait]
    impl LlmClient for MockLlmClient {
        async fn infer(&self, _request: InferenceRequest) -> PeridotResult<InferenceResponse> {
            Ok(InferenceResponse {
                message: peridot_model_gateway::Message::assistant(r#"{"summary": "Test", "modified_code": "test code"}"#),
                model: "test-model".to_string(),
                provider: "test".to_string(),
                usage: None,
                finish_reason: Some("stop".to_string()),
            })
        }
    }

    #[test]
    fn test_tool_dispatcher_creation() {
        let dispatcher = ToolDispatcher::new();

        assert!(dispatcher.has_tool("read_file"));
        assert!(dispatcher.has_tool("modify_code"));
        assert!(!dispatcher.has_tool("nonexistent"));
    }

    #[test]
    fn test_tool_call_creation() {
        let params = serde_json::json!({"file_path": "test.js"});
        let call = ToolCall::new("read_file", params.clone());

        assert_eq!(call.tool_id, "read_file");
        assert_eq!(call.params, params);
        assert!(!call.call_id.is_empty());
    }

    #[test]
    fn test_tool_definitions() {
        let dispatcher = ToolDispatcher::new();
        let defs = dispatcher.get_tool_definitions();

        assert!(!defs.is_empty());

        // Check that read_file is in definitions
        let has_read_file = defs.iter().any(|d| {
            d.get("function")
                .and_then(|f| f.get("name"))
                .and_then(|n| n.as_str())
                == Some("read_file")
        });
        assert!(has_read_file);
    }
}
