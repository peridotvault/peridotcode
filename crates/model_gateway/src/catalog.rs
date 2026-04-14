//! Model catalog for organizing available models
//!
//! The catalog provides a unified view of models across different providers.
//! It supports filtering, searching, and organizing models by capability and tier.
//!
//! # Model Tiers
//!
//! Models are organized into three tiers to help users make informed choices:
//!
//! - **Recommended** (★): Best models for PeridotCode workflows
//!   - Well-tested with templates
//!   - Good quality/cost balance
//!   - Reliable for game scaffolding
//!
//! - **Supported** (✓): Works but not primary recommendations
//!   - Function correctly
//!   - May have quirks
//!   - Good for specific use cases
//!
//! - **Experimental** (⚠): New, untested, or limited availability
//!   - May have unknown issues
//!   - Could be preview/beta
//!   - Use for exploration only

use crate::model::{ModelCapability, ModelDescriptor, ModelId, ModelTier};
use crate::provider::ProviderId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A catalog of available models
///
/// The catalog can be populated from:
/// - Built-in recommended models
/// - Provider API queries (for dynamic model lists)
/// - User configuration
///
/// Models are organized by tier (Recommended, Supported, Experimental)
/// to help users navigate options effectively.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelCatalog {
    /// Models indexed by ID
    models: HashMap<ModelId, ModelDescriptor>,
    /// Provider-specific model lists (for quick filtering)
    by_provider: HashMap<ProviderId, Vec<ModelId>>,
    /// Capability-based indexes
    by_capability: HashMap<ModelCapability, Vec<ModelId>>,
    /// Tier-based indexes
    by_tier: HashMap<ModelTier, Vec<ModelId>>,
}

