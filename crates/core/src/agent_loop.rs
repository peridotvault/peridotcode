//! Agent Loop System
//!
//! This module provides an autonomous agent system capable of:
//! - Multi-turn conversational interaction with LLM
//! - Tool selection and execution
//! - Structured response parsing
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                     AgentLoop                              │
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐   │
//! │  │ AgentState │  │  Message  │  │   Tool    │   │
//! │  │           │  │  History  │  │ Registry  │   │
//! │  └─────────────┘  └─────────────┘  └─────────────┘   │
//! └───────────────────────┬─────────────────────────────────────┘
//!                       │
//!                       ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │              GatewayClient (LLM)                         │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Usage
//!
//! ```rust,ignore
//! use peridot_core::agent_loop::{AgentLoop, AgentConfig};
//!
//! let config = AgentConfig::default();
//! let mut agent = AgentLoop::new(config);
//!
//! // Process prompt and get structured response
//! let result = agent.process("Create a platformer game").await?;
//! println!("Response: {}", result.response);
//! ```

use peridot_model_gateway::{InferenceRequest, Message, Role};
use serde::{Deserialize, Serialize};

use crate::gateway_integration::GatewayClient;

/// Configuration for the agent loop
#[derive(Debug, Clone)]
pub struct AgentConfig {
    /// System prompt to set agent behavior
    pub system_prompt: String,
    /// Maximum conversation turns to kept in history
    pub max_history: usize,
    /// Whether to enable tool use
    pub enable_tools: bool,
    /// Temperature for LLM responses
    pub temperature: f32,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            system_prompt: "You are a helpful game development assistant.".to_string(),
            max_history: 10,
            enable_tools: true,
            temperature: 0.7,
        }
    }
}

/// Role of a message in the conversation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageRole {
    /// System message (internal)
    System,
    /// User message
    User,
    /// Assistant (AI) message
    Assistant,
    /// Tool result message
    Tool,
}

impl MessageRole {
    /// Convert to model gateway Role type
    pub fn to_gateway_role(&self) -> Role {
        match self {
            MessageRole::System => Role::System,
            MessageRole::User => Role::User,
            MessageRole::Assistant => Role::Assistant,
            MessageRole::Tool => Role::User,
        }
    }
}

/// A single message in the agent conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    /// The role of the message sender
    pub role: MessageRole,
    /// The content of the message
    pub content: String,
    /// Optional tool call identifier
    pub tool_call_id: Option<String>,
}

impl AgentMessage {
    /// Create a new message
    pub fn new(role: MessageRole, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
            tool_call_id: None,
        }
    }

    /// Create a system message
    pub fn system(content: impl Into<String>) -> Self {
        Self::new(MessageRole::System, content)
    }

    /// Create a user message
    pub fn user(content: impl Into<String>) -> Self {
        Self::new(MessageRole::User, content)
    }

    /// Create an assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        Self::new(MessageRole::Assistant, content)
    }

    /// Create a tool result message
    pub fn tool(content: impl Into<String>, tool_call_id: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Tool,
            content: content.into(),
            tool_call_id: Some(tool_call_id.into()),
        }
    }
}

/// Result of agent processing
#[derive(Debug)]
pub struct AgentResult {
    /// The agent's response content
    pub response: String,
    /// Token count used in this request
    pub usage: Option<u32>,
    /// Whether tools were used
    pub tools_used: bool,
    /// Any error that occurred
    pub error: Option<AgentError>,
}

/// Standard LLM response schema (JSON)
///
/// This is the structured response format the agent returns.
/// The LLM is instructed to respond in this JSON format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredResponse {
    /// The action to take (e.g., "create_game", "add_feature", "modify_code", "ask_more", "done")
    pub action: String,
    /// Brief summary of what will happen
    pub summary: String,
    /// Detailed message to display to the user
    pub message: Option<String>,
    /// Parameters for the action (if any)
    pub params: Option<serde_json::Value>,
}

