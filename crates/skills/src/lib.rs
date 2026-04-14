//! Skills System Foundation
//!
//! This crate provides the foundation for PeridotCode's modular skill system.
//! Skills are modular feature packs that can add functionality to games,
//! such as inventory systems, dialogue systems, save/load, etc.
//!
//! # Design Philosophy
//!
//! - **Lightweight**: No plugin complexity, no marketplace, no dynamic loading
//! - **Type-safe**: Skills are Rust traits with compile-time safety
//! - **Extensible**: New skills implement the `Skill` trait
//! - **Template-based**: Skills generate code using the template engine
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
//! │   Orchestrator  │────▶│  SkillRegistry  │────▶│    Skill trait  │
//! └─────────────────┘     └─────────────────┘     └─────────────────┘
//!                                                        │
//!                    ┌──────────────┬───────────────────┼───────────────────┐
//!                    ▼              ▼                   ▼                   ▼
//!            ┌──────────┐   ┌──────────┐       ┌──────────┐       ┌──────────┐
//!            │Inventory │   │ Dialogue │       │SaveSystem│       │  Custom  │
//!            └──────────┘   └──────────┘       └──────────┘       └──────────┘
//! ```
//!
//! # Usage
//!
//! ## Registering Skills
//!
//! ```rust,no_run
//! use peridot_skills::{SkillRegistry, SkillId};
//! use peridot_skills::builtins::{InventorySkill, DialogueSkill};
//!
//! let mut registry = SkillRegistry::new();
//!
//! // Register built-in skills
//! registry.register(Box::new(InventorySkill::new()));
//! registry.register(Box::new(DialogueSkill::new()));
//! ```
//!
//! ## Looking Up Skills
//!
//! ```rust,no_run
//! use peridot_skills::{SkillRegistry, SkillId};
//!
//! let registry = SkillRegistry::with_builtins();
//!
//! if let Some(skill) = registry.get(&SkillId::new("inventory")) {
//!     println!("Found skill: {}", skill.name());
//! }
//! ```
//!
//! ## Checking Skill Compatibility
//!
//! ```rust,no_run
//! use peridot_skills::Skill;
//! use std::path::Path;
//!
//! fn check_skill(skill: &dyn Skill, project_path: &Path) {
//!     if skill.can_apply(project_path) {
//!         println!("✓ {} can be applied", skill.name());
//!     }
//! }
//! ```
//!
//! # Future Skills (Stubbed)
//!
//! The following skills have stub implementations that define their interface
//! but don't yet generate code:
//!
//! - **Inventory**: Item management system with slots, stacking, and categories
//! - **Dialogue**: Conversation trees with choices and conditions
//! - **SaveSystem**: Game state persistence with slots and serialization
//!
//! # Adding a New Skill
//!
//! 1. Create a struct implementing `Skill`
//! 2. Implement required methods: `id()`, `name()`, `description()`
//! 3. Implement `can_apply()` to check compatibility
//! 4. Implement `apply()` to generate/modify project files
//! 5. Register in `SkillRegistry`
//!
//! ```rust
//! use peridot_skills::{Skill, SkillId};
//! use peridot_shared::PeridotResult;
//! use std::path::Path;
//!
//! #[derive(Debug)]
//! pub struct MySkill;
//!
//! impl Skill for MySkill {
//!     fn id(&self) -> &SkillId {
//!         use std::sync::OnceLock;
//!         static ID: OnceLock<SkillId> = OnceLock::new();
//!         ID.get_or_init(|| SkillId::new("my-skill"))
//!     }
//!
//!     fn name(&self) -> &str {
//!         "My Skill"
//!     }
//!
//!     fn description(&self) -> &str {
//!         "Description of what this skill does"
//!     }
//!
//!     fn can_apply(&self, project_path: &Path) -> bool {
//!         // Check if project supports this skill
//!         project_path.join("package.json").exists()
//!     }
//!
//!     fn apply(&self, project_path: &Path) -> PeridotResult<()> {
//!         // Generate files, modify existing code, etc.
//!         Ok(())
//!     }
//! }
//! ```

#![warn(missing_docs)]

pub mod builtins;
pub mod manifest;
#[doc(hidden)]
pub mod orchestrator_example;
pub mod registry;

pub use builtins::{DialogueSkill, InventorySkill, SaveSystemSkill};
pub use manifest::{SkillDependency, SkillManifest, SkillVersion};
pub use registry::{SkillId, SkillMetadata, SkillRegistry};

use peridot_shared::{PeridotError, PeridotResult};
use std::path::Path;

/// The core skill trait that all skills must implement
///
/// This trait defines the interface between PeridotCode and modular
/// game features. Skills can generate code, modify existing files,
/// and add dependencies to projects.
///
/// # Implementation Notes
///
/// - Skills should be stateless or have cheap-to-clone state
/// - The `apply()` method should be idempotent (running twice = same result)
/// - Use `can_apply()` to validate before attempting application
/// - Return descriptive errors from `apply()` for debugging
///
/// # Lifecycle
///
/// 1. **Discovery**: Skill is registered in `SkillRegistry`
/// 2. **Selection**: User or planner selects a skill to add
/// 3. **Validation**: `can_apply()` checks if skill works with project
/// 4. **Application**: `apply()` generates/modifies files
/// 5. **Verification**: (Future) Verify skill was applied correctly
pub trait Skill: Send + Sync + std::fmt::Debug {
    /// Get the unique skill identifier
    ///
    /// This ID is used to lookup skills in the registry and should
    /// be stable across versions (e.g., "inventory", "dialogue", "save-system").
    fn id(&self) -> &SkillId;

    /// Get the human-readable skill name
    ///
    /// Used in UI displays and documentation.
    fn name(&self) -> &str;

