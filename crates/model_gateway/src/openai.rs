//! OpenAI provider adapter
//!
//! Implements the `Provider` trait for OpenAI (https://openai.com).
//!
//! # Design Notes
//!
//! - Uses the OpenAI Chat Completions API
//! - Model IDs are simple strings (e.g., "gpt-4o", "gpt-4o-mini")
//! - Minimal implementation focused on core functionality
//!
//! # Limitations
//!
//! This is a minimal implementation compared to OpenRouter:
//! - No streaming support (MVP scope)
//! - No advanced features like function calling, JSON mode
//! - Static model list (no dynamic fetching)
//! - Basic error handling
//!
//! # Example
//!
//! ```rust,ignore
//! use peridot_model_gateway::{OpenAIClient, ProviderConfig, ProviderId};
//!
//! let client = OpenAIClient::new("sk-...")?;
//! ```

use crate::{
    GatewayError, GatewayResult, InferenceRequest, InferenceResponse, Message, Provider,
    ProviderId, Role, UsageStats,
};
use crate::provider::ModelInfo;
use reqwest::header::{self, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// OpenAI API client
///
/// This is a concrete implementation of the Provider trait for OpenAI.
/// It provides basic chat completion functionality.
#[derive(Debug, Clone)]
pub struct OpenAIClient {
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
}

impl OpenAIClient {
    /// OpenAI's default base URL
    pub const DEFAULT_BASE_URL: &'static str = "https://api.openai.com/v1";

    /// Default model for OpenAI
    pub const DEFAULT_MODEL: &'static str = "gpt-4o-mini";

    /// Create a new OpenAI client
    ///
    /// # Arguments
    ///
    /// * `api_key` - The resolved API key (not a reference)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = OpenAIClient::new("sk-...")?;
    /// ```
    pub fn new(api_key: impl Into<String>) -> GatewayResult<Self> {
        Self::with_config(api_key, None, None)
    }

    /// Create a new OpenAI client with configuration
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
                "OpenAI API key is empty".to_string(),
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

        // Authorization header
        let auth_value = format!("Bearer {}", api_key);
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_str(&auth_value).map_err(|e| {
                GatewayError::ConfigError(format!("Invalid API key format: {}", e))
            })?,
        );

        // Content-Type
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

        // OpenAI organization header (optional, not used in MVP)
        // headers.insert("OpenAI-Organization", ...);

        // Build client
        reqwest::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(60))
            .build()
            .map_err(|e| {
                GatewayError::ProviderError {
                    provider: "openai".to_string(),
                    message: format!("Failed to build HTTP client: {}", e),
                }
            })
    }

    /// Get the chat completions endpoint URL
    fn chat_endpoint(&self) -> String {
        format!("{}/chat/completions", self.base_url)
    }

    /// Transform internal request to OpenAI format
    fn transform_request(&self, request: InferenceRequest) -> OpenAIRequest {
        let model = if request.model.is_empty() {
            self.default_model
                .clone()
                .unwrap_or_else(|| Self::DEFAULT_MODEL.to_string())
        } else {
            request.model
        };

        OpenAIRequest {
            model,
            messages: request.messages.into_iter().map(|m| m.into()).collect(),
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            stream: Some(false), // MVP: non-streaming only
        }
    }

    /// Transform OpenAI response to internal format
    fn transform_response(&self, response: OpenAIResponse) -> GatewayResult<InferenceResponse> {
        let choice = response.choices.into_iter().next().ok_or_else(|| {
            GatewayError::ProviderError {
                provider: "openai".to_string(),
                message: "No choices in response".to_string(),
            }
        })?;

        // Convert OpenAI message to internal Message
        let message: Message = choice.message.into();

        let usage = response.usage.map(|u| UsageStats {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
        });

        Ok(InferenceResponse {
            message,
            model: response.model,
            provider: "openai".to_string(),
            usage,
            finish_reason: choice.finish_reason,
        })
    }
}

#[async_trait::async_trait]
impl Provider for OpenAIClient {
    fn id(&self) -> ProviderId {
        ProviderId::openai()
    }

    fn name(&self) -> &str {
        "OpenAI"
    }

    fn is_configured(&self) -> bool {
        !self.api_key.is_empty()
    }

