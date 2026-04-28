//! JSON-Based Template System
//!
//! This module provides a modern, AI-native template system where templates
//! are defined as JSON files containing structured code knowledge rather than
//! copying existing project files.
//!
//! # Template Structure
//!
//! ```json
//! {
//!   "template_name": "2d_platformer_basic",
//!   "engine": "macroquad",
//!   "description": "A basic 2D platformer with physics",
//!   "features": ["gravity", "jump", "collision"],
//!   "dependencies": {
//!     "macroquad": "0.4"
//!   },
//!   "files": {
//!     "src/main.rs": "// Main entry point...",
//!     "src/player.rs": "// Player implementation...",
//!     "Cargo.toml": "[package]..."
//!   },
//!   "placeholders": {
//!     "project_name": "My Game",
//!     "description": "A fun platformer"
//!   },
//!   "extension_points": [
//!     {
//!       "name": "add_enemy",
//!       "description": "Add enemy entities",
//!       "files_to_modify": ["src/main.rs"],
//!       "suggested_code": "..."
//!     }
//!   ]
//! }
//! ```

use peridot_shared::{PeridotError, PeridotResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ─────────────────────────────────────────────────────────────────
// Embedded Templates
//
// These templates are compiled into the binary so that `cargo install`
// produces a self-contained executable. Disk-based templates in
// `./templates` or next to the exe still load afterwards and can
// override or supplement these defaults.
// ─────────────────────────────────────────────────────────────────

const EMBEDDED_TEMPLATES: &[&str] = &[
    include_str!("../../../templates/phaser_2d_starter.json"),
    include_str!("../../../templates/macroquad_2d_platformer.json"),
    include_str!("../../../templates/bevy_2d_minimal.json"),
    include_str!("../../../templates/vanilla_js_platformer.json"),
    include_str!("../../../templates/vanilla_js_shooter.json"),
];

/// A JSON-based code template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeTemplate {
    /// Template identifier
    pub template_name: String,
    /// Game engine/framework
    pub engine: GameEngine,
    /// Human-readable description
    pub description: String,
    /// List of features included
    pub features: Vec<String>,
    /// Dependencies required
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
    /// File contents (path -> content)
    pub files: HashMap<String, String>,
    /// Placeholder values for customization
    #[serde(default)]
    pub placeholders: HashMap<String, String>,
    /// Extension points for adding features
    #[serde(default)]
    pub extension_points: Vec<ExtensionPoint>,
    /// Template metadata
    #[serde(default)]
    pub metadata: TemplateMetadata,
}

/// Supported game engines
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GameEngine {
    /// Phaser HTML5 framework
    Phaser,
    /// Macroquad (Rust)
    Macroquad,
    /// Bevy Engine (Rust)
    Bevy,
    /// Godot Engine
    Godot,
    /// Custom/other
    Custom,
}

impl GameEngine {
    /// Get engine display name
    pub fn display_name(&self) -> &'static str {
        match self {
            GameEngine::Phaser => "Phaser",
            GameEngine::Macroquad => "Macroquad",
            GameEngine::Bevy => "Bevy",
            GameEngine::Godot => "Godot",
            GameEngine::Custom => "Custom",
        }
    }

    /// Get file extension for main language
    pub fn main_extension(&self) -> &'static str {
        match self {
            GameEngine::Phaser => "js",
            GameEngine::Macroquad => "rs",
            GameEngine::Bevy => "rs",
            GameEngine::Godot => "gd",
            GameEngine::Custom => "txt",
        }
    }

    /// Get default package file
    pub fn package_file(&self) -> &'static str {
        match self {
            GameEngine::Phaser => "package.json",
            GameEngine::Macroquad => "Cargo.toml",
            GameEngine::Bevy => "Cargo.toml",
            GameEngine::Godot => "project.godot",
            GameEngine::Custom => "README.md",
        }
    }
}

impl std::fmt::Display for GameEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Template metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TemplateMetadata {
    /// Template version
    pub version: String,
    /// Author/creator
    pub author: Option<String>,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Minimum complexity level (1-5)
    pub complexity: Option<u8>,
    /// Estimated setup time in minutes
    pub setup_time_minutes: Option<u32>,
}