    /// Get the skill description
    ///
    /// Should explain what the skill does in 1-2 sentences.
    fn description(&self) -> &str;

    /// Get the skill version
    ///
    /// Used for compatibility checking and updates.
    fn version(&self) -> SkillVersion {
        // Default to 1.0.0 for skills that don't specify
        SkillVersion::new(1, 0, 0)
    }

    /// Get the category this skill belongs to
    ///
    /// Used for grouping in UI (e.g., "Gameplay", "UI", "System").
    fn category(&self) -> SkillCategory {
        SkillCategory::Gameplay
    }

    /// Check if this skill can be applied to a project
    ///
    /// Validates that:
    /// - The project type is supported (e.g., Phaser project)
    /// - Required dependencies are present or can be added
    /// - No conflicting skills are installed
    ///
    /// # Arguments
    /// * `project_path` - Path to the project root
    ///
    /// # Returns
    /// `true` if the skill can be applied, `false` otherwise
    fn can_apply(&self, project_path: &Path) -> bool {
        // Default implementation: check if it's a valid project
        project_path.exists() && project_path.is_dir()
    }

    /// Apply the skill to a project
    ///
    /// This method:
    /// 1. Generates new files (if any)
    /// 2. Modifies existing files (if needed)
    /// 3. Adds dependencies to package.json (if needed)
    /// 4. Updates project configuration
    ///
    /// # Arguments
    /// * `project_path` - Path to the project root
    ///
    /// # Errors
    /// Returns an error if:
    /// - Files cannot be written
    /// - Required dependencies cannot be added
    /// - Project structure is incompatible
    fn apply(&self, project_path: &Path) -> PeridotResult<()> {
        // Default implementation returns an error
        // Real skills must override this
        Err(PeridotError::General(format!(
            "Skill '{}' does not implement apply() yet",
            self.name()
        )))
    }

    /// Get files that would be created/modified by this skill
    ///
    /// Used for preview/dry-run functionality. Returns relative paths
    /// from project root.
    ///
    /// # Returns
    /// Vector of (path, operation) tuples where operation is "create" or "modify"
    fn affected_files(&self, _project_path: &Path) -> Vec<(std::path::PathBuf, &'static str)> {
        // Default: return empty (skills should override)
        Vec::new()
    }

    /// Get dependencies this skill requires
    ///
    /// Returns npm package names and versions for Node.js projects.
    fn dependencies(&self) -> Vec<(&str, &str)> {
        // Default: no dependencies
        Vec::new()
    }

    /// Check if this skill conflicts with another skill
    ///
    /// Some skills cannot be used together (e.g., two different
    /// inventory systems).
    fn conflicts_with(&self, _other: &dyn Skill) -> bool {
        // Default: no conflicts
        false
    }
}

/// Category for organizing skills
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillCategory {
    /// Core gameplay mechanics (inventory, combat, etc.)
    Gameplay,
    /// User interface components
    UI,
    /// System-level features (save/load, audio, etc.)
    System,
    /// Development tools and debugging
    DevTool,
    /// Integration with external services
    Integration,
}

impl SkillCategory {
    /// Get display name for the category
    pub fn display_name(&self) -> &'static str {
        match self {
            SkillCategory::Gameplay => "Gameplay",
            SkillCategory::UI => "UI",
            SkillCategory::System => "System",
            SkillCategory::DevTool => "Development",
            SkillCategory::Integration => "Integration",
        }
    }
}

/// A boxed skill trait object
///
/// This type alias makes it easier to work with skills stored
/// in collections.
pub type BoxedSkill = Box<dyn Skill>;

/// Check if a project already has a skill applied
///
/// Looks for skill markers in the project (e.g., specific files,
/// configuration entries, etc.).
///
/// # TODO
/// - Implement actual detection logic
/// - Check for skill configuration files
/// - Look for skill-generated code patterns
pub fn skill_is_applied(_skill_id: &SkillId, _project_path: &Path) -> bool {
    // TODO: Implement skill detection
    // This could check for:
    // - Skill configuration files (.peridot/skills/{id}.toml)
    // - Generated code patterns
    // - Package.json dependencies added by skill
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test skill for verifying trait behavior
    #[derive(Debug)]
    struct TestSkill {
        id: SkillId,
    }

    impl TestSkill {
        fn new() -> Self {
            TestSkill {
                id: SkillId::new("test-skill"),
            }
        }
    }

    impl Skill for TestSkill {
        fn id(&self) -> &SkillId {
            &self.id
        }

        fn name(&self) -> &str {
            "Test Skill"
        }

        fn description(&self) -> &str {
            "A skill for testing"
        }

        fn category(&self) -> SkillCategory {
            SkillCategory::DevTool
        }
    }

    #[test]
    fn test_skill_trait() {
        let skill = TestSkill::new();

        assert_eq!(skill.id().as_str(), "test-skill");
        assert_eq!(skill.name(), "Test Skill");
        assert_eq!(skill.description(), "A skill for testing");
        assert_eq!(skill.category(), SkillCategory::DevTool);
        assert_eq!(skill.version(), SkillVersion::new(1, 0, 0));
    }

    #[test]
    fn test_skill_category_display() {
        assert_eq!(SkillCategory::Gameplay.display_name(), "Gameplay");
        assert_eq!(SkillCategory::System.display_name(), "System");
    }

    #[test]
    fn test_default_skill_methods() {
        let skill = TestSkill::new();
        let temp_dir = std::env::temp_dir();

        // Default implementations
        assert!(skill.dependencies().is_empty());
        assert!(skill.affected_files(&temp_dir).is_empty());
        assert!(!skill.conflicts_with(&skill));

        // apply() should return error by default
        assert!(skill.apply(&temp_dir).is_err());
    }
}
