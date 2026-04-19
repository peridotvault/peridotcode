//! Model identity and capabilities
//!
//! Provides ModelId type and capability metadata for models.
//! This module is the foundation for model selection and cataloging.

use serde::{Deserialize, Serialize};

/// Unique identifier for a model within a provider
///
/// Model IDs are provider-specific strings like:
/// - OpenRouter: "anthropic/claude-3.5-sonnet"
/// - OpenAI: "gpt-4o"
/// - Anthropic: "claude-3-opus-20240229"
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModelId(pub String);

impl ModelId {
    /// Create a new model ID
    pub fn new<S: Into<String>>(id: S) -> Self {
        ModelId(id.into())
    }

    /// Get the model ID as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get the provider prefix if the ID has one (e.g., "anthropic/claude-3.5-sonnet" -> "anthropic")
    pub fn provider_prefix(&self) -> Option<&str> {
        self.0.split('/').next()
    }

    /// Get the model name without provider prefix
    pub fn model_name(&self) -> &str {
        self.0.split('/').last().unwrap_or(&self.0)
    }
}

impl AsRef<str> for ModelId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ModelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Capability flags for models
///
/// Used to describe what a model can do. Not all providers support
/// querying capabilities, so these may be manually curated for some models.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelCapability {
    /// Model supports chat/completion
    Chat,
    /// Model supports function calling/tool use
    FunctionCalling,
    /// Model supports JSON mode/structured output
    JsonMode,
    /// Model supports vision (image input)
    Vision,
    /// Model is fine-tuned for code
    Code,
    /// Model is suitable for game development scaffolding
    GameScaffolding,
}

impl ModelCapability {
    /// Get a human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            ModelCapability::Chat => "Chat completion",
            ModelCapability::FunctionCalling => "Function calling",
            ModelCapability::JsonMode => "JSON mode",
            ModelCapability::Vision => "Vision",
            ModelCapability::Code => "Code generation",
            ModelCapability::GameScaffolding => "Game scaffolding",
        }
    }
}

/// Model tier classification for UI guidance
///
/// PeridotCode organizes models into three tiers to help users make
/// informed choices without being overwhelmed by options.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Default,
)]
pub enum ModelTier {
    /// **Recommended**: Best models for PeridotCode workflows
    ///
    /// These models are:
    /// - Well-tested with PeridotCode templates
    /// - Provide good quality/cost balance
    /// - Reliable for game scaffolding tasks
    /// - Actively maintained by providers
    #[default]
    Recommended,

    /// **Supported**: Works but not primary recommendations
    ///
    /// These models:
    /// - Function correctly with PeridotCode
    /// - May have quirks or limitations
    /// - Could be more expensive or slower
    /// - Useful for specific use cases
    Supported,

    /// **Experimental**: New, untested, or limited availability
    ///
    /// These models:
    /// - May have unknown issues with PeridotCode
    /// - Could be preview/beta versions
    /// - Might be removed or changed by providers
    /// - Use for exploration only
    Experimental,
}

impl ModelTier {
    /// Get a human-readable label for the tier
    pub fn label(&self) -> &'static str {
        match self {
            ModelTier::Recommended => "★ Recommended",
            ModelTier::Supported => "✓ Supported",
            ModelTier::Experimental => "⚠ Experimental",
        }
    }

    /// Get a short single-character symbol
    pub fn symbol(&self) -> &'static str {
        match self {
            ModelTier::Recommended => "★",
            ModelTier::Supported => "✓",
            ModelTier::Experimental => "⚠",
        }
    }

    /// Get description of what this tier means
    pub fn description(&self) -> &'static str {
        match self {
            ModelTier::Recommended => "Best choice for most users",
            ModelTier::Supported => "Works well, specific use cases",
            ModelTier::Experimental => "Try at your own risk",
        }
    }

    /// Check if this tier is suitable for production use
    pub fn is_production_ready(&self) -> bool {
        matches!(self, ModelTier::Recommended | ModelTier::Supported)
    }
}

/// Model cost tier for basic cost awareness
///
/// Lightweight cost classification without complex pricing logic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CostTier {
    /// Low cost (~$0.10-0.50 per 1M tokens)
    Low,
    /// Moderate cost (~$0.50-5.00 per 1M tokens)
    #[default]
    Moderate,
    /// High cost (~$5.00+ per 1M tokens)
    High,
}

impl CostTier {
    /// Get human-readable cost label
    pub fn label(&self) -> &'static str {
        match self {
            CostTier::Low => "$ Low",
            CostTier::Moderate => "$$ Moderate",
            CostTier::High => "$$$ High",
        }
    }
}

