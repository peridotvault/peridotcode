//! Run Instructions for Generated Projects
//!
//! Provides command suggestions and next-step guidance for generated projects.
//! This module does NOT execute commands automatically - it only provides
//! guidance for the user to run manually.

use std::path::Path;

/// Instructions for running a generated project
#[derive(Debug, Clone)]
pub struct RunInstructions {
    /// Project type detected
    pub project_type: ProjectType,
    /// Whether dependencies are installed
    pub dependencies_installed: bool,
    /// List of commands to run (in order)
    pub commands: Vec<CommandStep>,
    /// Additional notes or tips
    pub notes: Vec<String>,
}

/// Type of project detected
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectType {
    /// Phaser.js project
    Phaser,
    /// Node.js project (generic)
    Node,
    /// Unknown/unrecognized
    Unknown,
}

impl ProjectType {
    /// Get display name for the project type
    pub fn display_name(&self) -> &'static str {
        match self {
            ProjectType::Phaser => "Phaser Game",
            ProjectType::Node => "Node.js Project",
            ProjectType::Unknown => "Unknown Project",
        }
    }
}

/// A single command step to execute
#[derive(Debug, Clone)]
pub struct CommandStep {
    /// Description of what this step does
    pub description: String,
    /// The command to run
    pub command: String,
    /// Arguments for the command
    pub args: Vec<String>,
    /// Whether this is a required step
    pub required: bool,
}

impl CommandStep {
    /// Create a new command step
    pub fn new(description: &str, command: &str, args: &[&str], required: bool) -> Self {
        CommandStep {
            description: description.to_string(),
            command: command.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
            required,
        }
    }

    /// Format the command as a shell string
    pub fn format_shell(&self) -> String {
        let args_str = self.args.join(" ");
        if args_str.is_empty() {
            self.command.clone()
        } else {
            format!("{} {}", self.command, args_str)
        }
    }
}

impl RunInstructions {
    /// Generate instructions for a project directory
    ///
    /// Detects project type and provides appropriate commands.
    pub fn for_project(project_path: &Path) -> Self {
        let project_type = detect_project_type(project_path);
        let deps_installed = are_dependencies_installed(project_path, project_type);

        let commands = generate_commands(project_type, deps_installed);
        let notes = generate_notes(project_type);

        RunInstructions {
            project_type,
            dependencies_installed: deps_installed,
            commands,
            notes,
        }
    }

    /// Format instructions for display in CLI/TUI
    pub fn format_display(&self) -> String {
        let mut output = String::new();

        // Header
        output.push_str(&format!("\n{}\n", self.project_type.display_name()));
        output.push_str(&"=".repeat(self.project_type.display_name().len()));
        output.push('\n');

        // Dependencies status
        if !self.dependencies_installed {
            output.push_str("\n⚠ Dependencies not installed\n");
        }

        // Commands
        output.push_str("\nNext steps:\n");
        for (i, step) in self.commands.iter().enumerate() {
            let required_mark = if step.required { "*" } else { " " };
            output.push_str(&format!(
                "  {} {}. {}\n",
                required_mark,
                i + 1,
                step.description
            ));
            output.push_str(&format!("      $ {}\n", step.format_shell()));
        }

        // Notes
        if !self.notes.is_empty() {
            output.push_str("\nTips:\n");
            for note in &self.notes {
                output.push_str(&format!("  • {}\n", note));
            }
        }

        output
    }

    /// Get only the required commands
    pub fn required_commands(&self) -> Vec<&CommandStep> {
        self.commands.iter().filter(|c| c.required).collect()
    }

    /// Check if all required steps are completed
    pub fn is_ready_to_run(&self) -> bool {
        match self.project_type {
            ProjectType::Phaser | ProjectType::Node => self.dependencies_installed,
            ProjectType::Unknown => true,
        }
    }
}

/// Detect the type of project at the given path
fn detect_project_type(project_path: &Path) -> ProjectType {
    let package_json = project_path.join("package.json");

    if package_json.exists() {
        // Read package.json to check for Phaser
        if let Ok(content) = std::fs::read_to_string(&package_json) {
            // Simple check for Phaser dependency
            if content.contains("phaser") {
                return ProjectType::Phaser;
            }
        }
        return ProjectType::Node;
    }

    ProjectType::Unknown
}

