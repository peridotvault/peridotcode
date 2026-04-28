//! Provider abstraction and registry
//!
//! Defines the Provider trait and registry for managing available providers.
//!
//! # Implementing a Provider
//!
//! To add a new provider, implement the `Provider` trait:
//!
//! ```rust,ignore
//! use peridot_model_gateway::{
//!     Provider, ProviderId, InferenceRequest, InferenceResponse,
//!     GatewayResult, ModelInfo,
//! };
//!
//! #[derive(Debug)]
//! pub struct MyProvider {
//!     config: ProviderConfig,
//!     http_client: reqwest::Client,
//! }
//!
//! #[async_trait::async_trait]
//! impl Provider for MyProvider {
//!     fn id(&self) -> ProviderId {
//!         ProviderId::new("myprovider")
//!     }
//!
//!     fn name(&self) -> &str {
//!         "My Provider"
//!     }
//!
//!     fn is_configured(&self) -> bool {
//!         self.config.api_key.is_some()
//!     }
//!
//!     async fn infer(&self, request: InferenceRequest) -> GatewayResult<InferenceResponse> {
//!         // Implementation here
//!     }
//!
//!     async fn list_models(&self) -> GatewayResult<Vec<ModelInfo>> {
//!         // Implementation here
//!     }
//! }
//! ```

use crate::inference::{InferenceRequest, InferenceResponse};
use crate::model::ModelId;
use crate::{GatewayError, GatewayResult};
use serde::{Deserialize, Serialize};

/// Unique identifier for a provider
///
/// Providers are identified by short lowercase strings like:
/// - "openrouter"
/// - "openai"
/// - "anthropic"
/// - "gemini"
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProviderId(pub String);

impl ProviderId {
    /// Create a new provider ID
    pub fn new<S: Into<String>>(id: S) -> Self {
        ProviderId(id.into())
    }

    /// Get the provider ID as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// OpenRouter provider ID (MVP priority)
    pub fn openrouter() -> Self {
        ProviderId("openrouter".to_string())
    }

    /// OpenAI provider ID
    pub fn openai() -> Self {
        ProviderId("openai".to_string())
    }

    /// Anthropic provider ID
    pub fn anthropic() -> Self {
        ProviderId("anthropic".to_string())
    }

    /// Google Gemini provider ID
    pub fn gemini() -> Self {
        ProviderId("gemini".to_string())
    }

    /// Groq provider ID
    pub fn groq() -> Self {
        ProviderId("groq".to_string())
    }

    /// Local/Ollama provider ID (future)
    pub fn local() -> Self {
        ProviderId("local".to_string())
    }

    /// Get the default API key environment variable name for this provider
    pub fn default_env_var(&self) -> String {
        format!("{}_API_KEY", self.0.to_uppercase())
    }
}

impl AsRef<str> for ProviderId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ProviderId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Trait for AI model providers
///
/// Implementors handle the specifics of communicating with each provider's API
/// while presenting a unified interface to the rest of the system.
///
/// # Design Notes
///
/// - All methods are async to accommodate network calls
/// - The trait is object-safe (no generics)
/// - Implementations should be Send + Sync for use across tasks
#[async_trait::async_trait]
pub trait Provider: Send + Sync + std::fmt::Debug {
    /// Get the provider ID
    fn id(&self) -> ProviderId;

    /// Get the provider display name
    fn name(&self) -> &str;

    /// Check if the provider is properly configured and ready
    ///
    /// This should verify that:
    /// - API key is present
    /// - Base URL is valid (if applicable)
    /// - The provider can be reached (optional, can be done lazily)
    fn is_configured(&self) -> bool;

    /// Perform an inference request
    ///
    /// This is the main method for sending prompts to models.
    /// Implementations should:
    /// - Transform the normalized request into provider-specific format
    /// - Handle authentication
    /// - Parse the response into normalized format
    /// - Handle errors appropriately
    async fn infer(&self, request: InferenceRequest) -> GatewayResult<InferenceResponse>;

    /// Validate that the provider can be reached and credentials are valid
    ///
    /// This should perform a minimal non-destructive request (like listing models
    /// or checking auth status) to ensure the API key works.
    async fn validate_credentials(&self) -> GatewayResult<()>;

    /// List available models from this provider
    ///
    /// This is optional for MVP - can return a static list if the provider
    /// doesn't support dynamic model listing.
    async fn list_models(&self) -> GatewayResult<Vec<ModelInfo>>;

    /// Get information about a specific model
    ///
    /// Default implementation falls back to list_models.
    async fn get_model(&self, model_id: &ModelId) -> GatewayResult<Option<ModelInfo>> {
        let models = self.list_models().await?;
        Ok(models.into_iter().find(|m| m.id == model_id.0))
    }

