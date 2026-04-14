//! Skill Registry
//!
//! Manages the registration and discovery of available skills.
//! The registry maintains a collection of skills that can be applied
//! to game projects.
//!
//! # Usage
//!
//! ```rust,no_run
//! use peridot_skills::{SkillRegistry, SkillId, SkillCategory};
//! use peridot_skills::builtins::{InventorySkill, DialogueSkill};
//!
//! // Create registry with built-in skills
//! let registry = SkillRegistry::with_builtins();
//!
//! // Look up a skill
//! if let Some(skill) = registry.get(&SkillId::new("inventory")) {
//!     println!("Found: {}", skill.name());
//! }
//!
//! // List all gameplay skills
//! let gameplay_skills = registry.by_category(SkillCategory::Gameplay);
//! ```

use crate::{BoxedSkill, Skill, SkillCategory};
use std::collections::HashMap;
use std::fmt;

/// Registry of available skills
///
/// The registry maintains a map of skill IDs to skill trait objects.
/// It provides lookup, filtering, and discovery capabilities.
#[derive(Debug, Default)]
pub struct SkillRegistry {
    /// Map of skill ID to skill implementation
    skills: HashMap<SkillId, BoxedSkill>,
}

impl SkillRegistry {
    /// Create an empty registry
    pub fn new() -> Self {
        SkillRegistry {
            skills: HashMap::new(),
        }
    }

    /// Create a registry with all built-in skills
    ///
    /// This is a convenience constructor for typical usage.
    /// It registers: inventory, dialogue, save-system
    pub fn with_builtins() -> Self {
        let mut registry = SkillRegistry::new();

        use crate::builtins::{DialogueSkill, InventorySkill, SaveSystemSkill};

        registry.register(Box::new(InventorySkill::new()));
        registry.register(Box::new(DialogueSkill::new()));
        registry.register(Box::new(SaveSystemSkill::new()));

        registry
    }

    /// Register a skill
    ///
    /// # Arguments
    /// * `skill` - Boxed skill trait object
    ///
    /// # Example
    ///
    /// ```rust
    /// use peridot_skills::SkillRegistry;
    /// use peridot_skills::builtins::InventorySkill;
    ///
    /// let mut registry = SkillRegistry::new();
    /// registry.register(Box::new(InventorySkill::new()));
    ///
    /// assert_eq!(registry.len(), 1);
    /// ```
    pub fn register(&mut self, skill: BoxedSkill) {
        let id = skill.id().clone();
        self.skills.insert(id, skill);
    }

    /// Get a skill by ID
    ///
    /// Returns `Some(&dyn Skill)` if found, `None` otherwise.
    pub fn get(&self, id: &SkillId) -> Option<&dyn Skill> {
        self.skills.get(id).map(|s| s.as_ref())
    }

    /// Check if a skill is registered
    pub fn contains(&self, id: &SkillId) -> bool {
        self.skills.contains_key(id)
    }

    /// List all registered skills
    pub fn list(&self) -> Vec<&dyn Skill> {
        self.skills.values().map(|s| s.as_ref()).collect()
    }

    /// Get skills by category
    ///
    /// Returns all skills matching the given category.
    pub fn by_category(&self, category: SkillCategory) -> Vec<&dyn Skill> {
        self.skills
            .values()
            .filter(|s| s.category() == category)
            .map(|s| s.as_ref())
            .collect()
    }

    /// Get skills that can be applied to a project
    ///
    /// Filters skills by their `can_apply()` method.
    pub fn applicable_to(&self, project_path: &std::path::Path) -> Vec<&dyn Skill> {
        self.skills
            .values()
            .filter(|s| s.can_apply(project_path))
            .map(|s| s.as_ref())
            .collect()
    }

    /// Find skills by name (partial match)
    ///
    /// Case-insensitive search of skill names.
    pub fn find_by_name(&self, query: &str) -> Vec<&dyn Skill> {
        let query_lower = query.to_lowercase();
        self.skills
            .values()
            .filter(|s| s.name().to_lowercase().contains(&query_lower))
            .map(|s| s.as_ref())
            .collect()
    }

    /// Get the number of registered skills
    pub fn len(&self) -> usize {
        self.skills.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.skills.is_empty()
    }

    /// Remove a skill from the registry
    pub fn remove(&mut self, id: &SkillId) -> Option<BoxedSkill> {
        self.skills.remove(id)
    }

    /// Clear all skills
    pub fn clear(&mut self) {
        self.skills.clear();
    }
}

