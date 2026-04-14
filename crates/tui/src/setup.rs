//! Provider and Model Setup Flow
//!
//! Guides users through initial configuration when no provider is set up.
//!
//! # Flow
//!
//! 1. **Check Configuration** - On startup, check if provider is configured
//! 2. **Provider Selection** - Choose from available providers (prioritizing OpenRouter)
//! 3. **API Key Input** - Enter or reference API key
//! 4. **Validation** - Test the configuration
//! 5. **Model Selection** - Choose default model
//! 6. **Save Configuration** - Persist to config file

use peridot_model_gateway::{
    check_environment, ConfigManager, GatewayConfig, ModelId, ProviderConfig, ProviderId,
};

/// Current step in the setup flow
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetupStep {
    /// Not in setup
    None,
    /// Welcome to setup
    Welcome,
    /// Select provider
    SelectProvider,
    /// Enter API key
    EnterApiKey,
    /// Select model
    SelectModel,
    /// Validation in progress
    Validating,
    /// Setup complete
    Complete,
    /// Setup error
    Error,
}

impl SetupStep {
    /// Get display title for the step
    pub fn title(&self) -> &'static str {
        match self {
            SetupStep::None => "",
            SetupStep::Welcome => "Welcome to PeridotCode",
            SetupStep::SelectProvider => "Select Provider",
            SetupStep::EnterApiKey => "Enter API Key",
            SetupStep::SelectModel => "Select Model",
            SetupStep::Validating => "Validating...",
            SetupStep::Complete => "Setup Complete",
            SetupStep::Error => "Setup Error",
        }
    }

    /// Get help text for the step
    pub fn help_text(&self) -> &'static str {
        match self {
            SetupStep::None => "",
            SetupStep::Welcome => "PeridotCode needs an AI provider to generate game prototypes.",
            SetupStep::SelectProvider => {
                "Choose your AI provider. OpenRouter is recommended for access to multiple models."
            }
            SetupStep::EnterApiKey => {
                "Enter your API key or press 'e' to use an environment variable reference."
            }
            SetupStep::SelectModel => {
                "Choose your default model. You can change this later in settings."
            }
            SetupStep::Validating => "Testing your configuration...",
            SetupStep::Complete => "Your provider is configured and ready to use!",
            SetupStep::Error => {
                "There was a problem with your configuration. Please check your API key."
            }
        }
    }
}

/// Provider option for selection
#[derive(Debug, Clone)]
pub struct ProviderOption {
    /// Provider ID
    pub id: ProviderId,
    /// Display name
    pub name: String,
    /// Description
    pub description: String,
    /// Whether this is the recommended option
    pub recommended: bool,
    /// Environment variable name for API key
    pub env_var: String,
}

impl ProviderOption {
    /// Create OpenRouter option
    pub fn openrouter() -> Self {
        Self {
            id: ProviderId::openrouter(),
            name: "OpenRouter".to_string(),
            description: "Access Claude, GPT, and Gemini through one API (Recommended)".to_string(),
            recommended: true,
            env_var: "OPENROUTER_API_KEY".to_string(),
        }
    }

    /// Create OpenAI option
    pub fn openai() -> Self {
        Self {
            id: ProviderId::openai(),
            name: "OpenAI".to_string(),
            description: "Direct OpenAI API access (GPT-4, GPT-3.5)".to_string(),
            recommended: false,
            env_var: "OPENAI_API_KEY".to_string(),
        }
    }

    /// Create Anthropic option
    pub fn anthropic() -> Self {
        Self {
            id: ProviderId::anthropic(),
            name: "Anthropic".to_string(),
            description: "Direct Anthropic API access (Claude models)".to_string(),
            recommended: false,
            env_var: "ANTHROPIC_API_KEY".to_string(),
        }
    }

    /// Get all available provider options
    pub fn all() -> Vec<Self> {
        vec![Self::openrouter(), Self::openai(), Self::anthropic()]
    }

    /// Get the recommended provider
    pub fn recommended() -> Self {
        Self::openrouter()
    }
}

