//! Intent Classification
//!
//! Provides simple, deterministic classification of user prompts into
//! actionable intents. Uses keyword-based matching for MVP.
//!
//! # Classification Rules
//! - **CreateNewGame**: Keywords like "make", "create", "build", "new game"
//! - **AddFeature**: Keywords like "add", "include", "implement"
//! - **Unsupported**: Anything that doesn't match above

use peridot_shared::PromptInput;

/// Classification result with confidence score
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Classification {
    /// The detected intent
    pub intent: Intent,
    /// Confidence level (0-100)
    pub confidence: u8,
    /// Extracted parameters from the prompt
    pub params: IntentParams,
}

/// Supported intents for the orchestrator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Intent {
    /// Create a new game project
    CreateNewGame,
    /// Add a feature to an existing project
    AddFeature,
    /// Unsupported or unrecognized request
    Unsupported,
}

impl Intent {
    /// Get a human-readable name for this intent
    pub fn display_name(&self) -> &'static str {
        match self {
            Intent::CreateNewGame => "Create New Game",
            Intent::AddFeature => "Add Feature",
            Intent::Unsupported => "Unsupported",
        }
    }

    /// Check if this intent is supported
    pub fn is_supported(&self) -> bool {
        !matches!(self, Intent::Unsupported)
    }
}

/// Parameters extracted from the prompt
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IntentParams {
    /// Game genre (e.g., "platformer", "rpg")
    pub genre: Option<String>,
    /// Requested features (e.g., "inventory", "dialogue")
    pub features: Vec<String>,
    /// The feature to add (for AddFeature intent)
    pub feature_name: Option<String>,
    /// Raw prompt text
    pub raw_prompt: String,
}

/// Classifier for user intents
#[derive(Debug, Default)]
pub struct IntentClassifier;

impl IntentClassifier {
    /// Create a new classifier
    pub fn new() -> Self {
        IntentClassifier
    }

    /// Classify a prompt into an intent
    pub fn classify(&self, input: &PromptInput) -> Classification {
        let text = input.text.to_lowercase();
        let params = extract_params(&text);

        // Check for new game creation
        if has_keywords(&text, &["make", "create", "build", "generate", "new game"]) {
            return Classification {
                intent: Intent::CreateNewGame,
                confidence: calculate_confidence(&text, Intent::CreateNewGame),
                params,
            };
        }

        // Check for adding features
        if has_keywords(&text, &["add", "include", "implement"]) {
            return Classification {
                intent: Intent::AddFeature,
                confidence: calculate_confidence(&text, Intent::AddFeature),
                params,
            };
        }

        // Unsupported intent
        Classification {
            intent: Intent::Unsupported,
            confidence: 0,
            params,
        }
    }
}

fn has_keywords(text: &str, keywords: &[&str]) -> bool {
    keywords.iter().any(|&kw| text.contains(kw))
}

fn extract_params(text: &str) -> IntentParams {
    IntentParams {
        genre: extract_genre(text),
        features: extract_features(text),
        feature_name: extract_feature_name(text),
        raw_prompt: text.to_string(),
    }
}

fn extract_genre(text: &str) -> Option<String> {
    if text.contains("platformer") {
        Some("platformer".to_string())
    } else if text.contains("rpg") || text.contains("adventure") {
        Some("rpg".to_string())
    } else if text.contains("puzzle") {
        Some("puzzle".to_string())
    } else {
        None
    }
}

fn extract_features(text: &str) -> Vec<String> {
    let mut features = Vec::new();
    if text.contains("inventory") {
        features.push("inventory".to_string());
    }
    if text.contains("dialogue") {
        features.push("dialogue".to_string());
    }
    if text.contains("save") {
        features.push("save_system".to_string());
    }
    features
}

fn extract_feature_name(text: &str) -> Option<String> {
    if let Some(pos) = text.find("add") {
        let after = &text[pos + 3..];
        let words: Vec<&str> = after.split_whitespace().take(2).collect();
        if !words.is_empty() {
            return Some(words.join(" "));
        }
    }
    None
}

fn calculate_confidence(text: &str, intent: Intent) -> u8 {
    let keywords = match intent {
        Intent::CreateNewGame => &["make", "create", "build", "game"][..],
        Intent::AddFeature => &["add", "feature"][..],
        Intent::Unsupported => &[][..],
    };
    let matches = keywords.iter().filter(|&&kw| text.contains(kw)).count();
    ((matches as f32 / keywords.len().max(1) as f32) * 100.0) as u8
}
