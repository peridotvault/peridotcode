//! Configuration file management
//!
//! Handles loading and saving configuration from standard locations:
//!
//! ## Config File Locations
//!
//! - **Linux**: `~/.config/peridotcode/config.toml`
//! - **macOS**: `~/Library/Application Support/peridotcode/config.toml`
//! - **Windows**: `%APPDATA%\peridotcode\config.toml`
//!
//! ## Environment File Locations
//!
//! PeridotCode looks for `.env` files in the following order:
//! 1. Current working directory (project-specific)
//! 2. Config directory (user-specific)
//!
//! ## Configuration Format
//!
//! ```toml
//! # Default provider and model
//! default_provider = "openrouter"
//! default_model = "anthropic/claude-3.5-sonnet"
//!
//! [providers.openrouter]
//! enabled = true
//! api_key = "env:OPENROUTER_API_KEY"  # Reference to env var
//! base_url = "https://openrouter.ai/api/v1"
//! default_model = "anthropic/claude-3.5-sonnet"
//! timeout_seconds = 60
//!
//! [providers.openai]
//! enabled = false
//! api_key = "env:OPENAI_API_KEY"
//! ```

use crate::credentials::CredentialResolver;
use crate::{GatewayConfig, GatewayResult, ProviderConfig, ProviderId};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Manages configuration loading and persistence
#[derive(Debug)]
pub struct ConfigManager {
    /// Loaded configuration
    config: GatewayConfig,
    /// Path to the config file (if loaded from file)
    config_path: Option<PathBuf>,
    /// Credential resolver for expanding references
    credential_resolver: CredentialResolver,
    /// Whether dotenv was loaded
    dotenv_loaded: bool,
}

impl ConfigManager {
    /// Create a new config manager with empty configuration
    pub fn new() -> Self {
        Self {
            config: GatewayConfig::new(),
            config_path: None,
            credential_resolver: CredentialResolver::new(),
            dotenv_loaded: false,
        }
    }

    /// Create a config manager with an existing configuration
    pub fn with_config(config: GatewayConfig) -> Self {
        Self {
            config,
            config_path: None,
            credential_resolver: CredentialResolver::new(),
            dotenv_loaded: false,
        }
    }

    /// Load configuration from the standard location
    ///
    /// This attempts to load from the platform-specific config directory.
    /// If no config file exists, returns an empty configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the config file exists but cannot be parsed.
    pub fn load() -> GatewayResult<Self> {
        let config_path = Self::default_config_path()?;

        if config_path.exists() {
            Self::load_from_file(&config_path)
        } else {
            Ok(Self::new())
        }
    }

