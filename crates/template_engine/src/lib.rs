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
}

impl TemplateEngine {
    /// Create a new TemplateEngine with the default templates directory
    ///
    /// The default location is `./templates` relative to the current working directory.
    /// In production, this should be the installation directory.
    pub fn new() -> PeridotResult<Self> {
        // Default to templates/ in current directory
        // TODO: Use proper installation path in production
        let templates_path = PathBuf::from("templates");
        Self::with_path(templates_path)
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
            });
        }

        let registry = load_templates(&templates_path)?;

        Ok(TemplateEngine {
            registry,
            templates_path,
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

        let template_path = self.templates_path.join(template_id.as_ref());

        tracing::info!(
            "Generating scaffold from template '{}' ({}) to {:?}",
            template_id.as_ref(),
            template.name,
            output_path.as_ref()
        );

        // Validate template directory exists
        if !template_path.exists() {
            return Err(PeridotError::TemplateRenderError(format!(
                "Template directory not found: {}",
                template_path.display()
            )));
        }

        // Render the template
        let result = render_template(&template_path, output_path, context)?;

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
        // Create with empty registry if default path doesn't work
        Self {
            registry: TemplateRegistry::new(),
            templates_path: PathBuf::from("templates"),
        }
    }
}