/// Check if dependencies are installed
fn are_dependencies_installed(project_path: &Path, project_type: ProjectType) -> bool {
    match project_type {
        ProjectType::Phaser | ProjectType::Node => {
            // Check for node_modules directory
            project_path.join("node_modules").exists()
        }
        ProjectType::Unknown => true,
    }
}

/// Generate appropriate commands for the project type
fn generate_commands(project_type: ProjectType, deps_installed: bool) -> Vec<CommandStep> {
    let mut commands = Vec::new();

    match project_type {
        ProjectType::Phaser | ProjectType::Node => {
            if !deps_installed {
                commands.push(CommandStep::new(
                    "Install dependencies",
                    "npm",
                    &["install"],
                    true,
                ));
            }

            commands.push(CommandStep::new(
                "Start development server",
                "npm",
                &["run", "dev"],
                true,
            ));

            commands.push(CommandStep::new(
                "Build for production",
                "npm",
                &["run", "build"],
                false,
            ));
        }
        ProjectType::Unknown => {
            commands.push(CommandStep::new(
                "Check project structure",
                "ls",
                &["-la"],
                false,
            ));
        }
    }

    commands
}

/// Generate helpful notes for the project type
fn generate_notes(project_type: ProjectType) -> Vec<String> {
    match project_type {
        ProjectType::Phaser => vec![
            "Open http://localhost:8080 (or the URL shown) in your browser".to_string(),
            "Edit files in src/ to modify the game".to_string(),
            "Add assets to an assets/ folder".to_string(),
        ],
        ProjectType::Node => vec![
            "Check package.json for available scripts".to_string(),
            "Run 'npm run' to see all available commands".to_string(),
        ],
        ProjectType::Unknown => vec![
            "Could not detect project type".to_string(),
            "Check project documentation for how to run".to_string(),
        ],
    }
}

/// Quick helper to get instructions for a Phaser starter project
///
/// This is a convenience function for the common case.
pub fn phaser_starter_instructions() -> RunInstructions {
    RunInstructions {
        project_type: ProjectType::Phaser,
        dependencies_installed: false,
        commands: vec![
            CommandStep::new("Install dependencies", "npm", &["install"], true),
            CommandStep::new("Start development server", "npm", &["run", "dev"], true),
        ],
        notes: vec![
            "Open http://localhost:8080 in your browser".to_string(),
            "Edit src/scenes/GameScene.js to customize gameplay".to_string(),
        ],
    }
}

/// Format instructions as a simple string list
///
/// Useful for logging or simple output.
pub fn format_instructions_list(instructions: &RunInstructions) -> Vec<String> {
    let mut lines = Vec::new();

    lines.push(format!(
        "Project type: {}",
        instructions.project_type.display_name()
    ));

    if !instructions.dependencies_installed {
        lines.push("Dependencies: Not installed (run 'npm install')".to_string());
    }

    lines.push("Commands:".to_string());
    for step in &instructions.commands {
        let indicator = if step.required { "→" } else { "  " };
        lines.push(format!("  {} {}", indicator, step.format_shell()));
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_detect_phaser_project() {
        let temp_dir = TempDir::new().unwrap();
        let package_json = r#"{"dependencies": {"phaser": "^3.60.0"}}"#;
        fs::write(temp_dir.path().join("package.json"), package_json).unwrap();

        let project_type = detect_project_type(temp_dir.path());
        assert_eq!(project_type, ProjectType::Phaser);
    }

    #[test]
    fn test_detect_node_project() {
        let temp_dir = TempDir::new().unwrap();
        let package_json = r#"{"name": "test", "version": "1.0.0"}"#;
        fs::write(temp_dir.path().join("package.json"), package_json).unwrap();

        let project_type = detect_project_type(temp_dir.path());
        assert_eq!(project_type, ProjectType::Node);
    }

    #[test]
    fn test_phaser_instructions_format() {
        let instructions = phaser_starter_instructions();

        assert_eq!(instructions.project_type, ProjectType::Phaser);
        assert!(!instructions.dependencies_installed);
        assert_eq!(instructions.commands.len(), 2);

        let display = instructions.format_display();
        assert!(display.contains("npm install"));
        assert!(display.contains("npm run dev"));
        assert!(display.contains("localhost:8080"));
    }

    #[test]
    fn test_command_step_formatting() {
        let step = CommandStep::new("Test", "npm", &["run", "dev"], true);
        assert_eq!(step.format_shell(), "npm run dev");

        let step_no_args = CommandStep::new("Test", "ls", &[], false);
        assert_eq!(step_no_args.format_shell(), "ls");
    }
}
