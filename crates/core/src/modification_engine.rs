//! Modification Engine
//!
//! Handles the construction of prompts for iterative code editing and
//! parsing of the AI's response into file modifications.
//!
//! This is the OpenCode-style editing engine that enables users to edit
//! their game projects through natural language prompts.

use regex::Regex;
use std::path::PathBuf;

/// Engine for iterative project modifications
#[derive(Debug)]
pub struct ModificationEngine;

/// Result of parsing AI modifications
#[derive(Debug)]
pub struct ModificationResult {
    /// Files that were modified
    pub modified_files: Vec<FileModification>,
    /// Any explanation text from the AI
    pub explanation: Option<String>,
}

/// A single file modification
#[derive(Debug, Clone)]
pub struct FileModification {
    /// Path to the file (relative to project root)
    pub path: PathBuf,
    /// New content for the file
    pub content: String,
    /// Whether this is a new file or modification
    pub _is_new: bool,
}

impl ModificationEngine {
    /// Create a new modification engine
    pub fn new() -> Self {
        Self
    }

    /// Build a prompt for the AI to modify the project
    ///
    /// # Arguments
    /// * `user_prompt` - The user's request
    /// * `files` - List of (relative_path, content) for existing project files
    pub fn build_prompt(&self, user_prompt: &str, files: &[(PathBuf, String)]) -> String {
        let mut prompt = format!(
            "You are helping modify a Phaser 2D game project. The user request is:\n\n\
             \"{}\"\n\n\
             Below are the current project files. Your task:\n\
             1. Analyze the existing code to understand the game\n\
             2. Apply the user's requested changes\n\
             3. Return the COMPLETE updated content for each modified file\n\n\
             **CRITICAL RULES:**\n\
             - Use EXACTLY this format for each file:\n\
             <file path=\"path/to/file.js\">\n\
             [FULL FILE CONTENT]\n\
             </file>\n\
             - Include ONLY files that need changes\n\
             - Provide FULL file content, not diffs or partial changes\n\
             - Do NOT wrap content in markdown code blocks\n\n\
             **Current Project Files:**\n\n",
            user_prompt
        );

        for (path, content) in files {
            prompt.push_str(&format!(
                "<file path=\"{}\">\n{}\n</file>\n\n",
                path.display(),
                content
            ));
        }

        prompt.push_str("\n**Your response with modifications:**\n");
        prompt
    }

    /// Parse the AI's response to extract file changes
    ///
    /// # Arguments
    /// * `response` - The raw string response from the AI
    ///
    /// # Returns
    /// A ModificationResult containing all modifications and any explanation
    pub fn parse_response(&self, response: &str) -> ModificationResult {
        let mut modifications = Vec::new();
        let mut explanation = None;

        // First, try to extract any explanation text (text before the first <file> tag)
        if let Some(first_file_pos) = response.find("<file path=") {
            let before_files = &response[..first_file_pos].trim();
            if !before_files.is_empty() {
                explanation = Some(before_files.to_string());
            }
        }

        // Use regex to find <file path="...">...</file> blocks
        let re = Regex::new(r#"(?s)<file\s+path\s*=\s*"([^"]+)"\s*>\n?(.*?)\n?</file>"#).unwrap();

        for cap in re.captures_iter(response) {
            let path_str = &cap[1];
            let content = Self::clean_content(&cap[2]);

            if !content.is_empty() {
                modifications.push(FileModification {
                    path: PathBuf::from(path_str),
                    content,
                    _is_new: false, // Will be determined by caller based on file existence
                });
            }
        }

        ModificationResult {
            modified_files: modifications,
            explanation,
        }
    }

    /// Clean content by removing markdown code fences and trimming
    fn clean_content(content: &str) -> String {
        let mut cleaned = content.to_string();

        // Remove markdown code block markers if present
        // Handle both ```javascript and ``` fences
        let trimmed = cleaned.trim();
        if trimmed.starts_with("```") {
            // Find the first newline after the opening fence
            if let Some(first_newline) = trimmed.find('\n') {
                let after_fence = &trimmed[first_newline + 1..];
                // Remove trailing fence if present
                if let Some(last_fence_pos) = after_fence.rfind("```") {
                    cleaned = after_fence[..last_fence_pos].trim().to_string();
                } else {
                    cleaned = after_fence.trim().to_string();
                }
            } else {
                cleaned = trimmed.to_string();
            }
        }

        cleaned.trim().to_string()
    }

    /// Get the system prompt for modification mode
    pub fn system_prompt(&self) -> &str {
        "You are an expert game developer specializing in Phaser 2D games. \
         Your task is to modify existing game code based on user requests. \
         You receive the current project files and the user's request. \
         You MUST respond with modified files in <file path=\"...\"> tags. \
         Always provide the COMPLETE file content, never partial updates or diffs. \
         Be concise but complete in your implementation."
    }

    /// Get supported file extensions for context gathering
    pub fn supported_extensions(&self) -> &[&str] {
        &[
            "js", "ts", "json", "html", "css", "md", "txt", "toml", "yaml", "yml",
        ]
    }

    /// Check if a file path should be included in context
    pub fn should_include_file(&self, path: &PathBuf) -> bool {
        let path_str = path.to_string_lossy();

        // Skip node_modules and hidden directories
        if path_str.contains("node_modules") || path_str.contains("/.") {
            return false;
        }

        // Check extension
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            self.supported_extensions().contains(&ext_str.as_str())
        } else {
            // Include files without extension if they're not hidden
            !path_str.starts_with('.')
        }
    }
}

impl Default for ModificationEngine {
    fn default() -> Self {
        Self::new()
    }
}
