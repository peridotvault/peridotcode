//! Prompt Builder
//!
//! Builds system prompts and merges context from multiple sources
//! to create the final prompt for the LLM.

use crate::unified_context::{
    conversation::ConversationMemory,
    file_loader::FileInput,
    input_parser::ParsedInput,
    ProjectContextInfo, UnifiedContext,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// System prompt template for different intents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPromptTemplate {
    /// Template identifier
    pub id: String,
    /// Template name
    pub name: String,
    /// Base system prompt
    pub base_prompt: String,
    /// Intent-specific additions
    pub intent_prompts: HashMap<String, String>,
    /// Output format instructions
    pub output_format: OutputFormat,
    /// Constraints and rules
    pub constraints: Vec<String>,
}

/// Output format specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputFormat {
    /// Format type (json, markdown, text)
    pub format_type: FormatType,
    /// JSON schema for structured output
    pub schema: Option<serde_json::Value>,
    /// Example output
    pub example: Option<String>,
    /// Required fields
    pub required_fields: Vec<String>,
}

/// Format types for output
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FormatType {
    /// JSON structured output
    Json,
    /// Markdown formatted text
    Markdown,
    /// Plain text
    Text,
    /// Code block
    Code,
}

impl OutputFormat {
    /// Create a JSON output format
    pub fn json(schema: serde_json::Value) -> Self {
        Self {
            format_type: FormatType::Json,
            schema: Some(schema),
            example: None,
            required_fields: Vec::new(),
        }
    }

    /// Create a code output format
    pub fn code(language: impl Into<String>) -> Self {
        Self {
            format_type: FormatType::Code,
            schema: None,
            example: None,
            required_fields: vec![language.into()],
        }
    }

    /// Get format instructions
    pub fn instructions(&self) -> String {
        match self.format_type {
            FormatType::Json => {
                let mut instr = "Respond with valid JSON.".to_string();
                if let Some(schema) = &self.schema {
                    instr.push_str(&format!("\nSchema: {}", serde_json::to_string_pretty(schema).unwrap_or_default()));
                }
                instr
            }
            FormatType::Markdown => "Respond with Markdown formatting.".to_string(),
            FormatType::Text => "Respond with plain text.".to_string(),
            FormatType::Code => {
                let lang = self.required_fields.first().cloned().unwrap_or_default();
                format!("Respond with {} code in a code block.", lang)
            }
        }
    }
}

/// Predefined system prompt templates
impl SystemPromptTemplate {
    /// Create the default template
    pub fn default_template() -> Self {
        Self {
            id: "default".to_string(),
            name: "Default".to_string(),
            base_prompt: DEFAULT_BASE_PROMPT.to_string(),
            intent_prompts: Self::default_intent_prompts(),
            output_format: OutputFormat {
                format_type: FormatType::Json,
                schema: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "action": {"type": "string"},
                        "summary": {"type": "string"},
                        "message": {"type": "string"},
                        "params": {"type": "object"}
                    },
                    "required": ["action", "summary"]
                })),
                example: None,
                required_fields: vec!["action".to_string(), "summary".to_string()],
            },
            constraints: vec![
                "Be concise and helpful".to_string(),
                "Ask for clarification if needed".to_string(),
            ],
        }
    }

    /// Create a template for new game creation
    pub fn new_game_template() -> Self {
        Self {
            id: "new_game".to_string(),
            name: "New Game Creation".to_string(),
            base_prompt: NEW_GAME_PROMPT.to_string(),
            intent_prompts: HashMap::new(),
            output_format: OutputFormat {
                format_type: FormatType::Json,
                schema: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "action": {"type": "string", "enum": ["create_game"]},
                        "genre": {"type": "string"},
                        "features": {"type": "array", "items": {"type": "string"}},
                        "description": {"type": "string"}
                    },
                    "required": ["action", "genre"]
                })),
                example: None,
                required_fields: vec!["action".to_string(), "genre".to_string()],
            },
            constraints: vec![
                "Focus on playable scaffolds".to_string(),
                "Keep code editable and understandable".to_string(),
                "Use the provided template structure".to_string(),
            ],
        }
    }

    /// Create a template for code modification
    pub fn modify_code_template() -> Self {
        Self {
            id: "modify_code".to_string(),
            name: "Code Modification".to_string(),
            base_prompt: MODIFY_CODE_PROMPT.to_string(),
            intent_prompts: HashMap::new(),
            output_format: OutputFormat::code("javascript"),
            constraints: vec![
                "Preserve existing code structure".to_string(),
                "Make minimal targeted changes".to_string(),
                "Ensure code remains syntactically valid".to_string(),
            ],
        }
    }

    /// Build the complete system prompt
    pub fn build_prompt(&self, intent: Option<&str>) -> String {
        let mut prompt = self.base_prompt.clone();
        
        // Add intent-specific prompt if available
        if let Some(intent) = intent {
            if let Some(intent_prompt) = self.intent_prompts.get(intent) {
                prompt.push_str("\n\n");
                prompt.push_str(intent_prompt);
            }
        }

        // Add constraints
        if !self.constraints.is_empty() {
            prompt.push_str("\n\nConstraints:\n");
            for constraint in &self.constraints {
                prompt.push_str(&format!("- {}\n", constraint));
            }
        }

        // Add output format instructions
        prompt.push_str("\n\n");
        prompt.push_str(&self.output_format.instructions());

        prompt
    }

    fn default_intent_prompts() -> HashMap<String, String> {
        let mut prompts = HashMap::new();
        
        prompts.insert(
            "new_game".to_string(),
            "The user wants to create a new game. Help them define the genre, features, and structure.".to_string(),
        );
        
        prompts.insert(
            "add_feature".to_string(),
            "The user wants to add a feature to an existing game. Help them integrate it properly.".to_string(),
        );
        
        prompts.insert(
            "modify_project".to_string(),
            "The user wants to modify existing code. Make precise, targeted changes.".to_string(),
        );
        
        prompts
    }
}