impl ModelCatalog {
    /// Create an empty catalog
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
            by_provider: HashMap::new(),
            by_capability: HashMap::new(),
            by_tier: HashMap::new(),
        }
    }

    /// Create a catalog with built-in recommended models
    pub fn with_recommended() -> Self {
        let mut catalog = Self::new();

        // Add OpenRouter models from all tiers
        for model in crate::model::recommended::openrouter_recommended() {
            catalog.add(model);
        }
        for model in crate::model::recommended::openrouter_supported() {
            catalog.add(model);
        }
        for model in crate::model::recommended::openrouter_experimental() {
            catalog.add(model);
        }

        catalog
    }

    /// Create a catalog with only recommended models
    pub fn with_recommended_only() -> Self {
        let mut catalog = Self::new();

        for model in crate::model::recommended::openrouter_recommended() {
            catalog.add(model);
        }

        catalog
    }

    /// Add a model to the catalog
    pub fn add(&mut self, model: ModelDescriptor) {
        let id = model.id.clone();
        let provider = model.provider.clone();
        let capabilities = model.capabilities.capabilities.clone();
        let tier = model.capabilities.tier;

        // Add to main index
        self.models.insert(id.clone(), model);

        // Add to provider index
        self.by_provider
            .entry(provider)
            .or_default()
            .push(id.clone());

        // Add to capability indexes
        for cap in capabilities {
            self.by_capability.entry(cap).or_default().push(id.clone());
        }

        // Add to tier index
        self.by_tier.entry(tier).or_default().push(id.clone());
    }

    /// Get a model by ID
    pub fn get(&self, id: &ModelId) -> Option<&ModelDescriptor> {
        self.models.get(id)
    }

    /// Check if a model exists
    pub fn contains(&self, id: &ModelId) -> bool {
        self.models.contains_key(id)
    }

    /// List all models
    pub fn all(&self) -> Vec<&ModelDescriptor> {
        self.models.values().collect()
    }

    /// Get count of models
    pub fn len(&self) -> usize {
        self.models.len()
    }

    /// Check if catalog is empty
    pub fn is_empty(&self) -> bool {
        self.models.is_empty()
    }

    /// List models for a specific provider
    pub fn for_provider(&self, provider: &ProviderId) -> Vec<&ModelDescriptor> {
        self.by_provider
            .get(provider)
            .map(|ids| ids.iter().filter_map(|id| self.models.get(id)).collect())
            .unwrap_or_default()
    }

    /// List models with a specific capability
    pub fn with_capability(&self, capability: ModelCapability) -> Vec<&ModelDescriptor> {
        self.by_capability
            .get(&capability)
            .map(|ids| ids.iter().filter_map(|id| self.models.get(id)).collect())
            .unwrap_or_default()
    }

    /// List models suitable for game scaffolding
    pub fn for_game_scaffolding(&self) -> Vec<&ModelDescriptor> {
        self.with_capability(ModelCapability::GameScaffolding)
    }

    /// Get recommended models (★ Recommended tier)
    pub fn recommended(&self) -> Vec<&ModelDescriptor> {
        self.by_tier
            .get(&ModelTier::Recommended)
            .map(|ids| ids.iter().filter_map(|id| self.models.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get supported models (✓ Supported tier)
    pub fn supported(&self) -> Vec<&ModelDescriptor> {
        self.by_tier
            .get(&ModelTier::Supported)
            .map(|ids| ids.iter().filter_map(|id| self.models.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get experimental models (⚠ Experimental tier)
    pub fn experimental(&self) -> Vec<&ModelDescriptor> {
        self.by_tier
            .get(&ModelTier::Experimental)
            .map(|ids| ids.iter().filter_map(|id| self.models.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get models by tier
    pub fn by_tier(&self, tier: ModelTier) -> Vec<&ModelDescriptor> {
        match tier {
            ModelTier::Recommended => self.recommended(),
            ModelTier::Supported => self.supported(),
            ModelTier::Experimental => self.experimental(),
        }
    }

    /// Get production-ready models (Recommended + Supported)
    pub fn production_ready(&self) -> Vec<&ModelDescriptor> {
        let mut models = self.recommended();
        models.extend(self.supported());
        models
    }

    /// Search models by name or ID
    pub fn search(&self, query: &str) -> Vec<&ModelDescriptor> {
        let query_lower = query.to_lowercase();
        self.models
            .values()
            .filter(|m| {
                m.name.to_lowercase().contains(&query_lower)
                    || m.id.0.to_lowercase().contains(&query_lower)
            })
            .collect()
    }

    /// Remove a model from the catalog
    pub fn remove(&mut self, id: &ModelId) -> Option<ModelDescriptor> {
        self.models.remove(id).map(|model| {
            // Clean up provider index
            if let Some(ids) = self.by_provider.get_mut(&model.provider) {
                ids.retain(|i| i != id);
            }

            // Clean up capability indexes
            for cap in &model.capabilities.capabilities {
                if let Some(ids) = self.by_capability.get_mut(cap) {
                    ids.retain(|i| i != id);
                }
            }

            // Clean up tier index
            if let Some(ids) = self.by_tier.get_mut(&model.capabilities.tier) {
                ids.retain(|i| i != id);
            }

            model
        })
    }

    /// Clear all models
    pub fn clear(&mut self) {
        self.models.clear();
        self.by_provider.clear();
        self.by_capability.clear();
        self.by_tier.clear();
    }

    /// Merge another catalog into this one
    pub fn merge(&mut self, other: &ModelCatalog) {
        for model in other.all() {
            self.add(model.clone());
        }
    }

    /// Get the default model for MVP
    pub fn mvp_default(&self) -> Option<&ModelDescriptor> {
        let default_id = crate::model::recommended::mvp_default();
        self.get(&default_id)
    }
}

/// Filter options for querying the catalog
#[derive(Debug, Clone, Default)]
pub struct ModelFilter {
    /// Filter by provider
    pub provider: Option<ProviderId>,
    /// Filter by capability
    pub capability: Option<ModelCapability>,
    /// Filter by tier
    pub tier: Option<ModelTier>,
    /// Only recommended models (legacy, use tier instead)
    pub recommended_only: bool,
    /// Exclude experimental models
    pub exclude_experimental: bool,
    /// Minimum context window
    pub min_context: Option<usize>,
}

impl ModelFilter {
    /// Create a new filter
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by provider
    pub fn provider(mut self, provider: ProviderId) -> Self {
        self.provider = Some(provider);
        self
    }

    /// Filter by capability
    pub fn capability(mut self, capability: ModelCapability) -> Self {
        self.capability = Some(capability);
        self
    }

    /// Filter by tier
    pub fn tier(mut self, tier: ModelTier) -> Self {
        self.tier = Some(tier);
        self
    }

    /// Only show recommended models (★ Recommended tier)
    pub fn recommended(mut self) -> Self {
        self.tier = Some(ModelTier::Recommended);
        self.recommended_only = true;
        self
    }

    /// Only show production-ready models (Recommended + Supported)
    pub fn production_ready(mut self) -> Self {
        self.exclude_experimental = true;
        self
    }

    /// Minimum context window
    pub fn min_context(mut self, tokens: usize) -> Self {
        self.min_context = Some(tokens);
        self
    }

    /// Apply filter to a catalog
    pub fn apply<'a>(&self, catalog: &'a ModelCatalog) -> Vec<&'a ModelDescriptor> {
        let mut results = catalog.all();

        if let Some(ref provider) = self.provider {
            results = catalog.for_provider(provider);
        }

        if let Some(capability) = self.capability {
            results = catalog.with_capability(capability);
        }

        // Handle tier filtering
        if let Some(tier) = self.tier {
            results = results
                .into_iter()
                .filter(|m| m.capabilities.tier == tier)
                .collect();
        } else if self.exclude_experimental {
            results = results
                .into_iter()
                .filter(|m| m.capabilities.tier != ModelTier::Experimental)
                .collect();
        }

        // Legacy recommended filter
        if self.recommended_only && self.tier.is_none() {
            results = results
                .into_iter()
                .filter(|m| m.capabilities.recommended)
                .collect();
        }

        if let Some(min_ctx) = self.min_context {
            results = results
                .into_iter()
                .filter(|m| m.capabilities.context_window >= min_ctx)
                .collect();
        }

        results
    }
}