    /// Load configuration from a specific file path
    pub fn load_from_file(path: &Path) -> GatewayResult<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            crate::GatewayError::ConfigError(format!("Failed to read config file: {}", e))
        })?;

        let config: GatewayConfig = toml::from_str(&content).map_err(|e| {
            crate::GatewayError::ConfigError(format!("Failed to parse config file: {}", e))
        })?;

        Ok(Self {
            config,
            config_path: Some(path.to_path_buf()),
            credential_resolver: CredentialResolver::new(),
            dotenv_loaded: false,
        })
    }

    /// Create a config manager and load .env file
    ///
    /// This is the recommended way to initialize for CLI applications.
    /// It will:
    /// 1. Load .env from current directory (if exists)
    /// 2. Load config from standard location (if exists)
    pub fn initialize() -> GatewayResult<Self> {
        // Try to load .env files first
        let _ = dotenvy::dotenv(); // Ignore errors, .env is optional

        Self::load()
    }

    /// Get the current configuration
    pub fn config(&self) -> &GatewayConfig {
        &self.config
    }

    /// Get mutable access to configuration
    pub fn config_mut(&mut self) -> &mut GatewayConfig {
        &mut self.config
    }

    /// Get configuration status for UI display
    ///
    /// Returns a summary of what's configured.
    pub fn config_status(&self) -> crate::ConfigStatus {
        crate::ConfigStatus {
            has_provider: self.config.default_provider.is_some()
                || !self.config.providers.is_empty(),
            provider_ready: self.config.default_provider.as_ref().map_or(false, |p| {
                self.config
                    .get_provider(p)
                    .map_or(false, |cfg| cfg.is_valid())
            }),
            has_model: self.config.default_model.is_some(),
            provider_name: self.config.default_provider.as_ref().map(|p| p.to_string()),
            model_name: self.config.default_model.as_ref().map(|m| m.to_string()),
        }
    }

    /// Get the path to the config file (if loaded from file)
    pub fn config_path(&self) -> Option<&Path> {
        self.config_path.as_deref()
    }

    /// Set the config file path
    pub fn set_config_path(&mut self, path: PathBuf) {
        self.config_path = Some(path);
    }

    /// Save configuration to the current config file path
    ///
    /// If no path is set, saves to the default location.
    pub fn save(&self) -> GatewayResult<()> {
        let path = match &self.config_path {
            Some(p) => p.clone(),
            None => Self::default_config_path()?,
        };

        self.save_to_file(&path)
    }

    /// Save configuration to a specific file path
    pub fn save_to_file(&self, path: &Path) -> GatewayResult<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                crate::GatewayError::ConfigError(format!(
                    "Failed to create config directory: {}",
                    e
                ))
            })?;
        }

        let content = toml::to_string_pretty(&self.config).map_err(|e| {
            crate::GatewayError::ConfigError(format!("Failed to serialize config: {}", e))
        })?;

        std::fs::write(path, content).map_err(|e| {
            crate::GatewayError::ConfigError(format!("Failed to write config file: {}", e))
        })?;

        Ok(())
    }

    /// Resolve credentials for a provider
    ///
    /// This expands credential references like `env:VAR_NAME` to actual values.
    pub fn resolve_credentials(&self, provider_id: &ProviderId) -> GatewayResult<Option<String>> {
        let config = self.config.get_provider(provider_id).ok_or_else(|| {
            crate::GatewayError::ConfigError(format!("Provider '{}' not configured", provider_id))
        })?;

        match &config.api_key {
            Some(ref_key) => {
                let resolved = self
                    .credential_resolver
                    .resolve(ref_key)
                    .map_err(|e| crate::GatewayError::CredentialError(e.to_string()))?;
                Ok(Some(resolved))
            }
            None => Ok(None),
        }
    }

    /// Check if credentials for the default provider are valid and present
    pub fn validate_credentials(&self) -> GatewayResult<()> {
        let provider_id = self.config.default_provider.as_ref()
            .ok_or_else(|| crate::GatewayError::ConfigError("No default provider selected".to_string()))?;

        match self.resolve_credentials(provider_id) {
            Ok(Some(key)) if !key.trim().is_empty() => Ok(()),
            _ => Err(crate::GatewayError::CredentialError(format!(
                "No API key configured for {}. Set {}_API_KEY in .env or run setup again.",
                provider_id.as_str(),
                provider_id.as_str().to_uppercase()
            ))),
        }
    }

    /// Get the default config file path for the current platform
    pub fn default_config_path() -> GatewayResult<PathBuf> {
        let config_dir = dirs::config_dir().ok_or_else(|| {
            crate::GatewayError::ConfigError("Could not determine config directory".to_string())
        })?;

        Ok(config_dir.join("peridotcode").join("config.toml"))
    }

    /// Get the default data directory for the current platform
    ///
    /// This is where additional data like caches might be stored.
    pub fn default_data_dir() -> GatewayResult<PathBuf> {
        let data_dir = dirs::data_dir().ok_or_else(|| {
            crate::GatewayError::ConfigError("Could not determine data directory".to_string())
        })?;

        Ok(data_dir.join("peridotcode"))
    }

    /// Check if a configuration file exists at the default location
    pub fn config_exists() -> GatewayResult<bool> {
        let path = Self::default_config_path()?;
        Ok(path.exists())
    }

    /// Create a new configuration with defaults
    ///
    /// This creates a sensible default configuration for first-time setup.
    pub fn create_default() -> GatewayConfig {
        let mut config = GatewayConfig::new();

        // Set up OpenRouter as default
        let openrouter_id = ProviderId::openrouter();
        let openrouter_config = ProviderConfig {
            enabled: true,
            api_key: Some("env:OPENROUTER_API_KEY".to_string()),
            base_url: Some("https://openrouter.ai/api/v1".to_string()),
            default_model: Some("anthropic/claude-3.5-sonnet".to_string()),
            timeout_seconds: 60,
            extra: HashMap::new(),
        };

        config.set_provider(openrouter_id.clone(), openrouter_config);
        config.set_default_provider(openrouter_id);
        config.set_default_model(crate::ModelId::new("anthropic/claude-3.5-sonnet"));

        config
    }

    /// Validate the current configuration
    ///
    /// Returns a list of validation errors, or empty if valid.
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        // Check default provider
        if let Some(ref default) = self.config.default_provider {
            if !self.config.has_provider(default) {
                errors.push(format!("Default provider '{}' is not configured", default));
            }

            // Check that default provider has API key
            if let Some(provider_config) = self.config.get_provider(default) {
                if !provider_config.has_api_key() {
                    errors.push(format!(
                        "Default provider '{}' has no API key configured",
                        default
                    ));
                }
            }
        }

        // Check default model
        if let Some(ref model) = self.config.default_model {
            let model_str = model.to_string();
            if model_str.is_empty() {
                errors.push("Default model is empty".to_string());
            }
        }

        // Validate all provider configurations
        for (provider_id, provider_config) in &self.config.providers {
            if provider_config.enabled && !provider_config.has_api_key() {
                errors.push(format!("Enabled provider '{}' has no API key", provider_id));
            }
        }

        errors
    }

    /// Check if the configuration is valid
    pub fn is_valid(&self) -> bool {
        self.validate().is_empty()
    }

    /// Load and merge project-specific .env file
    ///
    /// This loads from the current working directory.
    pub fn load_project_env(&mut self) -> GatewayResult<()> {
        match dotenvy::from_path(".env") {
            Ok(_) => {
                self.dotenv_loaded = true;
                Ok(())
            }
            Err(dotenvy::Error::Io(io_err)) if io_err.kind() == std::io::ErrorKind::NotFound => {
                // .env not found is okay
                Ok(())
            }
            Err(e) => Err(crate::GatewayError::ConfigError(format!(
                "Failed to load .env file: {}",
                e
            ))),
        }
    }

    /// Get the resolved API key for a provider
    ///
    /// This resolves the credential reference and returns the actual key.
    pub fn get_api_key(&self, provider_id: &ProviderId) -> GatewayResult<Option<String>> {
        self.resolve_credentials(provider_id)
    }

    /// Check if all required environment variables are set
    ///
    /// Returns a map of provider_id -> env_var_name for missing variables.
    pub fn check_environment_variables(&self) -> HashMap<ProviderId, String> {
        let mut missing = HashMap::new();

        for (provider_id, config) in &self.config.providers {
            if let Some(api_key_ref) = &config.api_key {
                if let Some(env_var) = self.credential_resolver.get_env_var_name(api_key_ref) {
                    if std::env::var(env_var).is_err() {
                        missing.insert(provider_id.clone(), env_var.to_string());
                    }
                }
            }
        }

        missing
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_config_manager_new() {
        let manager = ConfigManager::new();
        assert!(manager.config().is_empty());
        assert!(manager.config_path().is_none());
    }

    #[test]
    fn test_config_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");

        // Create a config with OpenRouter
        let mut config = GatewayConfig::new();

        let provider_config = ProviderConfig {
            enabled: true,
            api_key: Some("env:OPENROUTER_API_KEY".to_string()),
            base_url: Some("https://openrouter.ai/api/v1".to_string()),
            default_model: Some("anthropic/claude-3.5-sonnet".to_string()),
            timeout_seconds: 60,
            extra: std::collections::HashMap::new(),
        };

        let provider_id = ProviderId::openrouter();
        config.set_provider(provider_id.clone(), provider_config);
        config.set_default_provider(provider_id.clone());
        config.set_default_model(crate::ModelId::new("anthropic/claude-3.5-sonnet"));

        let mut manager = ConfigManager::with_config(config);
        manager.set_config_path(config_path.clone());

        // Save the config
        manager.save().unwrap();
        assert!(config_path.exists());

        // Load the config back
        let loaded_manager = ConfigManager::load_from_file(&config_path).unwrap();

        // Verify the loaded config
        let status = loaded_manager.config_status();
        assert!(status.has_provider);
        assert!(status.has_model);
        assert_eq!(status.provider_name.as_deref(), Some("openrouter"));
    }

    #[test]
    fn test_credential_resolution() {
        let mut config = GatewayConfig::new();

        // Set up provider with env var reference
        let provider_config = ProviderConfig {
            enabled: true,
            api_key: Some("env:TEST_API_KEY_VAR".to_string()),
            base_url: None,
            default_model: None,
            timeout_seconds: 60,
            extra: std::collections::HashMap::new(),
        };

        let provider_id = ProviderId::openrouter();
        config.set_provider(provider_id.clone(), provider_config);
        let manager = ConfigManager::with_config(config);

        // Set the environment variable
        std::env::set_var("TEST_API_KEY_VAR", "test-secret-key");

        // Resolve credentials
        let resolved = manager.resolve_credentials(&provider_id).unwrap();
        assert_eq!(resolved, Some("test-secret-key".to_string()));

        // Clean up
        std::env::remove_var("TEST_API_KEY_VAR");
    }

    #[test]
    fn test_credential_resolution_missing_env() {
        let mut config = GatewayConfig::new();

        // Set up provider with env var that doesn't exist
        let provider_config = ProviderConfig {
            enabled: true,
            api_key: Some("env:NONEXISTENT_VAR_12345".to_string()),
            base_url: None,
            default_model: None,
            timeout_seconds: 60,
            extra: std::collections::HashMap::new(),
        };

        let provider_id = ProviderId::openrouter();
        config.set_provider(provider_id.clone(), provider_config);
        let manager = ConfigManager::with_config(config);

        // Try to resolve - should fail
        let result = manager.resolve_credentials(&provider_id);
        assert!(result.is_err());

        match result {
            Err(crate::GatewayError::CredentialError(msg)) => {
                assert!(msg.contains("NONEXISTENT_VAR_12345"));
            }
            _ => panic!("Expected CredentialError for missing env var"),
        }
    }

    #[test]
    fn test_config_validation() {
        // Empty config - no providers configured, but that's not a validation error
        // Validation errors are for misconfigurations, not missing config
        let manager = ConfigManager::new();
        let errors = manager.validate();
        // Empty config has no validation errors (just no configuration)
        assert!(errors.is_empty());

        // Add a provider without API key
        let mut config = GatewayConfig::new();
        let provider_config = ProviderConfig::new(); // No API key
        let provider_id = ProviderId::openrouter();
        config.set_provider(provider_id, provider_config);
        config.set_default_provider(ProviderId::openrouter());

        let manager = ConfigManager::with_config(config);

        // Should have validation errors because default provider has no API key
        let errors = manager.validate();
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.contains("API key")));
        assert!(!manager.is_valid());
    }

    #[test]
    fn test_check_environment_variables() {
        let mut config = GatewayConfig::new();

        let provider_config = ProviderConfig {
            enabled: true,
            api_key: Some("env:MISSING_ENV_VAR".to_string()),
            base_url: None,
            default_model: None,
            timeout_seconds: 60,
            extra: std::collections::HashMap::new(),
        };

        let provider_id = ProviderId::openrouter();
        config.set_provider(provider_id.clone(), provider_config);
        let manager = ConfigManager::with_config(config);

        // Check environment variables
        let missing = manager.check_environment_variables();
        assert!(missing.contains_key(&provider_id));
        assert_eq!(missing.get(&provider_id).unwrap(), "MISSING_ENV_VAR");
    }

    #[test]
    fn test_create_default_config() {
        let config = ConfigManager::create_default();

        assert!(config.default_provider.is_some());
        assert!(config.default_model.is_some());
        assert!(!config.providers.is_empty());

        let provider_id = ProviderId::openrouter();
        let provider_config = config.get_provider(&provider_id);
        assert!(provider_config.is_some());

        let provider_config = provider_config.unwrap();
        assert!(provider_config.enabled);
        assert!(provider_config.api_key.is_some());
    }
}

