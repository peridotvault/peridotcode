//! Anthropic provider adapter
//!
//! Implements the `Provider` trait for Anthropic Claude (https://anthropic.com).
//!
//! # Design Notes
//!
//! - Uses the Anthropic Messages API (v1)
//! - Model IDs are in format "claude-3-*" (e.g., "claude-3-opus-20240229")
//! - Requires "anthropic-version" header
//! - Minimal implementation focused on core functionality
//!
//! # Limitations
//!
//! This is a minimal implementation compared to OpenRouter:
//! - No streaming support (MVP scope)
//! - No tool use / function calling
//! - No vision support (even though some models support it)
//! - Static model list (no dynamic fetching)
//! - Basic error handling
//!
//! # Example
//!
//! ```rust,ignore
//! use peridot_model_gateway::{AnthropicClient, ProviderConfig, ProviderId};
//!
//! let client = AnthropicClient::new("sk-ant-...")?;
//! ```

use crate::{
    GatewayError, GatewayResult, InferenceRequest, InferenceResponse, Message, Provider,
    ProviderId, Role, UsageStats,
};
use crate::provider::ModelInfo;
use reqwest::header::{self, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Anthropic API client
///
/// This is a concrete implementation of the Provider trait for Anthropic Claude.
/// It provides basic message completion functionality.
#[derive(Debug, Clone)]
pub struct AnthropicClient {
    /// HTTP client for making requests
    http_client: reqwest::Client,
    /// API key (already resolved from credential reference)
    api_key: String,
    /// Base URL for API requests
    base_url: String,
    /// Default model to use if none specified
    default_model: Option<String>,
    /// Request timeout
    timeout: Duration,
    /// API version
    _api_version: String,
}

impl AnthropicClient {
    /// Anthropic's default base URL
    pub const DEFAULT_BASE_URL: &'static str = "https://api.anthropic.com/v1";

    /// Default model for Anthropic
    pub const DEFAULT_MODEL: &'static str = "claude-3-sonnet-20240229";

    /// API version
    pub const API_VERSION: &'static str = "2023-06-01";

    /// Create a new Anthropic client
    ///
    /// # Arguments
    ///
    /// * `api_key` - The resolved API key (not a reference)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = AnthropicClient::new("sk-ant-...")?;
    /// ```
    pub fn new(api_key: impl Into<String>) -> GatewayResult<Self> {
        Self::with_config(api_key, None, None)
    }

    /// Create a new Anthropic client with configuration
    ///
    /// # Arguments
    ///
    /// * `api_key` - The resolved API key
    /// * `base_url` - Optional base URL override
    /// * `default_model` - Optional default model ID
    pub fn with_config(
        api_key: impl Into<String>,
        base_url: Option<String>,
        default_model: Option<String>,
    ) -> GatewayResult<Self> {
        let api_key = api_key.into();
        let base_url = base_url.unwrap_or_else(|| Self::DEFAULT_BASE_URL.to_string());

        // Validate API key format (basic check)
        if api_key.is_empty() {
            return Err(GatewayError::CredentialError(
                "Anthropic API key is empty".to_string(),
            ));
        }

        // Build HTTP client with appropriate headers
        let http_client = Self::build_http_client(&api_key)?;

        Ok(Self {
            http_client,
            api_key,
            base_url,
            default_model,
            timeout: Duration::from_secs(60),
            _api_version: Self::API_VERSION.to_string(),
        })
    }

    /// Set request timeout
    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout = Duration::from_secs(seconds);
        self
    }

    /// Build the HTTP client with required headers
    fn build_http_client(api_key: &str) -> GatewayResult<reqwest::Client> {
        let mut headers = HeaderMap::new();

        // Authorization header (x-api-key for Anthropic)
        headers.insert(
            "x-api-key",
            HeaderValue::from_str(api_key).map_err(|e| {
                GatewayError::ConfigError(format!("Invalid API key format: {}", e))
            })?,
        );

        // Content-Type
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

        // Anthropic version header (required)
        headers.insert(
            "anthropic-version",
            HeaderValue::from_static(Self::API_VERSION),
        );

        // Build client
        reqwest::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(60))
            .build()
            .map_err(|e| {
                GatewayError::ProviderError {
                    provider: "anthropic".to_string(),
                    message: format!("Failed to build HTTP client: {}", e),
                }
            })
    }

    /// Get the messages endpoint URL
    fn messages_endpoint(&self) -> String {
        format!("{}/messages", self.base_url)
    }

    /// Transform internal request to Anthropic format
    fn transform_request(&self, request: InferenceRequest) -> AnthropicRequest {
        let mut model = if request.model.is_empty() {
            self.default_model
                .clone()
                .unwrap_or_else(|| Self::DEFAULT_MODEL.to_string())
        } else {
            request.model
        };

        // Normalize model ID: strip "anthropic/" prefix if present
        // This handles cases where the user is switching from OpenRouter to direct Anthropic
        if model.starts_with("anthropic/") {
            model = model.replacen("anthropic/", "", 1);
        }

        // Normalize dots to dashes for Claude 3.5 Sonnet if needed
        if model == "claude-3.5-sonnet" {
            model = "claude-3-5-sonnet-20240620".to_string();
        }

        // Separate system message from other messages
        let mut system: Option<String> = None;
        let mut messages: Vec<AnthropicMessage> = Vec::new();

        for msg in request.messages {
            match msg.role {
                Role::System => {
                    system = Some(msg.content);
                }
                _ => {
                    messages.push(msg.into());
                }
            }
        }

        AnthropicRequest {
            model,
            messages,
            system,
            max_tokens: request.max_tokens.unwrap_or(4096),
            temperature: request.temperature,
            stream: false, // MVP: non-streaming only
        }
    }

    /// Transform Anthropic response to internal format
    fn transform_response(&self, response: AnthropicResponse) -> GatewayResult<InferenceResponse> {
        // Anthropic returns content as a list of blocks, we take the first text block
        let content = response
            .content
            .into_iter()
            .find(|c| c.content_type == "text")
            .map(|c| c.text)
            .unwrap_or_default();

        let message = Message::assistant(content);

        let usage = response.usage.map(|u| UsageStats {
            prompt_tokens: u.input_tokens,
            completion_tokens: u.output_tokens,
            total_tokens: u.input_tokens + u.output_tokens,
        });

        Ok(InferenceResponse {
            message,
            model: response.model,
            provider: "anthropic".to_string(),
            usage,
            finish_reason: response.stop_reason,
        })
    }
}