    /// Validate that a model ID is valid for this provider
    ///
    /// Default implementation checks against list_models.
    async fn validate_model(&self, model_id: &ModelId) -> GatewayResult<bool> {
        Ok(self.get_model(model_id).await?.is_some())
    }
}

/// Information about an available model
///
/// This is a simplified view for provider responses.
/// For full model metadata, see `ModelDescriptor` in the `model` module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Model identifier (provider-specific)
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Provider that hosts this model
    pub provider: ProviderId,
    /// Context window size (if known)
    pub context_window: Option<usize>,
    /// Whether this is a recommended/default model
    pub recommended: bool,
}

impl ModelInfo {
    /// Create new model info
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        provider: ProviderId,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            provider,
            context_window: None,
            recommended: false,
        }
    }

    /// Set context window
    pub fn with_context_window(mut self, tokens: usize) -> Self {
        self.context_window = Some(tokens);
        self
    }

    /// Mark as recommended
    pub fn recommended(mut self) -> Self {
        self.recommended = true;
        self
    }
}

/// Registry of available providers
///
/// The registry tracks which providers are available but does not store
/// provider instances (those are created on-demand from configuration).
#[derive(Debug, Default, Clone)]
pub struct ProviderRegistry {
    provider_ids: Vec<ProviderId>,
    /// Static model lists for providers that don't support dynamic listing
    static_models: std::collections::HashMap<ProviderId, Vec<ModelInfo>>,
}

impl ProviderRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            provider_ids: Vec::new(),
            static_models: std::collections::HashMap::new(),
        }
    }

    /// Register a provider ID
    pub fn register(&mut self, id: ProviderId) {
        if !self.provider_ids.contains(&id) {
            self.provider_ids.push(id);
        }
    }

    /// Check if a provider is available
    pub fn is_available(&self, id: &ProviderId) -> bool {
        self.provider_ids.contains(id)
    }

    /// List all registered provider IDs
    pub fn list_providers(&self) -> &[ProviderId] {
        &self.provider_ids
    }

    /// Get count of registered providers
    pub fn len(&self) -> usize {
        self.provider_ids.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.provider_ids.is_empty()
    }

    /// Remove a provider from the registry
    pub fn unregister(&mut self, id: &ProviderId) {
        self.provider_ids.retain(|p| p != id);
        self.static_models.remove(id);
    }

    /// Register a static model list for a provider
    ///
    /// This is useful for providers that don't support dynamic model listing
    /// or for curated model lists.
    pub fn register_static_models(&mut self, provider: ProviderId, models: Vec<ModelInfo>) {
        self.static_models.insert(provider, models);
    }

    /// Get static models for a provider
    pub fn get_static_models(&self, provider: &ProviderId) -> Option<&Vec<ModelInfo>> {
        self.static_models.get(provider)
    }

    /// Get default MVP providers (OpenRouter first)
    pub fn mvp_providers() -> Vec<ProviderId> {
        vec![
            ProviderId::openrouter(),
            ProviderId::groq(),
        ]
    }

    /// Create a registry pre-populated with MVP providers
    pub fn with_mvp_providers() -> Self {
        let mut registry = Self::new();
        for provider in Self::mvp_providers() {
            registry.register(provider);
        }
        registry
    }
}

/// Placeholder provider for scaffolding (not functional)
///
/// This is used as a stub when a provider is configured but the actual
/// implementation is not yet available. It returns errors for all operations.
#[derive(Debug, Clone)]
pub struct PlaceholderProvider {
    id: ProviderId,
    name: String,
}

impl PlaceholderProvider {
    /// Create a new placeholder provider
    pub fn new(id: ProviderId) -> Self {
        let name = format!("{} (placeholder)", id);
        Self { id, name }
    }

    /// Create OpenRouter placeholder
    pub fn openrouter() -> Self {
        Self::new(ProviderId::openrouter())
    }
}

#[async_trait::async_trait]
impl Provider for PlaceholderProvider {
    fn id(&self) -> ProviderId {
        self.id.clone()
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn is_configured(&self) -> bool {
        false
    }

    async fn infer(&self, _request: InferenceRequest) -> GatewayResult<InferenceResponse> {
        Err(GatewayError::ProviderNotAvailable(self.id.to_string()))
    }

    async fn validate_credentials(&self) -> GatewayResult<()> {
        Err(GatewayError::ProviderNotAvailable(self.id.to_string()))
    }

    async fn list_models(&self) -> GatewayResult<Vec<ModelInfo>> {
        Err(GatewayError::ProviderNotAvailable(self.id.to_string()))
    }
}

// Re-export async_trait for implementors
pub use async_trait::async_trait;