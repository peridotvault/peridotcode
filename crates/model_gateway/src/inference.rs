//! Normalized inference types
//!
//! Provides provider-agnostic request/response formats for AI inference.

use serde::{Deserialize, Serialize};

/// A message in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Role of the message sender
    pub role: Role,
    /// Message content
    pub content: String,
}

impl Message {
    /// Create a new message
    pub fn new(role: Role, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
        }
    }

    /// Create a system message
    pub fn system(content: impl Into<String>) -> Self {
        Self::new(Role::System, content)
    }

    /// Create a user message
    pub fn user(content: impl Into<String>) -> Self {
        Self::new(Role::User, content)
    }

    /// Create an assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        Self::new(Role::Assistant, content)
    }
}

/// Role of a message sender
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// System instructions/prompt
    System,
    /// User input
    User,
    /// Model response
    Assistant,
}

/// An inference request to a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRequest {
    /// Model to use (provider-specific ID)
    pub model: String,
    /// Conversation messages
    pub messages: Vec<Message>,
    /// Temperature (0.0 - 2.0, default: 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Maximum tokens to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// Whether to stream the response
    #[serde(default)]
    pub stream: bool,
}

impl InferenceRequest {
    /// Create a new inference request
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            messages: Vec::new(),
            temperature: None,
            max_tokens: None,
            stream: false,
        }
    }

    /// Add a message to the request
    pub fn with_message(mut self, message: Message) -> Self {
        self.messages.push(message);
        self
    }

    /// Add multiple messages to the request
    pub fn with_messages(mut self, messages: Vec<Message>) -> Self {
        self.messages.extend(messages);
        self
    }

    /// Add a system message
    pub fn with_system(self, content: impl Into<String>) -> Self {
        self.with_message(Message::system(content))
    }

    /// Add a user message
    pub fn with_user(self, content: impl Into<String>) -> Self {
        self.with_message(Message::user(content))
    }

    /// Set the temperature
    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.temperature = Some(temp.clamp(0.0, 2.0));
        self
    }

    /// Set max tokens
    pub fn with_max_tokens(mut self, tokens: u32) -> Self {
        self.max_tokens = Some(tokens);
        self
    }

    /// Enable streaming
    pub fn with_streaming(mut self) -> Self {
        self.stream = true;
        self
    }
}

/// An inference response from a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResponse {
    /// Generated message
    pub message: Message,
    /// Model used for generation
    pub model: String,
    /// Provider that handled the request
    pub provider: String,
    /// Usage statistics (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<UsageStats>,
    /// Finish reason (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}

impl InferenceResponse {
    /// Get the content of the response
    pub fn content(&self) -> &str {
        &self.message.content
    }

    /// Check if the response has usage stats
    pub fn has_usage(&self) -> bool {
        self.usage.is_some()
    }
}

/// Statistics about token usage for an inference request
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UsageStats {
    /// Number of tokens in the prompt
    pub prompt_tokens: u32,
    /// Completion tokens generated
    pub completion_tokens: u32,
    /// Total tokens
    pub total_tokens: u32,
}
