//! Template Engine
//!
//! Manages game templates including:
//! - Loading templates from the templates directory
//! - Selecting the appropriate template based on user intent
//! - Rendering templates with placeholder substitution
//! - Validating template integrity
//!
//! Templates are defined by a `template.toml` manifest file and a directory
//! of source files that serve as the scaffold for new projects.

#![warn(missing_docs)]

pub mod registry;
pub mod renderer;
pub mod selector;

pub use registry::{load_templates, TemplateRegistry};
pub use renderer::{render_template, RenderResult, TemplateContext};
pub use selector::select_template;

// Re-export fs_engine types for convenience
pub use peridot_fs_engine::{ChangeType, FileChange};

use peridot_shared::{PeridotError, PeridotResult, TemplateId, TemplateManifest};
use std::path::{Path, PathBuf};

/// The main template engine that coordinates template operations
#[derive(Debug)]
pub struct TemplateEngine {
    /// Registry of available templates
    registry: TemplateRegistry,
    /// Path to the templates directory
    templates_path: PathBuf,
    /// Whether this engine uses embedded templates
    is_embedded: bool,
}

impl TemplateEngine {
    /// Create a new TemplateEngine with the default templates directory
    ///
    /// Searches for templates in the following order:
    /// 1. `./templates` relative to current working directory
    /// 2. `../templates` relative to the executable directory
    /// 3. `templates/` relative to the executable directory
    pub fn new() -> PeridotResult<Self> {
        // Try to find templates directory
        if let Some(templates_path) = Self::find_templates_path() {
            return Self::with_path(templates_path);
        }

        tracing::info!("No filesystem templates found, using embedded templates.");
        let registry = registry::load_templates_embedded()?;
        Ok(TemplateEngine {
            registry,
            templates_path: PathBuf::from("embedded"),
            is_embedded: true,
        })
    }

