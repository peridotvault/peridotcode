//! Groq provider adapter
//!
//! Implements the `Provider` trait for Groq (https://groq.com).
//!
//! # Design Notes
//!
//! - Groq uses an OpenAI-compatible API format
//! - Model IDs are in format like "llama-3.1-70b-versatile"
//! - Known for fast inference speeds
//! - Uses OpenAI SDK compatible endpoints

use crate::{
    GatewayError, GatewayResult, InferenceRequest, InferenceResponse, Message, Provider,
    ProviderId, Role, UsageStats,
};
use crate::provider::ModelInfo;
use reqwest::header::{self, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Groq API client
///
/// This is the concrete implementation of the Provider trait for Groq.
/// It handles authentication, request formatting, and response normalization.
#[derive(Debug, Clone)]
pub struct GroqClient {
    http_client: reqwest::Client,
    api_key: String,
    base_url: String,
    default_model: Option<String>,
    timeout: Duration,
}

impl GroqClient {
    pub const DEFAULT_BASE_URL: &'static str = "https://api.groq.com/openai/v1";

    pub fn new(api_key: impl Into<String>) -> GatewayResult<Self> {
        Self::with_config(api_key, None, None)
    }

    pub fn with_config(
        api_key: impl Into<String>,
        base_url: Option<String>,
        default_model: Option<String>,
    ) -> GatewayResult<Self> {
        let api_key = api_key.into();
        let base_url = base_url.unwrap_or_else(|| Self::DEFAULT_BASE_URL.to_string());

        if api_key.is_empty() {
            return Err(GatewayError::CredentialError(
                "Groq API key is empty".to_string(),
            ));
        }

        let http_client = Self::build_http_client(&api_key)?;

        Ok(Self {
            http_client,
            api_key,
            base_url,
            default_model,
            timeout: Duration::from_secs(60),
        })
    }

    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout = Duration::from_secs(seconds);
        self
    }

    fn build_http_client(api_key: &str) -> GatewayResult<reqwest::Client> {
        let mut headers = HeaderMap::new();

        let auth_value = format!("Bearer {}", api_key);
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_str(&auth_value).map_err(|e| {
                GatewayError::ConfigError(format!("Invalid API key format: {}", e))
            })?,
        );

        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

        reqwest::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(60))
            .build()
            .map_err(|e| GatewayError::ProviderError {
                provider: "groq".to_string(),
                message: format!("Failed to build HTTP client: {}", e),
            })
    }

    fn chat_endpoint(&self) -> String {
        format!("{}/chat/completions", self.base_url)
    }

    fn models_endpoint(&self) -> String {
        format!("{}/models", self.base_url)
    }

    fn transform_request(&self, request: InferenceRequest) -> GroqRequest {
        let model = if request.model.is_empty() {
            self.default_model.clone().unwrap_or_default()
        } else {
            request.model
        };

        GroqRequest {
            model,
            messages: request.messages.into_iter().map(|m| m.into()).collect(),
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            stream: Some(false),
        }
    }

    fn transform_response(&self, response: GroqResponse) -> GatewayResult<InferenceResponse> {
        let choice = response.choices.into_iter().next().ok_or_else(|| {
            GatewayError::ProviderError {
                provider: "groq".to_string(),
                message: "No choices in response".to_string(),
            }
        })?;

        let message: Message = choice.message.into();

        let usage = response.usage.map(|u| UsageStats {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
        });

        Ok(InferenceResponse {
            message,
            model: response.model,
            provider: "groq".to_string(),
            usage,
            finish_reason: choice.finish_reason,
        })
    }

    #[allow(dead_code)]
    async fn list_models_internal(&self) -> GatewayResult<Vec<ModelInfo>> {
        let url = self.models_endpoint();

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| GatewayError::ProviderError {
                provider: "groq".to_string(),
                message: format!("Failed to fetch models: {}", e),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(GatewayError::ProviderError {
                provider: "groq".to_string(),
                message: format!("HTTP {}: Failed to fetch models", status),
            });
        }

        let data: GroqModelsResponse = response.json().await.map_err(|e| {
            GatewayError::ProviderError {
                provider: "groq".to_string(),
                message: format!("Failed to parse models response: {}", e),
            }
        })?;

        let model_infos: Vec<ModelInfo> = data
            .data
            .into_iter()
            .map(|m| ModelInfo {
                id: m.id.clone(),
                name: m.name.unwrap_or_else(|| m.id.clone()),
                provider: ProviderId::groq(),
                context_window: m.context_length,
                recommended: RECOMMENDED_MODELS.contains(&m.id.as_str()),
            })
            .collect();

        Ok(model_infos)
    }
}

#[async_trait::async_trait]
impl Provider for GroqClient {
    fn id(&self) -> ProviderId {
        ProviderId::groq()
    }

    fn name(&self) -> &str {
        "Groq"
    }

    fn is_configured(&self) -> bool {
        !self.api_key.is_empty()
    }