/// Unique identifier for a skill
///
/// Skill IDs should be:
/// - Unique across all skills
/// - Stable across versions
/// - Lowercase with hyphens (kebab-case)
/// - Descriptive (e.g., "inventory", "dialogue", "save-system")
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SkillId(String);

impl SkillId {
    /// Create a new skill ID
    pub fn new<S: Into<String>>(id: S) -> Self {
        SkillId(id.into())
    }

    /// Get the ID as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SkillId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for SkillId {
    fn from(s: &str) -> Self {
        SkillId::new(s)
    }
}

impl From<String> for SkillId {
    fn from(s: String) -> Self {
        SkillId(s)
    }
}

/// Metadata about a skill
///
/// This is a lightweight struct for skill information without
/// the full trait object. Useful for listings and UI.
#[derive(Debug, Clone)]
pub struct SkillMetadata {
    /// Skill identifier
    pub id: SkillId,
    /// Human-readable name
    pub name: String,
    /// Skill description
    pub description: String,
    /// Category
    pub category: SkillCategory,
    /// Version string
    pub version: String,
    /// Whether this is a built-in skill
    pub is_builtin: bool,
}

impl SkillMetadata {
    /// Create new skill metadata
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        SkillMetadata {
            id: SkillId::new(id),
            name: name.into(),
            description: description.into(),
            category: SkillCategory::Gameplay,
            version: "1.0.0".to_string(),
            is_builtin: true,
        }
    }

    /// Extract metadata from a skill trait object
    pub fn from_skill(skill: &dyn Skill) -> Self {
        SkillMetadata {
            id: skill.id().clone(),
            name: skill.name().to_string(),
            description: skill.description().to_string(),
            category: skill.category(),
            version: skill.version().to_string(),
            is_builtin: true, // Assume built-in unless specified otherwise
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builtins::InventorySkill;

    #[test]
    fn test_skill_id() {
        let id = SkillId::new("test-skill");
        assert_eq!(id.as_str(), "test-skill");
        assert_eq!(id.to_string(), "test-skill");

        // Display trait
        let formatted = format!("{}", id);
        assert_eq!(formatted, "test-skill");
    }

    #[test]
    fn test_skill_id_from_str() {
        let id: SkillId = "test-skill".into();
        assert_eq!(id.as_str(), "test-skill");

        let id: SkillId = String::from("test-skill").into();
        assert_eq!(id.as_str(), "test-skill");
    }

    #[test]
    fn test_registry_register_and_get() {
        let mut registry = SkillRegistry::new();
        let skill = InventorySkill::new();

        registry.register(Box::new(skill));

        assert_eq!(registry.len(), 1);
        assert!(registry.contains(&SkillId::new("inventory")));

        let retrieved = registry.get(&SkillId::new("inventory"));
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name(), "Inventory System");
    }

    #[test]
    fn test_registry_with_builtins() {
        let registry = SkillRegistry::with_builtins();

        assert_eq!(registry.len(), 3);
        assert!(registry.contains(&SkillId::new("inventory")));
        assert!(registry.contains(&SkillId::new("dialogue")));
        assert!(registry.contains(&SkillId::new("save-system")));
    }

    #[test]
    fn test_registry_by_category() {
        let registry = SkillRegistry::with_builtins();

        let gameplay = registry.by_category(SkillCategory::Gameplay);
        assert_eq!(gameplay.len(), 2); // inventory, dialogue

        let system = registry.by_category(SkillCategory::System);
        assert_eq!(system.len(), 1); // save-system
    }

    #[test]
    fn test_registry_find_by_name() {
        let registry = SkillRegistry::with_builtins();

        let results = registry.find_by_name("inventory");
        assert_eq!(results.len(), 1);

        let results = registry.find_by_name("system");
        assert_eq!(results.len(), 3); // "Inventory System", "Dialogue System", "Save System"
    }

    #[test]
    fn test_skill_metadata() {
        let skill = InventorySkill::new();
        let metadata = SkillMetadata::from_skill(&skill);

        assert_eq!(metadata.id.as_str(), "inventory");
        assert_eq!(metadata.name, "Inventory System");
        assert_eq!(metadata.category, SkillCategory::Gameplay);
        assert!(metadata.is_builtin);
    }

    #[test]
    fn test_registry_list() {
        let registry = SkillRegistry::with_builtins();
        let skills = registry.list();

        assert_eq!(skills.len(), 3);
    }
}