impl StructuredResponse {
    /// Create a "done" response
    pub fn done(message: impl Into<String>) -> Self {
        Self {
            action: "done".to_string(),
            summary: "Task completed".to_string(),
            message: Some(message.into()),
            params: None,
        }
    }

    /// Create a response asking for more information
    pub fn ask_more(question: impl Into<String>) -> Self {
        Self {
            action: "ask_more".to_string(),
            summary: "Need more information".to_string(),
            message: Some(question.into()),
            params: None,
        }
    }

    /// Create a create_game response
    pub fn create_game(genre: String, features: Vec<String>) -> Self {
        Self {
            action: "create_game".to_string(),
            summary: format!("Creating {} game", genre),
            message: None,
            params: Some(serde_json::json!({
                "genre": genre,
                "features": features
            })),
        }
    }
}

/// Response parser for structured responses
///
/// Parses JSON responses from the LLM into StructuredResponse.
/// Falls back to text response if parsing fails.
pub struct ResponseParser;

impl ResponseParser {
    /// Parse a response string into StructuredResponse
    ///
    /// Attempts to parse JSON. If that fails, wraps the text in a basic response.
    pub fn parse(response: &str) -> Result<StructuredResponse, AgentError> {
        // Try to find JSON in the response (may be wrapped in markdown)
        let json_text = Self::extract_json(response)
            .ok_or_else(|| AgentError::ParseError("No JSON found".to_string()))?;

        // Parse JSON
        let parsed: StructuredResponse = serde_json::from_str(&json_text)
            .map_err(|e| AgentError::ParseError(format!("JSON parse error: {}", e)))?;

        Ok(parsed)
    }

    /// Extract JSON from response text (handles markdown code blocks)
    fn extract_json(text: &str) -> Option<String> {
        let text = text.trim();
        
        // Check for markdown code block
        if text.starts_with("```") {
            // Find the JSON content between code blocks
            let start = text.find("```\n").map(|i| i + 4);
            let end = text.rfind("```");
            
            if let (Some(start), Some(end)) = (start, end) {
                return Some(text[start..end].trim().to_string());
            }
        }
        
        // Try as plain JSON
        Some(text.to_string())
    }

    /// Parse or return text response
    pub fn parse_or_text(response: &str, fallback_message: &str) -> StructuredResponse {
        Self::parse(response).unwrap_or_else(|_| {
            StructuredResponse::done(fallback_message)
        })
    }
}

impl AgentResult {
    /// Check if the result is successful
    pub fn is_success(&self) -> bool {
        self.error.is_none()
    }
}

/// Errors that can occur in the agent
#[derive(Debug, Clone)]
pub enum AgentError {
    /// Not configured
    NotConfigured(String),
    /// LLM inference failed
    InferenceFailed(String),
    /// Structured response parsing failed
    ParseError(String),
    /// Tool execution failed
    ToolError(String),
}

impl std::fmt::Display for AgentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentError::NotConfigured(msg) => write!(f, "Not configured: {}", msg),
            AgentError::InferenceFailed(msg) => write!(f, "Inference failed: {}", msg),
            AgentError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            AgentError::ToolError(msg) => write!(f, "Tool error: {}", msg),
        }
    }
}

impl std::error::Error for AgentError {}

/// State maintained by the agent across interactions
#[derive(Debug)]
pub struct AgentState {
    /// Conversation history
    messages: Vec<AgentMessage>,
    /// Total tokens used
    pub total_tokens: u32,
    /// Number of interactions
    pub interaction_count: u32,
}

impl AgentState {
    /// Create new agent state
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            total_tokens: 0,
            interaction_count: 0,
        }
    }

    /// Add a message to history
    pub fn add_message(&mut self, message: AgentMessage) {
        self.messages.push(message);
    }

    /// Get conversation history
    pub fn messages(&self) -> &[AgentMessage] {
        &self.messages
    }

    /// Truncate history to max size
    pub fn truncate(&mut self, max_size: usize) {
        if self.messages.len() > max_size {
            let remove_count = self.messages.len() - max_size;
            self.messages.drain(0..remove_count);
        }
    }

    /// Increment interaction count
    pub fn increment_interactions(&mut self) {
        self.interaction_count += 1;
    }
}