    /// Find the templates directory by searching multiple locations
    fn find_templates_path() -> Option<PathBuf> {
        // 1. Check current working directory
        let cwd_templates = PathBuf::from("templates");
        if cwd_templates.exists() && cwd_templates.is_dir() {
            tracing::info!("Found templates at: {:?}", cwd_templates);
            return Some(cwd_templates);
        }

        // 2. Check relative to executable
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                // Try templates/ next to executable
                let exe_templates = exe_dir.join("templates");
                if exe_templates.exists() && exe_templates.is_dir() {
                    tracing::info!("Found templates at: {:?}", exe_templates);
                    return Some(exe_templates);
                }

                // Try ../templates (for cargo build layout: target/debug/peridotcode -> target/templates)
                if let Some(parent_dir) = exe_dir.parent() {
                    let parent_templates = parent_dir.join("templates");
                    if parent_templates.exists() && parent_templates.is_dir() {
                        tracing::info!("Found templates at: {:?}", parent_templates);
                        return Some(parent_templates);
                    }

                    // Try ../../templates (for cargo build: target/release/peridotcode -> project root)
                    if let Some(grandparent_dir) = parent_dir.parent() {
                        let grandparent_templates = grandparent_dir.join("templates");
                        if grandparent_templates.exists() && grandparent_templates.is_dir() {
                            tracing::info!("Found templates at: {:?}", grandparent_templates);
                            return Some(grandparent_templates);
                        }
                    }
                }
            }
        }

        None
    }

    /// Create a TemplateEngine with a specific templates directory
    pub fn with_path(templates_path: impl AsRef<Path>) -> PeridotResult<Self> {
        let templates_path = templates_path.as_ref().to_path_buf();

        // Ensure templates directory exists or create it
        if !templates_path.exists() {
            tracing::warn!(
                "Templates directory does not exist: {}. Creating empty registry.",
                templates_path.display()
            );
            return Ok(TemplateEngine {
                registry: TemplateRegistry::new(),
                templates_path,
                is_embedded: false,
            });
        }

        let registry = load_templates(&templates_path)?;

        Ok(TemplateEngine {
            registry,
            templates_path,
            is_embedded: false,
        })
    }

    /// Get the template registry
    pub fn registry(&self) -> &TemplateRegistry {
        &self.registry
    }

    /// Get a template manifest by ID
    pub fn get_template(&self, id: &TemplateId) -> Option<&TemplateManifest> {
        self.registry.get(id)
    }

    /// Check if a template exists
    pub fn has_template(&self, id: &TemplateId) -> bool {
        self.registry.contains(id)
    }

    /// Select the best template for given criteria
    ///
    /// For MVP, this simply returns the phaser-2d-starter template.
    /// In the future, this will match based on genre, features, etc.
    pub fn select_template(&self, _genre: Option<&str>) -> Option<&TemplateManifest> {
        // MVP: Always use the phaser-2d-starter template
        let default_id = TemplateId::new("phaser-2d-starter");

        if let Some(template) = self.registry.get(&default_id) {
            return Some(template);
        }

        // Fallback: return first available template
        self.registry.list().first().copied()
    }

    /// Generate a scaffold from a template
    ///
    /// # Arguments
    /// * `template_id` - The template to use
    /// * `output_path` - Where to write the scaffold
    /// * `context` - Template variables for placeholder substitution
    ///
    /// # Returns
    /// Result containing the render result with list of created files
    ///
    /// # Errors
    /// Returns an error if the template doesn't exist or file operations fail
    pub fn generate_scaffold(
        &self,
        template_id: &TemplateId,
        output_path: impl AsRef<Path>,
        context: &TemplateContext,
    ) -> PeridotResult<RenderResult> {
        let template = self
            .registry
            .get(template_id)
            .ok_or_else(|| PeridotError::TemplateNotFound(template_id.as_ref().to_string()))?;

        tracing::info!(
            "Generating scaffold from template '{}' ({}) to {:?}",
            template_id.as_ref(),
            template.name,
            output_path.as_ref()
        );

        let result = if self.is_embedded {
            renderer::render_template_embedded(template_id.as_ref(), output_path, context)?
        } else {
            let template_path = self.templates_path.join(template_id.as_ref());
            // Validate template directory exists
            if !template_path.exists() {
                return Err(PeridotError::TemplateRenderError(format!(
                    "Template directory not found: {}",
                    template_path.display()
                )));
            }
            // Render the template
            render_template(&template_path, output_path, context)?
        };

        tracing::info!(
            "Successfully generated {} files in {:?}",
            result.file_count,
            result.output_path
        );

        Ok(result)
    }

    /// Generate a scaffold using the default template selection
    ///
    /// This is a convenience method that selects the best template
    /// and generates the scaffold in one call.
    pub fn generate_with_auto_select(
        &self,
        genre: Option<&str>,
        output_path: impl AsRef<Path>,
        context: &TemplateContext,
    ) -> PeridotResult<RenderResult> {
        let template = self.select_template(genre).ok_or_else(|| {
            PeridotError::TemplateNotFound("No suitable template found".to_string())
        })?;

        self.generate_scaffold(&template.id.clone(), output_path, context)
    }

    /// List all available templates
    pub fn list_templates(&self) -> Vec<&TemplateManifest> {
        self.registry.list()
    }

    /// Get the templates directory path
    pub fn templates_path(&self) -> &Path {
        &self.templates_path
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            registry: TemplateRegistry::new(),
            templates_path: PathBuf::from("templates"),
            is_embedded: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    /// Integration test: Full scaffold generation happy path
    #[test]
    fn test_full_scaffold_generation() {
        let temp_dir = TempDir::new().unwrap();
        let templates_dir = temp_dir.path().join("templates");
        let template_dir = templates_dir.join("test-template");
        let output_dir = temp_dir.path().join("output");

        // Create template structure
        std::fs::create_dir_all(&template_dir).unwrap();
        std::fs::create_dir_all(template_dir.join("src")).unwrap();

        // Create template.toml manifest
        let manifest = r#"id = "test-template"
name = "Test Template"
description = "A test template for integration testing"
stack = "phaser"
files = ["index.html", "src/main.js"]
"#;
        let mut manifest_file = std::fs::File::create(template_dir.join("template.toml")).unwrap();
        manifest_file.write_all(manifest.as_bytes()).unwrap();

        // Create template files
        let index_html = r#"<!DOCTYPE html>
<html>
<head><title>{{game_title}}</title></head>
<body>
<h1>{{game_title}}</h1>
<p>{{game_description}}</p>
</body>
</html>"#;
        let mut index_file = std::fs::File::create(template_dir.join("index.html")).unwrap();
        index_file.write_all(index_html.as_bytes()).unwrap();

        let main_js = r#"console.log("Welcome to {{game_title}}!");"#;
        let mut main_file = std::fs::File::create(template_dir.join("src/main.js")).unwrap();
        main_file.write_all(main_js.as_bytes()).unwrap();

        // Create output directory
        std::fs::create_dir_all(&output_dir).unwrap();

        // Create template engine pointing to our test templates
        let engine = TemplateEngine::with_path(&templates_dir).unwrap();

        // Generate scaffold
        let ctx = TemplateContext::from_project("My Test Game", Some("A test game description"));
        let result = engine.generate_scaffold(&TemplateId::new("test-template"), &output_dir, &ctx);

        // Verify success
        assert!(
            result.is_ok(),
            "Scaffold generation failed: {:?}",
            result.err()
        );
        let result = result.unwrap();

        // Verify file count (template.toml should be excluded)
        assert_eq!(
            result.file_count, 3,
            "Expected 3 files (index.html, src/, src/main.js)"
        );

        // Verify files were created
        assert!(output_dir.join("index.html").exists());
        assert!(output_dir.join("src").exists());
        assert!(output_dir.join("src/main.js").exists());

        // Verify placeholder substitution
        let index_content = std::fs::read_to_string(output_dir.join("index.html")).unwrap();
        assert!(index_content.contains("My Test Game"));
        assert!(index_content.contains("A test game description"));

        let main_content = std::fs::read_to_string(output_dir.join("src/main.js")).unwrap();
        assert!(main_content.contains("My Test Game"));

        // Verify change tracking
        let created_files: Vec<_> = result
            .file_changes
            .iter()
            .filter(|c| c.change_type == ChangeType::Created)
            .collect();
        assert!(
            !created_files.is_empty(),
            "Should have tracked created files"
        );
    }

    /// Test template auto-selection
    #[test]
    fn test_auto_select_template() {
        let temp_dir = TempDir::new().unwrap();
        let templates_dir = temp_dir.path().join("templates");
        let template_dir = templates_dir.join("phaser-2d-starter");

        // Create phaser-2d-starter template
        std::fs::create_dir_all(&template_dir).unwrap();
        let manifest = r#"id = "phaser-2d-starter"
name = "Phaser 2D Starter"
description = "A simple Phaser 3 starter template"
stack = "phaser"
files = []
"#;
        let mut manifest_file = std::fs::File::create(template_dir.join("template.toml")).unwrap();
        manifest_file.write_all(manifest.as_bytes()).unwrap();

        let engine = TemplateEngine::with_path(&templates_dir).unwrap();
        let template = engine.select_template(None);

        assert!(template.is_some());
        assert_eq!(template.unwrap().id.as_ref(), "phaser-2d-starter");
    }

    /// Test that template.toml is excluded from output
    #[test]
    fn test_template_toml_excluded() {
        let temp_dir = TempDir::new().unwrap();
        let templates_dir = temp_dir.path().join("templates");
        let template_dir = templates_dir.join("test");
        let output_dir = temp_dir.path().join("output");

        std::fs::create_dir_all(&template_dir).unwrap();
        std::fs::create_dir_all(&output_dir).unwrap();

        // Create template.toml
        let manifest = r#"id = "test"
name = "Test"
description = "A test template"
stack = "phaser"
files = ["file.txt"]
"#;
        let mut manifest_file = std::fs::File::create(template_dir.join("template.toml")).unwrap();
        manifest_file.write_all(manifest.as_bytes()).unwrap();

        // Create a regular file
        let mut regular_file = std::fs::File::create(template_dir.join("file.txt")).unwrap();
        regular_file.write_all(b"content").unwrap();

        let engine = TemplateEngine::with_path(&templates_dir).unwrap();
        let ctx = TemplateContext::new();
        let result = engine
            .generate_scaffold(&TemplateId::new("test"), &output_dir, &ctx)
            .unwrap();

        // template.toml should NOT be in output
        assert!(!output_dir.join("template.toml").exists());
        assert!(output_dir.join("file.txt").exists());
        assert_eq!(result.file_count, 1);
    }

    /// Test embedded template discovery
    #[test]
    fn test_embedded_template_engine() {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().join("output");
        
        std::fs::create_dir_all(&output_dir).unwrap();

        // Create engine using embedded templates explicitly
        // If this project doesn't have an actual `templates/` directory populated during testing,
        // it may fail the test or skip it. Assuming `templates/phaser-2d-starter` is there in the codebase:
        let registry = registry::load_templates_embedded().unwrap();
        let engine = TemplateEngine {
            registry,
            templates_path: PathBuf::from("embedded"),
            is_embedded: true,
        };
        
        assert!(engine.registry().len() > 0, "Embedded templates should not be empty");
        let template_id = engine.list_templates()[0].id.clone();
        
        let ctx = TemplateContext::new();
        let result = engine.generate_scaffold(&template_id, &output_dir, &ctx);
        assert!(result.is_ok(), "Failed to generate embedded template scaffold");
    }
}
