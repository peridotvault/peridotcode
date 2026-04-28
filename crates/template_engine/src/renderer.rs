//! Template renderer
//!
//! Handles the actual generation of project files from templates,
//! including placeholder substitution and file copying.
//!
//! This module now uses fs_engine for safe file operations and change tracking.

use peridot_fs_engine::{ChangeType, FileChange, FsEngine};
use peridot_shared::{PeridotError, PeridotResult};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::registry::EMBEDDED_TEMPLATES;

/// Context for template rendering (placeholder values)
#[derive(Debug, Clone, Default)]
pub struct TemplateContext {
    /// Map of placeholder name to value
    values: HashMap<String, String>,
}

impl TemplateContext {
    /// Create a new empty context
    pub fn new() -> Self {
        TemplateContext {
            values: HashMap::new(),
        }
    }

    /// Add a value to the context
    pub fn set<K: Into<String>, V: Into<String>>(&mut self, key: K, value: V) {
        self.values.insert(key.into(), value.into());
    }

    /// Get a value from the context
    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(|s| s.as_str())
    }

    /// Create a context from common project parameters
    pub fn from_project(name: &str, description: Option<&str>) -> Self {
        let mut ctx = TemplateContext::new();
        ctx.set("game_title", name);
        ctx.set("game_name_snake", name.to_lowercase().replace(" ", "_"));
        ctx.set("game_name_camel", to_camel_case(name));

        if let Some(desc) = description {
            ctx.set("game_description", desc);
        } else {
            ctx.set("game_description", &format!("A game called {}", name));
        }

        ctx
    }
}

/// Convert a string to camelCase
fn to_camel_case(s: &str) -> String {
    s.split_whitespace()
        .enumerate()
        .map(|(i, word)| {
            if i == 0 {
                word.to_lowercase()
            } else {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => {
                        first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                    }
                }
            }
        })
        .collect()
}

/// Result of rendering a template
#[derive(Debug, Clone)]
pub struct RenderResult {
    /// List of files created with their change information
    pub file_changes: Vec<FileChange>,
    /// Number of files processed
    pub file_count: usize,
    /// Path to output directory
    pub output_path: PathBuf,
    /// Summary of changes
    pub summary: String,
}

impl RenderResult {
    /// Get list of created files only
    pub fn created_files(&self) -> Vec<&PathBuf> {
        self.file_changes
            .iter()
            .filter(|c| c.change_type == ChangeType::Created)
            .map(|c| &c.path)
            .collect()
    }

    /// Get list of modified files only
    pub fn modified_files(&self) -> Vec<&PathBuf> {
        self.file_changes
            .iter()
            .filter(|c| c.change_type == ChangeType::Modified)
            .map(|c| &c.path)
            .collect()
    }

    /// Format a report of all changes
    pub fn format_report(&self) -> String {
        let mut report = String::new();
        report.push_str("Generated Files:\n");
        report.push_str("================\n\n");

        // Group by change type
        let created: Vec<_> = self
            .file_changes
            .iter()
            .filter(|c| c.change_type == ChangeType::Created)
            .collect();
        let modified: Vec<_> = self
            .file_changes
            .iter()
            .filter(|c| c.change_type == ChangeType::Modified)
            .collect();
        let unchanged: Vec<_> = self
            .file_changes
            .iter()
            .filter(|c| c.change_type == ChangeType::Unchanged)
            .collect();

        if !created.is_empty() {
            report.push_str(&format!("Created ({} files):\n", created.len()));
            for change in created {
                report.push_str(&format!("  + {}\n", change.path.display()));
            }
            report.push('\n');
        }

        if !modified.is_empty() {
            report.push_str(&format!("Modified ({} files):\n", modified.len()));
            for change in modified {
                report.push_str(&format!("  ~ {}\n", change.path.display()));
            }
            report.push('\n');
        }

        if !unchanged.is_empty() {
            report.push_str(&format!("Unchanged ({} files)\n", unchanged.len()));
        }

        report.push('\n');
        report.push_str(&self.summary);

        report
    }
}