/// Model option for selection
#[derive(Debug, Clone)]
pub struct ModelOption {
    /// Model ID
    pub id: ModelId,
    /// Display name
    pub name: String,
    /// Description
    pub description: String,
    /// Context window size
    pub context_window: String,
    /// Whether this is recommended
    pub recommended: bool,
}

impl ModelOption {
    /// Get OpenRouter model options
    pub fn openrouter_models() -> Vec<Self> {
        vec![
            Self {
                id: ModelId::new("anthropic/claude-3.5-sonnet"),
                name: "Claude 3.5 Sonnet".to_string(),
                description: "Best quality for game scaffolding".to_string(),
                context_window: "200K".to_string(),
                recommended: true,
            },
            Self {
                id: ModelId::new("openai/gpt-4o-mini"),
                name: "GPT-4o Mini".to_string(),
                description: "Fast and cost-effective".to_string(),
                context_window: "128K".to_string(),
                recommended: false,
            },
            Self {
                id: ModelId::new("anthropic/claude-3-haiku"),
                name: "Claude 3 Haiku".to_string(),
                description: "Fast Claude model".to_string(),
                context_window: "200K".to_string(),
                recommended: false,
            },
            Self {
                id: ModelId::new("google/gemini-flash-1.5"),
                name: "Gemini Flash 1.5".to_string(),
                description: "Very large context window".to_string(),
                context_window: "1M".to_string(),
                recommended: false,
            },
        ]
    }

    /// Get OpenAI model options
    pub fn openai_models() -> Vec<Self> {
        vec![
            Self {
                id: ModelId::new("gpt-4o"),
                name: "GPT-4o".to_string(),
                description: "Latest GPT-4 model".to_string(),
                context_window: "128K".to_string(),
                recommended: true,
            },
            Self {
                id: ModelId::new("gpt-4o-mini"),
                name: "GPT-4o Mini".to_string(),
                description: "Fast and affordable".to_string(),
                context_window: "128K".to_string(),
                recommended: false,
            },
        ]
    }

    /// Get Anthropic model options
    pub fn anthropic_models() -> Vec<Self> {
        vec![
            Self {
                id: ModelId::new("claude-3-sonnet-20240229"),
                name: "Claude 3 Sonnet".to_string(),
                description: "Balanced performance".to_string(),
                context_window: "200K".to_string(),
                recommended: true,
            },
            Self {
                id: ModelId::new("claude-3-haiku-20240307"),
                name: "Claude 3 Haiku".to_string(),
                description: "Fast and efficient".to_string(),
                context_window: "200K".to_string(),
                recommended: false,
            },
        ]
    }

    /// Get models for a provider
    pub fn for_provider(provider: &ProviderId) -> Vec<Self> {
        match provider.as_str() {
            "openrouter" => Self::openrouter_models(),
            "openai" => Self::openai_models(),
            "anthropic" => Self::anthropic_models(),
            _ => Self::openrouter_models(),
        }
    }
}

/// Setup state manager
#[derive(Debug)]
pub struct SetupState {
    /// Current step
    pub step: SetupStep,
    /// Selected provider
    pub selected_provider: Option<ProviderOption>,
    /// Available providers
    pub provider_options: Vec<ProviderOption>,
    /// API key input
    pub api_key_input: String,
    /// Use environment variable
    pub use_env_var: bool,
    /// Selected model
    pub selected_model: Option<ModelOption>,
    /// Available models
    pub model_options: Vec<ModelOption>,
    /// Selection index
    pub selection_index: usize,
    /// Error message
    pub error_message: Option<String>,
    /// Configuration being built
    pub config: Option<GatewayConfig>,
}

impl SetupState {
    /// Create new setup state
    pub fn new() -> Self {
        Self {
            step: SetupStep::Welcome,
            selected_provider: None,
            provider_options: ProviderOption::all(),
            api_key_input: String::new(),
            use_env_var: true,
            selected_model: None,
            model_options: Vec::new(),
            selection_index: 0,
            error_message: None,
            config: None,
        }
    }

    /// Check if configuration exists and is valid
    pub fn is_configuration_needed() -> bool {
        match ConfigManager::initialize() {
            Ok(manager) => {
                let env_check = check_environment();
                !env_check.ready && !manager.is_valid()
            }
            Err(_) => true,
        }
    }

