use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::{GatewayResult, UsageStats};

/// Tracks token usage by model
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct UsageTracker {
    /// Total tokens used per model
    pub model_usage: HashMap<String, UsageStats>,
    /// Last updated timestamp (optional, for future)
    pub last_updated: Option<String>,
}

impl UsageTracker {
    /// Load from default location
    pub fn load_default() -> Self {
        let path = Self::default_path();
        Self::load_from_file(&path).unwrap_or_default()
    }

    /// Save to default location
    pub fn save_default(&self) -> GatewayResult<()> {
        let path = Self::default_path();
        self.save_to_file(&path)
    }

    fn default_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from(".config"));
        path.push("peridotcode");
        path.push("usage.json");
        path
    }

    /// Load tracking data from a specific file path
    pub fn load_from_file(path: &Path) -> GatewayResult<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(path).map_err(|e| {
            crate::GatewayError::ConfigError(format!("Failed to read usage file: {}", e))
        })?;
        serde_json::from_str(&content).map_err(|e| {
            crate::GatewayError::ConfigError(format!("Failed to parse usage file: {}", e))
        })
    }

    /// Save tracking data to a specific file path
    pub fn save_to_file(&self, path: &Path) -> GatewayResult<()> {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let content = serde_json::to_string_pretty(self).map_err(|e| {
            crate::GatewayError::ConfigError(format!("Failed to serialize usage: {}", e))
        })?;
        std::fs::write(path, content).map_err(|e| {
            crate::GatewayError::ConfigError(format!("Failed to write usage file: {}", e))
        })?;
        Ok(())
    }

    /// Record usage for a specific model
    pub fn record_usage(&mut self, model: &str, usage: UsageStats) {
        let entry = self.model_usage.entry(model.to_string()).or_default();
        entry.prompt_tokens += usage.prompt_tokens;
        entry.completion_tokens += usage.completion_tokens;
        entry.total_tokens += usage.total_tokens;
    }
}
