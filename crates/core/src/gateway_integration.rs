//! Model Gateway Integration
//!
//! This module provides the integration between the core orchestrator and the
//! model gateway. It demonstrates the complete flow from user prompt to AI
//! response while keeping core decoupled from provider-specific details.
//!
//! # Example Flow
//!
//! ```text
//! User Prompt
//!     │
//!     ▼
//! ┌─────────────────┐
//! │  Orchestrator   │──> Load context, classify intent
//! └────────┬────────┘
//!          │
//!          ▼ (if AI needed)
//! ┌─────────────────┐
//! │InferenceRequest │──> Normalized request with provider/model
//! └────────┬────────┘
//!          │
//!          ▼
//! ┌─────────────────┐
//! │  Model Gateway  │──> Route to selected provider
//! └────────┬────────┘
//!          │
//!          ▼
//! ┌─────────────────┐
//! │    Provider     │──> OpenRouter/OpenAI/Anthropic/etc
//! │   (Adapter)     │
//! └────────┬────────┘
//!          │
//!          ▼
//! ┌─────────────────┐
//! │InferenceResponse│──> Normalized response
//! └────────┬────────┘
//!          │
//!          ▼
//! ┌─────────────────┐
//! │  Orchestrator   │──> Use response for planning/execution
//! └─────────────────┘
//! ```

use peridot_model_gateway::{
    ConfigManager, ConfigStatus, GatewayError, InferenceRequest, InferenceResponse, Message,
    Provider, ProviderId,
};
use peridot_shared::PeridotError;

/// Status of an inference request
///
/// This provides UI-friendly status information for displaying
/// inference progress and results.
#[derive(Debug, Clone)]
pub enum InferenceStatus {
    /// Not configured (no provider/model set up)
    NotConfigured,
    /// Ready to perform inference
    Ready {
        /// Provider name
        provider: String,
        /// Model name
        model: String,
    },
    /// Inference in progress
    InProgress {
        /// Provider name
        provider: String,
        /// Model name
        model: String,
        /// Request start time
        started_at: std::time::Instant,
    },
    /// Inference completed successfully
    Success {
        /// Provider name
        provider: String,
        /// Model name
        model: String,
        /// Response content
        content: String,
        /// Token usage if available
        usage: Option<UsageInfo>,
        /// Duration of the request
        duration: std::time::Duration,
    },
    /// Inference failed
    Failed {
        /// Provider name (if available)
        provider: Option<String>,
        /// Error message
        error: String,
        /// Whether this is a configuration error
        is_config_error: bool,
    },
}

impl InferenceStatus {
    /// Get a display message for the current status
    pub fn display_message(&self) -> String {
        match self {
            InferenceStatus::NotConfigured => {
                "AI not configured. Run 'peridotcode' to set up a provider.".to_string()
            }
            InferenceStatus::Ready { provider, model } => {
                format!("Ready | {} / {}", provider, model)
            }
            InferenceStatus::InProgress { provider, model, .. } => {
                format!("Thinking... | {} / {}", provider, model)
            }
            InferenceStatus::Success {
                provider,
                model,
                usage,
                duration,
                ..
            } => {
                let mut msg = format!("Done | {} / {}", provider, model);
                if let Some(u) = usage {
                    msg.push_str(&format!(" | {} tokens", u.total_tokens));
                }
                msg.push_str(&format!(" | {:.1}s", duration.as_secs_f64()));
                msg
            }
            InferenceStatus::Failed { error, .. } => format!("Error: {}", error),
        }
    }

    /// Check if inference is currently in progress
    pub fn is_in_progress(&self) -> bool {
        matches!(self, InferenceStatus::InProgress { .. })
    }

    /// Check if inference completed successfully
    pub fn is_success(&self) -> bool {
        matches!(self, InferenceStatus::Success { .. })
    }