/// Default base prompt
const DEFAULT_BASE_PROMPT: &str = r#"You are PeridotCode, an AI game development assistant.

Your role is to help users create and modify game projects using the Phaser framework.

Guidelines:
- Provide clear, actionable responses
- Use the available tools when appropriate
- Ask for clarification if the request is unclear
- Prioritize working, playable code over perfection
- Keep generated code editable and understandable
"#;

/// New game creation prompt
const NEW_GAME_PROMPT: &str = r#"You are helping create a new Phaser game project.

Your task is to:
1. Understand the user's game concept
2. Select appropriate features and mechanics
3. Generate a playable scaffold using the template

Focus on:
- Core gameplay loop
- Essential features
- Runnable code
- Clear structure

Use the provided template as the foundation.
"#;

/// Code modification prompt
const MODIFY_CODE_PROMPT: &str = r#"You are modifying existing game code.

Your task is to:
1. Understand the current code structure
2. Apply the requested changes
3. Ensure the code remains functional

Rules:
- Preserve existing functionality
- Make minimal necessary changes
- Maintain code style consistency
- Add comments for complex logic
"#;

/// Builder for constructing prompts from unified context
#[derive(Debug)]
pub struct PromptBuilder {
    /// System prompt template
    system_template: SystemPromptTemplate,
    /// Buffer for building the prompt
    buffer: String,
}

impl PromptBuilder {
    /// Create a new prompt builder
    pub fn new(template: SystemPromptTemplate) -> Self {
        Self {
            system_template: template,
            buffer: String::new(),
        }
    }

    /// Create with default template
    pub fn default() -> Self {
        Self::new(SystemPromptTemplate::default_template())
    }

    /// Set the system template
    pub fn with_template(mut self, template: SystemPromptTemplate) -> Self {
        self.system_template = template;
        self
    }

    /// Add project context
    pub fn add_project_context(&mut self, project: &ProjectContextInfo) -> &mut Self {
        self.buffer.push_str("## Project Context\n\n");
        self.buffer.push_str(&format!("Path: {}\n", project.path.display()));
        self.buffer.push_str(&format!("Type: {}\n", project.project_type));
        self.buffer.push_str(&format!("Is New: {}\n", project.is_new));
        
        if !project.files.is_empty() {
            self.buffer.push_str(&format!("Files: {} total\n", project.files.len()));
        }
        
        self.buffer.push('\n');
        self
    }

    /// Add conversation history
    pub fn add_conversation(&mut self, conversation: &ConversationMemory, max_turns: usize) -> &mut Self {
        let context = conversation.to_context_string(max_turns);
        if !context.is_empty() {
            self.buffer.push_str(&context);
            self.buffer.push('\n');
        }
        self
    }