/// An extension point for adding features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionPoint {
    /// Extension identifier
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Files that need modification
    pub files_to_modify: Vec<String>,
    /// Suggested code to add
    pub suggested_code: String,
    /// Where to insert (e.g., "after:Player::new")
    pub insertion_point: Option<String>,
}

impl CodeTemplate {
    /// Create a new empty template
    pub fn new(name: impl Into<String>, engine: GameEngine) -> Self {
        Self {
            template_name: name.into(),
            engine,
            description: String::new(),
            features: Vec::new(),
            dependencies: HashMap::new(),
            files: HashMap::new(),
            placeholders: HashMap::new(),
            extension_points: Vec::new(),
            metadata: TemplateMetadata::default(),
        }
    }

    /// Load a template from a JSON file
    pub fn load_file(path: impl AsRef<Path>) -> PeridotResult<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)
            .map_err(|e| PeridotError::FsError(format!("Failed to read template {}: {}", path.display(), e)))?;
        
        Self::from_json(&content)
    }

    /// Parse template from JSON string
    pub fn from_json(json: &str) -> PeridotResult<Self> {
        serde_json::from_str(json)
            .map_err(|e| PeridotError::General(format!("Failed to parse template JSON: {}", e)))
    }

    /// Add a file to the template
    pub fn add_file(&mut self, path: impl Into<String>, content: impl Into<String>) -> &mut Self {
        self.files.insert(path.into(), content.into());
        self
    }

    /// Add a placeholder
    pub fn add_placeholder(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.placeholders.insert(key.into(), value.into());
        self
    }

    /// Add a feature
    pub fn add_feature(&mut self, feature: impl Into<String>) -> &mut Self {
        self.features.push(feature.into());
        self
    }

    /// Substitute placeholders in all file contents
    pub fn render(&self, custom_placeholders: &HashMap<String, String>) -> HashMap<String, String> {
        let mut result = HashMap::new();
        
        // Merge default and custom placeholders
        let mut all_placeholders = self.placeholders.clone();
        all_placeholders.extend(custom_placeholders.clone());
        
        // Add computed placeholders
        if let Some(name) = all_placeholders.get("project_name").cloned() {
            all_placeholders.insert("project_name_snake".to_string(), 
                name.to_lowercase().replace(" ", "_"));
            all_placeholders.insert("project_name_pascal".to_string(), 
                to_pascal_case(&name));
        }
        
        // Substitute in each file
        for (path, content) in &self.files {
            let rendered = substitute_placeholders(content, &all_placeholders);
            result.insert(path.clone(), rendered);
        }
        
        result
    }

    /// Get file by path
    pub fn get_file(&self, path: &str) -> Option<&String> {
        self.files.get(path)
    }

    /// Get extension point by name
    pub fn get_extension(&self, name: &str) -> Option<&ExtensionPoint> {
        self.extension_points.iter().find(|e| e.name == name)
    }

    /// Check if template has a feature
    pub fn has_feature(&self, feature: &str) -> bool {
        self.features.iter().any(|f| f.to_lowercase() == feature.to_lowercase())
    }

    /// Convert to JSON string
    pub fn to_json(&self) -> PeridotResult<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| PeridotError::General(format!("Failed to serialize template: {}", e)))
    }

    /// Generate a summary
    pub fn summary(&self) -> String {
        format!(
            "{} ({}) - {} features, {} files",
            self.template_name,
            self.engine,
            self.features.len(),
            self.files.len()
        )
    }
}

/// Registry for managing multiple code templates
#[derive(Debug, Default)]
pub struct CodeTemplateRegistry {
    templates: HashMap<String, CodeTemplate>,
}