impl Default for AgentState {
    fn default() -> Self {
        Self::new()
    }
}

/// Tool definition for the agent
#[derive(Debug, Clone)]
pub struct AgentTool {
    /// Unique tool identifier
    id: String,
    /// Tool name for display
    name: String,
    /// Tool description
    description: String,
    /// Parameter schema (JSON schema)
    parameter_schema: Option<String>,
}

impl AgentTool {
    /// Create a new tool
    pub fn new(id: impl Into<String>, name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            parameter_schema: None,
        }
    }

    /// Get tool ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get tool name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get tool description
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Set parameter schema
    pub fn with_parameters(mut self, schema: impl Into<String>) -> Self {
        self.parameter_schema = Some(schema.into());
        self
    }
}

/// Registry of available tools
#[derive(Debug, Default)]
pub struct ToolRegistry {
    /// Available tools
    tools: Vec<AgentTool>,
}

impl ToolRegistry {
    /// Create new tool registry
    pub fn new() -> Self {
        Self { tools: Vec::new() }
    }

    /// Register a tool
    pub fn register(&mut self, tool: AgentTool) {
        self.tools.push(tool);
    }

    /// Get all tools
    pub fn tools(&self) -> &[AgentTool] {
        &self.tools
    }

    /// Get tool by ID
    pub fn get(&self, id: &str) -> Option<&AgentTool> {
        self.tools.iter().find(|t| t.id() == id)
    }