    /// Move to next step
    pub fn next_step(&mut self) {
        self.step = match self.step {
            SetupStep::Welcome => SetupStep::SelectProvider,
            SetupStep::SelectProvider => SetupStep::EnterApiKey,
            SetupStep::EnterApiKey => SetupStep::SelectModel,
            SetupStep::SelectModel => SetupStep::Validating,
            SetupStep::Validating => SetupStep::Complete,
            SetupStep::Complete | SetupStep::Error | SetupStep::None => SetupStep::None,
        };
        self.selection_index = 0;
    }

    /// Move to previous step
    pub fn previous_step(&mut self) {
        self.step = match self.step {
            SetupStep::SelectProvider => SetupStep::Welcome,
            SetupStep::EnterApiKey => SetupStep::SelectProvider,
            SetupStep::SelectModel => SetupStep::EnterApiKey,
            SetupStep::Validating | SetupStep::Complete | SetupStep::Error => {
                SetupStep::SelectModel
            }
            SetupStep::Welcome | SetupStep::None => SetupStep::Welcome,
        };
        self.error_message = None;
    }

    /// Select provider at current index
    pub fn select_provider(&mut self) {
        if let Some(provider) = self.provider_options.get(self.selection_index) {
            self.selected_provider = Some(provider.clone());
            self.model_options = ModelOption::for_provider(&provider.id);
            // Set default selection to recommended model
            if let Some((index, _)) = self
                .model_options
                .iter()
                .enumerate()
                .find(|(_, m)| m.recommended)
            {
                self.selection_index = index;
            } else {
                self.selection_index = 0;
            }
        }
    }

    /// Select model at current index
    pub fn select_model(&mut self) {
        if let Some(model) = self.model_options.get(self.selection_index) {
            self.selected_model = Some(model.clone());
        }
    }

    /// Build configuration from selections
    pub fn build_config(&self) -> Option<GatewayConfig> {
        let provider = self.selected_provider.as_ref()?;
        let model = self.selected_model.as_ref()?;

        let mut config = GatewayConfig::new();

        // Build API key reference
        let api_key = if self.use_env_var {
            format!("env:{}", provider.env_var)
        } else {
            format!("key:{}", self.api_key_input)
        };

        let provider_config = ProviderConfig {
            enabled: true,
            api_key: Some(api_key),
            base_url: None,
            default_model: Some(model.id.to_string()),
            timeout_seconds: 60,
            extra: std::collections::HashMap::new(),
        };

        config.set_provider(provider.id.clone(), provider_config);
        config.set_default_provider(provider.id.clone());
        config.set_default_model(model.id.clone());

        Some(config)
    }

    /// Save configuration
    pub fn save_config(&self) -> Result<(), String> {
        if let Some(config) = &self.config {
            let manager = ConfigManager::with_config(config.clone());
            manager.save().map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    /// Move selection up
    pub fn selection_up(&mut self) {
        if self.selection_index > 0 {
            self.selection_index -= 1;
        }
    }

    /// Move selection down
    pub fn selection_down(&mut self) {
        let max = match self.step {
            SetupStep::SelectProvider => self.provider_options.len(),
            SetupStep::SelectModel => self.model_options.len(),
            _ => 0,
        };
        if self.selection_index < max.saturating_sub(1) {
            self.selection_index += 1;
        }
    }

    /// Get current selection count
    pub fn selection_count(&self) -> usize {
        match self.step {
            SetupStep::SelectProvider => self.provider_options.len(),
            SetupStep::SelectModel => self.model_options.len(),
            _ => 0,
        }
    }

    /// Toggle env var mode
    pub fn toggle_env_var(&mut self) {
        self.use_env_var = !self.use_env_var;
    }

    /// Insert character into API key input
    pub fn insert_api_key_char(&mut self, ch: char) {
        self.api_key_input.push(ch);
    }

    /// Backspace in API key input
    pub fn api_key_backspace(&mut self) {
        self.api_key_input.pop();
    }

    /// Clear API key input
    pub fn clear_api_key(&mut self) {
        self.api_key_input.clear();
    }
}

impl Default for SetupState {
    fn default() -> Self {
        Self::new()
    }
}