    async fn infer(&self, request: InferenceRequest) -> GatewayResult<InferenceResponse> {
        // Validate request
        if request.messages.is_empty() {
            return Err(GatewayError::ValidationError(
                "Request must have at least one message".to_string(),
            ));
        }

        let url = self.chat_endpoint();
        let openai_request = self.transform_request(request);

        tracing::debug!(
            "Sending request to OpenAI: model={}",
            openai_request.model
        );

        let response = self
            .http_client
            .post(&url)
            .json(&openai_request)
            .send()
            .await
            .map_err(|e| GatewayError::ProviderError {
                provider: "openai".to_string(),
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
            if let Ok(error_body) = serde_json::from_str::<OpenAIErrorResponse>(&error_text) {
                return Err(GatewayError::ProviderError {
                    provider: "openai".to_string(),
                    message: format!("{}: {}", error_body.error.code, error_body.error.message),
                });
            }

            return Err(GatewayError::ProviderError {
                provider: "openai".to_string(),
                message: format!("HTTP {}: {}", status, error_text),
            });
        }

        // Parse successful response
        let openai_response: OpenAIResponse = response.json().await.map_err(|e| {
            GatewayError::ProviderError {
                provider: "openai".to_string(),
                message: format!("Failed to parse response: {}", e),
            }
        })?;

        tracing::debug!(
            "Received response from OpenAI: model={}",
            openai_response.model
        );

        self.transform_response(openai_response)
    }

    async fn list_models(&self) -> GatewayResult<Vec<ModelInfo>> {
        // Return static list - OpenAI model list API is available but
        // requires additional permissions for some accounts
        // Using static list for simplicity and reliability
        Ok(static_model_list())
    }
}

/// OpenAI API request format
#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

/// OpenAI message format
#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

impl From<Message> for OpenAIMessage {
    fn from(msg: Message) -> Self {
        Self {
            role: match msg.role {
                Role::System => "system".to_string(),
                Role::User => "user".to_string(),
                Role::Assistant => "assistant".to_string(),
            },
            content: msg.content,
        }
    }
}

impl From<OpenAIMessage> for Message {
    fn from(msg: OpenAIMessage) -> Self {
        let role = match msg.role.as_str() {
            "system" => Role::System,
            "assistant" => Role::Assistant,
            _ => Role::User,
        };
        Self::new(role, msg.content)
    }
}

/// OpenAI API response format
#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    id: String,
    model: String,
    choices: Vec<OpenAIChoice>,
    usage: Option<OpenAIUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
    finish_reason: Option<String>,
    index: u32,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

/// OpenAI error response format
#[derive(Debug, Deserialize)]
struct OpenAIErrorResponse {
    error: OpenAIErrorDetail,
}

#[derive(Debug, Deserialize)]
struct OpenAIErrorDetail {
    code: String,
    message: String,
    #[serde(rename = "type")]
    error_type: Option<String>,
}

/// Get static model list for OpenAI
///
/// These are the recommended models for game scaffolding.
fn static_model_list() -> Vec<ModelInfo> {
    vec![
        ModelInfo {
            id: "gpt-4o".to_string(),
            name: "GPT-4o".to_string(),
            provider: ProviderId::openai(),
            context_window: Some(128_000),
            recommended: true,
        },
        ModelInfo {
            id: "gpt-4o-mini".to_string(),
            name: "GPT-4o Mini".to_string(),
            provider: ProviderId::openai(),
            context_window: Some(128_000),
            recommended: true,
        },
        ModelInfo {
            id: "gpt-4-turbo".to_string(),
            name: "GPT-4 Turbo".to_string(),
            provider: ProviderId::openai(),
            context_window: Some(128_000),
            recommended: false,
        },
        ModelInfo {
            id: "gpt-3.5-turbo".to_string(),
            name: "GPT-3.5 Turbo".to_string(),
            provider: ProviderId::openai(),
            context_window: Some(16_385),
            recommended: false,
        },
    ]
}

/// Create an OpenAI client from a ConfigManager
///
/// This is a convenience function that resolves credentials and creates the client.
pub async fn create_openai_client(
    config_manager: &crate::ConfigManager,
) -> GatewayResult<OpenAIClient> {
    let provider_id = ProviderId::openai();

    // Get provider configuration
    let provider_config = config_manager
        .config()
        .get_provider(&provider_id)
        .ok_or_else(|| {
            GatewayError::ConfigError("OpenAI provider not configured".to_string())
        })?;

    // Resolve API key
    let api_key = config_manager
        .resolve_credentials(&provider_id)?
        .ok_or_else(|| {
            GatewayError::CredentialError("OpenAI API key not configured".to_string())
        })?;

    // Create client
    let client = OpenAIClient::with_config(
        api_key,
        provider_config.base_url.clone(),
        provider_config.default_model.clone(),
    )?;

    Ok(client.with_timeout(provider_config.timeout_seconds))
}
