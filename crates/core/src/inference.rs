//! Model Gateway Integration (Legacy)
//!
//! ⚠️ **DEPRECATED**: This module is deprecated in favor of `gateway_integration`.
//! Use `GatewayClient` from `crate::gateway_integration` instead.
//!
//! This module is kept for backward compatibility but will be removed in a future
//! release. All new code should use `GatewayClient`.
//!
//! # Migration Guide
//!
//! ```rust,ignore
//! // Old (deprecated)
//! use peridot_core::inference::{InferenceClient, InferenceRequest};
//! let client = InferenceClient::from_config_manager(&config_manager).await?;
//!
//! // New (recommended)
//! use peridot_core::gateway::{GatewayClient, InferenceStatus};
//! let client = GatewayClient::from_config_manager(&config_manager).await;
//! ```

#![allow(deprecated)]

use peridot_model_gateway::{
    ConfigManager, GatewayError, InferenceResponse, Message, Provider,
};
use peridot_shared::PeridotError;

/// Configuration for inference
///
/// **DEPRECATED**: Use `GatewayClient` directly from `crate::gateway_integration`.
/// This type will be removed in a future release.
#[deprecated(since = "0.1.0", note = "Use GatewayClient from gateway_integration module instead")]
#[derive(Debug, Clone)]
pub struct InferenceConfig {
    /// System prompt to prepend to all requests
    pub system_prompt: Option<String>,
    /// Default temperature (0.0 - 2.0)
    pub temperature: f32,
    /// Maximum tokens to generate
    pub max_tokens: u32,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            system_prompt: None,
            temperature: 0.7,
            max_tokens: 2048,
        }
    }
}

/// Client for AI inference through the model gateway
///
/// **DEPRECATED**: Use `GatewayClient` from `crate::gateway_integration` instead.
/// This type will be removed in a future release.
///
/// This was the primary interface for core to use AI models. It wraps the
/// model gateway's provider implementation and provides a simplified,
/// configuration-driven interface.
#[deprecated(since = "0.1.0", note = "Use GatewayClient from gateway_integration module instead")]
#[derive(Debug)]
pub struct InferenceClient {
    /// The underlying provider client
    provider: Box<dyn Provider>,
    /// Configuration
    config: InferenceConfig,
    /// Model ID to use
    model: String,
}

impl InferenceClient {
    /// Create a new inference client with a provider
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use peridot_model_gateway::create_openrouter_client;
    ///
    /// let provider = create_openrouter_client(&config_manager).await?;
    /// let client = InferenceClient::new(provider, "anthropic/claude-3.5-sonnet");
    /// ```
    pub fn new(provider: Box<dyn Provider>, model: impl Into<String>) -> Self {
        Self {
            provider,
            config: InferenceConfig::default(),
            model: model.into(),
        }
    }

    /// Create a new inference client from a ConfigManager
    ///
    /// This is the recommended way to create a client. It automatically:
    /// - Detects the configured provider
    /// - Creates the appropriate provider client
    /// - Uses the configured default model
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No provider is configured
    /// - Provider credentials are invalid
    /// - Provider client creation fails
    pub async fn from_config_manager(
        config_manager: &ConfigManager,
    ) -> Result<Self, InferenceError> {
        let status = config_manager.config_status();

        if !status.has_provider {
            return Err(InferenceError::NotConfigured);
        }

        if !status.provider_ready {
            return Err(InferenceError::ProviderNotReady(
                status.provider_name.unwrap_or_else(|| "Unknown".to_string()),
            ));
        }

        if !status.has_model {
            return Err(InferenceError::NoModelSelected);
        }

        let provider_id = config_manager
            .config()
            .default_provider
            .as_ref()
            .ok_or(InferenceError::NotConfigured)?;

        let model_id = config_manager
            .config()
            .default_model
            .as_ref()
            .ok_or(InferenceError::NoModelSelected)?;

        // Create provider-specific client
        let provider: Box<dyn Provider> = match provider_id.as_str() {
            "openrouter" => {
                let client =
                    peridot_model_gateway::create_openrouter_client(config_manager).await?;
                Box::new(client)
            }
            _ => {
                return Err(InferenceError::UnsupportedProvider(
                    provider_id.to_string(),
                ))
            }
        };

        Ok(Self {
            provider,
            config: InferenceConfig::default(),
            model: model_id.to_string(),
        })
    }