    /// Check if this is a configuration error
    pub fn is_config_error(&self) -> bool {
        matches!(self, InferenceStatus::Failed { is_config_error: true, .. })
    }
}

/// Token usage information
#[derive(Debug, Clone)]
pub struct UsageInfo {
    /// Prompt tokens consumed
    pub prompt_tokens: u32,
    /// Completion tokens generated
    pub completion_tokens: u32,
    /// Total tokens
    pub total_tokens: u32,
}

impl From<&peridot_model_gateway::UsageStats> for UsageInfo {
    fn from(stats: &peridot_model_gateway::UsageStats) -> Self {
        Self {
            prompt_tokens: stats.prompt_tokens,
            completion_tokens: stats.completion_tokens,
            total_tokens: stats.total_tokens,
        }
    }
}

/// Gateway client for core orchestrator
///
/// This is a thin wrapper around the model gateway that provides:
/// - Status tracking for UI display
/// - Simplified interface for the orchestrator
/// - Error translation to PeridotError
#[derive(Debug)]
pub struct GatewayClient {
    /// The underlying provider client
    provider: Option<Box<dyn Provider>>,
    /// Current configuration status
    config_status: ConfigStatus,
    /// Provider ID
    _provider_id: Option<ProviderId>,
    /// Model ID
    model_id: Option<String>,
}

impl Clone for GatewayClient {
    fn clone(&self) -> Self {
        Self {
            provider: None,
            config_status: self.config_status.clone(),
            _provider_id: self._provider_id.clone(),
            model_id: self.model_id.clone(),
        }
    }
}

impl GatewayClient {
    /// Create a new gateway client
    pub fn new() -> Self {
        Self {
            provider: None,
            config_status: ConfigStatus {
                has_provider: false,
                provider_ready: false,
                has_model: false,
                provider_name: None,
                model_name: None,
            },
            _provider_id: None,
            model_id: None,
        }
    }

    /// Initialize from a ConfigManager
    ///
    /// This attempts to create a provider client from the configuration.
    /// If configuration is incomplete, the client will be in a non-ready state
    /// but can still be used (operations will return appropriate errors).
    pub async fn from_config_manager(config_manager: &ConfigManager) -> Self {
        let config_status = config_manager.config_status();

        if !config_status.is_ready() {
            return Self {
                provider: None,
                config_status,
                _provider_id: None,
                model_id: None,
            };
        }

        let provider_id = config_manager
            .config()
            .default_provider
            .clone()
            .expect("checked above");

        let model_id = config_manager
            .config()
            .default_model
            .as_ref()
            .map(|m| m.to_string());

        // Try to create provider client
        let provider = Self::create_provider(config_manager, &provider_id).await;

        Self {
            provider,
            config_status,
            _provider_id: Some(provider_id),
            model_id,
        }
    }

    /// Create the appropriate provider client
    ///
    /// Supports OpenRouter, OpenAI, and Anthropic providers.
    /// OpenRouter is the primary supported provider for MVP.
    async fn create_provider(
        config_manager: &ConfigManager,
        provider_id: &ProviderId,
    ) -> Option<Box<dyn Provider>> {
        match provider_id.as_str() {
            "openrouter" => {
                match peridot_model_gateway::create_openrouter_client(config_manager).await {
                    Ok(client) => Some(Box::new(client)),
                    Err(e) => {
                        tracing::warn!("Failed to create OpenRouter client: {}", e);
                        None
                    }
                }
            }
            "openai" => {
                match peridot_model_gateway::create_openai_client(config_manager).await {
                    Ok(client) => {
                        tracing::info!("OpenAI client created successfully");
                        Some(Box::new(client))
                    }
                    Err(e) => {
                        tracing::warn!("Failed to create OpenAI client: {}", e);
                        None
                    }
                }
            }
            "anthropic" => {
                match peridot_model_gateway::create_anthropic_client(config_manager).await {
                    Ok(client) => {
                        tracing::info!("Anthropic client created successfully");
                        Some(Box::new(client))
                    }
                    Err(e) => {
                        tracing::warn!("Failed to create Anthropic client: {}", e);
                        None
                    }
                }
            }
            "groq" => {
                match peridot_model_gateway::create_groq_client(config_manager).await {
                    Ok(client) => {
                        tracing::info!("Groq client created successfully");
                        Some(Box::new(client))
                    }
                    Err(e) => {
                        tracing::warn!("Failed to create Groq client: {}", e);
                        None
                    }
                }
            }
            _ => {
                tracing::warn!("Unsupported provider: {}", provider_id);
                None
            }
        }
    }