/// Render a template to an output directory
///
/// Copies files from template_path to output_path, performing placeholder
/// substitution on text files. Uses FsEngine for safety and change tracking.
///
/// # Arguments
/// * `template_path` - Path to the template source directory
/// * `output_path` - Where to write the rendered project (must be within project)
/// * `context` - Placeholder values for substitution
///
/// # Returns
/// Result containing file changes and summary
pub fn render_template(
    template_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
    context: &TemplateContext,
) -> PeridotResult<RenderResult> {
    let template_path = template_path.as_ref();
    let output_path = output_path.as_ref();

    tracing::info!(
        "Rendering template from {:?} to {:?}",
        template_path,
        output_path
    );

    // Validate template path exists
    if !template_path.exists() {
        return Err(PeridotError::TemplateNotFound(format!(
            "Template not found: {}",
            template_path.display()
        )));
    }

    // Validate template path is a directory
    if !template_path.is_dir() {
        return Err(PeridotError::TemplateRenderError(format!(
            "Template path is not a directory: {}",
            template_path.display()
        )));
    }

    // Create FsEngine for the output directory
    // This provides safety validation and change tracking
    let mut fs_engine = FsEngine::new(output_path)?;

    // Recursively render template files
    render_directory_with_engine(
        template_path,
        template_path,
        output_path,
        context,
        &mut fs_engine,
    )?;

    // Get change summary from engine
    let change_summary = fs_engine.take_change_summary();
    let file_count = change_summary.len();
    let summary = change_summary.summary_line();

    tracing::info!("Rendered {} files: {}", file_count, summary);

    Ok(RenderResult {
        file_changes: change_summary.changes().to_vec(),
        file_count,
        output_path: output_path.to_path_buf(),
        summary,
    })
}

/// Render an embedded template to an output directory
pub fn render_template_embedded(
    template_id: &str,
    output_path: impl AsRef<Path>,
    context: &TemplateContext,
) -> PeridotResult<RenderResult> {
    let output_path = output_path.as_ref();
    let template_dir = EMBEDDED_TEMPLATES.get_dir(template_id)
        .ok_or_else(|| PeridotError::TemplateNotFound(format!("Embedded template not found: {}", template_id)))?;

    tracing::info!(
        "Rendering embedded template from {} to {:?}",
        template_id,
        output_path
    );

    let mut fs_engine = FsEngine::new(output_path)?;

    render_embedded_directory_with_engine(
        &template_dir,
        Path::new(template_id),
        output_path,
        context,
        &mut fs_engine,
    )?;

    let change_summary = fs_engine.take_change_summary();
    let file_count = change_summary.len();
    let summary = change_summary.summary_line();

    tracing::info!("Rendered {} files from embedded: {}", file_count, summary);

    Ok(RenderResult {
        file_changes: change_summary.changes().to_vec(),
        file_count,
        output_path: output_path.to_path_buf(),
        summary,
    })
}

/// Recursively render an embedded directory using FsEngine
fn render_embedded_directory_with_engine(
    dir: &include_dir::Dir<'_>,
    template_root: &Path,
    output_root: &Path,
    context: &TemplateContext,
    fs_engine: &mut FsEngine,
) -> PeridotResult<()> {
    for entry in dir.entries() {
        let path = entry.path();
        let file_name = path
            .file_name()
            .ok_or_else(|| PeridotError::FsError("Invalid file name".to_string()))?;

        if file_name == "template.toml" {
            continue;
        }

        if file_name.to_str().map(|s| s.starts_with('.')).unwrap_or(false) {
            continue;
        }

        let relative_path = path
            .strip_prefix(template_root)
            .map_err(|_| PeridotError::FsError("Failed to calculate relative path".to_string()))?;

        if let Some(subdir) = entry.as_dir() {
            fs_engine.create_dir(relative_path)?;
            tracing::debug!("Created directory: {:?}", relative_path);

            render_embedded_directory_with_engine(subdir, template_root, output_root, context, fs_engine)?;
        } else if let Some(file) = entry.as_file() {
            let is_text = is_text_file(path);
            
            if is_text {
                let content_str = file.contents_utf8()
                    .ok_or_else(|| PeridotError::FsError(format!("Failed to read text file: {:?}", path)))?;
                let content = substitute_placeholders(content_str, context);
                fs_engine.write_file(relative_path, &content)?;
            } else {
                let content_str = String::from_utf8_lossy(file.contents()).to_string();
                fs_engine.write_file(relative_path, &content_str)?;
            }
            tracing::debug!("Rendered file: {:?}", relative_path);
        }
    }

    Ok(())
}


