//! Input Parser
//!
//! Parses raw user input into structured form with intent detection metadata.

use peridot_shared::{GameIntent, PromptInput};
use serde::{Deserialize, Serialize};

/// Parsed user input with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedInput {
    /// The raw text input from the user
    pub raw_text: String,
    /// Timestamp of the input
    pub timestamp: std::time::SystemTime,
    /// Detected intent (if any)
    pub intent: Option<GameIntent>,
    /// Extracted entities (e.g., game genre, features)
    pub entities: Vec<Entity>,
    /// Confidence score for intent detection (0.0 - 1.0)
    pub confidence: f32,
    /// Any flags or special markers detected
    pub flags: InputFlags,
}

/// Entity extracted from user input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// Entity type
    pub entity_type: EntityType,
    /// The extracted value
    pub value: String,
    /// Position in the raw text
    pub position: Option<(usize, usize)>,
    /// Confidence score
    pub confidence: f32,
}

/// Types of entities that can be extracted
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EntityType {
    /// Game genre (platformer, rpg, puzzle, etc.)
    Genre,
    /// Programming language or framework
    Language,
    /// Feature name
    Feature,
    /// File path
    FilePath,
    /// Template name
    Template,
    /// Unknown/Other
    Other,
}

/// Special flags detected in input
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InputFlags {
    /// Input contains a question
    pub is_question: bool,
    /// Input is requesting help
    pub is_help_request: bool,
    /// Input is urgent/immediate
    pub is_urgent: bool,
    /// Input contains code
    pub contains_code: bool,
    /// Input references external files
    pub references_files: bool,
}

/// Input parser for processing user prompts
#[derive(Debug)]
pub struct InputParser;

impl InputParser {
    /// Create a new input parser
    pub fn new() -> Self {
        Self
    }

    /// Parse raw input text
    pub fn parse(&self, input: impl Into<String>) -> ParsedInput {
        let raw_text = input.into();
        let timestamp = std::time::SystemTime::now();

        // Detect basic intent patterns
        let (intent, confidence) = self.detect_intent(&raw_text);

        // Extract entities
        let entities = self.extract_entities(&raw_text);

        // Detect flags
        let flags = self.detect_flags(&raw_text);

        ParsedInput {
            raw_text,
            timestamp,
            intent,
            entities,
            confidence,
            flags,
        }
    }

    /// Parse from PromptInput
    pub fn parse_prompt(&self, prompt: &PromptInput) -> ParsedInput {
        self.parse(&prompt.text)
    }

    /// Detect intent from raw text (basic rule-based)
    fn detect_intent(&self, text: &str) -> (Option<GameIntent>, f32) {
        let lower = text.to_lowercase();

        // New game patterns
        if Self::matches_any(&lower, &[
            "create a game",
            "make a game",
            "new game",
            "build a game",
            "generate a game",
            "start a game",
        ]) {
            let genre = self.extract_genre(&lower).unwrap_or_else(|| "game".to_string());
            let features = self.extract_features(&lower);
            
            return (
                Some(GameIntent::NewGame {
                    genre,
                    features,
                    description: Some(text.to_string()),
                }),
                0.8,
            );
        }

        // Add feature patterns
        if Self::matches_any(&lower, &[
            "add",
            "implement",
            "create a",
            "add a",
            "i need",
        ]) && Self::matches_any(&lower, &[
            "feature",
            "system",
            "mechanic",
            "component",
            "to my game",
            "to the game",
        ]) {
            let feature = self.extract_feature_name(&lower);
            
            return (
                Some(GameIntent::AddFeature {
                    feature,
                    context: Some(text.to_string()),
                }),
                0.7,
            );
        }

        // Modify project patterns
        if Self::matches_any(&lower, &[
            "change",
            "modify",
            "update",
            "fix",
            "refactor",
            "edit",
        ]) {
            return (
                Some(GameIntent::ModifyProject {
                    modification: text.to_string(),
                }),
                0.6,
            );
        }

        (None, 0.0)
    }

    /// Extract entities from text
    fn extract_entities(&self, text: &str) -> Vec<Entity> {
        let mut entities = Vec::new();
        let lower = text.to_lowercase();

        // Extract genre
        if let Some(genre) = self.extract_genre(&lower) {
            entities.push(Entity {
                entity_type: EntityType::Genre,
                value: genre,
                position: None,
                confidence: 0.8,
            });
        }

        // Extract features
        for feature in self.extract_features(&lower) {
            entities.push(Entity {
                entity_type: EntityType::Feature,
                value: feature,
                position: None,
                confidence: 0.7,
            });
        }

        // Extract file paths
        entities.extend(self.extract_file_paths(text));

        entities
    }

    /// Detect input flags
    fn detect_flags(&self, text: &str) -> InputFlags {
        let lower = text.to_lowercase();

        InputFlags {
            is_question: lower.ends_with('?') || lower.contains("how do i") || lower.contains("what is"),
            is_help_request: lower.contains("help") || lower.contains("how to"),
            is_urgent: lower.contains("urgent") || lower.contains("asap") || lower.contains("quick"),
            contains_code: lower.contains("```") || lower.contains("function") || lower.contains("class"),
            references_files: lower.contains(".js") || lower.contains(".ts") || lower.contains("file") || lower.contains("in "),
        }
    }

    /// Helper: Check if text matches any pattern
    fn matches_any(text: &str, patterns: &[&str]) -> bool {
        patterns.iter().any(|&p| text.contains(p))
    }