impl CodeTemplateRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }

    /// Register a template
    pub fn register(&mut self, template: CodeTemplate) {
        self.templates.insert(template.template_name.clone(), template);
    }

    /// Get a template by name
    pub fn get(&self, name: &str) -> Option<&CodeTemplate> {
        self.templates.get(name)
    }

    /// Get mutable reference to template
    pub fn get_mut(&mut self, name: &str) -> Option<&mut CodeTemplate> {
        self.templates.get_mut(name)
    }

    /// Check if template exists
    pub fn has(&self, name: &str) -> bool {
        self.templates.contains_key(name)
    }

    /// List all template names
    pub fn list(&self) -> Vec<&String> {
        self.templates.keys().collect()
    }

    /// List templates by engine
    pub fn by_engine(&self, engine: GameEngine) -> Vec<&CodeTemplate> {
        self.templates
            .values()
            .filter(|t| t.engine == engine)
            .collect()
    }

    /// Load all templates from a directory
    pub fn load_from_dir(&mut self, dir: impl AsRef<Path>) -> PeridotResult<usize> {
        let dir = dir.as_ref();
        let mut count = 0;

        if !dir.exists() {
            return Ok(0);
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                match CodeTemplate::load_file(&path) {
                    Ok(template) => {
                        tracing::info!("Loaded template: {} from {:?}", template.template_name, path);
                        self.register(template);
                        count += 1;
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load template from {:?}: {}", path, e);
                    }
                }
            }
        }

        Ok(count)
    }

    /// Get default template for an engine
    pub fn get_default_for_engine(&self, engine: &GameEngine) -> Option<&CodeTemplate> {
        self.templates
            .values()
            .find(|t| &t.engine == engine)
    }

    /// Get template that best matches features
    pub fn find_by_features(&self, required_features: &[String]) -> Option<&CodeTemplate> {
        self.templates
            .values()
            .max_by_key(|t| {
                required_features
                    .iter()
                    .filter(|f| t.has_feature(f))
                    .count()
            })
    }
}

/// Helper: Convert string to PascalCase
fn to_pascal_case(s: &str) -> String {
    s.split(|c: char| c.is_whitespace() || c == '_' || c == '-')
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
            }
        })
        .collect()
}

/// Helper: Substitute placeholders in content
fn substitute_placeholders(content: &str, placeholders: &HashMap<String, String>) -> String {
    let mut result = content.to_string();
    
    for (key, value) in placeholders {
        let placeholder = format!("{{{{{}}}}}", key);
        result = result.replace(&placeholder, value);
    }
    
    result
}

/// Scaffold generator that writes templates to disk
#[derive(Debug)]
pub struct ScaffoldGenerator {
    registry: CodeTemplateRegistry,
}

impl ScaffoldGenerator {
    /// Create a new scaffold generator
    ///
    /// Automatically loads embedded templates so the binary is usable
    /// immediately after `cargo install` without requiring external
    /// template files.
    pub fn new() -> Self {
        let mut generator = Self {
            registry: CodeTemplateRegistry::new(),
        };
        generator.load_embedded_templates();
        generator
    }

    /// Load templates compiled into the binary via `include_str!`
    fn load_embedded_templates(&mut self) {
        for (i, json) in EMBEDDED_TEMPLATES.iter().enumerate() {
            match CodeTemplate::from_json(json) {
                Ok(template) => {
                    tracing::info!(
                        "Loaded embedded template: {} ({})",
                        template.template_name,
                        template.engine
                    );
                    self.registry.register(template);
                }
                Err(e) => {
                    tracing::warn!("Failed to parse embedded template #{}: {}", i, e);
                }
            }
        }
        tracing::info!(
            "Embedded templates loaded: {} total",
            self.registry.list().len()
        );
    }

    /// Create with templates from a directory
    pub fn with_templates_dir(dir: impl AsRef<Path>) -> PeridotResult<Self> {
        let mut generator = Self::new();
        generator.load_templates(dir)?;
        Ok(generator)
    }

    /// Load templates from directory
    pub fn load_templates(&mut self, dir: impl AsRef<Path>) -> PeridotResult<usize> {
        self.registry.load_from_dir(dir)
    }

    /// Generate scaffold from template
    pub fn generate(
        &self,
        template_name: &str,
        output_path: impl AsRef<Path>,
        placeholders: &HashMap<String, String>,
    ) -> PeridotResult<GenerationResult> {
        let template = self.registry
            .get(template_name)
            .ok_or_else(|| PeridotError::TemplateNotFound(template_name.to_string()))?;
        
        let output_path = output_path.as_ref();
        
        // Create output directory if needed
        std::fs::create_dir_all(output_path)?;
        
        // Render template with placeholders
        let rendered_files = template.render(placeholders);
        
        let mut created_files = Vec::new();
        
        // Write each file
        for (file_path, content) in rendered_files {
            let full_path = output_path.join(&file_path);
            
            // Create parent directories
            if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            
            // Write file
            std::fs::write(&full_path, content)?;
            created_files.push(file_path);
            
            tracing::info!("Created file: {:?}", full_path);
        }
        
        Ok(GenerationResult {
            template_name: template.template_name.clone(),
            engine: template.engine.clone(),
            created_files,
            features: template.features.clone(),
        })
    }

