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

        let mut attempt = 1;
        let max_attempts = 4; // 1 initial + 3 retries

        loop {
            let response_result = self
                .http_client
                .post(&url)
                .headers(self.build_request_headers())
                .json(&openrouter_request)
                .send()
                .await;

            match response_result {
                Ok(response) if response.status().is_success() => {
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

                    return self.transform_response(openrouter_response);
                }
                Ok(response) => {
                    let status = response.status();
                    let is_client_error = status.is_client_error();
                    
                    let error_text = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    
                    if is_client_error || attempt >= max_attempts {
                        // Don't retry 4xx errors, or we've run out of retries
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
                    
                    tracing::warn!("OpenRouter API error (attempt {}): HTTP {} - {}", attempt, status, error_text);
                }
                Err(e) => {
                    // Reqwest network error or timeout
                    if attempt >= max_attempts {
                        return Err(GatewayError::ProviderError {
                            provider: "openrouter".to_string(),
                            message: format!("Request failed after {} attempts: {}", attempt, e),
                        });
                    }
                    tracing::warn!("OpenRouter network error (attempt {}): {}", attempt, e);
                }
            }
            
            let delay_secs = 1 << (attempt - 1);
            tokio::time::sleep(tokio::time::Duration::from_secs(delay_secs)).await;
            attempt += 1;
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openrouter_client_creation() {
        let client = OpenRouterClient::new("test-api-key");
        assert!(client.is_ok());
        
        let client = client.unwrap();
        assert!(client.is_configured());
        assert_eq!(client.id().as_str(), "openrouter");
        assert_eq!(client.name(), "OpenRouter");
    }

    #[test]
    fn test_empty_api_key_fails() {
        let client = OpenRouterClient::new("");
        assert!(client.is_err());
        match client {
            Err(GatewayError::CredentialError(_)) => {},
            _ => panic!("Expected CredentialError for empty API key"),
        }
    }

    #[test]
    fn test_request_transformation() {
        let client = OpenRouterClient::new("test-key").unwrap();
        
        let request = InferenceRequest::new("anthropic/claude-3.5-sonnet")
            .with_user("Hello, world!");
        
        let openrouter_req = client.transform_request(request);
        
        assert_eq!(openrouter_req.model, "anthropic/claude-3.5-sonnet");
        assert_eq!(openrouter_req.messages.len(), 1);
        assert_eq!(openrouter_req.messages[0].content, "Hello, world!");
        assert_eq!(openrouter_req.messages[0].role, "user");
    }

    #[test]
    fn test_default_model_fallback() {
        let client = OpenRouterClient::with_config(
            "test-key",
            None,
            Some("default-model".to_string()),
        ).unwrap();
        
        let request = InferenceRequest::new(""); // Empty model
        let openrouter_req = client.transform_request(request);
        
        assert_eq!(openrouter_req.model, "default-model");
    }

    #[test]
    fn test_message_conversion() {
        // Test Message -> OpenRouterMessage
        let msg = Message::user("Test content");
        let or_msg: OpenRouterMessage = msg.into();
        assert_eq!(or_msg.role, "user");
        assert_eq!(or_msg.content, "Test content");

        // Test system message
        let msg = Message::system("System prompt");
        let or_msg: OpenRouterMessage = msg.into();
        assert_eq!(or_msg.role, "system");

        // Test assistant message
        let msg = Message::assistant("Assistant response");
        let or_msg: OpenRouterMessage = msg.into();
        assert_eq!(or_msg.role, "assistant");
    }

    #[test]
    fn test_openrouter_message_to_message() {
        let or_msg = OpenRouterMessage {
            role: "assistant".to_string(),
            content: "Hello".to_string(),
        };
        let msg: Message = or_msg.into();
        assert!(matches!(msg.role, Role::Assistant));
        assert_eq!(msg.content, "Hello");
    }

    #[test]
    fn test_static_model_list() {
        let models = static_model_list();
        assert!(!models.is_empty());
        
        // Check that recommended models are included
        let model_ids: Vec<&str> = models.iter().map(|m| m.id.as_str()).collect();
        assert!(model_ids.contains(&"anthropic/claude-3.5-sonnet"));
    }

    #[tokio::test]
    async fn test_list_models_returns_models() {
        let client = OpenRouterClient::new("test-key").unwrap();
        
        // This should return the static list since the API call will fail with invalid key
        let models = client.list_models().await;
        assert!(models.is_ok());
        
        let models = models.unwrap();
        assert!(!models.is_empty());
    }

    #[tokio::test]
    async fn test_infer_retry_success() {
        use wiremock::{MockServer, Mock, matchers, ResponseTemplate};
        
        // Disable retry delays in tests internally if possible, but here we just use small timeouts
        // Note: the test will take 1+2+4=7 seconds if it retries 3 times. We can just test 2 retries (3 seconds).
        // Actually, to make it fast we would need to mock the delay or pass a time_multiplier. Let's just run it!
        // We will mock one 500 error then a 200 success.
        let mock_server = MockServer::start().await;
        
        let response_200 = ResponseTemplate::new(200)
            .set_body_json(serde_json::json!({
                "id": "test-id",
                "model": "test-model",
                "choices": [
                    {
                        "message": { "role": "assistant", "content": "Success!" },
                        "index": 0
                    }
                ]
            }));

        // Use sequential logic for wiremock
        Mock::given(matchers::method("POST"))
            .and(matchers::path("/chat/completions"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;

        Mock::given(matchers::method("POST"))
            .and(matchers::path("/chat/completions"))
            .respond_with(response_200)
            .mount(&mock_server) // Fallback for subsequent requests
            .await;

        let client = OpenRouterClient::with_config(
            "test-key",
            Some(mock_server.uri()),
            Some("default-model".to_string()),
        ).unwrap();

        let request = InferenceRequest::new("default-model").with_user("Hi");
        let result = client.infer(request).await;

        assert!(result.is_ok());
        let msg = result.unwrap().message;
        assert_eq!(msg.content, "Success!");
    }

    #[tokio::test]
    async fn test_infer_retry_exhausted() {
        use wiremock::{MockServer, Mock, matchers, ResponseTemplate};
        let mock_server = MockServer::start().await;
        
        Mock::given(matchers::method("POST"))
            .and(matchers::path("/chat/completions"))
            .respond_with(ResponseTemplate::new(502).set_body_string("Bad Gateway"))
            .mount(&mock_server)
            .await;

        let client = OpenRouterClient::with_config(
            "test-key",
            Some(mock_server.uri()),
            Some("default-model".to_string()),
        ).unwrap();

        let request = InferenceRequest::new("default-model").with_user("Hi");
        
        // Start a timeout to ensure it actually aborts
        let result = tokio::time::timeout(tokio::time::Duration::from_secs(10), client.infer(request)).await;
        
        assert!(result.is_ok()); // Did not timeout
        let err = result.unwrap();
        assert!(err.is_err());
        match err.unwrap_err() {
            GatewayError::ProviderError { message, .. } => {
                assert!(message.contains("HTTP 502") || message.contains("Bad Gateway"));
            }
            _ => panic!("Expected ProviderError"),
        }
    }
}