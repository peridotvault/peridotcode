//! Skill Manifest
//!
//! Defines metadata structures for skills, including version handling
//! and dependency management.
//!
//! # Skill Manifest Format (Future)
//!
//! Skills will be definable via TOML manifests for external skills:
//!
//! ```toml
//! [skill]
//! id = "inventory"
//! name = "Inventory System"
//! version = "1.0.0"
//! category = "gameplay"
//!
//! [dependencies]
//! npm = [
//!   { name = "lodash", version = "^4.17.0" }
//! ]
//!
//! [files]
//! create = [
//!   "src/systems/Inventory.js",
//!   "src/ui/InventoryUI.js"
//! ]
//! modify = [
//!   "src/scenes/GameScene.js"
//! ]
//! ```

use std::fmt;

/// Semantic version for skills
///
/// Follows semver format: MAJOR.MINOR.PATCH
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SkillVersion {
    /// Major version (breaking changes)
    pub major: u32,
    /// Minor version (new features, backwards compatible)
    pub minor: u32,
    /// Patch version (bug fixes)
    pub patch: u32,
}

impl SkillVersion {
    /// Create a new version
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        SkillVersion {
            major,
            minor,
            patch,
        }
    }

    /// Parse version from string
    ///
    /// # Examples
    ///
    /// ```
    /// use peridot_skills::SkillVersion;
    ///
    /// let v = SkillVersion::parse("1.2.3").unwrap();
    /// assert_eq!(v.major, 1);
    /// assert_eq!(v.minor, 2);
    /// assert_eq!(v.patch, 3);
    /// ```
    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return None;
        }

        let major = parts[0].parse().ok()?;
        let minor = parts[1].parse().ok()?;
        let patch = parts[2].parse().ok()?;

        Some(SkillVersion::new(major, minor, patch))
    }

    /// Check if this version is compatible with another
    ///
    /// Two versions are compatible if they have the same major version
    /// and this version is >= the required version.
    pub fn is_compatible_with(&self, required: &SkillVersion) -> bool {
        self.major == required.major && self >= required
    }
}

impl fmt::Display for SkillVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl Default for SkillVersion {
    fn default() -> Self {
        SkillVersion::new(1, 0, 0)
    }
}

/// A dependency on another skill
#[derive(Debug, Clone)]
pub struct SkillDependency {
    /// Skill ID
    pub skill_id: String,
    /// Version requirement (e.g., "^1.0.0" or ">=1.0.0")
    pub version_req: String,
    /// Whether this is an optional dependency
    pub optional: bool,
}

impl SkillDependency {
    /// Create a new required dependency
    pub fn new(skill_id: impl Into<String>, version_req: impl Into<String>) -> Self {
        SkillDependency {
            skill_id: skill_id.into(),
            version_req: version_req.into(),
            optional: false,
        }
    }

    /// Create a new optional dependency
    pub fn optional(skill_id: impl Into<String>, version_req: impl Into<String>) -> Self {
        SkillDependency {
            skill_id: skill_id.into(),
            version_req: version_req.into(),
            optional: true,
        }
    }
}

/// Manifest for a skill
///
/// Defines metadata, dependencies, and file operations for a skill.
/// This is the static description; runtime behavior is in the `Skill` trait.
#[derive(Debug, Clone)]
pub struct SkillManifest {
    /// Skill identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description
    pub description: String,
    /// Version
    pub version: SkillVersion,
    /// Category
    pub category: String,
    /// Author
    pub author: Option<String>,
    /// Skill dependencies
    pub dependencies: Vec<SkillDependency>,
    /// NPM package dependencies
    pub npm_dependencies: Vec<(String, String)>,
    /// Files to create
    pub files_create: Vec<String>,
    /// Files to modify
    pub files_modify: Vec<String>,
}