    /// Add file inputs
    pub fn add_files(&mut self, files: &[FileInput]) -> &mut Self {
        if files.is_empty() {
            return self;
        }

        self.buffer.push_str("## Reference Files\n\n");
        
        for file in files {
            self.buffer.push_str(&file.to_context_format());
            self.buffer.push('\n');
        }
        
        self
    }

    /// Add user input
    pub fn add_user_input(&mut self, input: &ParsedInput) -> &mut Self {
        self.buffer.push_str("## Current Request\n\n");
        self.buffer.push_str(&input.raw_text);
        self.buffer.push_str("\n\n");
        
        // Add intent info if available
        if let Some(intent) = &input.intent {
            self.buffer.push_str(&format!("Intent: {}\n", intent.display_name()));
        }
        
        self
    }

    /// Build the final prompt
    pub fn build(self) -> BuiltPrompt {
        BuiltPrompt {
            system_prompt: self.system_template.build_prompt(None),
            user_prompt: self.buffer,
            output_format: self.system_template.output_format.clone(),
        }
    }

    /// Build with intent-specific prompt
    pub fn build_with_intent(self, intent: &str) -> BuiltPrompt {
        BuiltPrompt {
            system_prompt: self.system_template.build_prompt(Some(intent)),
            user_prompt: self.buffer,
            output_format: self.system_template.output_format.clone(),
        }
    }
}

impl Default for PromptBuilder {
    fn default() -> Self {
        Self::default()
    }
}

/// A fully built prompt ready for LLM inference
#[derive(Debug, Clone)]
pub struct BuiltPrompt {
    /// System prompt
    pub system_prompt: String,
    /// User/context prompt
    pub user_prompt: String,
    /// Expected output format
    pub output_format: OutputFormat,
}

impl BuiltPrompt {
    /// Get the complete prompt (system + user)
    pub fn full_prompt(&self) -> String {
        format!(
            "{system}\n\n{user}",
            system = self.system_prompt,
            user = self.user_prompt
        )
    }

    /// Get messages for chat API
    pub fn to_messages(&self) -> Vec<peridot_model_gateway::Message> {
        vec![
            peridot_model_gateway::Message::system(&self.system_prompt),
            peridot_model_gateway::Message::user(&self.user_prompt),
        ]
    }
}

/// High-level context merger for building prompts from UnifiedContext
pub struct ContextMerger;

impl ContextMerger {
    /// Build a prompt from unified context
    pub fn build(context: &UnifiedContext) -> BuiltPrompt {
        // Select appropriate template based on intent
        let template = if context.is_new_game_request() {
            SystemPromptTemplate::new_game_template()
        } else if context.is_modify_request() {
            SystemPromptTemplate::modify_code_template()
        } else {
            SystemPromptTemplate::default_template()
        };

        let mut builder = PromptBuilder::new(template);

        // Add project context
        builder.add_project_context(&context.project);

        // Add conversation history
        builder.add_conversation(&context.conversation, 5);

        // Add file inputs
        builder.add_files(&context.files);

        // Add user input
        builder.add_user_input(&context.user_input);

        // Build with appropriate intent
        let intent_str = context.intent().map(|i| i.display_name());
        match intent_str {
            Some("New Game") => builder.build_with_intent("new_game"),
            Some("Add Feature") => builder.build_with_intent("add_feature"),
            Some("Modify Project") => builder.build_with_intent("modify_project"),
            _ => builder.build(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_format_instructions() {
        let json_format = OutputFormat::json(serde_json::json!({"type": "object"}));
        assert!(json_format.instructions().contains("JSON"));

        let code_format = OutputFormat::code("javascript");
        assert!(code_format.instructions().contains("javascript"));
    }

    #[test]
    fn test_system_prompt_template_build() {
        let template = SystemPromptTemplate::default_template();
        let prompt = template.build_prompt(None);

        assert!(prompt.contains("PeridotCode"));
        assert!(prompt.contains("JSON"));
    }

    #[test]
    fn test_prompt_builder() {
        let builder = PromptBuilder::default();
        let prompt = builder.build();

        assert!(!prompt.system_prompt.is_empty());
        assert!(prompt.user_prompt.is_empty()); // No context added
    }

    #[test]
    fn test_built_prompt_full() {
        let prompt = BuiltPrompt {
            system_prompt: "System: Be helpful".to_string(),
            user_prompt: "User: Hello".to_string(),
            output_format: OutputFormat::json(serde_json::json!({})),
        };

        let full = prompt.full_prompt();
        assert!(full.contains("System: Be helpful"));
        assert!(full.contains("User: Hello"));
    }
}
