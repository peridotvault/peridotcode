//! OpenRouter provider adapter
//!
//! Implements the `Provider` trait for OpenRouter (https://openrouter.ai).
//!
//! # Design Notes
//!
//! - OpenRouter uses an OpenAI-compatible API format
//! - Model IDs are in format "provider/model" (e.g., "anthropic/claude-3.5-sonnet")
//! - Supports streaming (not implemented in MVP)
//! - Requires HTTP Referer and X-Title headers for ranking
//!
//! # Example
//!
//! ```rust,ignore
//! use peridot_model_gateway::{OpenRouterClient, ProviderConfig, ProviderId};
//!
//! let config = ProviderConfig::openrouter("env:OPENROUTER_API_KEY");
//! let client = OpenRouterClient::new(config)?;
//! ```

use crate::{
    GatewayError, GatewayResult, InferenceRequest, InferenceResponse, Message, Provider,
    ProviderId, Role, UsageStats,
};
use crate::provider::ModelInfo;
use reqwest::header::{self, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// OpenRouter API client
///
/// This is the concrete implementation of the Provider trait for OpenRouter.
/// It handles authentication, request formatting, and response normalization.
#[derive(Debug, Clone)]
pub struct OpenRouterClient {
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
    /// HTTP referer for OpenRouter ranking
    http_referer: Option<String>,
    /// App title for OpenRouter ranking
    app_title: Option<String>,
}

impl OpenRouterClient {
    /// OpenRouter's default base URL
    pub const DEFAULT_BASE_URL: &'static str = "https://openrouter.ai/api/v1";

    /// Create a new OpenRouter client
    ///
    /// # Arguments
    ///
    /// * `api_key` - The resolved API key (not a reference)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = OpenRouterClient::new("sk-or-v1-xxx")?;
    /// ```
    pub fn new(api_key: impl Into<String>) -> GatewayResult<Self> {
        Self::with_config(api_key, None, None)
    }

    /// Create a new OpenRouter client with configuration
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
                "OpenRouter API key is empty".to_string(),
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
            http_referer: None,
            app_title: Some("PeridotCode".to_string()),
        })
    }

    /// Set the HTTP referer for OpenRouter ranking
    pub fn with_http_referer(mut self, referer: impl Into<String>) -> Self {
        self.http_referer = Some(referer.into());
        self
    }

    /// Set the app title for OpenRouter ranking
    pub fn with_app_title(mut self, title: impl Into<String>) -> Self {
        self.app_title = Some(title.into());
        self
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

        // Build client
        reqwest::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(60))
            .build()
            .map_err(|e| {
                GatewayError::ProviderError {
                    provider: "openrouter".to_string(),
                    message: format!("Failed to build HTTP client: {}", e),
                }
            })
    }

    /// Get the chat completions endpoint URL
    fn chat_endpoint(&self) -> String {
        format!("{}/chat/completions", self.base_url)
    }

    /// Get the models endpoint URL
    fn models_endpoint(&self) -> String {
        format!("{}/models", self.base_url)
    }

    /// Build request headers including OpenRouter-specific headers
    fn build_request_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();

        // OpenRouter ranking headers
        if let Some(ref referer) = self.http_referer {
            if let Ok(value) = HeaderValue::from_str(referer) {
                headers.insert("HTTP-Referer", value);
            }
        }

        if let Some(ref title) = self.app_title {
            if let Ok(value) = HeaderValue::from_str(title) {
                headers.insert("X-Title", value);
            }
        }

        headers
    }

    /// Transform internal request to OpenRouter format
    fn transform_request(&self, request: InferenceRequest) -> OpenRouterRequest {
        let model = if request.model.is_empty() {
            self.default_model.clone().unwrap_or_default()
        } else {
            request.model
        };

        OpenRouterRequest {
            model,
            messages: request.messages.into_iter().map(|m| m.into()).collect(),
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            stream: Some(false), // MVP: non-streaming only
        }
    }

    /// Transform OpenRouter response to internal format
    fn transform_response(&self, response: OpenRouterResponse) -> GatewayResult<InferenceResponse> {
        let choice = response.choices.into_iter().next().ok_or_else(|| {
            GatewayError::ProviderError {
                provider: "openrouter".to_string(),
                message: "No choices in response".to_string(),
            }
        })?;

        // Convert OpenRouter message to internal Message
        let message: Message = choice.message.into();

        let usage = response.usage.map(|u| UsageStats {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
        });

        Ok(InferenceResponse {
            message,
            model: response.model,
            provider: "openrouter".to_string(),
            usage,
            finish_reason: choice.finish_reason,
        })
    }

    /// Fetch available models from OpenRouter API
    async fn fetch_models(&self) -> GatewayResult<Vec<OpenRouterModel>> {
        let url = self.models_endpoint();

        let response = self
            .http_client
            .get(&url)
            .headers(self.build_request_headers())
            .send()
            .await
            .map_err(|e| GatewayError::ProviderError {
                provider: "openrouter".to_string(),
                message: format!("Failed to fetch models: {}", e),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(GatewayError::ProviderError {
                provider: "openrouter".to_string(),
                message: format!("API error {}: {}", status, text),
            });
        }

        let data: OpenRouterModelsResponse = response.json().await.map_err(|e| {
            GatewayError::ProviderError {
                provider: "openrouter".to_string(),
                message: format!("Failed to parse models response: {}", e),
            }
        })?;

        Ok(data.data)
    }
}