    /// Auto-select template based on engine and generate
    pub fn generate_auto(
        &self,
        engine: &GameEngine,
        output_path: impl AsRef<Path>,
        placeholders: &HashMap<String, String>,
    ) -> PeridotResult<GenerationResult> {
        let template = self.registry
            .get_default_for_engine(engine)
            .ok_or_else(|| PeridotError::TemplateNotFound(
                format!("No template found for engine: {}", engine)
            ))?;
        
        self.generate(&template.template_name, output_path, placeholders)
    }

    /// Get registry reference
    pub fn registry(&self) -> &CodeTemplateRegistry {
        &self.registry
    }

    /// Get registry mutable reference
    pub fn registry_mut(&mut self) -> &mut CodeTemplateRegistry {
        &mut self.registry
    }
}

/// Result of scaffold generation
#[derive(Debug, Clone)]
pub struct GenerationResult {
    /// Template name used
    pub template_name: String,
    /// Engine type
    pub engine: GameEngine,
    /// Files that were created
    pub created_files: Vec<String>,
    /// Features included
    pub features: Vec<String>,
}

impl GenerationResult {
    /// Generate a summary report
    pub fn report(&self) -> String {
        let mut report = format!("Generated project: {} ({} engine)\n\n", 
            self.template_name, self.engine);
        
        report.push_str(&format!("Features ({}):\n", self.features.len()));
        for feature in &self.features {
            report.push_str(&format!("  - {}\n", feature));
        }
        
        report.push_str(&format!("\nCreated files ({}):\n", self.created_files.len()));
        for file in &self.created_files {
            report.push_str(&format!("  + {}\n", file));
        }
        
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_template_creation() {
        let mut template = CodeTemplate::new("test_platformer", GameEngine::Macroquad);
        template
            .add_feature("gravity")
            .add_feature("jump")
            .add_file("src/main.rs", "fn main() { println!(\"{{project_name}}\"); }")
            .add_placeholder("project_name", "My Game");
        
        assert_eq!(template.template_name, "test_platformer");
        assert_eq!(template.engine, GameEngine::Macroquad);
        assert_eq!(template.features.len(), 2);
        assert!(template.has_feature("gravity"));
    }

    #[test]
    fn test_placeholder_substitution() {
        let mut template = CodeTemplate::new("test", GameEngine::Macroquad);
        template
            .add_file("main.rs", "// {{project_name}} by {{author}}")
            .add_placeholder("project_name", "Awesome Game")
            .add_placeholder("author", "Player1");
        
        let mut custom = HashMap::new();
        custom.insert("author".to_string(), "Developer".to_string());
        
        let rendered = template.render(&custom);
        let content = rendered.get("main.rs").unwrap();
        
        assert!(content.contains("Awesome Game"));
        assert!(content.contains("Developer"));
        assert!(!content.contains("{{"));
    }

    #[test]
    fn test_json_roundtrip() {
        let template = CodeTemplate::new("test", GameEngine::Bevy)
            .add_feature("3d")
            .add_file("main.rs", "fn main() {}")
            .add_placeholder("name", "Test");
        
        let json = template.to_json().unwrap();
        let parsed = CodeTemplate::from_json(&json).unwrap();
        
        assert_eq!(parsed.template_name, template.template_name);
        assert_eq!(parsed.engine, template.engine);
    }

    #[test]
    fn test_registry() {
        let mut registry = CodeTemplateRegistry::new();
        
        let template1 = CodeTemplate::new("platformer", GameEngine::Macroquad);
        let template2 = CodeTemplate::new("shooter", GameEngine::Bevy);
        
        registry.register(template1);
        registry.register(template2);
        
        assert_eq!(registry.list().len(), 2);
        assert!(registry.has("platformer"));
        assert!(!registry.has("rpg"));
    }

    #[test]
    fn test_game_engine_display() {
        assert_eq!(GameEngine::Macroquad.display_name(), "Macroquad");
        assert_eq!(GameEngine::Phaser.display_name(), "Phaser");
        assert_eq!(GameEngine::Bevy.package_file(), "Cargo.toml");
    }
}