    /// Get the current status
    pub fn status(&self) -> InferenceStatus {
        if self.provider.is_none() {
            if !self.config_status.has_provider {
                return InferenceStatus::NotConfigured;
            }
            if !self.config_status.provider_ready {
                return InferenceStatus::Failed {
                    provider: self.config_status.provider_name.clone(),
                    error: "Provider not ready. Check API key configuration.".to_string(),
                    is_config_error: true,
                };
            }
            if !self.config_status.has_model {
                return InferenceStatus::Failed {
                    provider: self.config_status.provider_name.clone(),
                    error: "No model selected. Run setup to choose a model.".to_string(),
                    is_config_error: true,
                };
            }
            return InferenceStatus::Failed {
                provider: self.config_status.provider_name.clone(),
                error: "Failed to initialize provider client".to_string(),
                is_config_error: true,
            };
        }

        InferenceStatus::Ready {
            provider: self.config_status.provider_name.clone().unwrap_or_default(),
            model: self.config_status.model_name.clone().unwrap_or_default(),
        }
    }

    /// Check if the client is ready to perform inference
    pub fn is_ready(&self) -> bool {
        self.provider.is_some()
    }

    /// Perform inference with the configured provider
    ///
    /// # Arguments
    ///
    /// * `prompt` - The user prompt to send
    /// * `system_prompt` - Optional system prompt to set context
    ///
    /// # Returns
    ///
    /// Returns the inference response or an error. Also returns the final
    /// status which can be used for UI display.
    pub async fn infer(
        &self,
        prompt: impl Into<String>,
        system_prompt: Option<&str>,
    ) -> Result<(InferenceResponse, InferenceStatus), InferenceError> {
        let provider = self
            .provider
            .as_ref()
            .ok_or_else(|| InferenceError::NotConfigured {
                message: "No AI provider configured".to_string(),
            })?;

        let provider_name = self
            .config_status
            .provider_name
            .clone()
            .unwrap_or_else(|| "Unknown".to_string());
        let model_name = self
            .model_id
            .clone()
            .unwrap_or_else(|| "Unknown".to_string());

        let started_at = std::time::Instant::now();

        // Build request
        let mut messages = Vec::new();
        if let Some(system) = system_prompt {
            messages.push(Message::system(system.to_string()));
        }
        messages.push(Message::user(prompt.into()));

        let model = self.model_id.clone().ok_or_else(|| InferenceError::NoModel {
            message: "No model selected".to_string(),
        })?;

        let request = InferenceRequest::new(model).with_messages(messages);

        // Send request
        let result = provider.infer(request).await;

        let duration = started_at.elapsed();

        match result {
            Ok(response) => {
                // Record usage persistence
                if let Some(ref usage) = response.usage {
                    let mut tracker = peridot_model_gateway::UsageTracker::load_default();
                    tracker.record_usage(&model_name, usage.clone());
                    let _ = tracker.save_default();
                }

                let usage = response.usage.as_ref().map(UsageInfo::from);
                let status = InferenceStatus::Success {
                    provider: provider_name,
                    model: model_name,
                    content: response.content().to_string(),
                    usage,
                    duration,
                };
                Ok((response, status))
            }
            Err(e) => {
                let (error_msg, is_config) = Self::classify_error(&e);
                let status = InferenceStatus::Failed {
                    provider: Some(provider_name),
                    error: error_msg.clone(),
                    is_config_error: is_config,
                };
                Err(InferenceError::Provider {
                    message: error_msg,
                    status,
                })
            }
        }
    }

