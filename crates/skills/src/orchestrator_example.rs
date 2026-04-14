//! Orchestrator Integration Example
//!
//! This module demonstrates how the orchestrator could reference
//! and use skills in the future. This is documentation/example code,
//! not currently active in the orchestrator.
//!
//! # Integration Pattern
//!
//! The orchestrator would:
//! 1. Hold a `SkillRegistry` instance
//! 2. Look up skills by ID when processing "AddSkill" actions
//! 3. Validate skills can be applied with `can_apply()`
//! 4. Apply skills using `apply()` method
//!
//! # Example Usage
//!
//! ```ignore
//! use peridot_core::Orchestrator;
//! use peridot_skills::{SkillRegistry, SkillId};
//!
//! async fn example() {
//!     let mut orchestrator = Orchestrator::new(Default::default()).unwrap();
//!     
//!     // Initialize skill registry
//!     orchestrator.init_skills(SkillRegistry::with_builtins());
//!     
//!     // Add a skill to the current project
//!     orchestrator.add_skill(&SkillId::new("inventory")).await.unwrap();
//! }
//! ```

use crate::{SkillId, SkillRegistry};
use peridot_shared::PeridotResult;
use std::path::Path;

/// Example: How the orchestrator would integrate with skills
///
/// This struct shows the pattern for orchestrator-skill integration.
/// It would be part of the real Orchestrator struct.
pub struct SkillIntegrationExample {
    /// Registry of available skills
    skill_registry: SkillRegistry,
    /// Currently applied skills in the project
    applied_skills: Vec<SkillId>,
}

impl SkillIntegrationExample {
    /// Create new skill integration
    pub fn new(registry: SkillRegistry) -> Self {
        SkillIntegrationExample {
            skill_registry: registry,
            applied_skills: Vec::new(),
        }
    }

    /// Example: Add a skill to the project
    ///
    /// This shows the pattern the orchestrator would use:
    /// 1. Look up skill by ID
    /// 2. Validate it can be applied
    /// 3. Apply the skill
    /// 4. Track the applied skill
    pub fn add_skill(&mut self, skill_id: &SkillId, project_path: &Path) -> PeridotResult<()> {
        // Step 1: Look up skill
        let skill = self.skill_registry.get(skill_id).ok_or_else(|| {
            peridot_shared::PeridotError::General(format!(
                "Skill '{}' not found in registry",
                skill_id
            ))
        })?;

        // Step 2: Check if already applied
        if self.applied_skills.contains(skill_id) {
            return Err(peridot_shared::PeridotError::General(format!(
                "Skill '{}' is already applied",
                skill_id
            )));
        }

        // Step 3: Validate compatibility
        if !skill.can_apply(project_path) {
            return Err(peridot_shared::PeridotError::General(format!(
                "Skill '{}' cannot be applied to this project. \
                     Check that the project type is supported.",
                skill.name()
            )));
        }

        // Step 4: Check conflicts with existing skills
        for applied_id in &self.applied_skills {
            if let Some(applied_skill) = self.skill_registry.get(applied_id) {
                if skill.conflicts_with(applied_skill) {
                    return Err(peridot_shared::PeridotError::General(format!(
                        "Skill '{}' conflicts with '{}'",
                        skill.name(),
                        applied_skill.name()
                    )));
                }
            }
        }

        // Step 5: Apply the skill
        tracing::info!("Applying skill '{}' to {:?}", skill.name(), project_path);
        skill.apply(project_path)?;

        // Step 6: Track applied skill
        self.applied_skills.push(skill_id.clone());

        tracing::info!("Successfully applied skill '{}'", skill.name());
        Ok(())
    }

    /// Example: Get applicable skills for current project
    ///
    /// Shows how to filter skills by project compatibility
    pub fn get_applicable_skills(&self, project_path: &Path) -> Vec<&dyn crate::Skill> {
        self.skill_registry.applicable_to(project_path)
    }