/// Metadata about a model's capabilities and characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapabilities {
    /// Maximum context window (input + output tokens)
    pub context_window: usize,
    /// Maximum output tokens
    pub max_output_tokens: Option<usize>,
    /// Supported capabilities
    pub capabilities: Vec<ModelCapability>,
    /// Whether streaming is supported
    pub supports_streaming: bool,
    /// Whether the model is recommended for general use
    pub recommended: bool,
    /// Cost tier (relative, 1 = baseline) - DEPRECATED: Use cost_tier_enum instead
    pub cost_tier: u8,
    /// Model tier for UI guidance
    #[serde(default)]
    pub tier: ModelTier,
    /// Cost tier for basic cost awareness
    #[serde(default)]
    pub cost_tier_enum: CostTier,
    /// Why this model is recommended (or not)
    #[serde(default)]
    pub recommendation_reason: Option<String>,
    /// Tags for filtering and grouping
    #[serde(default)]
    pub tags: Vec<String>,
}

impl ModelCapabilities {
    /// Create default capabilities
    pub fn new(context_window: usize) -> Self {
        Self {
            context_window,
            max_output_tokens: None,
            capabilities: vec![ModelCapability::Chat],
            supports_streaming: true,
            recommended: false,
            cost_tier: 1,
            tier: ModelTier::Supported,
            cost_tier_enum: CostTier::Moderate,
            recommendation_reason: None,
            tags: Vec::new(),
        }
    }

    /// Check if model has a specific capability
    pub fn has(&self, capability: ModelCapability) -> bool {
        self.capabilities.contains(&capability)
    }

    /// Add a capability
    pub fn with_capability(mut self, capability: ModelCapability) -> Self {
        if !self.capabilities.contains(&capability) {
            self.capabilities.push(capability);
        }
        self
    }

    /// Set as recommended (sets tier to Recommended)
    pub fn recommended(mut self) -> Self {
        self.recommended = true;
        self.tier = ModelTier::Recommended;
        self
    }

    /// Set the model tier
    pub fn with_tier(mut self, tier: ModelTier) -> Self {
        self.tier = tier;
        // Also update legacy recommended flag for backward compatibility
        if tier == ModelTier::Recommended {
            self.recommended = true;
        }
        self
    }

    /// Set cost tier (legacy numeric)
    pub fn with_cost_tier(mut self, tier: u8) -> Self {
        self.cost_tier = tier;
        self
    }

    /// Set cost tier (enum)
    pub fn with_cost_tier_enum(mut self, tier: CostTier) -> Self {
        self.cost_tier_enum = tier;
        self
    }

    /// Set max output tokens
    pub fn with_max_output(mut self, tokens: usize) -> Self {
        self.max_output_tokens = Some(tokens);
        self
    }

    /// Set recommendation reason
    pub fn with_recommendation_reason(mut self, reason: impl Into<String>) -> Self {
        self.recommendation_reason = Some(reason.into());
        self
    }

    /// Add a tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
}

impl Default for ModelCapabilities {
    fn default() -> Self {
        Self::new(4096)
    }
}

/// Full model information for catalog entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDescriptor {
    /// Model identifier
    pub id: ModelId,
    /// Human-readable name
    pub name: String,
    /// Provider ID
    pub provider: super::ProviderId,
    /// Model capabilities
    pub capabilities: ModelCapabilities,
    /// Description of the model
    pub description: Option<String>,
    /// Version or date
    pub version: Option<String>,
}

impl ModelDescriptor {
    /// Create a new model descriptor
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        provider: super::ProviderId,
        context_window: usize,
    ) -> Self {
        Self {
            id: ModelId::new(id),
            name: name.into(),
            provider,
            capabilities: ModelCapabilities::new(context_window),
            description: None,
            version: None,
        }
    }

    /// Add a description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Add a capability
    pub fn with_capability(mut self, cap: ModelCapability) -> Self {
        self.capabilities = self.capabilities.with_capability(cap);
        self
    }

    /// Mark as recommended
    pub fn with_recommended(mut self) -> Self {
        self.capabilities = self.capabilities.recommended();
        self
    }

    /// Set model tier
    pub fn with_tier(mut self, tier: ModelTier) -> Self {
        self.capabilities = self.capabilities.with_tier(tier);
        self
    }

    /// Set cost tier
    pub fn with_cost_tier(mut self, tier: CostTier) -> Self {
        self.capabilities = self.capabilities.with_cost_tier_enum(tier);
        self
    }

    /// Set recommendation reason
    pub fn with_recommendation_reason(mut self, reason: impl Into<String>) -> Self {
        self.capabilities = self.capabilities.with_recommendation_reason(reason);
        self
    }

    /// Add a tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.capabilities = self.capabilities.with_tag(tag);
        self
    }

    /// Check if model supports game scaffolding
    pub fn is_suitable_for_games(&self) -> bool {
        self.capabilities.has(ModelCapability::GameScaffolding)
            || self.capabilities.context_window >= 8000
    }

    /// Get tier symbol for display
    pub fn tier_symbol(&self) -> &'static str {
        self.capabilities.tier.symbol()
    }

    /// Check if this is a recommended model
    pub fn is_recommended(&self) -> bool {
        self.capabilities.tier == ModelTier::Recommended
    }
}