    /// Classify a gateway error into user-friendly message and config flag
    fn classify_error(error: &GatewayError) -> (String, bool) {
        match error {
            GatewayError::CredentialError(_) => (
                "API key invalid or missing. Check your configuration.".to_string(),
                true,
            ),
            GatewayError::ProviderNotAvailable(_) => (
                "Provider not available. Please try again later.".to_string(),
                false,
            ),
            GatewayError::ProviderError { message, .. } => {
                let mut display_msg = format!("Provider error: {}", message);
                
                // Add specific guidance for common errors like 404 (No endpoints found)
                if message.contains("404") || message.to_lowercase().contains("not found") {
                    display_msg.push_str("\n\nGuidance: This usually means the model ID is incorrect for your chosen provider. \
                        If you recently switched providers, try re-selecting your model in the settings (/models).");
                }
                
                (display_msg, false)
            }
            GatewayError::InferenceError(msg) => (msg.clone(), false),
            GatewayError::ValidationError(msg) => (format!("Invalid request: {}", msg), false),
            _ => (error.to_string(), false),
        }
    }

    /// Get provider name for display
    pub fn provider_name(&self) -> Option<&str> {
        self.config_status.provider_name.as_deref()
    }

    /// Get model name for display
    pub fn model_name(&self) -> Option<&str> {
        self.config_status.model_name.as_deref()
    }

    /// Perform a network-based validation of the current credentials
    pub async fn validate_network(&self) -> Result<(), String> {
        let provider = self.provider.as_ref().ok_or_else(|| {
            "No AI provider initialized. Please check your configuration.".to_string()
        })?;

        provider.validate_credentials().await.map_err(|e| e.to_string())
    }

    /// Get the provider client for direct inference
    ///
    /// This allows external components to perform inference directly
    /// using the configured provider.
    pub fn provider(&self) -> Option<&Box<dyn Provider>> {
        self.provider.as_ref()
    }
}

impl Default for GatewayClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur during inference
#[derive(Debug, Clone)]
pub enum InferenceError {
    /// Not configured
    NotConfigured {
        /// Error message
        message: String,
    },
    /// No model selected
    NoModel {
        /// Error message
        message: String,
    },
    /// Provider error
    Provider {
        /// Error message
        message: String,
        /// Status with error details
        status: InferenceStatus,
    },
}

impl std::fmt::Display for InferenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InferenceError::NotConfigured { message } => write!(f, "{}", message),
            InferenceError::NoModel { message } => write!(f, "{}", message),
            InferenceError::Provider { message, .. } => write!(f, "{}", message),
        }
    }
}

impl std::error::Error for InferenceError {}

impl From<InferenceError> for PeridotError {
    fn from(err: InferenceError) -> Self {
        PeridotError::General(err.to_string())
    }
}

