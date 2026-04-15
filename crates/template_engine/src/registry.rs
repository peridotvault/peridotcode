//! Template registry
//!
//! Manages loading and accessing available templates.

use peridot_shared::{constants, PeridotError, PeridotResult, TemplateId, TemplateManifest};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use include_dir::{include_dir, Dir};

/// Directory containing embedded templates at compile time
pub static EMBEDDED_TEMPLATES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../../templates");

/// Registry of available templates
#[derive(Debug, Default)]
pub struct TemplateRegistry {
    /// Map of template ID to manifest
    templates: HashMap<TemplateId, TemplateManifest>,
}

impl TemplateRegistry {
    /// Create an empty registry
    pub fn new() -> Self {
        TemplateRegistry {
            templates: HashMap::new(),
        }
    }

    /// Register a template
    pub fn register(&mut self, manifest: TemplateManifest) {
        self.templates.insert(manifest.id.clone(), manifest);
    }

    /// Get a template by ID
    pub fn get(&self, id: &TemplateId) -> Option<&TemplateManifest> {
        self.templates.get(id)
    }

    /// Check if a template exists
    pub fn contains(&self, id: &TemplateId) -> bool {
        self.templates.contains_key(id)
    }

    /// List all registered templates
    pub fn list(&self) -> Vec<&TemplateManifest> {
        self.templates.values().collect()
    }

    /// Get the number of registered templates
    pub fn len(&self) -> usize {
        self.templates.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.templates.is_empty()
    }
}

/// Load all templates from the templates directory
///
/// Scans the directory for subdirectories containing `template.toml` files
/// and loads them into a registry.
///
/// # Errors
/// Returns an error if the templates directory cannot be read or
/// if a template manifest is invalid.
pub fn load_templates(templates_dir: impl AsRef<Path>) -> PeridotResult<TemplateRegistry> {
    let templates_dir = templates_dir.as_ref();
    let mut registry = TemplateRegistry::new();

    if !templates_dir.exists() {
        tracing::warn!(
            "Templates directory does not exist: {}",
            templates_dir.display()
        );
        return Ok(registry);
    }

    // Iterate through template directories
    for entry in fs::read_dir(templates_dir)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let manifest_path = path.join(constants::TEMPLATE_MANIFEST);
        if manifest_path.exists() {
            match load_template_manifest(&manifest_path) {
                Ok(manifest) => {
                    tracing::info!("Loaded template: {}", manifest.id.as_ref());
                    registry.register(manifest);
                }
                Err(e) => {
                    tracing::error!("Failed to load template from {:?}: {}", path, e);
                }
            }
        }
    }

    Ok(registry)
}

/// Load a single template manifest from a file
fn load_template_manifest(path: &Path) -> PeridotResult<TemplateManifest> {
    let content = fs::read_to_string(path)?;
    let manifest: TemplateManifest = toml::from_str(&content).map_err(|e| {
        PeridotError::Serialization(format!("Failed to parse template manifest: {e}"))
    })?;

    Ok(manifest)
}

/// Load templates from embedded compiled representations
pub fn load_templates_embedded() -> PeridotResult<TemplateRegistry> {
    let mut registry = TemplateRegistry::new();

    for entry in EMBEDDED_TEMPLATES.entries() {
        if let Some(dir) = entry.as_dir() {
            let path = dir.path();
            let manifest_path = path.join(constants::TEMPLATE_MANIFEST);
            
            if let Some(manifest_file) = dir.get_file(manifest_path) {
                if let Some(content) = manifest_file.contents_utf8() {
                    match toml::from_str::<TemplateManifest>(content) {
                        Ok(manifest) => {
                            tracing::info!("Loaded embedded template: {}", manifest.id.as_ref());
                            registry.register(manifest);
                        }
                        Err(e) => {
                            tracing::error!("Failed to parse embedded template manifest from {:?}: {}", path, e);
                        }
                    }
                }
            }
        }
    }

    Ok(registry)
}