#[async_trait::async_trait]
impl Provider for OpenRouterClient {
    fn id(&self) -> ProviderId {
        ProviderId::openrouter()
    }

    fn name(&self) -> &str {
        "OpenRouter"
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
        let openrouter_request = self.transform_request(request);

        tracing::debug!(
            "Sending request to OpenRouter: model={}",
            openrouter_request.model
        );

        let response = self
            .http_client
            .post(&url)
            .headers(self.build_request_headers())
            .json(&openrouter_request)
            .send()
            .await
            .map_err(|e| GatewayError::ProviderError {
                provider: "openrouter".to_string(),
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
            if let Ok(error_body) =
                serde_json::from_str::<OpenRouterErrorResponse>(&error_text)
            {
                return Err(GatewayError::ProviderError {
                    provider: "openrouter".to_string(),
                    message: format!("{}: {}", error_body.error.code, error_body.error.message),
                });
            }

            return Err(GatewayError::ProviderError {
                provider: "openrouter".to_string(),
                message: format!("HTTP {}: {}", status, error_text),
            });
        }

        // Parse successful response
        let openrouter_response: OpenRouterResponse = response.json().await.map_err(|e| {
            GatewayError::ProviderError {
                provider: "openrouter".to_string(),
                message: format!("Failed to parse response: {}", e),
            }
        })?;

        tracing::debug!(
            "Received response from OpenRouter: model={}",
            openrouter_response.model
        );

        self.transform_response(openrouter_response)
    }

    async fn list_models(&self) -> GatewayResult<Vec<ModelInfo>> {
        // Try to fetch from API first
        match self.fetch_models().await {
            Ok(models) => {
                let model_infos: Vec<ModelInfo> = models
                    .into_iter()
                    .map(|m| ModelInfo {
                        id: m.id.clone(),
                        name: m.name.unwrap_or_else(|| m.id.clone()),
                        provider: ProviderId::openrouter(),
                        context_window: Some(m.context_length),
                        recommended: RECOMMENDED_MODELS.contains(&m.id.as_str()),
                    })
                    .collect();
                Ok(model_infos)
            }
            Err(_) => {
                // Fall back to static list if API fails
                tracing::warn!("Failed to fetch models from OpenRouter API, using static list");
                Ok(static_model_list())
            }
        }
    }
}

/// OpenRouter API request format
#[derive(Debug, Serialize)]
struct OpenRouterRequest {
    model: String,
    messages: Vec<OpenRouterMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

/// OpenRouter message format
#[derive(Debug, Serialize, Deserialize)]
struct OpenRouterMessage {
    role: String,
    content: String,
}

impl From<Message> for OpenRouterMessage {
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

impl From<OpenRouterMessage> for Message {
    fn from(msg: OpenRouterMessage) -> Self {
        let role = match msg.role.as_str() {
            "system" => Role::System,
            "assistant" => Role::Assistant,
            _ => Role::User,
        };
        Self::new(role, msg.content)
    }
}

/// OpenRouter API response format
#[derive(Debug, Deserialize)]
struct OpenRouterResponse {
    id: String,
    model: String,
    choices: Vec<OpenRouterChoice>,
    usage: Option<OpenRouterUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenRouterChoice {
    message: OpenRouterMessage,
    finish_reason: Option<String>,
    index: u32,
}

#[derive(Debug, Deserialize)]
struct OpenRouterUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

/// OpenRouter error response format
#[derive(Debug, Deserialize)]
struct OpenRouterErrorResponse {
    error: OpenRouterErrorDetail,
}

#[derive(Debug, Deserialize)]
struct OpenRouterErrorDetail {
    code: u32,
    message: String,
    #[serde(rename = "type")]
    error_type: Option<String>,
}

/// OpenRouter model information
#[derive(Debug, Deserialize)]
struct OpenRouterModel {
    id: String,
    name: Option<String>,
    context_length: usize,
    pricing: Option<OpenRouterPricing>,
}

#[derive(Debug, Deserialize)]
struct OpenRouterPricing {
    prompt: f64,
    completion: f64,
}

/// OpenRouter models list response
#[derive(Debug, Deserialize)]
struct OpenRouterModelsResponse {
    data: Vec<OpenRouterModel>,
}

/// List of recommended OpenRouter models for game scaffolding
const RECOMMENDED_MODELS: &[&str] = &[
    "anthropic/claude-3.5-sonnet",
    "anthropic/claude-3-opus",
    "openai/gpt-4o",
    "openai/gpt-4o-mini",
    "anthropic/claude-3-haiku",
    "google/gemini-flash-1.5",
];

/// Get static model list as fallback when API is unavailable
fn static_model_list() -> Vec<ModelInfo> {
    vec![
        ModelInfo {
            id: "anthropic/claude-3.5-sonnet".to_string(),
            name: "Claude 3.5 Sonnet".to_string(),
            provider: ProviderId::openrouter(),
            context_window: Some(200_000),
            recommended: true,
        },
        ModelInfo {
            id: "openai/gpt-4o-mini".to_string(),
            name: "GPT-4o Mini".to_string(),
            provider: ProviderId::openrouter(),
            context_window: Some(128_000),
            recommended: true,
        },
        ModelInfo {
            id: "anthropic/claude-3-haiku".to_string(),
            name: "Claude 3 Haiku".to_string(),
            provider: ProviderId::openrouter(),
            context_window: Some(200_000),
            recommended: true,
        },
        ModelInfo {
            id: "google/gemini-flash-1.5".to_string(),
            name: "Gemini Flash 1.5".to_string(),
            provider: ProviderId::openrouter(),
            context_window: Some(1_000_000),
            recommended: true,
        },
    ]
}

/// Create an OpenRouter client from a ConfigManager
///
/// This is a convenience function that resolves credentials and creates the client.
pub async fn create_openrouter_client(
    config_manager: &crate::ConfigManager,
) -> GatewayResult<OpenRouterClient> {
    let provider_id = ProviderId::openrouter();

    // Get provider configuration
    let provider_config = config_manager
        .config()
        .get_provider(&provider_id)
        .ok_or_else(|| GatewayError::ConfigError(
            "OpenRouter provider not configured".to_string()
        ))?;

    // Resolve API key
    let api_key = config_manager
        .resolve_credentials(&provider_id)?
        .ok_or_else(|| GatewayError::CredentialError(
            "OpenRouter API key not configured".to_string()
        ))?;

    // Create client
    let client = OpenRouterClient::with_config(
        api_key,
        provider_config.base_url.clone(),
        provider_config.default_model.clone(),
    )?;

    Ok(client.with_timeout(provider_config.timeout_seconds))
}