/// Configuration sources in order of precedence (highest first)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigSource {
    /// Command-line argument
    CommandLine,
    /// Environment variable
    Environment,
    /// Project .env file
    ProjectEnv,
    /// User config file
    UserConfig,
    /// Default values
    Default,
}

/// Helper to create initial configuration interactively
///
/// This can be used by the TUI to guide first-time setup.
pub mod interactive {
    use super::*;

    /// Configuration choices for interactive setup
    #[derive(Debug, Clone)]
    pub struct SetupChoices {
        /// Selected provider
        pub provider: ProviderId,
        /// API key or reference
        pub api_key: String,
        /// Selected model
        pub model: String,
        /// Use env var reference instead of direct key
        pub use_env_var: bool,
    }

    impl SetupChoices {
        /// Create setup choices for OpenRouter
        pub fn openrouter(api_key: impl Into<String>, model: impl Into<String>) -> Self {
            Self {
                provider: ProviderId::openrouter(),
                api_key: api_key.into(),
                model: model.into(),
                use_env_var: true,
            }
        }

        /// Convert to gateway configuration
        pub fn to_config(&self) -> GatewayConfig {
            let mut config = GatewayConfig::new();

            let api_key = if self.use_env_var {
                format!("env:{}", self.provider.default_env_var())
            } else {
                format!("key:{}", self.api_key)
            };

            let provider_config = ProviderConfig {
                enabled: true,
                api_key: Some(api_key),
                base_url: None,
                default_model: Some(self.model.clone()),
                timeout_seconds: 60,
                extra: HashMap::new(),
            };

            config.set_provider(self.provider.clone(), provider_config);
            config.set_default_provider(self.provider.clone());
            config.set_default_model(crate::ModelId::new(&self.model));

            config
        }
    }