    async fn infer(&self, request: InferenceRequest) -> GatewayResult<InferenceResponse> {
        if request.messages.is_empty() {
            return Err(GatewayError::ValidationError(
                "Request must have at least one message".to_string(),
            ));
        }

        let url = self.chat_endpoint();
        let groq_request = self.transform_request(request);

        tracing::debug!(
            "Sending request to Groq: model={}",
            groq_request.model
        );

        let response = self
            .http_client
            .post(&url)
            .json(&groq_request)
            .send()
            .await
            .map_err(|e| GatewayError::ProviderError {
                provider: "groq".to_string(),
                message: format!("Request failed: {}", e),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(GatewayError::ProviderError {
                provider: "groq".to_string(),
                message: format!("HTTP {}: {}", status, text),
            });
        }

        let groq_response: GroqResponse = response.json().await.map_err(|e| {
            GatewayError::ProviderError {
                provider: "groq".to_string(),
                message: format!("Failed to parse response: {}", e),
            }
        })?;

        self.transform_response(groq_response)
    }

    async fn list_models(&self) -> GatewayResult<Vec<ModelInfo>> {
        GroqClient::list_models(self).await
    }

    async fn validate_credentials(&self) -> GatewayResult<()> {
        let url = self.models_endpoint();

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| GatewayError::ProviderError {
                provider: "groq".to_string(),
                message: format!("Network error during validation: {}", e),
            })?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(GatewayError::CredentialError(
                "API key validation failed".to_string(),
            ))
        }
    }
}

#[derive(Debug, Serialize)]
struct GroqRequest {
    model: String,
    messages: Vec<GroqMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GroqMessage {
    role: String,
    content: String,
}

impl From<Message> for GroqMessage {
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

impl From<GroqMessage> for Message {
    fn from(msg: GroqMessage) -> Self {
        let role = match msg.role.as_str() {
            "system" => Role::System,
            "assistant" => Role::Assistant,
            _ => Role::User,
        };
        Self::new(role, msg.content)
    }
}

#[derive(Debug, Deserialize)]
struct GroqResponse {
    id: String,
    model: String,
    choices: Vec<GroqChoice>,
    usage: Option<GroqUsage>,
}

#[derive(Debug, Deserialize)]
struct GroqChoice {
    message: GroqMessage,
    finish_reason: Option<String>,
    index: u32,
}

#[derive(Debug, Deserialize)]
struct GroqUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct GroqModelsResponse {
    data: Vec<GroqModel>,
}

#[derive(Debug, Deserialize)]
struct GroqModel {
    id: String,
    name: Option<String>,
    #[serde(rename = "context_length")]
    context_length: Option<usize>,
}

const RECOMMENDED_MODELS: &[&str] = &[
    "llama-3.1-70b-versatile",
    "llama-3.1-405b-reasoning",
    "llama-3.1-8b-instant",
    "mixtral-8x7b-32768",
    "gemma2-9b-it",
];

pub fn static_model_list() -> Vec<ModelInfo> {
    vec![
        ModelInfo {
            id: "llama-3.1-70b-versatile".to_string(),
            name: "Llama 3.1 70B Versatile".to_string(),
            provider: ProviderId::groq(),
            context_window: Some(128_000),
            recommended: true,
        },
        ModelInfo {
            id: "llama-3.1-405b-reasoning".to_string(),
            name: "Llama 3.1 405B Reasoning".to_string(),
            provider: ProviderId::groq(),
            context_window: Some(128_000),
            recommended: true,
        },
        ModelInfo {
            id: "llama-3.1-8b-instant".to_string(),
            name: "Llama 3.1 8B Instant".to_string(),
            provider: ProviderId::groq(),
            context_window: Some(128_000),
            recommended: true,
        },
        ModelInfo {
            id: "mixtral-8x7b-32768".to_string(),
            name: "Mixtral 8x7B".to_string(),
            provider: ProviderId::groq(),
            context_window: Some(32_768),
            recommended: true,
        },
        ModelInfo {
            id: "gemma2-9b-it".to_string(),
            name: "Gemma 2 9B".to_string(),
            provider: ProviderId::groq(),
            context_window: Some(8_192),
            recommended: true,
        },
    ]
}

pub async fn create_groq_client(
    config_manager: &crate::ConfigManager,
) -> GatewayResult<GroqClient> {
    let provider_id = ProviderId::groq();

    let provider_config = config_manager
        .config()
        .get_provider(&provider_id)
        .ok_or_else(|| GatewayError::ConfigError(
            "Groq provider not configured".to_string()
        ))?;

    let api_key = config_manager
        .resolve_credentials(&provider_id)?
        .ok_or_else(|| GatewayError::CredentialError(
            "Groq API key not configured".to_string()
        ))?;

    let client = GroqClient::with_config(
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
    fn test_groq_client_creation() {
        let client = GroqClient::new("test-api-key");
        assert!(client.is_ok());

        let client = client.unwrap();
        assert!(client.is_configured());
        assert_eq!(client.id().as_str(), "groq");
        assert_eq!(client.name(), "Groq");
    }

    #[test]
    fn test_empty_api_key_fails() {
        let client = GroqClient::new("");
        assert!(client.is_err());
    }

    #[test]
    fn test_request_transformation() {
        let client = GroqClient::new("test-key").unwrap();

        let request = InferenceRequest::new("llama-3.1-70b-versatile")
            .with_user("Hello");

        let groq_req = client.transform_request(request);

        assert_eq!(groq_req.model, "llama-3.1-70b-versatile");
        assert_eq!(groq_req.messages.len(), 1);
    }

    #[test]
    fn test_static_model_list() {
        let models = static_model_list();
        assert!(!models.is_empty());
    }
}