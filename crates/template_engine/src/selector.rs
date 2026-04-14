//! Template selector
//!
//! Chooses the most appropriate template based on user intent.
//!
//! MVP Implementation:
//! - Always returns the default Phaser 2D template if available
//! - Future: Match genre and features to available templates

use peridot_shared::TemplateId;

/// Select the best template for a given user intent
///
/// # Arguments
/// * `_intent_hint` - Hint about the desired template (e.g., genre)
/// * `available_templates` - List of template IDs that can be used
///
/// # Returns
/// The selected template ID, or None if no suitable template exists
pub fn select_template(
    _intent_hint: Option<&str>,
    available_templates: &[TemplateId],
) -> Option<TemplateId> {
    // MVP: Always use the phaser-2d-starter template if available
    let default_id = TemplateId::new("phaser-2d-starter");

    if available_templates.contains(&default_id) {
        return Some(default_id);
    }

    // If default not available, return the first available template
    available_templates.first().cloned()
}

/// Match score for a template against intent
#[derive(Debug, Clone)]
pub struct TemplateMatch {
    /// The template ID
    pub template_id: TemplateId,
    /// Match score (0-100, higher is better)
    pub score: u8,
}

/// Calculate how well a template matches the intent
///
/// TODO: Implement proper scoring algorithm based on:
/// - Game genre matching
/// - Feature support
/// - Stack compatibility
pub fn calculate_match_score(intent_hint: Option<&str>, template_id: &TemplateId) -> u8 {
    // Placeholder: return high score if template matches phaser-2d-starter
    if template_id.as_ref() == "phaser-2d-starter" {
        return 90;
    }

    // Lower score for other templates
    let _ = intent_hint;
    50
}
