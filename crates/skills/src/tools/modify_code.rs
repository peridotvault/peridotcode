//! Modify Code Tool
//!
//! Tool for modifying existing code files using LLM assistance.
//!
//! # Workflow
//!
//! 1. Read the existing file
//! 2. Send code + instruction to LLM
//! 3. Parse LLM response to extract modified code
//! 4. Apply and save changes via fs_engine

use peridot_fs_engine::read::read_file as fs_read_file;
use peridot_fs_engine::write::write_file as fs_write_file;
use peridot_model_gateway::InferenceRequest;
use peridot_shared::PeridotResult;
use serde::{Deserialize, Serialize};

use crate::tools::{Tool, ToolContext, ToolResult};

/// Parameters for the modify_code tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModifyCodeParams {
    /// Path to the file to modify
    pub file_path: String,
    /// Description of the change to make
    pub instruction: String,
    /// Context about why the change is needed (optional)
    #[serde(default)]
    pub context: Option<String>,
}

/// Result of modifying code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModifyCodeResult {
    /// Path to the modified file
    pub file_path: String,
    /// Whether changes were applied
    pub changes_applied: bool,
    /// Description of what was changed
    pub change_summary: String,
    /// Number of lines changed
    pub lines_changed: usize,
    /// Original content (for diff purposes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_content: Option<String>,
    /// Modified content (for diff purposes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_content: Option<String>,
}

/// LLM response format for code modifications
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CodeModificationResponse {
    /// Brief summary of changes made
    pub summary: String,
    /// The modified code (complete file content)
    pub modified_code: String,
    /// Explanation of the changes
    pub explanation: Option<String>,
}

/// Modify code tool implementation
#[derive(Debug)]
pub struct ModifyCodeTool;

#[async_trait::async_trait]
impl Tool for ModifyCodeTool {
    fn id(&self) -> &str {
        "modify_code"
    }

    fn name(&self) -> &str {
        "Modify Code"
    }

    fn description(&self) -> &str {
        "Modify existing code in a file. \
         Reads the file, sends it to LLM with your instructions, \
         and applies the suggested changes. \
         Use this to fix bugs, add features, or refactor code."
    }

    fn parameter_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Path to the file to modify (relative to project root)"
                },
                "instruction": {
                    "type": "string",
                    "description": "Description of the change to make (e.g., 'Add a jump method to the Player class')"
                },
                "context": {
                    "type": "string",
                    "description": "Additional context about why this change is needed (optional)"
                }
            },
            "required": ["file_path", "instruction"]
        })
    }

    async fn execute(
        &self,
        _params: serde_json::Value,
        _context: &ToolContext,
    ) -> ToolResult {
        let _parsed: ModifyCodeParams = match serde_json::from_value(_params) {
            Ok(p) => p,
            Err(e) => {
                return ToolResult::error_with_details(
                    "Invalid parameters",
                    format!("Failed to parse parameters: {}", e),
                );
            }
        };

        // Note: This requires a model gateway client which we don't have access to here
        // The actual implementation would need to be called from the dispatcher with the client
        ToolResult::error("modify_code requires model gateway client - use modify_code() function directly")
    }
}

/// Context for LLM-based operations
pub struct LlmContext<'a> {
    /// Model gateway client for inference
    pub client: &'a dyn LlmClient,
    /// Model to use for inference
    pub model: String,
}

/// Trait for LLM clients (to allow mocking in tests)
#[async_trait::async_trait]
pub trait LlmClient: Send + Sync {
    /// Perform inference with the LLM
    async fn infer(&self, request: InferenceRequest) -> PeridotResult<peridot_model_gateway::InferenceResponse>;
}

/// Modify code using LLM assistance
///
/// This is the main function that performs the code modification workflow:
/// 1. Read the existing file
/// 2. Build a prompt with the code and instructions
/// 3. Send to LLM
/// 4. Parse the response
/// 5. Apply the changes
pub async fn modify_code(
    params: &ModifyCodeParams,
    tool_context: &ToolContext,
    llm_context: &LlmContext<'_>,
) -> PeridotResult<ModifyCodeResult> {
    let file_path = tool_context.resolve_path(&params.file_path);

    // Safety check: ensure path is within project
    if !tool_context.is_within_project(&file_path) {
        return Err(peridot_shared::PeridotError::FsError(format!(
            "Path '{}' is outside project boundaries",
            params.file_path
        )));
    }

    // Step 1: Read the existing file
    let original_content = fs_read_file(&file_path)?;
    let original_lines: Vec<&str> = original_content.lines().collect();

    tracing::info!(
        "Modifying file '{}' ({} lines)",
        params.file_path,
        original_lines.len()
    );

    // Step 2: Build the modification prompt
    let prompt = build_modification_prompt(&original_content, &params.instruction, params.context.as_deref());

    // Step 3: Send to LLM
    let request = InferenceRequest::new(&llm_context.model)
        .with_system(CODE_MODIFICATION_SYSTEM_PROMPT)
        .with_user(prompt)
        .with_temperature(0.2); // Lower temperature for code generation

    let response = llm_context.client.infer(request).await?;
    let response_text = response.content().to_string();

    // Step 4: Parse the response
    let modification = parse_modification_response(&response_text)?;

    // Step 5: Check if there are actual changes
    if modification.modified_code == original_content {
        tracing::info!("No changes needed for '{}'", params.file_path);
        return Ok(ModifyCodeResult {
            file_path: params.file_path.clone(),
            changes_applied: false,
            change_summary: "No changes were necessary".to_string(),
            lines_changed: 0,
            original_content: None,
            modified_content: None,
        });
    }

    // Step 6: Apply changes
    fs_write_file(&file_path, &modification.modified_code)?;

    // Calculate lines changed (rough estimate)
    let modified_lines: Vec<&str> = modification.modified_code.lines().collect();
    let lines_changed = if modified_lines.len() > original_lines.len() {
        modified_lines.len() - original_lines.len()
    } else {
        original_lines.len() - modified_lines.len()
    };

    tracing::info!(
        "Successfully modified '{}': {}",
        params.file_path,
        modification.summary
    );

    Ok(ModifyCodeResult {
        file_path: params.file_path.clone(),
        changes_applied: true,
        change_summary: modification.summary,
        lines_changed,
        original_content: Some(original_content),
        modified_content: Some(modification.modified_code),
    })
}

