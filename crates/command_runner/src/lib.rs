//! Command Runner
//!
//! Executes local system commands and provides environment diagnostics.
//! This crate is responsible for:
//! - Running development servers
//! - Checking environment prerequisites
//! - Providing run instructions for generated projects
//! - Executing build/publish commands (future)
//!
//! # Safety
//!
//! This crate does NOT run destructive commands automatically. Commands like
//! `npm install` that modify the filesystem are only executed when explicitly
//! requested by the user.
//!
//! # Usage
//!
//! ```rust,no_run
//! use peridot_command_runner::{CommandRunner, RunInstructions};
//! use std::path::Path;
//!
//! async fn example() -> Result<(), peridot_shared::PeridotError> {
//!     // Check environment
//!     let runner = CommandRunner::current_dir();
//!     let status = runner.check_environment().await?;
//!
//!     if !status.is_ready() {
//!         println!("Missing: {}", status.missing.join(", "));
//!     }
//!
//!     // Get run instructions for a project
//!     let instructions = RunInstructions::for_project(Path::new("./my-game"));
//!     println!("{}", instructions.format_display());
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]

pub mod doctor;
pub mod instructions;
pub mod run;

pub use doctor::{check_environment, EnvironmentStatus};
pub use instructions::{CommandStep, phaser_starter_instructions, ProjectType, RunInstructions};
pub use run::{CommandResult, npm_install, npm_run_dev, run_command};

use peridot_shared::PeridotResult;
use std::path::{Path, PathBuf};

/// The main command runner that executes system commands
#[derive(Debug, Clone)]
pub struct CommandRunner {
    /// Working directory for command execution
    working_dir: PathBuf,
    /// Whether to allow destructive operations
    allow_destructive: bool,
}

impl CommandRunner {
    /// Create a new CommandRunner for the given working directory
    ///
    /// By default, destructive operations are not allowed.
    pub fn new(working_dir: impl AsRef<Path>) -> Self {
        CommandRunner {
            working_dir: working_dir.as_ref().to_path_buf(),
            allow_destructive: false,
        }
    }

    /// Create a CommandRunner for the current directory
    pub fn current_dir() -> Self {
        CommandRunner {
            working_dir: std::env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf()),
            allow_destructive: false,
        }
    }

    /// Enable destructive operations (npm install, etc.)
    ///
    /// # Safety
    /// Only call this when the user has explicitly requested the operation.
    pub fn allow_destructive(mut self) -> Self {
        self.allow_destructive = true;
        self
    }

    /// Get the working directory
    pub fn working_dir(&self) -> &Path {
        &self.working_dir
    }

    /// Check if destructive operations are allowed
    pub fn destructive_allowed(&self) -> bool {
        self.allow_destructive
    }

    /// Check if the environment is ready for PeridotCode
    ///
    /// Verifies that required tools (Node.js, npm) are installed.
    /// This is a non-destructive diagnostic operation.
    pub async fn check_environment(&self) -> PeridotResult<EnvironmentStatus> {
        doctor::check_environment().await
    }

    /// Get run instructions for the project in the working directory
    ///
    /// Detects project type and provides appropriate commands.
    /// This does not execute any commands.
    pub fn get_run_instructions(&self) -> RunInstructions {
        RunInstructions::for_project(&self.working_dir)
    }

    /// Run a command in the working directory
    ///
    /// # Arguments
    /// * `program` - The program to execute
    /// * `args` - Arguments to pass
    pub async fn run(&self, program: &str, args: &[&str]) -> PeridotResult<CommandResult> {
        run::run_command(program, args, Some(&self.working_dir)).await
    }

    /// Run npm install (destructive operation)
    ///
    /// # Errors
    /// Returns an error if destructive operations are not allowed.
    pub async fn install_dependencies(&self) -> PeridotResult<CommandResult> {
        if !self.allow_destructive {
            return Err(peridot_shared::PeridotError::CommandError(
                "Install dependencies requires explicit user consent. \
                 Use CommandRunner::allow_destructive() or run 'npm install' manually.".to_string()
            ));
        }

        tracing::info!("Installing dependencies in {:?}", self.working_dir);
        run::npm_install(&self.working_dir).await
    }

    /// Run npm run dev
    ///
    /// This starts the development server. The process will run until
    /// the user stops it (Ctrl+C).
    pub async fn start_dev_server(&self) -> PeridotResult<CommandResult> {
        tracing::info!("Starting dev server in {:?}", self.working_dir);
        run::npm_run_dev(&self.working_dir).await
    }

    /// Run a development server interactively
    ///
    /// This streams output to stdout/stderr and waits for the process.
    /// The user can stop it with Ctrl+C.
    ///
    /// # TODO
    /// - Stream output to TUI instead of stdout
    /// - Handle graceful shutdown
    pub async fn run_dev_server_interactive(&self) -> PeridotResult<()> {
        let mut cmd = tokio::process::Command::new("npm");
        cmd.arg("run")
            .arg("dev")
            .current_dir(&self.working_dir)
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit());

        tracing::info!("Starting dev server (interactive mode)");
        tracing::info!("Press Ctrl+C to stop");

        let mut child = cmd
            .spawn()
            .map_err(|e| peridot_shared::PeridotError::CommandError(
                format!("Failed to start dev server: {}", e)
            ))?;

        // Wait for the process
        let status = child.wait().await.map_err(|e| {
            peridot_shared::PeridotError::CommandError(format!("Dev server error: {}", e))
        })?;

        if status.success() {
            Ok(())
        } else {
            Err(peridot_shared::PeridotError::CommandError(
                "Dev server exited with error".to_string()
            ))
        }
    }
}

/// Quick diagnostic for the current environment
///
/// Prints environment status to stdout.
///
/// # Example
///
/// ```rust,no_run
/// use peridot_command_runner::quick_diagnostic;
///
/// async fn example() -> Result<(), peridot_shared::PeridotError> {
///     quick_diagnostic().await?;
///     Ok(())
/// }
/// ```
pub async fn quick_diagnostic() -> PeridotResult<()> {
    println!("PeridotCode Environment Check");
    println!("==============================\n");

    let status = doctor::check_environment().await?;

    if status.node_installed {
        println!("  Node.js: {}", status.node_version.as_deref().unwrap_or("unknown"));
    } else {
        println!("  Node.js: Not found (required)");
    }

    if status.npm_installed {
        println!("  npm: {}", status.npm_version.as_deref().unwrap_or("unknown"));
    } else {
        println!("  npm: Not found (required)");
    }

    println!();

    if status.is_ready() {
        println!("Environment ready!");
    } else {
        println!("Missing dependencies:");
        for missing in &status.missing {
            println!("  - {}", missing);
        }
        println!("\nPlease install Node.js from https://nodejs.org/");
    }

    Ok(())
}

/// Get run instructions for a project at the given path
///
/// Convenience function that doesn't require creating a CommandRunner.
///
/// # Example
///
/// ```rust,no_run
/// use peridot_command_runner::get_instructions_for;
///
/// let instructions = get_instructions_for("./my-game");
/// println!("{}", instructions.format_display());
/// ```
pub fn get_instructions_for(path: impl AsRef<Path>) -> RunInstructions {
    RunInstructions::for_project(path.as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_runner_default_safe() {
        let runner = CommandRunner::current_dir();
        assert!(!runner.destructive_allowed());
    }

    #[test]
    fn test_command_runner_allow_destructive() {
        let runner = CommandRunner::current_dir().allow_destructive();
        assert!(runner.destructive_allowed());
    }
}