    /// Extract game genre
    fn extract_genre(&self, text: &str) -> Option<String> {
        let genres = [
            ("platformer", vec!["platformer", "platform"]),
            ("rpg", vec!["rpg", "role playing", "role-playing"]),
            ("puzzle", vec!["puzzle", "puzzles"]),
            ("shooter", vec!["shooter", "shooting", "fps"]),
            ("strategy", vec!["strategy", "rts", "turn-based"]),
            ("adventure", vec!["adventure", "action-adventure"]),
            ("arcade", vec!["arcade", "retro"]),
        ];

        for (genre, patterns) in genres {
            if patterns.iter().any(|&p| text.contains(p)) {
                return Some(genre.to_string());
            }
        }

        None
    }

    /// Extract features mentioned
    fn extract_features(&self, text: &str) -> Vec<String> {
        let feature_keywords = [
            ("jump", vec!["jump", "jumping"]),
            ("shoot", vec!["shoot", "shooting", "fire"]),
            ("inventory", vec!["inventory", "items", "storage"]),
            ("dialogue", vec!["dialogue", "conversation", "talk"]),
            ("save system", vec!["save", "save system", "saving"]),
            ("ui", vec!["ui", "interface", "menu", "hud"]),
            ("physics", vec!["physics", "gravity", "collision"]),
            ("enemy", vec!["enemy", "enemies", "ai", "npc"]),
        ];

        let mut features = Vec::new();
        for (feature, patterns) in feature_keywords {
            if patterns.iter().any(|&p| text.contains(p)) {
                features.push(feature.to_string());
            }
        }

        features
    }

    /// Extract feature name from add feature intent
    fn extract_feature_name(&self, text: &str) -> String {
        // Simple extraction: look for patterns like "add a X" or "implement X"
        let patterns = [
            ("add a ", " system"),
            ("add ", " system"),
            ("implement ", ""),
            ("create a ", ""),
        ];

        for (prefix, suffix) in patterns {
            if let Some(start) = text.find(prefix) {
                let rest = &text[start + prefix.len()..];
                let end = rest.find(' ').unwrap_or(rest.len());
                return rest[..end].to_string();
            }
        }

        "feature".to_string()
    }

    /// Extract file paths from text
    fn extract_file_paths(&self, text: &str) -> Vec<Entity> {
        let mut entities = Vec::new();
        
        // Simple pattern matching for file paths
        let extensions = [".js", ".ts", ".json", ".html", ".css", ".md"];
        let words: Vec<&str> = text.split_whitespace().collect();
        
        for (i, word) in words.iter().enumerate() {
            for ext in &extensions {
                if word.ends_with(ext) || word.contains('/') {
                    entities.push(Entity {
                        entity_type: EntityType::FilePath,
                        value: word.to_string(),
                        position: Some((i, i + 1)),
                        confidence: 0.9,
                    });
                    break;
                }
            }
        }

        entities
    }
}

impl Default for InputParser {
    fn default() -> Self {
        Self::new()
    }
}

impl ParsedInput {
    /// Create a simple parsed input (for testing)
    pub fn new(text: impl Into<String>) -> Self {
        InputParser::new().parse(text)
    }

    /// Get entities of a specific type
    pub fn entities_of_type(&self, entity_type: EntityType) -> Vec<&Entity> {
        self.entities
            .iter()
            .filter(|e| e.entity_type == entity_type)
            .collect()
    }

    /// Get file paths mentioned in input
    pub fn file_paths(&self) -> Vec<&str> {
        self.entities_of_type(EntityType::FilePath)
            .iter()
            .map(|e| e.value.as_str())
            .collect()
    }

    /// Check if this is a high-confidence parse
    pub fn is_confident(&self) -> bool {
        self.confidence >= 0.7
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_new_game_intent() {
        let parser = InputParser::new();
        
        // Test with "create a game" pattern
        let input1 = parser.parse("Create a game with jumping");
        assert!(matches!(input1.intent, Some(GameIntent::NewGame { .. })));
        assert!(input1.confidence > 0.5);
        
        // Test with "make a game" pattern
        let input2 = parser.parse("Make a game");
        assert!(matches!(input2.intent, Some(GameIntent::NewGame { .. })));
        
        // Test with "build a game" pattern
        let input3 = parser.parse("Build a game");
        assert!(matches!(input3.intent, Some(GameIntent::NewGame { .. })));
    }

    #[test]
    fn test_parse_add_feature_intent() {
        let parser = InputParser::new();
        let input = parser.parse("Add an inventory system to my game");

        assert!(matches!(input.intent, Some(GameIntent::AddFeature { .. })));
    }

    #[test]
    fn test_parse_modify_intent() {
        let parser = InputParser::new();
        let input = parser.parse("Fix the player movement bug");

        assert!(matches!(input.intent, Some(GameIntent::ModifyProject { .. })));
    }

    #[test]
    fn test_entity_extraction() {
        let parser = InputParser::new();
        let input = parser.parse("Create a platformer with jumping and shooting in src/game.js");

        let genres: Vec<_> = input.entities_of_type(EntityType::Genre);
        assert!(!genres.is_empty());
        assert_eq!(genres[0].value, "platformer");

        let files: Vec<_> = input.entities_of_type(EntityType::FilePath);
        assert!(!files.is_empty());
    }

    #[test]
    fn test_flag_detection() {
        let parser = InputParser::new();
        
        let input1 = parser.parse("How do I create a game?");
        assert!(input1.flags.is_question);

        let input2 = parser.parse("I need help with phaser");
        assert!(input2.flags.is_help_request);

        let input3 = parser.parse("```javascript\nfunction test() {}\n```");
        assert!(input3.flags.contains_code);
    }

    #[test]
    fn test_extract_genre() {
        let parser = InputParser::new();
        
        assert_eq!(parser.extract_genre("create an rpg"), Some("rpg".to_string()));
        assert_eq!(parser.extract_genre("make a puzzle game"), Some("puzzle".to_string()));
        assert_eq!(parser.extract_genre("do something"), None);
    }
}
