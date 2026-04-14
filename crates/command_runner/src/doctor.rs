//! Environment diagnostics (the `doctor` command)
//!
//! Checks that all required dependencies are installed and available.
//! Provides actionable feedback for fixing missing dependencies.

use peridot_shared::{PeridotError, PeridotResult};
use std::process::Command;

/// Status of the development environment
#[derive(Debug, Clone)]
pub struct EnvironmentStatus {
    /// Whether Node.js is installed
    pub node_installed: bool,
    /// Node.js version if available
    pub node_version: Option<String>,
    /// Whether npm is installed
    pub npm_installed: bool,
    /// npm version if available
    pub npm_version: Option<String>,
    /// List of missing requirements
    pub missing: Vec<String>,
    /// Additional warnings (optional tools, version issues)
    pub warnings: Vec<String>,
}

impl EnvironmentStatus {
    /// Create an empty status (all unknown)
    pub fn new() -> Self {
        EnvironmentStatus {
            node_installed: false,
            node_version: None,
            npm_installed: false,
            npm_version: None,
            missing: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Check if all requirements are satisfied
    pub fn is_ready(&self) -> bool {
        self.node_installed && self.npm_installed
    }

    /// Get a human-readable summary
    pub fn summary(&self) -> String {
        if self.is_ready() {
            "Environment ready for PeridotCode".to_string()
        } else {
            format!("Missing: {}", self.missing.join(", "))
        }
    }

    /// Get installation instructions for missing dependencies
    pub fn install_instructions(&self) -> Vec<String> {
        let mut instructions = Vec::new();

        if !self.node_installed {
            instructions.push("Install Node.js from https://nodejs.org/".to_string());
            instructions.push("  (LTS version recommended)".to_string());
        }

        if self.node_installed && !self.npm_installed {
            instructions.push("npm should be included with Node.js".to_string());
            instructions.push("  Try reinstalling Node.js".to_string());
        }

        instructions
    }

    /// Format a full diagnostic report
    pub fn format_report(&self) -> String {
        let mut report = String::new();

        report.push_str("Environment Diagnostic Report\n");
        report.push_str("=============================\n\n");

        // Node.js
        report.push_str("Node.js:\n");
        if self.node_installed {
            report.push_str(&format!(
                "  Status: ✓ Installed\n  Version: {}\n",
                self.node_version.as_deref().unwrap_or("unknown")
            ));
        } else {
            report.push_str("  Status: ✗ Not found\n");
        }
        report.push('\n');

        // npm
        report.push_str("npm:\n");
        if self.npm_installed {
            report.push_str(&format!(
                "  Status: ✓ Installed\n  Version: {}\n",
                self.npm_version.as_deref().unwrap_or("unknown")
            ));
        } else {
            report.push_str("  Status: ✗ Not found\n");
        }
        report.push('\n');

        // Summary
        if self.is_ready() {
            report.push_str("Result: ✓ Environment ready\n");
        } else {
            report.push_str("Result: ✗ Missing dependencies\n\n");
            report.push_str("Installation:\n");
            for instruction in self.install_instructions() {
                report.push_str(&format!("  {}\n", instruction));
            }
        }

        // Warnings
        if !self.warnings.is_empty() {
            report.push_str("\nWarnings:\n");
            for warning in &self.warnings {
                report.push_str(&format!("  ⚠ {}\n", warning));
            }
        }

        report
    }
}

impl Default for EnvironmentStatus {
    fn default() -> Self {
        Self::new()
    }
}

/// Check the environment for required dependencies
///
/// Currently checks for:
/// - Node.js (required for Phaser projects, >= 18 recommended)
/// - npm (required for package management)
///
/// # Example
///
/// ```rust,no_run
/// use peridot_command_runner::doctor::check_environment;
///
/// async fn example() -> Result<(), peridot_shared::PeridotError> {
///     let status = check_environment().await?;
///     if status.is_ready() {
///         println!("Ready to go!");
///     } else {
///         println!("Missing: {:?}", status.missing);
///     }
///     Ok(())
/// }
/// ```
pub async fn check_environment() -> PeridotResult<EnvironmentStatus> {
    let mut status = EnvironmentStatus::new();

    // Check Node.js
    match get_command_output("node", &["--version"]).await {
        Ok(version) => {
            let version = version.trim().to_string();
            status.node_installed = true;
            status.node_version = Some(version.clone());
            tracing::info!("Node.js found: {}", version);

            // Check version (v18+ recommended)
            if let Some(major) = parse_major_version(&version) {
                if major < 18 {
                    status.warnings.push(format!(
                        "Node.js {} is older than recommended (v18+)",
                        version
                    ));
                }
            }
        }
        Err(e) => {
            status.missing.push("Node.js".to_string());
            tracing::warn!("Node.js not found: {}", e);
        }
    }

    // Check npm
    match get_command_output("npm", &["--version"]).await {
        Ok(version) => {
            let version = version.trim().to_string();
            status.npm_installed = true;
            status.npm_version = Some(version.clone());
            tracing::info!("npm found: {}", version);
        }
        Err(e) => {
            status.missing.push("npm".to_string());
            tracing::warn!("npm not found: {}", e);
        }
    }

    Ok(status)
}

/// Run a command and return its stdout
async fn get_command_output(command: &str, args: &[&str]) -> PeridotResult<String> {
    // Use blocking execution for simplicity
    // In production, consider tokio::process for true async

    let output = Command::new(command)
        .args(args)
        .output()
        .map_err(|e| PeridotError::CommandError(format!("Failed to execute {}: {}", command, e)))?;

    if !output.status.success() {
        return Err(PeridotError::CommandError(format!(
            "{} returned error: {}",
            command,
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    String::from_utf8(output.stdout)
        .map_err(|e| PeridotError::CommandError(format!("Invalid UTF-8 output: {}", e)))
}

/// Parse major version number from version string like "v18.12.1" or "18.12.1"
fn parse_major_version(version: &str) -> Option<u32> {
    let version = version.trim().trim_start_matches('v');
    version
        .split('.')
        .next()
        .and_then(|v| v.parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_major_version() {
        assert_eq!(parse_major_version("v18.12.1"), Some(18));
        assert_eq!(parse_major_version("18.12.1"), Some(18));
        assert_eq!(parse_major_version("20.0.0"), Some(20));
        assert_eq!(parse_major_version("invalid"), None);
    }

    #[test]
    fn test_environment_status() {
        let mut status = EnvironmentStatus::new();
        assert!(!status.is_ready());

        status.node_installed = true;
        status.npm_installed = true;
        assert!(status.is_ready());
        assert_eq!(status.summary(), "Environment ready for PeridotCode");
    }

    #[test]
    fn test_environment_status_missing() {
        let mut status = EnvironmentStatus::new();
        status.missing.push("Node.js".to_string());

        assert!(!status.is_ready());
        assert_eq!(status.summary(), "Missing: Node.js");

        let instructions = status.install_instructions();
        assert!(!instructions.is_empty());
        assert!(instructions[0].contains("nodejs.org"));
    }
}
