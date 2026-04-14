//! Credential resolution for API keys
//!
//! Supports:
//! - Direct API key strings (for testing)
//! - Environment variable references (e.g., "env:OPENROUTER_API_KEY")
//! - Future: secure credential stores

use crate::GatewayError;

/// Resolves credentials from various sources
#[derive(Debug, Clone)]
pub struct CredentialResolver;

impl CredentialResolver {
    /// Create a new credential resolver
    pub fn new() -> Self {
        CredentialResolver
    }

    /// Resolve an API key from a credential reference
    ///
    /// Supports formats:
    /// - `env:VAR_NAME` - Read from environment variable
    /// - `key:<actual_key>` - Use key directly (for testing)
    /// - `<raw_key>` - Use as-is (for simple configs)
    pub fn resolve(&self, reference: &str) -> Result<String, GatewayError> {
        if reference.starts_with("env:") {
            // Environment variable reference
            let var_name = &reference[4..];
            self.from_env(var_name)
        } else if reference.starts_with("key:") {
            // Direct key with prefix
            Ok(reference[4..].to_string())
        } else {
            // Assume it's the key directly
            Ok(reference.to_string())
        }
    }

    /// Read API key from environment variable
    pub fn from_env(&self, var_name: &str) -> Result<String, GatewayError> {
        std::env::var(var_name).map_err(|_| {
            GatewayError::CredentialError(format!("Environment variable '{}' not set", var_name))
        })
    }

    /// Check if a credential reference can be resolved without errors
    pub fn can_resolve(&self, reference: &str) -> bool {
        self.resolve(reference).is_ok()
    }

    /// Get the environment variable name from a reference (if applicable)
    pub fn get_env_var_name<'a>(&self, reference: &'a str) -> Option<&'a str> {
        if reference.starts_with("env:") {
            Some(&reference[4..])
        } else {
            None
        }
    }
}

impl Default for CredentialResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Common environment variable names for providers
pub mod env_vars {
    /// OpenRouter API key environment variable
    pub const OPENROUTER_API_KEY: &str = "OPENROUTER_API_KEY";
    /// OpenAI API key environment variable
    pub const OPENAI_API_KEY: &str = "OPENAI_API_KEY";
    /// Anthropic API key environment variable
    pub const ANTHROPIC_API_KEY: &str = "ANTHROPIC_API_KEY";
    /// Google/Gemini API key environment variable
    pub const GEMINI_API_KEY: &str = "GEMINI_API_KEY";

    /// Get the standard env var name for a provider
    pub fn for_provider(provider_id: &str) -> &'static str {
        match provider_id {
            "openrouter" => OPENROUTER_API_KEY,
            "openai" => OPENAI_API_KEY,
            "anthropic" => ANTHROPIC_API_KEY,
            "gemini" | "google" => GEMINI_API_KEY,
            _ => "API_KEY",
        }
    }
}
