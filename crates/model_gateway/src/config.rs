//! Configuration for model gateway
//!
//! Handles provider configuration, default selections, and persistence.

use crate::model::ModelId;
use crate::provider::ProviderId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Top-level configuration for the model gateway
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    /// Default provider to use when none specified
    pub default_provider: Option<ProviderId>,
    /// Default model ID (provider-specific format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_model: Option<ModelId>,
    /// Provider-specific configurations
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub providers: HashMap<ProviderId, ProviderConfig>,
}

impl GatewayConfig {
    /// Create a new empty configuration
    pub fn new() -> Self {
        Self {
            default_provider: None,
            default_model: None,
            providers: HashMap::new(),
        }
    }

    /// Get configuration for a specific provider
    pub fn get_provider(&self, id: &ProviderId) -> Option<&ProviderConfig> {
        self.providers.get(id)
    }

    /// Get mutable configuration for a specific provider
    pub fn get_provider_mut(&mut self, id: &ProviderId) -> Option<&mut ProviderConfig> {
        self.providers.get_mut(id)
    }

    /// Set configuration for a provider
    pub fn set_provider(&mut self, id: ProviderId, config: ProviderConfig) {
        self.providers.insert(id, config);
    }

    /// Check if a provider is configured
    pub fn has_provider(&self, id: &ProviderId) -> bool {
        self.providers.contains_key(id)
    }

    /// Get the default provider configuration
    pub fn default_provider_config(&self) -> Option<(ProviderId, &ProviderConfig)> {
        self.default_provider
            .as_ref()
            .and_then(|id| self.providers.get(id).map(|cfg| (id.clone(), cfg)))
    }

    /// Set the default provider
    pub fn set_default_provider(&mut self, id: ProviderId) {
        self.default_provider = Some(id);
    }

    /// Set the default model
    pub fn set_default_model(&mut self, model: ModelId) {
        self.default_model = Some(model);
    }

    /// Set default provider and model from string IDs (convenience method)
    pub fn set_defaults(&mut self, provider: impl Into<String>, model: impl Into<String>) {
        self.default_provider = Some(ProviderId::new(provider));
        self.default_model = Some(ModelId::new(model));
    }

    /// Remove a provider configuration
    pub fn remove_provider(&mut self, id: &ProviderId) -> Option<ProviderConfig> {
        self.providers.remove(id)
    }

    /// List all configured provider IDs
    pub fn list_providers(&self) -> Vec<&ProviderId> {
        self.providers.keys().collect()
    }

    /// Check if configuration has any providers set up
    pub fn is_empty(&self) -> bool {
        self.providers.is_empty()
    }

    /// Get count of configured providers
    pub fn len(&self) -> usize {
        self.providers.len()
    }
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for a specific provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Whether this provider is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// API key (can be actual key or env var reference like "env:OPENROUTER_API_KEY")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    /// Base URL for API (optional, for custom endpoints or proxies)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    /// Default model for this provider
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_model: Option<String>,
    /// Timeout for requests in seconds (default: 60)
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
    /// Additional provider-specific settings
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub extra: HashMap<String, String>,
}

fn default_true() -> bool {
    true
}

fn default_timeout() -> u64 {
    60
}

impl ProviderConfig {
    /// Create a new provider configuration
    pub fn new() -> Self {
        Self {
            enabled: true,
            api_key: None,
            base_url: None,
            default_model: None,
            timeout_seconds: default_timeout(),
            extra: HashMap::new(),
        }
    }

    /// Create configuration with API key
    pub fn with_api_key<S: Into<String>>(api_key: S) -> Self {
        let mut cfg = Self::new();
        cfg.api_key = Some(api_key.into());
        cfg
    }

    /// Check if this configuration is valid (enabled and has API key)
    pub fn is_valid(&self) -> bool {
        self.enabled && self.api_key.is_some()
    }

    /// Check if this configuration has an API key set
    pub fn has_api_key(&self) -> bool {
        self.api_key.is_some()
    }

    /// Set the API key (supports "env:VAR_NAME" format)
    pub fn set_api_key<S: Into<String>>(&mut self, key: S) {
        self.api_key = Some(key.into());
    }

    /// Set the base URL
    pub fn set_base_url<S: Into<String>>(&mut self, url: S) {
        self.base_url = Some(url.into());
    }

    /// Set the default model for this provider
    pub fn set_default_model<S: Into<String>>(&mut self, model: S) {
        self.default_model = Some(model.into());
    }

    /// Set timeout
    pub fn set_timeout(&mut self, seconds: u64) {
        self.timeout_seconds = seconds;
    }

    /// Get an extra setting
    pub fn get_extra(&self, key: &str) -> Option<&str> {
        self.extra.get(key).map(|s| s.as_str())
    }

    /// Set an extra setting
    pub fn set_extra(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.extra.insert(key.into(), value.into());
    }

    /// Create OpenRouter configuration with API key
    pub fn openrouter(api_key: impl Into<String>) -> Self {
        let mut cfg = Self::with_api_key(api_key);
        cfg.base_url = Some("https://openrouter.ai/api/v1".to_string());
        cfg.default_model = Some("anthropic/claude-3.5-sonnet".to_string());
        cfg
    }

    /// Create OpenAI configuration with API key
    pub fn openai(api_key: impl Into<String>) -> Self {
        let mut cfg = Self::with_api_key(api_key);
        cfg.set_base_url("https://api.openai.com/v1");
        cfg
    }

    /// Create Anthropic configuration with API key
    pub fn anthropic(api_key: impl Into<String>) -> Self {
        let mut cfg = Self::with_api_key(api_key);
        cfg.set_base_url("https://api.anthropic.com/v1");
        cfg
    }
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Recommended models for MVP (for quick reference)
pub mod recommended {
    /// OpenRouter recommended models as (id, display_name) pairs
    /// These models are verified to work and provide good results
    pub const OPENROUTER_MODELS: &[(&str, &str)] = &[
        (
            "anthropic/claude-3.5-sonnet",
            "Claude 3.5 Sonnet - Best overall",
        ),
        ("openai/gpt-4o", "GPT-4o - Great performance"),
        ("openai/gpt-4o-mini", "GPT-4o Mini - Fast & cheap"),
        ("anthropic/claude-3.5-haiku", "Claude 3.5 Haiku - Fast"),
        ("anthropic/claude-3-opus", "Claude 3 Opus - Most powerful"),
    ];

    /// Get display name for a model ID
    pub fn get_display_name(model_id: &str) -> Option<&'static str> {
        OPENROUTER_MODELS
            .iter()
            .find(|(id, _)| *id == model_id)
            .map(|(_, name)| *name)
    }

    /// Get the default MVP model ID
    pub fn mvp_default_model() -> &'static str {
        "anthropic/claude-3.5-sonnet"
    }
}