#[async_trait::async_trait]
impl Provider for AnthropicClient {
    fn id(&self) -> ProviderId {
        ProviderId::anthropic()
    }

    fn name(&self) -> &str {
        "Anthropic"
    }

    fn is_configured(&self) -> bool {
        !self.api_key.is_empty()
    }

    async fn validate_credentials(&self) -> GatewayResult<()> {
        if self.api_key.is_empty() {
            return Err(crate::GatewayError::CredentialError("Anthropic API key is empty".to_string()));
        }
        // Minimal validation - actual network check TODO
        Ok(())
    }

    async fn infer(&self, request: InferenceRequest) -> GatewayResult<InferenceResponse> {
        // Validate request
        if request.messages.is_empty() {
            return Err(GatewayError::ValidationError(
                "Request must have at least one message".to_string(),
            ));
        }

        // Check that we have at least one non-system message
        let has_user_message = request.messages.iter().any(|m| m.role != Role::System);
        if !has_user_message {
            return Err(GatewayError::ValidationError(
                "Anthropic requires at least one user or assistant message".to_string(),
            ));
        }

        let url = self.messages_endpoint();
        let anthropic_request = self.transform_request(request);

        tracing::debug!(
            "Sending request to Anthropic: model={}",
            anthropic_request.model
        );

        let response = self
            .http_client
            .post(&url)
            .json(&anthropic_request)
            .send()
            .await
            .map_err(|e| GatewayError::ProviderError {
                provider: "anthropic".to_string(),
                message: format!("Request failed: {}", e),
            })?;

        // Handle HTTP errors
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            // Try to parse error details
            if let Ok(error_body) = serde_json::from_str::<AnthropicErrorResponse>(&error_text) {
                return Err(GatewayError::ProviderError {
                    provider: "anthropic".to_string(),
                    message: format!("{}: {}", error_body.error.error_type, error_body.error.message),
                });
            }

            return Err(GatewayError::ProviderError {
                provider: "anthropic".to_string(),
                message: format!("HTTP {}: {}", status, error_text),
            });
        }