impl SkillManifest {
    /// Create a new manifest
    pub fn new(id: impl Into<String>, name: impl Into<String>, version: SkillVersion) -> Self {
        SkillManifest {
            id: id.into(),
            name: name.into(),
            description: String::new(),
            version,
            category: "gameplay".to_string(),
            author: None,
            dependencies: Vec::new(),
            npm_dependencies: Vec::new(),
            files_create: Vec::new(),
            files_modify: Vec::new(),
        }
    }

    /// Set the description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Set the category
    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = category.into();
        self
    }

    /// Add a skill dependency
    pub fn with_dependency(mut self, dep: SkillDependency) -> Self {
        self.dependencies.push(dep);
        self
    }

    /// Add an NPM dependency
    pub fn with_npm(mut self, name: impl Into<String>, version: impl Into<String>) -> Self {
        self.npm_dependencies.push((name.into(), version.into()));
        self
    }

    /// Add a file to create
    pub fn creates_file(mut self, path: impl Into<String>) -> Self {
        self.files_create.push(path.into());
        self
    }

    /// Add a file to modify
    pub fn modifies_file(mut self, path: impl Into<String>) -> Self {
        self.files_modify.push(path.into());
        self
    }
}

impl Default for SkillManifest {
    fn default() -> Self {
        SkillManifest {
            id: "unknown".to_string(),
            name: "Unknown Skill".to_string(),
            description: String::new(),
            version: SkillVersion::default(),
            category: "gameplay".to_string(),
            author: None,
            dependencies: Vec::new(),
            npm_dependencies: Vec::new(),
            files_create: Vec::new(),
            files_modify: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing() {
        let v = SkillVersion::parse("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);

        assert!(SkillVersion::parse("1.2").is_none());
        assert!(SkillVersion::parse("invalid").is_none());
    }

    #[test]
    fn test_version_display() {
        let v = SkillVersion::new(2, 5, 1);
        assert_eq!(v.to_string(), "2.5.1");
    }

    #[test]
    fn test_version_comparison() {
        let v1 = SkillVersion::new(1, 0, 0);
        let v2 = SkillVersion::new(1, 1, 0);
        let v3 = SkillVersion::new(2, 0, 0);

        assert!(v1 < v2);
        assert!(v2 < v3);
        assert!(v1 < v3);
    }

    #[test]
    fn test_version_compatibility() {
        let v1_0_0 = SkillVersion::new(1, 0, 0);
        let v1_2_0 = SkillVersion::new(1, 2, 0);
        let v2_0_0 = SkillVersion::new(2, 0, 0);

        // Same major, higher or equal minor = compatible
        assert!(v1_2_0.is_compatible_with(&v1_0_0));
        assert!(v1_0_0.is_compatible_with(&v1_0_0));

        // Different major = incompatible
        assert!(!v2_0_0.is_compatible_with(&v1_0_0));
        assert!(!v1_0_0.is_compatible_with(&v2_0_0));

        // Lower version = incompatible
        assert!(!v1_0_0.is_compatible_with(&v1_2_0));
    }

    #[test]
    fn test_skill_dependency() {
        let dep = SkillDependency::new("inventory", "^1.0.0");
        assert_eq!(dep.skill_id, "inventory");
        assert_eq!(dep.version_req, "^1.0.0");
        assert!(!dep.optional);

        let opt_dep = SkillDependency::optional("dialogue", ">=0.5.0");
        assert!(opt_dep.optional);
    }

    #[test]
    fn test_skill_manifest() {
        let manifest = SkillManifest::new("test", "Test Skill", SkillVersion::new(1, 0, 0))
            .with_description("A test skill")
            .with_category("system")
            .with_dependency(SkillDependency::new("other", "^1.0.0"))
            .with_npm("lodash", "^4.17.0")
            .creates_file("src/test.js");

        assert_eq!(manifest.id, "test");
        assert_eq!(manifest.name, "Test Skill");
        assert_eq!(manifest.description, "A test skill");
        assert_eq!(manifest.category, "system");
        assert_eq!(manifest.dependencies.len(), 1);
        assert_eq!(manifest.npm_dependencies.len(), 1);
        assert_eq!(manifest.files_create.len(), 1);
    }
}