/// Example: Complete inference flow
///
/// This demonstrates how a user prompt flows through the system
/// from the orchestrator to the model gateway and back.
///
/// # Example
///
/// ```rust,ignore
/// use peridot_core::gateway_integration::{example_inference_flow, GatewayClient};
/// use peridot_model_gateway::ConfigManager;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Load configuration
///     let config_manager = ConfigManager::initialize()?;
///     
///     // Create gateway client
///     let client = GatewayClient::from_config_manager(&config_manager).await;
///     
///     // Run example flow
///     example_inference_flow(&client, "Create a 2D platformer game").await?;
///     
///     Ok(())
/// }
/// ```
pub async fn example_inference_flow(
    client: &GatewayClient,
    user_prompt: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Inference Flow Example ===\n");

    // Step 1: Check configuration status
    let status = client.status();
    println!("1. Configuration Status:");
    println!("   {}", status.display_message());

    if !client.is_ready() {
        println!("\n   Cannot proceed - provider not configured.");
        println!("   Run 'peridotcode' to set up your AI provider.");
        return Ok(());
    }

    // Step 2: Prepare the request
    println!("\n2. User Prompt:");
    println!("   \"{}\"", user_prompt);

    let system_prompt = "You are a helpful game development assistant. \
        Provide concise, actionable advice for creating game projects.";
    println!("\n3. System Context:");
    println!("   \"{}\"", system_prompt);

    // Step 3: Send request
    println!("\n4. Sending request to model...");

    match client.infer(user_prompt, Some(system_prompt)).await {
        Ok((response, status)) => {
            // Step 4: Display response
            println!("\n5. Response:");
            println!("   Content: {}", response.content());

            if let InferenceStatus::Success {
                usage, duration, ..
            } = status
            {
                if let Some(u) = usage {
                    println!("\n6. Usage Statistics:");
                    println!("   Prompt tokens: {}", u.prompt_tokens);
                    println!("   Completion tokens: {}", u.completion_tokens);
                    println!("   Total tokens: {}", u.total_tokens);
                }
                println!("\n7. Request Duration: {:.2}s", duration.as_secs_f64());
            }

            println!("\n=== Flow Complete ===");
        }
        Err(e) => {
            println!("\n5. Error:");
            println!("   {}", e);
            println!("\n=== Flow Failed ===");
        }
    }

    Ok(())
}

/// Example: AI-powered intent classification
///
/// This shows how the orchestrator can use AI to enhance intent classification
/// without being tightly coupled to any specific provider.
pub async fn example_ai_intent_classification(
    client: &GatewayClient,
    user_prompt: &str,
) -> Result<String, InferenceError> {
    let system_prompt = r#"You are an intent classifier for a game development tool.
Classify the user's request into exactly one of these categories:
- "create_game" - User wants to create a new game
- "add_feature" - User wants to add a feature to existing game
- "modify" - User wants to modify existing code
- "unknown" - Cannot determine intent

Respond with ONLY the category name."#;

    let (response, _) = client.infer(user_prompt, Some(system_prompt)).await?;
    Ok(response.content().trim().to_lowercase())
}

/// Example: Generate scaffold specification
///
/// This shows how AI can enhance template generation by providing
/// more detailed specifications based on user prompts.
pub async fn example_enhance_scaffold(
    client: &GatewayClient,
    user_prompt: &str,
) -> Result<String, InferenceError> {
    let system_prompt = r#"You are a game scaffold designer. Given a game description,
create a concise specification including:
1. Genre
2. Core mechanics
3. Key files needed
4. Recommended structure

Keep your response under 200 words."#;

    let (response, _) = client.infer(user_prompt, Some(system_prompt)).await?;
    Ok(response.content().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inference_status_display() {
        let ready = InferenceStatus::Ready {
            provider: "openrouter".to_string(),
            model: "claude-3.5-sonnet".to_string(),
        };
        assert!(ready.display_message().contains("openrouter"));
        assert!(ready.display_message().contains("claude-3.5-sonnet"));

        let not_config = InferenceStatus::NotConfigured;
        assert!(not_config.display_message().contains("not configured"));
    }

    #[test]
    fn test_inference_status_checks() {
        let success = InferenceStatus::Success {
            provider: "test".to_string(),
            model: "test".to_string(),
            content: "test".to_string(),
            usage: None,
            duration: std::time::Duration::from_secs(1),
        };
        assert!(success.is_success());
        assert!(!success.is_in_progress());

        let in_progress = InferenceStatus::InProgress {
            provider: "test".to_string(),
            model: "test".to_string(),
            started_at: std::time::Instant::now(),
        };
        assert!(in_progress.is_in_progress());
        assert!(!in_progress.is_success());
    }
}
