//! Example: Using the Agent to Modify Code
//!
//! This example demonstrates how to use the UnifiedContext and tools
//! to modify code from a prompt programmatically.

use peridot_core::unified_context::{
    conversation::ConversationMemory,
    input_parser::InputParser,
    prompt_builder::ContextMerger,
    templates::load_template,
    UnifiedContext,
};
use std::path::PathBuf;

/// Example: Modify code in a test project
fn main() {
    println!("=== PeridotCode Agent Testing Example ===\n");

    // Setup test scenario
    let project_path = PathBuf::from("./test-project");
    
    println!("1. Testing Input Parsing:");
    test_input_parsing();
    
    println!("\n2. Testing Unified Context:");
    test_unified_context(&project_path);
    
    println!("\n3. Testing Template Loading:");
    test_template_loading();
    
    println!("\n4. Testing Context Merger:");
    test_context_merger(&project_path);
    
    println!("\n=== Test Complete ===");
}

fn test_input_parsing() {
    let parser = InputParser::new();
    
    // Test new game intent
    let input = parser.parse("Create a platformer game with jumping");
    println!("  Input: '{}'", input.raw_text);
    println!("  Intent: {:?}", input.intent.as_ref().map(|i| i.display_name()));
    println!("  Confidence: {:.2}", input.confidence);
    
    // Test modify intent
    let input2 = parser.parse("Fix the player movement bug in src/player.js");
    println!("  Input: '{}'", input2.raw_text);
    println!("  Intent: {:?}", input2.intent.as_ref().map(|i| i.display_name()));
    
    // Show entities
    if !input.entities.is_empty() {
        println!("  Entities found:");
        for entity in &input.entities {
            println!("    - {:?}: {}", entity.entity_type, entity.value);
        }
    }
}

fn test_unified_context(project_path: &PathBuf) {
    use peridot_core::unified_context::input_parser::ParsedInput;
    
    // Create parsed input
    let parsed_input = ParsedInput::new("Add a jump method to the Player class");
    
    // Build unified context
    let context = UnifiedContext::new(parsed_input)
        .with_project(project_path)
        .unwrap_or_else(|_| {
            println!("  Note: Project path doesn't exist, using empty project context");
            UnifiedContext::new(ParsedInput::new("test"))
        })
        .with_conversation(ConversationMemory::with_capacity(5))
        .with_metadata("session_id", "test-session-123")
        .with_metadata("test_mode", "true");
    
    println!("  Context created successfully");
    println!("  Project type: {}", context.project.project_type);
    println!("  Is new project: {}", context.project.is_new);
    println!("  Metadata entries: {}", context.metadata.len());
    
    // Check intent detection
    if let Some(intent) = context.intent() {
        println!("  Detected intent: {}", intent.display_name());
    }
}

fn test_template_loading() {
    match load_template("phaser-2d-starter") {
        Ok(template) => {
            println!("  Loaded template: {}", template.name);
            println!("  Description: {}", template.description);
            println!("  Stack: {:?}", template.stack);
            println!("  Files: {} files included", template.files.len());
        }
        Err(e) => {
            println!("  Template loading requires filesystem templates");
            println!("  Error: {}", e);
        }
    }
}

fn test_context_merger(project_path: &PathBuf) {
    use peridot_core::unified_context::input_parser::ParsedInput;
    
    // Create a context with a modification request
    let parsed_input = ParsedInput::new("Add a health system to the player");
    
    let context = UnifiedContext::new(parsed_input)
        .with_project(project_path)
        .unwrap_or_else(|_| UnifiedContext::new(ParsedInput::new("test")))
        .with_conversation(ConversationMemory::new());
    
    // Build the prompt
    let built_prompt = ContextMerger::build(&context);
    
    println!("  System prompt length: {} chars", built_prompt.system_prompt.len());
    println!("  User prompt length: {} chars", built_prompt.user_prompt.len());
    
    // Show a snippet
    let system_snippet: String = built_prompt.system_prompt.chars().take(100).collect();
    println!("  System prompt snippet: '{}'...", system_snippet);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;
    
    #[test]
    fn test_full_agent_workflow() {
        // Create a temporary test project
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();
        
        // Create a test file
        let test_file = project_path.join("player.js");
        let mut file = std::fs::File::create(&test_file).unwrap();
        writeln!(file, "class Player {{}}").unwrap();
        
        // Parse input
        let parser = InputParser::new();
        let input = parser.parse("Add a jump method to Player");
        
        // Verify intent detection
        assert!(input.intent.is_some());
        
        // Build context
        let context = UnifiedContext::new(input)
            .with_project(project_path)
            .expect("Should create context");
        
        // Verify project detection
        assert!(context.project.files.len() > 0);
        
        // Build prompt
        let prompt = ContextMerger::build(&context);
        
        // Verify prompt was built
        assert!(!prompt.system_prompt.is_empty());
        assert!(!prompt.user_prompt.is_empty());
        
        println!("Full workflow test passed!");
    }
    
    #[test]
    fn test_file_loading_into_context() {
        use peridot_core::unified_context::file_loader::FileInputLoader;
        
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();
        
        // Create test files
        let js_file = project_path.join("game.js");
        std::fs::write(&js_file, "console.log('test');").unwrap();
        
        let md_file = project_path.join("README.md");
        std::fs::write(&md_file, "# Test Project").unwrap();
        
        // Load files
        let mut loader = FileInputLoader::new();
        let count = loader.load_code_files(project_path).expect("Should load files");
        
        assert_eq!(count, 1); // Should find game.js
        assert_eq!(loader.files().len(), 1);
        
        // Verify file was loaded
        let file = &loader.files()[0];
        assert_eq!(file.file_type.name(), "javascript");
        assert!(file.content.contains("console.log"));
        
        println!("File loading test passed!");
    }
}