        // Parse successful response
        let anthropic_response: AnthropicResponse = response.json().await.map_err(|e| {
            GatewayError::ProviderError {
                provider: "anthropic".to_string(),
                message: format!("Failed to parse response: {}", e),
            }
        })?;

        tracing::debug!(
            "Received response from Anthropic: model={}",
            anthropic_response.model
        );

        self.transform_response(anthropic_response)
    }

    async fn list_models(&self) -> GatewayResult<Vec<ModelInfo>> {
        // Return static list - Anthropic has a models API but
        // using static list for simplicity and reliability
        Ok(static_model_list())
    }
}

/// Anthropic API request format
#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    stream: bool,
}

/// Anthropic message format
#[derive(Debug, Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

impl From<Message> for AnthropicMessage {
    fn from(msg: Message) -> Self {
        // Anthropic only supports "user" and "assistant" roles
        // System messages are handled separately in the request
        Self {
            role: match msg.role {
                Role::Assistant => "assistant".to_string(),
                _ => "user".to_string(), // Both User and System map to user (system handled separately)
            },
            content: msg.content,
        }
    }
}

/// Anthropic content block
#[derive(Debug, Deserialize)]
struct AnthropicContentBlock {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

/// Anthropic API response format
#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    _id: String,
    model: String,
    content: Vec<AnthropicContentBlock>,
    #[serde(rename = "stop_reason")]
    stop_reason: Option<String>,
    usage: Option<AnthropicUsage>,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    #[serde(rename = "input_tokens")]
    input_tokens: u32,
    #[serde(rename = "output_tokens")]
    output_tokens: u32,
}

/// Anthropic error response format
#[derive(Debug, Deserialize)]
struct AnthropicErrorResponse {
    error: AnthropicErrorDetail,
    #[serde(rename = "type")]
    _error_type: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicErrorDetail {
    #[serde(rename = "type")]
    error_type: String,
    message: String,
}

/// Get static model list for Anthropic
///
/// These are the recommended models for game scaffolding.
fn static_model_list() -> Vec<ModelInfo> {
    vec![
        ModelInfo {
            id: "claude-3-opus-20240229".to_string(),
            name: "Claude 3 Opus".to_string(),
            provider: ProviderId::anthropic(),
            context_window: Some(200_000),
            recommended: true,
        },
        ModelInfo {
            id: "claude-3-sonnet-20240229".to_string(),
            name: "Claude 3 Sonnet".to_string(),
            provider: ProviderId::anthropic(),
            context_window: Some(200_000),
            recommended: true,
        },
        ModelInfo {
            id: "claude-3-haiku-20240307".to_string(),
            name: "Claude 3 Haiku".to_string(),
            provider: ProviderId::anthropic(),
            context_window: Some(200_000),
            recommended: true,
        },
    ]
}

/// Create an Anthropic client from a ConfigManager
///
/// This is a convenience function that resolves credentials and creates the client.
pub async fn create_anthropic_client(
    config_manager: &crate::ConfigManager,
) -> GatewayResult<AnthropicClient> {
    let provider_id = ProviderId::anthropic();

    // Get provider configuration
    let provider_config = config_manager
        .config()
        .get_provider(&provider_id)
        .ok_or_else(|| {
            GatewayError::ConfigError("Anthropic provider not configured".to_string())
        })?;

    // Resolve API key
    let api_key = config_manager
        .resolve_credentials(&provider_id)?
        .ok_or_else(|| {
            GatewayError::CredentialError("Anthropic API key not configured".to_string())
        })?;

    // Create client
    let client = AnthropicClient::with_config(
        api_key,
        provider_config.base_url.clone(),
        provider_config.default_model.clone(),
    )?;

    Ok(client.with_timeout(provider_config.timeout_seconds))
}
