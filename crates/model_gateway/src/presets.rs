//! Configuration presets and quick-setup helpers
//!
//! This module provides pre-built configuration templates for common setups.
//! Use these to quickly configure PeridotCode without manually writing TOML.

use crate::{ConfigManager, GatewayConfig, GatewayResult, ModelId, ProviderConfig, ProviderId};
use std::collections::HashMap;

/// Quick configuration presets for common providers
pub struct ConfigPresets;

impl ConfigPresets {
    /// Create a minimal working configuration for OpenRouter
    ///
    /// This is the recommended quick-start configuration.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use peridot_model_gateway::{ConfigPresets, ConfigManager};
    ///
    /// // Create config that references env var
    /// let config = ConfigPresets::openrouter_env();
    ///
    /// // Or create config with direct key (not recommended for production)
    /// let config = ConfigPresets::openrouter_key("sk-or-v1-xxx");
    /// ```
    pub fn openrouter_env() -> GatewayConfig {
        let mut config = GatewayConfig::new();

        let provider_id = ProviderId::openrouter();
        let provider_config = ProviderConfig {
            enabled: true,
            api_key: Some("env:OPENROUTER_API_KEY".to_string()),
            base_url: None,
            default_model: Some("anthropic/claude-3.5-sonnet".to_string()),
            timeout_seconds: 60,
            extra: HashMap::new(),
        };

        config.set_provider(provider_id.clone(), provider_config);
        config.set_default_provider(provider_id);
        config.set_default_model(ModelId::new("anthropic/claude-3.5-sonnet"));

        config
    }

    /// Create OpenRouter configuration with a direct API key
    ///
    /// ⚠️ **Security Warning**: Only use this for testing. In production,
    /// use `openrouter_env()` with environment variables.
    pub fn openrouter_key(api_key: impl Into<String>) -> GatewayConfig {
        let mut config = Self::openrouter_env();

        if let Some(provider_config) = config.get_provider_mut(&ProviderId::openrouter()) {
            provider_config.set_api_key(format!("key:{}", api_key.into()));
        }

        config
    }

    /// Create configuration for OpenAI
    ///
    /// Future provider - OpenRouter is recommended for MVP.
    pub fn openai_env() -> GatewayConfig {
        let mut config = GatewayConfig::new();

        let provider_id = ProviderId::openai();
        let provider_config = ProviderConfig {
            enabled: true,
            api_key: Some("env:OPENAI_API_KEY".to_string()),
            base_url: None,
            default_model: Some("gpt-4o-mini".to_string()),
            timeout_seconds: 60,
            extra: HashMap::new(),
        };

        config.set_provider(provider_id.clone(), provider_config);
        config.set_default_provider(provider_id);
        config.set_default_model(ModelId::new("gpt-4o-mini"));

        config
    }

    /// Create configuration for Anthropic
    ///
    /// Future provider - OpenRouter is recommended for MVP.
    pub fn anthropic_env() -> GatewayConfig {
        let mut config = GatewayConfig::new();

        let provider_id = ProviderId::anthropic();
        let provider_config = ProviderConfig {
            enabled: true,
            api_key: Some("env:ANTHROPIC_API_KEY".to_string()),
            base_url: None,
            default_model: Some("claude-3-sonnet-20240229".to_string()),
            timeout_seconds: 60,
            extra: HashMap::new(),
        };

        config.set_provider(provider_id.clone(), provider_config);
        config.set_default_provider(provider_id);
        config.set_default_model(ModelId::new("claude-3-sonnet-20240229"));

        config
    }

    /// Create a development configuration with placeholder credentials
    ///
    /// This creates a configuration that expects environment variables
    /// but has sensible defaults. Useful for development setups.
    pub fn development() -> GatewayConfig {
        let mut config = GatewayConfig::new();

        // Configure OpenRouter as primary
        let openrouter_config = ProviderConfig {
            enabled: true,
            api_key: Some("env:OPENROUTER_API_KEY".to_string()),
            base_url: None,
            default_model: Some("anthropic/claude-3.5-sonnet".to_string()),
            timeout_seconds: 60,
            extra: HashMap::new(),
        };
        config.set_provider(ProviderId::openrouter(), openrouter_config);

        // Configure OpenAI as secondary (disabled by default)
        let openai_config = ProviderConfig {
            enabled: false,
            api_key: Some("env:OPENAI_API_KEY".to_string()),
            base_url: None,
            default_model: Some("gpt-4o-mini".to_string()),
            timeout_seconds: 60,
            extra: HashMap::new(),
        };
        config.set_provider(ProviderId::openai(), openai_config);

        // Set defaults
        config.set_default_provider(ProviderId::openrouter());
        config.set_default_model(ModelId::new("anthropic/claude-3.5-sonnet"));

        config
    }
}

/// Configuration builder for programmatic configuration
///
/// This provides a fluent API for building configuration:
///
/// ```rust,ignore
/// use peridot_model_gateway::ConfigBuilder;
///
/// let config = ConfigBuilder::new()
///     .with_provider_openrouter()
///     .with_default_model("anthropic/claude-3.5-sonnet")
///     .build();
/// ```
#[derive(Debug, Default)]
pub struct ConfigBuilder {
    config: GatewayConfig,
}