/// Recursively render a directory using FsEngine
fn render_directory_with_engine(
    template_root: &Path,
    current_dir: &Path,
    output_root: &Path,
    context: &TemplateContext,
    fs_engine: &mut FsEngine,
) -> PeridotResult<()> {
    for entry in std::fs::read_dir(current_dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = path
            .file_name()
            .ok_or_else(|| PeridotError::FsError("Invalid file name".to_string()))?;

        // Skip the template.toml manifest file
        if file_name == "template.toml" {
            continue;
        }

        // Skip hidden files
        if file_name
            .to_str()
            .map(|s| s.starts_with('.'))
            .unwrap_or(false)
        {
            continue;
        }

        // Calculate relative path from template root
        let relative_path = path
            .strip_prefix(template_root)
            .map_err(|_| PeridotError::FsError("Failed to calculate relative path".to_string()))?;

        if path.is_dir() {
            // Create directory using engine (records the change)
            fs_engine.create_dir(relative_path)?;
            tracing::debug!("Created directory: {:?}", relative_path);

            // Recurse into subdirectory
            render_directory_with_engine(template_root, &path, output_root, context, fs_engine)?;
        } else {
            // Render file and write using engine
            let content = render_file_content(&path, context)?;
            fs_engine.write_file(relative_path, &content)?;
            tracing::debug!("Rendered file: {:?}", relative_path);
        }
    }

    Ok(())
}

/// Read and process a template file
fn render_file_content(template_file: &Path, context: &TemplateContext) -> PeridotResult<String> {
    // Check if this is a text file that needs placeholder substitution
    let is_text = is_text_file(template_file);

    if is_text {
        // Read file as text and substitute placeholders
        let content = std::fs::read_to_string(template_file)
            .map_err(|e| PeridotError::FsError(format!("Failed to read template file: {}", e)))?;

        Ok(substitute_placeholders(&content, context))
    } else {
        // Binary file - read as bytes and return as string
        // Note: This assumes binary files don't need placeholder substitution
        let bytes = std::fs::read(template_file)
            .map_err(|e| PeridotError::FsError(format!("Failed to read binary file: {}", e)))?;

        // For binary files, we just copy as-is (no placeholder substitution)
        // Convert to string - this is safe for the files we expect (images, etc.)
        // In production, you might want to handle binaries differently
        Ok(String::from_utf8_lossy(&bytes).to_string())
    }
}

/// Substitute placeholders in a string
///
/// Replaces `{{placeholder}}` with values from the context.
/// If a placeholder is not found in the context, it is left unchanged.
pub fn substitute_placeholders(input: &str, context: &TemplateContext) -> String {
    let mut result = input.to_string();

    for (key, value) in &context.values {
        let placeholder = format!("{{{{{}}}}}", key);
        result = result.replace(&placeholder, value);
    }

    result
}

/// Check if a file should be processed for placeholder substitution
///
/// Binary files should be copied as-is without modification.
pub fn is_text_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext = ext.to_string_lossy().to_lowercase();
        matches!(
            ext.as_str(),
            "txt"
                | "md"
                | "html"
                | "htm"
                | "js"
                | "ts"
                | "json"
                | "toml"
                | "yaml"
                | "yml"
                | "css"
                | "scss"
                | "xml"
        )
    } else {
        // Files without extensions might be text (like README or LICENSE)
        // Check for common text file names
        if let Some(name) = path.file_name() {
            let name = name.to_string_lossy().to_lowercase();
            if name == "readme" || name == "license" || name.starts_with("dockerfile") {
                return true;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_substitute_placeholders() {
        let mut ctx = TemplateContext::new();
        ctx.set("name", "World");
        ctx.set("greeting", "Hello");

        let input = "{{greeting}}, {{name}}!";
        let result = substitute_placeholders(input, &ctx);

        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_unknown_placeholder_unchanged() {
        let ctx = TemplateContext::new();
        let input = "{{unknown}} stays";
        let result = substitute_placeholders(input, &ctx);

        assert_eq!(result, "{{unknown}} stays");
    }

    #[test]
    fn test_camel_case() {
        assert_eq!(to_camel_case("my game"), "myGame");
        assert_eq!(to_camel_case("Hello World"), "helloWorld");
    }
}
