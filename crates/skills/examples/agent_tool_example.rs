//! Agent Tool Integration Example
//!
//! This example demonstrates how the agent can use tools to modify code
//! from a natural language prompt.
//!
//! # Example Usage
//!
//! ```rust,no_run
//! use peridot_core::agent_loop::{AgentLoop, AgentConfig};
//! use peridot_skills::tools::{ToolDispatcher, ToolContext, ToolCall};
//! use peridot_skills::tools::dispatcher::ModelGatewayClient;
//!
//! // Create the agent
//! let config = AgentConfig::default();
//! let agent = AgentLoop::new(config, gateway_client);
//!
//! // Create tool dispatcher
//! let mut dispatcher = ToolDispatcher::new();
//! let tool_context = ToolContext::new("/path/to/project");
//!
//! // Process a prompt that requires code modification
//! let result = agent.process("Add a jump method to the Player class in src/player.js").await?;
//!
//! // The agent can now dispatch tool calls based on the response
//! if result.tools_used {
//!     // Execute the tool call with LLM support
//!     let call = ToolCall::new("modify_code", params);
//!     let tool_result = dispatcher.execute_with_llm(
//!         call,
//!         &tool_context,
//!         &llm_client,
//!         "openai/gpt-4"
//!     ).await;
//! }
//! ```

use peridot_skills::tools::{
    dispatcher::{ModelGatewayClient, ToolCall, ToolDispatcher},
    modify_code::ModifyCodeParams,
    read_file::ReadFileParams,
    ToolContext,
};

/// Example: Agent modifies code from a prompt
///
/// This function demonstrates the end-to-end flow:
/// 1. User enters a prompt
/// 2. Agent classifies the intent (modify_code)
/// 3. Tool dispatcher executes the modification
/// 4. Changes are applied and saved
async fn example_modify_code_from_prompt() {
    // Setup
    let project_path = "/path/to/game-project";
    let tool_context = ToolContext::new(project_path);
    let mut dispatcher = ToolDispatcher::new();

    // Step 1: User prompt
    let user_prompt = "Add a jump method to the Player class in src/player.js";
    println!("User: {}", user_prompt);

    // Step 2: Agent would classify this as a modify_code intent
    // (This would come from the AgentLoop's LLM classification)
    let tool_call = ToolCall::new(
        "modify_code",
        serde_json::json!({
            "file_path": "src/player.js",
            "instruction": "Add a jump method to the Player class that sets a vertical velocity",
            "context": "The player should be able to jump when the spacebar is pressed"
        }),
    );

    // Step 3: Execute the tool call
    println!("Executing tool: {}", tool_call.tool_id);

    // In real usage, you'd have an LLM client from the model gateway
    // let provider = ...; // Get from ConfigManager
    // let llm_client = ModelGatewayClient::new(&provider);
    // let model = "openai/gpt-4";
    // let result = dispatcher.execute_with_llm(tool_call, &tool_context, &llm_client, model).await;

    println!("Tool execution would proceed here with LLM client...");
}

/// Example: Reading a file before modification
///
/// Best practice: Read the file first to understand its structure,
/// then make targeted modifications.
async fn example_read_then_modify() {
    let project_path = "/path/to/game-project";
    let tool_context = ToolContext::new(project_path);
    let mut dispatcher = ToolDispatcher::new();

    // Step 1: Read the file first
    let read_call = ToolCall::new(
        "read_file",
        serde_json::json!({
            "file_path": "src/player.js",
            "max_lines": 50
        }),
    );

    println!("Step 1: Reading file to understand structure...");
    // Without LLM - just read
    // let read_result = dispatcher.execute(read_call, &tool_context).await;

    // Step 2: Now modify based on what we read
    let modify_call = ToolCall::new(
        "modify_code",
        serde_json::json!({
            "file_path": "src/player.js",
            "instruction": "Add a jump() method that applies upward velocity",
            "context": "The Player class uses a physics body with velocity property"
        }),
    );

    println!("Step 2: Modifying code based on file contents...");
    // With LLM for code generation
    // let modify_result = dispatcher.execute_with_llm(modify_call, &tool_context, &llm_client, model).await;
}

/// Example: Tool definitions for LLM function calling
///
/// These definitions can be sent to the LLM so it knows what tools are available.
fn example_tool_definitions() {
    let dispatcher = ToolDispatcher::new();
    let definitions = dispatcher.get_tool_definitions();

    println!("Available tools for LLM:");
    for def in definitions {
        let name = def
            .get("function")
            .and_then(|f| f.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("unknown");
        let description = def
            .get("function")
            .and_then(|f| f.get("description"))
            .and_then(|d| d.as_str())
            .unwrap_or("");
        println!("  - {}: {}", name, description);
    }
}

/// Main example runner
fn main() {
    println!("=== PeridotCode Agent Tool Examples ===\n");

    println!("1. Tool Definitions:");
    example_tool_definitions();

    println!("\n2. Example: Modify code from prompt");
    println!("   (This would be an async function in practice)");

    println!("\n3. Example: Read then modify");
    println!("   (This would be an async function in practice)");

    println!("\n=== End of Examples ===");
    println!("\nThe agent can now:");
    println!("  - Read files safely with read_file tool");
    println!("  - Modify code with LLM assistance via modify_code tool");
    println!("  - Route actions through ToolDispatcher");
    println!("  - Execute tools with proper error handling");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_tool_dispatcher_read_file() {
        // Create a temp project
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // Create a test file
        let test_file = project_path.join("test.js");
        let mut file = std::fs::File::create(&test_file).unwrap();
        writeln!(file, "function test() {{}}").unwrap();

        // Setup dispatcher
        let mut dispatcher = ToolDispatcher::new();
        let tool_context = ToolContext::new(project_path);

        // Create tool call
        let call = ToolCall::new(
            "read_file",
            serde_json::json!({
                "file_path": "test.js"
            }),
        );

        // Execute (without LLM - just read)
        let result = dispatcher.execute(call, &tool_context).await;

        assert!(result.result.is_success());
        assert_eq!(result.tool_id, "read_file");
    }

    #[test]
    fn test_tool_context_creation() {
        let ctx = ToolContext::new("/project");
        assert_eq!(ctx.project_path, std::path::PathBuf::from("/project"));

        let resolved = ctx.resolve_path("src/main.js");
        assert_eq!(resolved, std::path::PathBuf::from("/project/src/main.js"));
    }
}
