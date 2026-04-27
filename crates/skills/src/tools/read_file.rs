//! Read File Tool
//!
//! Tool for safely reading file contents with size limits and encoding detection.
//! This is the foundation tool used by other tools like modify_code.

use peridot_fs_engine::read::read_file as fs_read_file;
use peridot_shared::PeridotResult;
use serde::{Deserialize, Serialize};

use crate::tools::{Tool, ToolContext, ToolResult};

/// Parameters for the read_file tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadFileParams {
    /// Path to the file to read (relative to project root)
    pub file_path: String,
    /// Maximum number of lines to read (optional, for large files)
    #[serde(default)]
    pub max_lines: Option<usize>,
    /// Offset to start reading from (optional)
    #[serde(default)]
    pub offset: Option<usize>,
}

/// Result of reading a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadFileResult {
    /// The file contents
    pub content: String,
    /// Total lines in file
    pub total_lines: usize,
    /// Lines read (may be less than total if max_lines was specified)
    pub lines_read: usize,
    /// Whether the file was truncated
    pub truncated: bool,
    /// File path that was read
    pub file_path: String,
}

/// Read file tool implementation
#[derive(Debug)]
pub struct ReadFileTool;

#[async_trait::async_trait]
impl Tool for ReadFileTool {
    fn id(&self) -> &str {
        "read_file"
    }

    fn name(&self) -> &str {
        "Read File"
    }

    fn description(&self) -> &str {
        "Read the contents of a file safely. Returns the file content as text. \
         Use this to examine existing code before modifying it. \
         Supports reading partial content with offset and max_lines parameters."
    }

    fn parameter_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Path to the file to read (relative to project root)"
                },
                "max_lines": {
                    "type": "integer",
                    "description": "Maximum number of lines to read (optional, for large files)",
                    "minimum": 1,
                    "maximum": 1000
                },
                "offset": {
                    "type": "integer",
                    "description": "Line number to start reading from (0-indexed, optional)",
                    "minimum": 0
                }
            },
            "required": ["file_path"]
        })
    }

    async fn execute(
        &self,
        params: serde_json::Value,
        context: &ToolContext,
    ) -> ToolResult {
        let params: ReadFileParams = match serde_json::from_value(params) {
            Ok(p) => p,
            Err(e) => {
                return ToolResult::error_with_details(
                    "Invalid parameters",
                    format!("Failed to parse parameters: {}", e),
                );
            }
        };

        match read_file_tool(&params, context).await {
            Ok(result) => match ToolResult::success_with_data("File read successfully", result) {
                Ok(r) => r,
                Err(e) => ToolResult::error(format!("Failed to serialize result: {}", e)),
            },
            Err(e) => ToolResult::error(format!("Failed to read file: {}", e)),
        }
    }
}

/// Execute the read_file tool
pub async fn read_file_tool(params: &ReadFileParams, context: &ToolContext) -> PeridotResult<ReadFileResult> {
    let file_path = context.resolve_path(&params.file_path);

    // Safety check: ensure path is within project
    if !context.is_within_project(&file_path) {
        return Err(peridot_shared::PeridotError::FsError(format!(
            "Path '{}' is outside project boundaries",
            params.file_path
        )));
    }

    // Read the file
    let content = fs_read_file(&file_path)?;

    // Calculate line information
    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();

    // Apply offset if specified
    let start_line = params.offset.unwrap_or(0);
    let lines: Vec<&str> = lines.into_iter().skip(start_line).collect();

    // Apply max_lines limit if specified
    let (content, lines_read, truncated) = if let Some(max) = params.max_lines {
        if lines.len() > max {
            let limited: Vec<_> = lines.into_iter().take(max).collect();
            let content = limited.join("\n");
            (content, max, true)
        } else {
            let content = lines.join("\n");
            (content, lines.len(), false)
        }
    } else {
        let content = lines.join("\n");
        (content, lines.len(), false)
    };

    tracing::info!(
        "Read file '{}' - {} lines ({} total)",
        params.file_path,
        lines_read,
        total_lines
    );

    Ok(ReadFileResult {
        content,
        total_lines,
        lines_read,
        truncated,
        file_path: params.file_path.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_read_file_tool() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();
        let file_path = project_path.join("test.txt");

        // Create test file
        let mut file = std::fs::File::create(&file_path).unwrap();
        writeln!(file, "Line 1").unwrap();
        writeln!(file, "Line 2").unwrap();
        writeln!(file, "Line 3").unwrap();

        let context = ToolContext::new(project_path);
        let params = ReadFileParams {
            file_path: "test.txt".to_string(),
            max_lines: None,
            offset: None,
        };

        let result = read_file_tool(&params, &context).await.unwrap();

        assert_eq!(result.total_lines, 3);
        assert_eq!(result.lines_read, 3);
        assert!(!result.truncated);
        assert!(result.content.contains("Line 1"));
    }

    #[tokio::test]
    async fn test_read_file_with_offset_and_limit() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();
        let file_path = project_path.join("test.txt");

        // Create test file with multiple lines
        let mut file = std::fs::File::create(&file_path).unwrap();
        for i in 1..=10 {
            writeln!(file, "Line {}", i).unwrap();
        }

        let context = ToolContext::new(project_path);
        let params = ReadFileParams {
            file_path: "test.txt".to_string(),
            max_lines: Some(3),
            offset: Some(2), // Start from Line 3
        };

        let result = read_file_tool(&params, &context).await.unwrap();

        assert_eq!(result.total_lines, 10);
        assert_eq!(result.lines_read, 3);
        assert!(result.truncated);
        assert!(result.content.contains("Line 3"));
        assert!(result.content.contains("Line 5"));
        assert!(!result.content.contains("Line 1"));
    }

    #[tokio::test]
    async fn test_read_file_outside_project() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        let context = ToolContext::new(project_path);
        let params = ReadFileParams {
            file_path: "../outside.txt".to_string(),
            max_lines: None,
            offset: None,
        };

        let result = read_file_tool(&params, &context).await;
        assert!(result.is_err());
    }
}
