//! Command execution utilities
//!
//! Provides functions for running system commands with proper
//! error handling and output capture.
//!
//! # Safety
//!
//! Commands in this module are executed as-is. Use the higher-level
//! `CommandRunner` API for safety checks and user confirmation.

use peridot_shared::{PeridotError, PeridotResult};
use std::process::Stdio;
use tokio::process::Command;

/// Result of a command execution
#[derive(Debug, Clone)]
pub struct CommandResult {
    /// The command that was executed
    pub command: String,
    /// Exit code (0 = success)
    pub exit_code: i32,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Whether the command succeeded
    pub success: bool,
}

impl CommandResult {
    /// Create a result from command output
    pub fn new(command: &str, output: &std::process::Output) -> Self {
        CommandResult {
            command: command.to_string(),
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            success: output.status.success(),
        }
    }

    /// Check if the command succeeded
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Format as a display string
    pub fn format(&self) -> String {
        if self.success {
            format!("✓ {} (exit: {})", self.command, self.exit_code)
        } else {
            format!(
                "✗ {} failed (exit: {})\n  stderr: {}",
                self.command,
                self.exit_code,
                self.stderr.trim()
            )
        }
    }
}

/// Run a command and return the result
///
/// # Arguments
/// * `program` - The program to execute
/// * `args` - Arguments to pass
/// * `working_dir` - Working directory for execution
///
/// # Errors
/// Returns an error if the command cannot be executed or returns non-zero exit code.
///
/// # Example
///
/// ```rust,no_run
/// use peridot_command_runner::run::run_command;
///
/// async fn example() -> Result<(), peridot_shared::PeridotError> {
///     let result = run_command("node", &["--version"], None).await?;
///     println!("Output: {}", result.stdout);
///     Ok(())
/// }
/// ```
pub async fn run_command(
    program: &str,
    args: &[&str],
    working_dir: Option<&std::path::Path>,
) -> PeridotResult<CommandResult> {
    let mut cmd = Command::new(program);
    cmd.args(args);

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    // Capture both stdout and stderr
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let command_str = format!("{} {}", program, args.join(" "));
    tracing::info!("Executing: {}", command_str);

    let output = cmd
        .output()
        .await
        .map_err(|e| PeridotError::CommandError(format!("Failed to execute '{}': {}", command_str, e)))?;

    let result = CommandResult::new(&command_str, &output);

    if result.success {
        tracing::info!("Command succeeded: {}", command_str);
    } else {
        tracing::warn!(
            "Command failed: {} (exit: {})\nstderr: {}",
            command_str,
            result.exit_code,
            result.stderr
        );
    }

    Ok(result)
}

/// Run npm install in the given directory
///
/// # Arguments
/// * `working_dir` - Directory containing package.json
///
/// # Errors
/// Returns an error if npm is not installed or install fails.
///
/// # Example
///
/// ```rust,no_run
/// use peridot_command_runner::run::npm_install;
///
/// async fn example() -> Result<(), peridot_shared::PeridotError> {
///     let result = npm_install("./my-project".as_ref()).await?;
///     if result.success {
///         println!("Dependencies installed!");
///     }
///     Ok(())
/// }
/// ```
pub async fn npm_install(working_dir: &std::path::Path) -> PeridotResult<CommandResult> {
    tracing::info!("Running npm install in {:?}", working_dir);
    run_command("npm", &["install"], Some(working_dir)).await
}

/// Run npm run dev in the given directory
///
/// # Arguments
/// * `working_dir` - Directory containing package.json
///
/// # Errors
/// Returns an error if npm is not installed or the dev script fails.
///
/// # Note
/// This command typically starts a long-running dev server. Use
/// `run_dev_server_interactive` in `CommandRunner` for interactive use.
pub async fn npm_run_dev(working_dir: &std::path::Path) -> PeridotResult<CommandResult> {
    tracing::info!("Running npm run dev in {:?}", working_dir);
    run_command("npm", &["run", "dev"], Some(working_dir)).await
}

/// Check if a command is available in PATH
///
/// # Arguments
/// * `command` - The command to check
///
/// # Returns
/// `true` if the command is available, `false` otherwise.
///
/// # Example
///
/// ```rust,no_run
/// use peridot_command_runner::run::is_command_available;
///
/// # async fn example() {
/// // Check if git is installed
/// let has_git = is_command_available("git").await;
/// # }
/// ```
pub async fn is_command_available(command: &str) -> bool {
    Command::new("which")
        .arg(command)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .map(|s: std::process::ExitStatus| s.success())
        .unwrap_or(false)
}

/// Run a command and stream output to the terminal
///
/// This is useful for long-running commands where you want
/// to see real-time output.
///
/// # Arguments
/// * `program` - The program to execute
/// * `args` - Arguments to pass
/// * `working_dir` - Working directory for execution
///
/// # Errors
/// Returns an error if the command fails to start or exits with error.
pub async fn run_command_streaming(
    program: &str,
    args: &[&str],
    working_dir: Option<&std::path::Path>,
) -> PeridotResult<()> {
    let mut cmd = Command::new(program);
    cmd.args(args);

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    // Inherit parent's stdio for streaming
    cmd.stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    let command_str = format!("{} {}", program, args.join(" "));
    tracing::info!("Executing (streaming): {}", command_str);

    let status = cmd
        .status()
        .await
        .map_err(|e| PeridotError::CommandError(format!("Failed to execute '{}': {}", command_str, e)))?;

    if status.success() {
        Ok(())
    } else {
        Err(PeridotError::CommandError(format!(
            "Command '{}' exited with code {:?}",
            command_str,
            status.code()
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_result_new() {
        let output = std::process::Output {
            status: std::process::ExitStatus::default(),
            stdout: b"hello\n".to_vec(),
            stderr: b"".to_vec(),
        };

        let result = CommandResult::new("test cmd", &output);
        assert_eq!(result.command, "test cmd");
        assert_eq!(result.stdout, "hello\n");
        assert!(result.stderr.is_empty());
    }

    #[test]
    fn test_command_result_format_success() {
        // Use a successful exit status (exit code 0)
        let output = std::process::Output {
            status: std::process::ExitStatus::default(), // Success by default
            stdout: b"output".to_vec(),
            stderr: b"".to_vec(),
        };

        let result = CommandResult::new("test", &output);
        let formatted = result.format();
        assert!(formatted.contains("test")); // Command name shown
        assert!(formatted.contains("exit: 0")); // Success exit code
    }

    #[test]
    fn test_command_result_format_failure() {
        // Create a failed exit status using std::process::Command
        // We can't easily construct ExitStatus directly, so test the format logic differently
        let _output = std::process::Output {
            status: std::process::ExitStatus::default(),
            stdout: b"".to_vec(),
            stderr: b"error".to_vec(),
        };

        // Manually create a CommandResult with failed state
        let result = CommandResult {
            command: "test".to_string(),
            exit_code: 1,
            stdout: String::new(),
            stderr: "error".to_string(),
            success: false,
        };

        let formatted = result.format();
        assert!(formatted.contains("test failed"), "Expected 'test failed' in: {}", formatted);
        assert!(formatted.contains("error"), "Expected 'error' in: {}", formatted);
    }
}