    /// Set the system prompt
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.config.system_prompt = Some(prompt.into());
        self
    }

    /// Set the temperature
    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.config.temperature = temp.clamp(0.0, 2.0);
        self
    }

    /// Set the maximum tokens
    pub fn with_max_tokens(mut self, tokens: u32) -> Self {
        self.config.max_tokens = tokens;
        self
    }

    /// Perform inference
    ///
    /// Sends the request to the configured AI model and returns the response.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let request = InferenceRequest::new("Create a game idea...");
    /// let response = client.infer(request).await?;
    /// ```
    pub async fn infer(&self, request: InferenceRequest) -> Result<InferenceResponse, InferenceError> {
        // Build messages
        let mut messages = Vec::new();

        // Add system prompt if configured
        if let Some(ref system) = self.config.system_prompt {
            messages.push(Message::system(system.clone()));
        }

        // Add user message
        messages.push(Message::user(request.user_prompt));

        // Create gateway request
        let mut gateway_request = peridot_model_gateway::InferenceRequest::new(&self.model)
            .with_temperature(self.config.temperature)
            .with_max_tokens(self.config.max_tokens);
        
        for message in messages {
            gateway_request = gateway_request.with_message(message);
        }

        // Send to provider
        let response = self.provider.infer(gateway_request).await?;

        Ok(response)
    }

    /// Get the configured model name
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Check if the client is ready to use
    pub fn is_ready(&self) -> bool {
        self.provider.is_configured()
    }
}

/// Request for inference
#[derive(Debug, Clone)]
pub struct InferenceRequest {
    /// The user's prompt
    pub user_prompt: String,
    /// Optional context or additional instructions
    pub context: Option<String>,
}

impl InferenceRequest {
    /// Create a new inference request
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            user_prompt: prompt.into(),
            context: None,
        }
    }

    /// Add context to the request
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }
}

/// Errors that can occur during inference
#[derive(Debug, Clone)]
pub enum InferenceError {
    /// No AI provider is configured
    NotConfigured,
    /// Provider is configured but not ready (missing API key, etc.)
    ProviderNotReady(String),
    /// No model is selected
    NoModelSelected,
    /// Provider is not supported
    UnsupportedProvider(String),
    /// Gateway error
    GatewayError(String),
    /// Request validation error
    ValidationError(String),
}

impl std::fmt::Display for InferenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InferenceError::NotConfigured => write!(f, "No AI provider configured. Run setup first."),
            InferenceError::ProviderNotReady(provider) => {
                write!(f, "Provider '{}' is not ready. Check your API key.", provider)
            }
            InferenceError::NoModelSelected => write!(f, "No model selected. Run setup to choose a model."),
            InferenceError::UnsupportedProvider(provider) => {
                write!(f, "Provider '{}' is not supported yet", provider)
            }
            InferenceError::GatewayError(msg) => write!(f, "AI request failed: {}", msg),
            InferenceError::ValidationError(msg) => write!(f, "Invalid request: {}", msg),
        }
    }
}

impl std::error::Error for InferenceError {}

impl From<GatewayError> for InferenceError {
    fn from(err: GatewayError) -> Self {
        InferenceError::GatewayError(err.to_string())
    }
}

impl From<InferenceError> for PeridotError {
    fn from(err: InferenceError) -> Self {
        PeridotError::General(err.to_string())
    }
}

/// Example: Simple AI-powered intent classification
///
/// This demonstrates how the orchestrator can use AI to classify user prompts.
pub async fn example_classify_intent(
    client: &InferenceClient,
    user_prompt: &str,
) -> Result<String, InferenceError> {
    let request = InferenceRequest::new(format!(
        r#"Classify this game development request into one of these categories:
- "new_game" - User wants to create a new game
- "add_feature" - User wants to add a feature to an existing game
- "modify" - User wants to modify existing code
- "unknown" - Cannot determine intent

User request: "{}"

Respond with ONLY the category name."#,
        user_prompt
    ));

    let response = client.infer(request).await?;
    let category = response.content().trim().to_lowercase();

    Ok(category)
}

/// Example: Generate game scaffold description
///
/// This demonstrates how the orchestrator can use AI to enhance template generation.
pub async fn example_enhance_scaffold(
    client: &InferenceClient,
    user_prompt: &str,
) -> Result<String, InferenceError> {
    let request = InferenceRequest::new(format!(
        r#"Given this game description, create a detailed scaffold specification:

User description: "{}"

Provide:
1. Game genre
2. Key features needed
3. Recommended template structure
4. File organization suggestions

Be concise but specific."#,
        user_prompt
    ))
    .with_context("This is for a Phaser 2D JavaScript game template.");

    let response = client.infer(request).await?;
    Ok(response.content().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inference_request_builder() {
        let request = InferenceRequest::new("Create a platformer")
            .with_context("2D web game");

        assert_eq!(request.user_prompt, "Create a platformer");
        assert_eq!(request.context, Some("2D web game".to_string()));
    }

    #[test]
    fn test_inference_config_defaults() {
        let config = InferenceConfig::default();
        assert_eq!(config.temperature, 0.7);
        assert_eq!(config.max_tokens, 2048);
        assert!(config.system_prompt.is_none());
    }
}