/// System prompt for code modification
const CODE_MODIFICATION_SYSTEM_PROMPT: &str = r#"You are a code modification assistant. Your task is to modify code according to the user's instructions.

Rules:
1. Return ONLY valid, complete code - no explanations outside the code block
2. Preserve the original file structure and style
3. Make minimal, targeted changes
4. Ensure the code remains syntactically correct
5. Include all necessary imports and dependencies

You must respond in this exact JSON format:
```json
{
  "summary": "Brief description of changes made",
  "modified_code": "The complete modified file content here",
  "explanation": "Optional: explain complex changes"
}
```

The modified_code field must contain the COMPLETE file content, not just the changed parts."#;

/// Build the prompt for code modification
fn build_modification_prompt(code: &str, instruction: &str, context: Option<&str>) -> String {
    let mut prompt = format!(
        "Please modify the following code according to this instruction:\n\n{}\n\n",
        instruction
    );

    if let Some(ctx) = context {
        prompt.push_str(&format!("Context: {}\n\n", ctx));
    }

    prompt.push_str("Here is the current code:\n\n");
    prompt.push_str("```\n");
    prompt.push_str(code);
    prompt.push_str("\n```\n\n");

    prompt.push_str("Please provide the complete modified code in the JSON format specified.");

    prompt
}

/// Parse the LLM response to extract the code modification
fn parse_modification_response(response: &str) -> PeridotResult<CodeModificationResponse> {
    // Try to find JSON in the response
    let json_text = extract_json_from_response(response)
        .ok_or_else(|| peridot_shared::PeridotError::General(
            "Could not find valid JSON in LLM response".to_string()
        ))?;

    // Parse the JSON
    let modification: CodeModificationResponse = serde_json::from_str(&json_text)
        .map_err(|e| peridot_shared::PeridotError::General(
            format!("Failed to parse modification response: {}", e)
        ))?;

    Ok(modification)
}

/// Extract JSON from an LLM response (handles markdown code blocks)
fn extract_json_from_response(text: &str) -> Option<String> {
    let text = text.trim();

    // Try to find JSON in markdown code block
    if let Some(start) = text.find("```json") {
        // Find the closing ``` after the opening ```json
        let search_start = start + 7; // Length of "```json"
        if let Some(end) = text[search_start..].find("```") {
            let json_content = &text[search_start..search_start + end];
            return Some(json_content.trim().to_string());
        }
    }

    // Try regular code block
    if let Some(start) = text.find("```") {
        let search_start = start + 3;
        if let Some(end) = text[search_start..].find("```") {
            let json_content = &text[search_start..search_start + end];
            return Some(json_content.trim().to_string());
        }
    }

    // Try plain JSON (look for curly braces)
    if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            return Some(text[start..=end].to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_extract_json_from_response() {
        // Test markdown code block with json label
        let response1 = r#"Some text
```json
{"summary": "test", "modified_code": "code"}
```"#;
        let json1 = extract_json_from_response(response1).unwrap();
        assert!(json1.contains("summary"));

        // Test plain code block
        let response2 = r#"```
{"summary": "test", "modified_code": "code"}
```"#;
        let json2 = extract_json_from_response(response2).unwrap();
        assert!(json2.contains("summary"));

        // Test plain JSON
        let response3 = r#"{"summary": "test", "modified_code": "code"}"#;
        let json3 = extract_json_from_response(response3).unwrap();
        assert!(json3.contains("test"));
    }

    #[test]
    fn test_parse_modification_response() {
        let response = r#"```json
{
  "summary": "Added console.log statement",
  "modified_code": "console.log('Hello');",
  "explanation": "Added logging"
}
```"#;

        let result = parse_modification_response(response).unwrap();
        assert_eq!(result.summary, "Added console.log statement");
        assert_eq!(result.modified_code, "console.log('Hello');");
        assert_eq!(result.explanation, Some("Added logging".to_string()));
    }

    #[test]
    fn test_build_modification_prompt() {
        let code = "function test() {}";
        let instruction = "Add a return statement";
        let context = Some("This is needed for debugging");

        let prompt = build_modification_prompt(code, instruction, context);

        assert!(prompt.contains("Add a return statement"));
        assert!(prompt.contains("function test()"));
        assert!(prompt.contains("This is needed for debugging"));
    }
}