    /// Get provider options for interactive selection
    pub fn provider_options() -> Vec<(ProviderId, &'static str, &'static str)> {
        vec![
            (
                ProviderId::openrouter(),
                "OpenRouter",
                "Access multiple AI models through one API (recommended)",
            ),
            // TODO: Enable these when fully implemented
            // (ProviderId::openai(), "OpenAI", "Direct OpenAI API access"),
            // (
            //     ProviderId::anthropic(),
            //     "Anthropic",
            //     "Direct Anthropic Claude API access",
            // ),
        ]
    }

    /// Get model options for a provider
    pub fn model_options(provider: &ProviderId) -> Vec<(&'static str, &'static str)> {
        match provider.as_str() {
            "openrouter" => vec![
                (
                    "anthropic/claude-3.5-sonnet",
                    "Claude 3.5 Sonnet (Recommended)",
                ),
                ("openai/gpt-4o-mini", "GPT-4o Mini (Fast & Cheap)"),
                ("anthropic/claude-3-haiku", "Claude 3 Haiku (Fast)"),
            ],
            "openai" => vec![
                ("gpt-4o", "GPT-4o"),
                ("gpt-4o-mini", "GPT-4o Mini"),
                ("gpt-3.5-turbo", "GPT-3.5 Turbo"),
            ],
            "anthropic" => vec![
                ("claude-3-opus-20240229", "Claude 3 Opus"),
                ("claude-3-sonnet-20240229", "Claude 3 Sonnet"),
                ("claude-3-haiku-20240307", "Claude 3 Haiku"),
            ],
            _ => vec![],
        }
    }
}