/// Common models for game scaffolding organized by tier
///
/// # Model Organization
///
/// Models are organized into three tiers:
///
/// ## Recommended (★)
/// Best models for PeridotCode workflows:
/// - `claude-3.5-sonnet` - Best overall balance
/// - `gpt-4o-mini` - Fast and cost-effective
/// - `claude-3-haiku` - Quick iterations
/// - `gemini-flash-1.5` - Large context needs
///
/// ## Supported (✓)
/// Reliable alternatives:
/// - `gpt-4o` - High quality, higher cost
/// - `claude-3-opus` - Premium quality
/// - `claude-3-sonnet` - Good mid-range option
///
/// ## Experimental (⚠)
/// New or preview models:
/// - Latest preview models
/// - Beta features
/// - Use for testing only
pub mod recommended {
    use super::*;

    /// Get ★ Recommended models (OpenRouter)
    ///
    /// These are the best choices for most users:
    /// - Well-tested with PeridotCode
    /// - Good quality/cost balance
    /// - Reliable for game scaffolding
    pub fn openrouter_recommended() -> Vec<ModelDescriptor> {
        vec![
            // ★ Primary recommendation
            ModelDescriptor::new(
                "anthropic/claude-3.5-sonnet",
                "Claude 3.5 Sonnet",
                super::super::ProviderId::openrouter(),
                200_000,
            )
            .with_description("Best overall balance of quality and speed for game scaffolding")
            .with_capability(ModelCapability::GameScaffolding)
            .with_capability(ModelCapability::Code)
            .with_capability(ModelCapability::JsonMode)
            .with_tier(ModelTier::Recommended)
            .with_cost_tier(CostTier::Moderate)
            .with_recommendation_reason("Best overall: excellent quality at reasonable cost")
            .with_tag("balanced")
            .with_tag("latest"),
            // ★ Secondary variant for Claude 3.5 Sonnet (dashes)
            ModelDescriptor::new(
                "anthropic/claude-3-5-sonnet",
                "Claude 3.5 Sonnet (Alt)",
                super::super::ProviderId::openrouter(),
                200_000,
            )
            .with_description("Alternative slug for Claude 3.5 Sonnet")
            .with_capability(ModelCapability::GameScaffolding)
            .with_tier(ModelTier::Supported)
            .with_cost_tier(CostTier::Moderate)
            .with_tag("balanced"),
            // ★ Cost-effective option
            ModelDescriptor::new(
                "openai/gpt-4o-mini",
                "GPT-4o Mini",
                super::super::ProviderId::openrouter(),
                128_000,
            )
            .with_description("Fast and cost-effective for simple scaffolding tasks")
            .with_capability(ModelCapability::GameScaffolding)
            .with_capability(ModelCapability::Code)
            .with_tier(ModelTier::Recommended)
            .with_cost_tier(CostTier::Low)
            .with_recommendation_reason("Best value: fast and inexpensive for simple tasks")
            .with_tag("fast"),
            // ★ Fast iteration option
            ModelDescriptor::new(
                "anthropic/claude-3-haiku",
                "Claude 3 Haiku",
                super::super::ProviderId::openrouter(),
                200_000,
            )
            .with_description("Fast Claude model for quick iterations")
            .with_capability(ModelCapability::GameScaffolding)
            .with_capability(ModelCapability::Code)
            .with_tier(ModelTier::Recommended)
            .with_cost_tier(CostTier::Low)
            .with_recommendation_reason("Best for rapid prototyping and quick iterations")
            .with_tag("fast"),
            // ★ Newer fast option
            ModelDescriptor::new(
                "anthropic/claude-3.5-haiku",
                "Claude 3.5 Haiku",
                super::super::ProviderId::openrouter(),
                200_000,
            )
            .with_description("Latest fast Claude model with improved capabilities")
            .with_capability(ModelCapability::GameScaffolding)
            .with_capability(ModelCapability::Code)
            .with_tier(ModelTier::Recommended)
            .with_cost_tier(CostTier::Low)
            .with_recommendation_reason("Best speed/value ratio for most tasks")
            .with_tag("fast"),
        ]
    }