    /// Generate tool definitions for LLM
    pub fn to_tool_definitions(&self) -> String {
        if self.tools.is_empty() {
            return String::new();
        }

        let mut definitions = Vec::new();
        for tool in &self.tools {
            let mut def = format!(
                r#"{{
                    "type": "function",
                    "function": {{
                        "name": "{}",
                        "description": "{}""#,
                tool.id(), tool.description()
            );
            if let Some(ref schema) = tool.parameter_schema {
                def.push_str(&format!(r#", "parameters": {}"#, schema));
            }
            def.push_str("}}");
            definitions.push(def);
        }

        format!("[{}]", definitions.join(","))
    }
}

/// Main agent loop
///
/// Coordinates conversation with LLM, maintains state, and handles tools.
#[derive(Debug)]
pub struct AgentLoop {
    /// Configuration
    config: AgentConfig,
    /// Gateway client for LLM inference
    gateway_client: GatewayClient,
    /// Agent state (history, tokens, etc.)
    state: AgentState,
    /// Tool registry
    tools: ToolRegistry,
}

impl AgentLoop {
    /// Create a new agent loop
    ///
    /// Requires a GatewayClient for LLM inference.
    pub fn new(config: AgentConfig, gateway_client: GatewayClient) -> Self {
        let tools = Self::default_tools();
        
        Self {
            config,
            gateway_client,
            state: AgentState::new(),
            tools,
        }
    }

    /// Create with default tool registry
    fn default_tools() -> ToolRegistry {
        let mut registry = ToolRegistry::new();
        
        // Register default game development tools
        registry.register(AgentTool::new(
            "create_game",
            "create_game",
            "Create a new game project with specified genre and features",
        ).with_parameters(r#"{
            "type": "object",
            "properties": {
                "genre": {"type": "string", "description": "Game genre (platformer, rpg, puzzle, etc.)"},
                "features": {"type": "array", "items": {"type": "string"}, "description": "List of features to include"}
            },
            "required": ["genre"]
        }"#));

        registry.register(AgentTool::new(
            "add_feature",
            "add_feature",
            "Add a new feature to an existing game project",
        ).with_parameters(r#"{
            "type": "object",
            "properties": {
                "feature_name": {"type": "string", "description": "Name of the feature to add"},
                "description": {"type": "string", "description": "Description of the feature"}
            },
            "required": ["feature_name"]
        }"#));

        registry.register(AgentTool::new(
            "modify_code",
            "modify_code",
            "Modify existing code in the game project",
        ).with_parameters(r#"{
            "type": "object",
            "properties": {
                "file_path": {"type": "string", "description": "Path to the file to modify"},
                "change_type": {"type": "string", "enum": ["add", "update", "fix"]},
                "description": {"type": "string", "description": "Description of the change"}
            },
            "required": ["file_path", "change_type"]
        }"#));

        registry
    }

    /// Check if agent is ready
    pub fn is_ready(&self) -> bool {
        self.gateway_client.is_ready()
    }

    /// Get agent status
    pub fn status(&self) -> String {
        if self.is_ready() {
            format!(
                "Ready | {} | Interactions: {}",
                self.gateway_client.provider_name().unwrap_or("unknown"),
                self.state.interaction_count
            )
        } else {
            "Not ready".to_string()
        }
    }

    /// Process a user prompt and get structured response
    ///
    /// This is the main entry point for agent interaction.
    /// Pipeline: prompt -> add to history -> call LLM -> parse response -> return result
    pub async fn process(&mut self, prompt: impl Into<String>) -> AgentResult {
        let prompt_text = prompt.into();
        
        // Check if ready
        if !self.is_ready() {
            return AgentResult {
                response: String::new(),
                usage: None,
                tools_used: false,
                error: Some(AgentError::NotConfigured(
                    "Gateway client not ready".to_string()
                )),
            };
        }

        tracing::info!("Agent processing prompt: {}", &prompt_text);

        // Add user message to history
        self.state.add_message(AgentMessage::user(&prompt_text));

        // Build messages for LLM with JSON response instruction
        let messages = self.build_messages_with_schema();

        // Call LLM
        let result = self.call_llm(messages).await;

        match result {
            Ok((response, usage)) => {
                tracing::info!("LLM response received ({} tokens)", usage);
                tracing::debug!("Raw response: {}", &response);

                // Parse structured response
                let structured = ResponseParser::parse_or_text(&response, &response);

                tracing::info!("Parsed action: {} - {}", structured.action, structured.summary);

                // Add assistant response to history
                self.state.add_message(AgentMessage::assistant(&response));
                self.state.total_tokens += usage;
                self.state.increment_interactions();

                AgentResult {
                    response,
                    usage: Some(usage),
                    tools_used: structured.action != "done" && structured.action != "ask_more",
                    error: None,
                }
            }
            Err(e) => {
                tracing::error!("LLM call failed: {}", e);
                AgentResult {
                    response: String::new(),
                    usage: None,
                    tools_used: false,
                    error: Some(e),
                }
            }
        }
    }

    /// Build messages with JSON response schema instruction
    fn build_messages_with_schema(&self) -> Vec<Message> {
        let schema_instruction = r#"
You must respond with valid JSON in this exact format:
```json
{
  "action": "create_game" | "add_feature" | "modify_code" | "ask_more" | "done",
  "summary": "brief description",
  "message": "detailed message to user (optional)",
  "params": {"key": "value"} // optional parameters
}
```
"#;

        let mut messages = vec![
            Message::system(&self.config.system_prompt),
            Message::system(schema_instruction),
        ];

        // Add conversation history
        let history = self.state.messages();
        let max = self.config.max_history;
        let start = history.len().saturating_sub(max);
        
        for msg in &history[start..] {
            if msg.role == MessageRole::System {
                continue;
            }
            let role = msg.role.to_gateway_role();
            messages.push(Message::new(role, msg.content.clone()));
        }

        messages
    }

    /// Process with system prompt override
    pub async fn process_with_system(
        &mut self,
        prompt: impl Into<String>,
        system_prompt: &str,
    ) -> AgentResult {
        // Temporarily prepend system message
        let prompt_text = prompt.into();
        
        if !self.is_ready() {
            return AgentResult {
                response: String::new(),
                usage: None,
                tools_used: false,
                error: Some(AgentError::NotConfigured(
                    "Gateway client not ready".to_string()
                )),
            };
        }

        // Build messages with custom system prompt
        let messages = vec![
            Message::system(system_prompt.to_string()),
            Message::user(prompt_text),
        ];

        // Call LLM
        let result = self.call_llm_direct(messages).await;

        match result {
            Ok((response, usage)) => {
                self.state.total_tokens += usage;
                self.state.increment_interactions();

                AgentResult {
                    response,
                    usage: Some(usage),
                    tools_used: false,
                    error: None,
                }
            }
            Err(e) => AgentResult {
                response: String::new(),
                usage: None,
                tools_used: false,
                error: Some(e),
            },
        }
    }

    /// Build messages for LLM from history
    fn build_messages(&self) -> Vec<Message> {
        let mut messages = Vec::new();

        // Add system prompt first
        messages.push(Message::system(&self.config.system_prompt));

        // Add conversation history (respecting max_history)
        let max = self.config.max_history;
        let history = self.state.messages();
        let start = history.len().saturating_sub(max);
        
        for msg in &history[start..] {
if msg.role == MessageRole::System {
            continue; // Skip system messages in history (already added separately)
        }
        
        let role = msg.role.to_gateway_role();
        messages.push(Message::new(role, msg.content.clone()));
        }

        messages
    }

    /// Call LLM with built messages
    async fn call_llm(
        &self,
        messages: Vec<Message>,
    ) -> Result<(String, u32), AgentError> {
        let provider = self.gateway_client.provider()
            .ok_or_else(|| AgentError::NotConfigured("No provider".to_string()))?;

        let model = self.gateway_client.model_name()
            .ok_or_else(|| AgentError::NotConfigured("No model selected".to_string()))?
            .to_string();

        let request = InferenceRequest::new(model)
            .with_messages(messages)
            .with_temperature(self.config.temperature);

        let started_at = std::time::Instant::now();

        match provider.infer(request).await {
            Ok(response) => {
                let token_count = response.usage
                    .as_ref()
                    .map(|u| u.total_tokens)
                    .unwrap_or(0);
                
                let content = response.content().to_string();
                
                tracing::debug!(
                    "LLM response: {} tokens in {:.2}s",
                    token_count,
                    started_at.elapsed().as_secs_f64()
                );

                Ok((content, token_count))
            }
            Err(e) => Err(AgentError::InferenceFailed(e.to_string())),
        }
    }

    /// Call LLM with direct messages (for custom system prompts)
    async fn call_llm_direct(
        &self,
        messages: Vec<Message>,
    ) -> Result<(String, u32), AgentError> {
        self.call_llm(messages).await
    }

    /// Get the tool registry
    pub fn tools(&self) -> &ToolRegistry {
        &self.tools
    }

    /// Get conversation history
    pub fn history(&self) -> &[AgentMessage] {
        self.state.messages()
    }

    /// Get interaction count
    pub fn interaction_count(&self) -> u32 {
        self.state.interaction_count
    }

    /// Get total tokens used
    pub fn total_tokens(&self) -> u32 {
        self.state.total_tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_config_default() {
        let config = AgentConfig::default();
        assert_eq!(config.max_history, 10);
        assert_eq!(config.temperature, 0.7);
    }

    #[test]
    fn test_agent_message_creation() {
        let msg = AgentMessage::user("Hello");
        assert_eq!(msg.role, MessageRole::User);
        assert_eq!(msg.content, "Hello");
    }

    #[test]
    fn test_tool_registry() {
        let mut registry = ToolRegistry::new();
        registry.register(AgentTool::new("test", "test", "A test tool"));
        
        let tool = registry.get("test");
        assert!(tool.is_some());
        assert_eq!(tool.unwrap().name(), "test");
    }

    #[test]
    fn test_agent_state() {
        let mut state = AgentState::new();
        state.add_message(AgentMessage::user("Hello"));
        state.add_message(AgentMessage::assistant("Hi there"));
        
        assert_eq!(state.messages().len(), 2);
    }
}