//! Model Gateway
//!
//! Provides abstraction over AI model providers. Handles:
//! - Provider configuration and registry
//! - Model catalog and capabilities
//! - Credential resolution from environment/config
//! - Normalized inference requests/responses
//! - OpenRouter adapter (MVP priority - fully implemented)
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use peridot_model_gateway::{
//!     ConfigManager, create_openrouter_client, InferenceRequest, Provider,
//! };
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Load configuration (with .env support)
//!     let manager = ConfigManager::initialize()?;
//!     
//!     // Create OpenRouter client
//!     let client = create_openrouter_client(&manager).await?;
//!     
//!     // Create and send request
//!     let request = InferenceRequest::new("anthropic/claude-3.5-sonnet")
//!         .with_user("Hello!");
//!     
//!     let response = client.infer(request).await?;
//!     println!("{}", response.content());
//!     
//!     Ok(())
//! }
//! ```
//!
//! # Architecture Overview
//!
//! The model_gateway crate provides a layered abstraction:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                      ModelGateway                           │
//! │                (Main entry point)                           │
//! └─────────────────────────────────────────────────────────────┘
//!                               │
//!           ┌───────────────────┼───────────────────┐
//!           ▼                   ▼                   ▼
//! ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
//! │ ProviderRegistry│  │  GatewayConfig  │  │  ModelCatalog   │
//! │                 │  │                 │  │                 │
//! └─────────────────┘  └─────────────────┘  └─────────────────┘
//!           │                   │                   │
//!           ▼                   ▼                   ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │                      Module Layer                           │
//! ├─────────────┬─────────────┬─────────────┬───────────────────┤
//! │  provider   │   config    │ credentials │  inference        │
//! │  (adapters) │  (settings) │   (auth)    │  (protocol)       │
//! ├─────────────┴─────────────┴─────────────┴───────────────────┤
//! │                      model                                  │
//! │              (identity & capabilities)                      │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Extension Points
//!
//! ## Adding a New Provider
//!
//! 1. **Implement the `Provider` trait** in a new module:
//!
//! ```rust,ignore
//! use peridot_model_gateway::{
//!     Provider, ProviderId, ModelId, InferenceRequest, InferenceResponse,
//!     GatewayResult, ModelInfo, async_trait,
//! };
//!
//! #[derive(Debug)]
//! pub struct MyProvider {
//!     api_key: String,
//!     base_url: String,
//! }
//!
//! #[async_trait]
//! impl Provider for MyProvider {
//!     fn id(&self) -> ProviderId {
//!         ProviderId::new("myprovider")
//!     }
//!
//!     fn name(&self) -> &str {
//!         "My AI Provider"
//!     }
//!
//!     fn is_configured(&self) -> bool {
//!         !self.api_key.is_empty()
//!     }
//!
//!     async fn infer(&self, request: InferenceRequest) -> GatewayResult<InferenceResponse> {
//!         // Transform normalized request to provider format
//!         // Make HTTP request
//!         // Transform response to normalized format
//!     }
//!
//!     async fn list_models(&self) -> GatewayResult<Vec<ModelInfo>> {
//!         // Return available models or static list
//!     }
//! }
//! ```
//!
//! 2. **Register the provider** with the gateway:
//!
//! ```rust,ignore
//! let mut gateway = ModelGateway::new();
//! gateway.registry_mut().register(ProviderId::new("myprovider"));
//! ```
//!
//! 3. **Add configuration support** (optional):
//!
//! ```rust,ignore
//! let config = ProviderConfig::with_api_key("my-api-key");
//! gateway.config_mut().set_provider(ProviderId::new("myprovider"), config);
//! ```
//!
//! ## Adding Model Capabilities
//!
//! To add a new capability:
//!
//! 1. Add the capability to `ModelCapability` enum in `model.rs`
//! 2. Update capability detection in your provider implementation
//! 3. Use the new capability with `ModelFilter`
//!
//! ## Custom Model Catalogs
//!
//! You can create custom model catalogs:
//!
//! ```rust,ignore
//! let mut catalog = ModelCatalog::new();
//! 
//! catalog.add(ModelDescriptor::new(
//!     "custom-model",
//!     "My Custom Model",
//!     ProviderId::openrouter(),
//!     128000,
//! ).with_capability(ModelCapability::GameScaffolding));
//! ```
//!
//! # Future Providers
//!
//! The architecture supports these providers (not yet implemented):
//!
//! - **OpenAI** (`openai`) - GPT-4, GPT-4o, GPT-3.5
//! - **Anthropic** (`anthropic`) - Claude 3 Opus, Sonnet, Haiku
//! - **Google** (`gemini`) - Gemini Pro, Flash
//! - **Local** (`local`) - Ollama, llama.cpp for on-device inference
//!
//! Each should implement the `Provider` trait and handle their specific:
//! - Authentication methods
//! - Request/response formats
//! - Model listing capabilities
//! - Error handling

#![warn(missing_docs)]

pub mod anthropic;
pub mod catalog;
pub mod config;
pub mod config_file;
pub mod credentials;
pub mod inference;
pub mod model;
/// Observability and tracking module
pub mod observability;
pub mod openai;
pub mod openrouter;
pub mod presets;
pub mod provider;

// Re-exports for convenience
pub use anthropic::{create_anthropic_client, AnthropicClient};
pub use catalog::{ModelCatalog, ModelFilter};
pub use config::{GatewayConfig, ProviderConfig};
pub use config_file::{interactive, ConfigManager, ConfigSource};
pub use credentials::CredentialResolver;
pub use inference::{InferenceRequest, InferenceResponse, Message, Role, UsageStats};
pub use model::{
    CostTier, ModelCapability, ModelDescriptor, ModelId, ModelTier, recommended,
};
pub use observability::*;
pub use openai::{create_openai_client, OpenAIClient};
pub use openrouter::{create_openrouter_client, OpenRouterClient};
pub use presets::{check_environment, ConfigBuilder, ConfigPresets, EnvironmentReport};
pub use provider::{Provider, ProviderId, ProviderRegistry};

/// Main entry point for model gateway operations
///
/// This is the primary interface used by the `core` crate to interact
/// with AI providers. It coordinates configuration, provider registry,
/// and model catalog.
///
/// # Usage Example
///
/// ```rust,ignore
/// use peridot_model_gateway::ModelGateway;
///
/// // Create gateway with recommended models
/// let gateway = ModelGateway::with_recommended_models();
///
/// // Check if ready
/// if gateway.config_status().is_ready() {
///     println!("Ready to use AI features!");
/// }
/// ```
#[derive(Debug)]
pub struct ModelGateway {
    /// Registry of available providers
    registry: ProviderRegistry,
    /// Configuration for providers
    config: GatewayConfig,
    /// Catalog of available models
    catalog: ModelCatalog,
}

impl ModelGateway {
    /// Create a new model gateway with default configuration
    ///
    /// This initializes an empty gateway. Use `with_recommended_models()`
    /// to populate with built-in recommended models.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let gateway = ModelGateway::new();
    /// assert!(!gateway.has_any_provider());
    /// ```
    pub fn new() -> Self {
        Self {
            registry: ProviderRegistry::new(),
            config: GatewayConfig::new(),
            catalog: ModelCatalog::new(),
        }
    }

    /// Create a new model gateway with custom configuration
    ///
    /// Use this when loading configuration from a file.
    pub fn with_config(config: GatewayConfig) -> Self {
        let mut gateway = Self {
            registry: ProviderRegistry::new(),
            catalog: ModelCatalog::new(),
            config,
        };
        gateway.initialize_providers();
        gateway
    }

    /// Create a new model gateway with recommended models populated
    ///
    /// This is the recommended way to initialize for most use cases.
    /// It pre-populates the catalog with known good models for game scaffolding.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let gateway = ModelGateway::with_recommended_models();
    /// assert!(gateway.catalog().len() > 0);
    /// ```
    pub fn with_recommended_models() -> Self {
        Self {
            registry: ProviderRegistry::new(),
            config: GatewayConfig::new(),
            catalog: ModelCatalog::with_recommended(),
        }
    }

    /// Get the provider registry
    ///
    /// The registry tracks which providers are available.
    pub fn registry(&self) -> &ProviderRegistry {
        &self.registry
    }

    /// Get mutable access to the provider registry
    pub fn registry_mut(&mut self) -> &mut ProviderRegistry {
        &mut self.registry
    }

    /// Get the current configuration
    pub fn config(&self) -> &GatewayConfig {
        &self.config
    }

    /// Get mutable access to the configuration
    pub fn config_mut(&mut self) -> &mut GatewayConfig {
        &mut self.config
    }

    /// Get the model catalog
    ///
    /// The catalog contains metadata about available models.
    pub fn catalog(&self) -> &ModelCatalog {
        &self.catalog
    }

    /// Get mutable access to the model catalog
    pub fn catalog_mut(&mut self) -> &mut ModelCatalog {
        &mut self.catalog
    }

    /// Check if a provider is configured and ready to use
    ///
    /// A provider is "ready" if:
    /// - It is registered in the registry
    /// - It has configuration in the gateway config
    /// - The configuration has a valid API key
    pub fn is_provider_ready(&self, provider_id: &ProviderId) -> bool {
        self.registry.is_available(provider_id) && 
        self.config.get_provider(provider_id)
            .map(|c| c.is_valid())
            .unwrap_or(false)
    }

    /// Get the default provider if configured
    pub fn default_provider(&self) -> Option<ProviderId> {
        self.config.default_provider.clone()
    }

    /// Get the default model if configured
    pub fn default_model(&self) -> Option<ModelId> {
        self.config.default_model.clone()
    }

    /// Check if any provider is configured
    pub fn has_any_provider(&self) -> bool {
        self.config.default_provider.is_some() || !self.config.providers.is_empty()
    }

    /// Check if the gateway has a default model configured
    pub fn has_default_model(&self) -> bool {
        self.config.default_model.is_some()
    }

    /// Get the complete configuration status
    ///
    /// Returns a summary of what's configured for UI display.
    pub fn config_status(&self) -> ConfigStatus {
        ConfigStatus {
            has_provider: self.has_any_provider(),
            provider_ready: self.default_provider()
                .map(|p| self.is_provider_ready(&p))
                .unwrap_or(false),
            has_model: self.has_default_model(),
            provider_name: self.default_provider()
                .map(|p| p.to_string()),
            model_name: self.default_model()
                .map(|m| m.to_string()),
        }
    }

    /// Initialize providers from configuration
    ///
    /// This should be called after loading configuration to register
    /// all configured providers with the registry.
    pub fn initialize_providers(&mut self) {
        // Register all configured providers
        for (provider_id, _config) in &self.config.providers {
            self.registry.register(provider_id.clone());
        }
        
        // If we have a default provider, ensure it's registered
        if let Some(ref default) = self.config.default_provider {
            self.registry.register(default.clone());
        }
    }
}

impl Default for ModelGateway {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration status for UI display
#[derive(Debug, Clone)]
pub struct ConfigStatus {
    /// Whether any provider is configured
    pub has_provider: bool,
    /// Whether the default provider is ready
    pub provider_ready: bool,
    /// Whether a default model is selected
    pub has_model: bool,
    /// Name of the default provider (if any)
    pub provider_name: Option<String>,
    /// Name of the default model (if any)
    pub model_name: Option<String>,
}

impl ConfigStatus {
    /// Check if configuration is complete enough to use
    ///
    /// This means: provider configured + provider ready + model selected
    pub fn is_ready(&self) -> bool {
        self.has_provider && self.provider_ready && self.has_model
    }

    /// Get a human-readable status message
    pub fn message(&self) -> String {
        if self.is_ready() {
            format!(
                "Ready: {} / {}",
                self.provider_name.as_deref().unwrap_or("Unknown"),
                self.model_name.as_deref().unwrap_or("Unknown")
            )
        } else if !self.has_provider {
            "No provider configured".to_string()
        } else if !self.provider_ready {
            "Provider not ready - check API key".to_string()
        } else {
            "No model selected".to_string()
        }
    }
}

/// Errors specific to model gateway operations
#[derive(Debug, thiserror::Error)]
pub enum GatewayError {
    /// Provider not found or not available
    #[error("Provider not available: {0}")]
    ProviderNotAvailable(String),
    
    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    /// Credential resolution failed
    #[error("Failed to resolve credentials: {0}")]
    CredentialError(String),
    
    /// Inference request failed
    #[error("Inference failed: {0}")]
    InferenceError(String),
    
    /// Provider-specific error
    #[error("Provider error ({provider}): {message}")]
    ProviderError {
        /// Provider that failed
        provider: String,
        /// Error message
        message: String,
    },
    
    /// Model not found
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    
    /// Request validation error
    #[error("Invalid request: {0}")]
    ValidationError(String),
}

/// Result type for gateway operations
pub type GatewayResult<T> = Result<T, GatewayError>;

/// Version of the model gateway API
pub const API_VERSION: &str = "0.1.0";

/// Check if a provider is supported by this version
///
/// Returns true for providers that have scaffolding support
/// (even if not fully implemented yet).
pub fn is_provider_supported(provider_id: &ProviderId) -> bool {
    matches!(
        provider_id.as_str(),
        "openrouter" | "openai" | "anthropic" | "gemini" | "local"
    )
}