    /// Get ✓ Supported models (OpenRouter)
    ///
    /// These models work well but are not primary recommendations:
    /// - May be more expensive
    /// - Could be slower
    /// - Good for specific use cases
    pub fn openrouter_supported() -> Vec<ModelDescriptor> {
        vec![
            // ✓ High quality option
            ModelDescriptor::new(
                "anthropic/claude-3-opus",
                "Claude 3 Opus",
                super::super::ProviderId::openrouter(),
                200_000,
            )
            .with_description("Highest quality Claude model for demanding tasks")
            .with_capability(ModelCapability::GameScaffolding)
            .with_capability(ModelCapability::Code)
            .with_tier(ModelTier::Supported)
            .with_cost_tier(CostTier::High)
            .with_recommendation_reason("Use when you need maximum quality, accept higher cost")
            .with_tag("premium"),
            // ✓ Alternative mid-range
            ModelDescriptor::new(
                "openai/gpt-4o",
                "GPT-4o",
                super::super::ProviderId::openrouter(),
                128_000,
            )
            .with_description("High-quality GPT-4 model with good capabilities")
            .with_capability(ModelCapability::GameScaffolding)
            .with_capability(ModelCapability::Code)
            .with_capability(ModelCapability::Vision)
            .with_tier(ModelTier::Supported)
            .with_cost_tier(CostTier::High)
            .with_recommendation_reason("OpenAI's best model, higher cost but excellent quality")
            .with_tag("premium"),
            // ✓ Budget option
            ModelDescriptor::new(
                "openai/gpt-3.5-turbo",
                "GPT-3.5 Turbo",
                super::super::ProviderId::openrouter(),
                16_385,
            )
            .with_description("Older but reliable model for basic tasks")
            .with_capability(ModelCapability::GameScaffolding)
            .with_capability(ModelCapability::Code)
            .with_tier(ModelTier::Supported)
            .with_cost_tier(CostTier::Low)
            .with_recommendation_reason("Budget option for simple scaffolding, limited context")
            .with_tag("budget"),
        ]
    }

    /// Get ⚠ Experimental models (OpenRouter)
    ///
    /// These models are new, untested, or may have issues:
    /// - Use for exploration only
    /// - May be unstable
    /// - Could be removed or changed
    pub fn openrouter_experimental() -> Vec<ModelDescriptor> {
        vec![
            // ⚠ Preview/beta models
            // Add new preview models here as they become available
            // Example:
            // ModelDescriptor::new(
            //     "anthropic/claude-3-5-sonnet-20241022",
            //     "Claude 3.5 Sonnet (Preview)",
            //     ...
            // )
            // .with_tier(ModelTier::Experimental),
        ]
    }

    /// Get all OpenRouter models (legacy compatibility)
    #[deprecated(since = "0.1.0", note = "Use openrouter_recommended() instead")]
    pub fn openrouter_models() -> Vec<ModelDescriptor> {
        let mut models = openrouter_recommended();
        models.extend(openrouter_supported());
        models.extend(openrouter_experimental());
        models
    }

    /// Get default model for MVP
    pub fn mvp_default() -> ModelId {
        // Use the most standard OpenRouter slug for MVP
        ModelId::new("anthropic/claude-3.5-sonnet")
    }

    /// Find a model by partial name match
    pub fn find_by_name<'a>(
        models: &'a [ModelDescriptor],
        query: &str,
    ) -> Option<&'a ModelDescriptor> {
        let query_lower = query.to_lowercase();
        models.iter().find(|m| {
            m.name.to_lowercase().contains(&query_lower)
                || m.id.0.to_lowercase().contains(&query_lower)
        })
    }

    /// Get model selection guidance
    pub fn selection_guidance() -> &'static str {
        r#"Model Selection Guide:

★ Recommended - Start here:
  • Claude 3.5 Sonnet: Best overall balance (recommended)
  • GPT-4o Mini: Fastest and cheapest
  • Claude 3 Haiku: Best for quick iterations
  • Gemini Flash 1.5: Best for large projects

✓ Supported - For specific needs:
  • Claude 3 Opus: Maximum quality (expensive)
  • GPT-4o: OpenAI's best (higher cost)
  • GPT-3.5 Turbo: Budget option (limited features)

⚠ Experimental - Try at your own risk:
  • Preview and beta models
  • May have unknown issues
"#
    }
}