    /// Example: Preview skill changes
    ///
    /// Shows what files would be created/modified without applying
    pub fn preview_skill_changes(
        &self,
        skill_id: &SkillId,
        project_path: &Path,
    ) -> PeridotResult<Vec<(std::path::PathBuf, &'static str)>> {
        let skill = self.skill_registry.get(skill_id).ok_or_else(|| {
            peridot_shared::PeridotError::General(format!("Skill '{}' not found", skill_id))
        })?;

        Ok(skill.affected_files(project_path))
    }

    /// Example: Check skill status
    ///
    /// Returns whether a skill is available, already applied, etc.
    pub fn skill_status(&self, skill_id: &SkillId) -> SkillStatus {
        if !self.skill_registry.contains(skill_id) {
            return SkillStatus::NotFound;
        }

        if self.applied_skills.contains(skill_id) {
            return SkillStatus::AlreadyApplied;
        }

        SkillStatus::Available
    }
}

/// Status of a skill relative to the current project
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillStatus {
    /// Skill is not in the registry
    NotFound,
    /// Skill is available and can be applied
    Available,
    /// Skill is already applied to this project
    AlreadyApplied,
}

/// Example extension trait for Orchestrator
///
/// Shows how the orchestrator would implement skill-related methods
pub trait OrchestratorSkillsExt {
    /// Initialize the skill system with a registry
    fn init_skills(&mut self, registry: SkillRegistry);

    /// Add a skill to the current project
    fn add_skill(
        &mut self,
        skill_id: &SkillId,
    ) -> impl std::future::Future<Output = PeridotResult<()>> + Send;

    /// Get list of skills that can be applied
    fn get_available_skills(&self) -> Vec<&dyn crate::Skill>;

    /// Preview what a skill would do
    fn preview_skill(
        &self,
        skill_id: &SkillId,
    ) -> PeridotResult<Vec<(std::path::PathBuf, &'static str)>>;
}

// Example implementation showing how Orchestrator would use this:
//
// impl Orchestrator {
//     pub fn init_skills(&mut self, registry: SkillRegistry) {
//         self.skills = Some(SkillIntegration::new(registry));
//     }
//
//     pub async fn add_skill(&mut self, skill_id: &SkillId) -> PeridotResult<()> {
//         let skills = self.skills.as_mut()
//             .ok_or_else(|| PeridotError::General("Skills not initialized".into()))?;
//
//         skills.add_skill(skill_id, self.context.path())
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builtins::InventorySkill;
    use std::path::PathBuf;

    #[test]
    fn test_skill_integration_pattern() {
        let registry = crate::SkillRegistry::with_builtins();
        let mut integration = SkillIntegrationExample::new(registry);

        // Test status
        assert_eq!(
            integration.skill_status(&SkillId::new("inventory")),
            SkillStatus::Available
        );

        assert_eq!(
            integration.skill_status(&SkillId::new("nonexistent")),
            SkillStatus::NotFound
        );
    }

    #[test]
    fn test_skill_status_tracking() {
        let registry = crate::SkillRegistry::with_builtins();
        let mut integration = SkillIntegrationExample::new(registry);

        // Initially not applied
        assert_eq!(
            integration.skill_status(&SkillId::new("inventory")),
            SkillStatus::Available
        );
    }

    #[test]
    fn test_get_applicable_skills() {
        // Create a temp directory that looks like a Phaser project
        let temp_dir = std::env::temp_dir().join("test_phaser_project");
        std::fs::create_dir_all(&temp_dir).ok();
        std::fs::write(
            temp_dir.join("package.json"),
            r#"{"dependencies": {"phaser": "^3.0.0"}}"#,
        )
        .ok();

        let registry = crate::SkillRegistry::with_builtins();
        let integration = SkillIntegrationExample::new(registry);

        // Get skills that can apply to this project
        let applicable = integration.get_applicable_skills(&temp_dir);

        // Inventory, Dialogue should apply (Phaser projects)
        // SaveSystem applies to any Node project
        assert!(applicable.len() >= 2);

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
    }
}