impl ConfigBuilder {
    /// Create a new configuration builder
    pub fn new() -> Self {
        Self {
            config: GatewayConfig::new(),
        }
    }

    /// Add OpenRouter provider configuration
    pub fn with_provider_openrouter(mut self) -> Self {
        let config = ProviderConfig {
            enabled: true,
            api_key: Some("env:OPENROUTER_API_KEY".to_string()),
            base_url: None,
            default_model: Some("anthropic/claude-3.5-sonnet".to_string()),
            timeout_seconds: 60,
            extra: HashMap::new(),
        };
        self.config.set_provider(ProviderId::openrouter(), config);
        self
    }

    /// Add OpenAI provider configuration
    pub fn with_provider_openai(mut self) -> Self {
        let config = ProviderConfig {
            enabled: true,
            api_key: Some("env:OPENAI_API_KEY".to_string()),
            base_url: None,
            default_model: Some("gpt-4o-mini".to_string()),
            timeout_seconds: 60,
            extra: HashMap::new(),
        };
        self.config.set_provider(ProviderId::openai(), config);
        self
    }

    /// Add Anthropic provider configuration
    pub fn with_provider_anthropic(mut self) -> Self {
        let config = ProviderConfig {
            enabled: true,
            api_key: Some("env:ANTHROPIC_API_KEY".to_string()),
            base_url: None,
            default_model: Some("claude-3-sonnet-20240229".to_string()),
            timeout_seconds: 60,
            extra: HashMap::new(),
        };
        self.config.set_provider(ProviderId::anthropic(), config);
        self
    }

    /// Set the default provider
    pub fn with_default_provider(mut self, provider: impl Into<String>) -> Self {
        self.config.set_default_provider(ProviderId::new(provider));
        self
    }

    /// Set the default model
    pub fn with_default_model(mut self, model: impl Into<String>) -> Self {
        self.config.set_default_model(ModelId::new(model));
        self
    }

    /// Set API key for a provider
    pub fn with_api_key(mut self, provider: impl Into<String>, api_key: impl Into<String>) -> Self {
        let provider_id = ProviderId::new(provider);
        let api_key_str = api_key.into();

        // Ensure provider exists
        if self.config.get_provider(&provider_id).is_none() {
            let provider_config = ProviderConfig::new();
            self.config
                .set_provider(provider_id.clone(), provider_config);
        }

        // Set API key
        if let Some(config) = self.config.get_provider_mut(&provider_id) {
            config.set_api_key(api_key_str);
        }

        self
    }

    /// Build the configuration
    pub fn build(self) -> GatewayConfig {
        self.config
    }

    /// Build and save to the default config location
    pub fn build_and_save(self) -> GatewayResult<GatewayConfig> {
        let config = self.config;
        let manager = ConfigManager::with_config(config.clone());
        manager.save()?;
        Ok(config)
    }
}

/// Check if the environment is properly configured
///
/// Returns a report of what's configured and what's missing.
pub fn check_environment() -> EnvironmentReport {
    use crate::credentials::env_vars;

    let mut report = EnvironmentReport::default();

    // Check for OpenRouter
    if std::env::var(env_vars::OPENROUTER_API_KEY).is_ok() {
        report.available_providers.push(ProviderId::openrouter());
    } else {
        report.missing_env_vars.push((
            ProviderId::openrouter(),
            env_vars::OPENROUTER_API_KEY.to_string(),
        ));
    }

    // Check for OpenAI
    if std::env::var(env_vars::OPENAI_API_KEY).is_ok() {
        report.available_providers.push(ProviderId::openai());
    } else {
        report
            .missing_env_vars
            .push((ProviderId::openai(), env_vars::OPENAI_API_KEY.to_string()));
    }

    // Check for Anthropic
    if std::env::var(env_vars::ANTHROPIC_API_KEY).is_ok() {
        report.available_providers.push(ProviderId::anthropic());
    } else {
        report.missing_env_vars.push((
            ProviderId::anthropic(),
            env_vars::ANTHROPIC_API_KEY.to_string(),
        ));
    }

    report.ready = !report.available_providers.is_empty();
    report
}

/// Report on environment configuration status
#[derive(Debug, Clone, Default)]
pub struct EnvironmentReport {
    /// Whether at least one provider is ready
    pub ready: bool,
    /// Providers that have environment variables set
    pub available_providers: Vec<ProviderId>,
    /// Providers missing environment variables
    pub missing_env_vars: Vec<(ProviderId, String)>,
}

impl EnvironmentReport {
    /// Get a human-readable summary
    pub fn summary(&self) -> String {
        if self.ready {
            format!(
                "Ready with {} provider(s): {}",
                self.available_providers.len(),
                self.available_providers
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        } else {
            "No providers configured. Set OPENROUTER_API_KEY environment variable.".to_string()
        }
    }

    /// Check if a specific provider is available
    pub fn has_provider(&self, provider: &ProviderId) -> bool {
        self.available_providers.contains(provider)
    }
}
