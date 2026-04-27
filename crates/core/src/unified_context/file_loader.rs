//! File Input Loader
//!
//! Loads and processes file inputs (markdown, code files) into the context.

use peridot_shared::PeridotResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Type of file input
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileType {
    /// Markdown documentation
    Markdown,
    /// JavaScript code
    JavaScript,
    /// TypeScript code
    TypeScript,
    /// JSON configuration
    Json,
    /// HTML file
    Html,
    /// CSS stylesheet
    Css,
    /// Plain text
    Text,
    /// Unknown type
    Unknown,
}

impl FileType {
    /// Detect file type from extension
    pub fn from_path(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "md" | "markdown" => FileType::Markdown,
            "js" | "mjs" => FileType::JavaScript,
            "ts" | "tsx" => FileType::TypeScript,
            "json" => FileType::Json,
            "html" | "htm" => FileType::Html,
            "css" => FileType::Css,
            "txt" | "text" => FileType::Text,
            _ => FileType::Unknown,
        }
    }

    /// Get the file type name
    pub fn name(&self) -> &'static str {
        match self {
            FileType::Markdown => "markdown",
            FileType::JavaScript => "javascript",
            FileType::TypeScript => "typescript",
            FileType::Json => "json",
            FileType::Html => "html",
            FileType::Css => "css",
            FileType::Text => "text",
            FileType::Unknown => "unknown",
        }
    }

    /// Check if this is a code file
    pub fn is_code(&self) -> bool {
        matches!(
            self,
            FileType::JavaScript | FileType::TypeScript | FileType::Html | FileType::Css
        )
    }

    /// Check if this is documentation
    pub fn is_documentation(&self) -> bool {
        matches!(self, FileType::Markdown | FileType::Text)
    }
}

/// A loaded file input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInput {
    /// Original path (as provided by user)
    pub original_path: String,
    /// Absolute resolved path
    pub resolved_path: PathBuf,
    /// File type
    pub file_type: FileType,
    /// File content
    pub content: String,
    /// File size in bytes
    pub size: usize,
    /// Line count
    pub line_count: usize,
    /// Whether this file was truncated (too large)
    pub truncated: bool,
    /// File metadata
    pub metadata: HashMap<String, String>,
}

impl FileInput {
    /// Create a new file input
    pub fn new(
        original_path: impl Into<String>,
        resolved_path: impl AsRef<Path>,
        content: impl Into<String>,
    ) -> Self {
        let content = content.into();
        let resolved_path = resolved_path.as_ref().to_path_buf();
        let file_type = FileType::from_path(&resolved_path);
        let size = content.len();
        let line_count = content.lines().count();

        Self {
            original_path: original_path.into(),
            resolved_path,
            file_type,
            content,
            size,
            line_count,
            truncated: false,
            metadata: HashMap::new(),
        }
    }

    /// Load a file from disk
    pub fn load(path: impl AsRef<Path>) -> PeridotResult<Self> {
        let path = path.as_ref();
        let content = peridot_fs_engine::read::read_file(path)?;
        
        Ok(Self::new(
            path.to_string_lossy(),
            path,
            content,
        ))
    }

    /// Load with size limit
    pub fn load_limited(path: impl AsRef<Path>, max_size: usize) -> PeridotResult<Self> {
        let path = path.as_ref();
        let metadata = std::fs::metadata(path)?;
        
        let (content, truncated) = if metadata.len() as usize > max_size {
            // Read first max_size bytes
            let bytes = std::fs::read(path)?;
            let truncated_content = String::from_utf8_lossy(&bytes[..max_size]);
            (truncated_content.to_string(), true)
        } else {
            (peridot_fs_engine::read::read_file(path)?, false)
        };

        let mut file_input = Self::new(
            path.to_string_lossy(),
            path,
            content,
        );
        file_input.truncated = truncated;

        Ok(file_input)
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Get content formatted for context insertion
    pub fn to_context_format(&self) -> String {
        format!(
            "### File: {} ({} lines, {} type)\n```{}\n{}\n```\n",
            self.original_path,
            self.line_count,
            self.file_type.name(),
            self.file_type.name(),
            self.content
        )
    }

    /// Get a summary of the file
    pub fn summary(&self) -> String {
        format!(
            "{} ({} lines, {} bytes)",
            self.original_path,
            self.line_count,
            self.size
        )
    }
}

/// Loader for multiple file inputs
#[derive(Debug)]
pub struct FileInputLoader {
    /// Maximum file size to load (bytes)
    max_file_size: usize,
    /// Maximum total size of all files
    max_total_size: usize,
    /// Loaded files
    loaded_files: Vec<FileInput>,
}

impl FileInputLoader {
    /// Create a new file input loader with default limits
    pub fn new() -> Self {
        Self::with_limits(1024 * 1024, 5 * 1024 * 1024) // 1MB per file, 5MB total
    }

    /// Create with custom limits
    pub fn with_limits(max_file_size: usize, max_total_size: usize) -> Self {
        Self {
            max_file_size,
            max_total_size,
            loaded_files: Vec::new(),
        }
    }

    /// Load a single file, returns the index of the loaded file
    pub fn load_file(&mut self, path: impl AsRef<Path>) -> PeridotResult<usize> {
        let file_input = FileInput::load_limited(&path, self.max_file_size)?;
        
        // Check total size
        let current_total: usize = self.loaded_files.iter().map(|f| f.size).sum();
        if current_total + file_input.size > self.max_total_size {
            return Err(peridot_shared::PeridotError::General(
                format!("Loading this file would exceed total size limit of {} bytes", self.max_total_size)
            ));
        }

        let index = self.loaded_files.len();
        self.loaded_files.push(file_input);
        Ok(index)
    }

    /// Load multiple files, returns indices of loaded files
    pub fn load_files(&mut self, paths: &[impl AsRef<Path>]) -> Vec<PeridotResult<usize>> {
        paths.iter().map(|p| self.load_file(p)).collect()
    }

    /// Load files by pattern (e.g., *.md), returns number of files loaded
    pub fn load_pattern(&mut self, project_path: impl AsRef<Path>, pattern: &str) -> PeridotResult<usize> {
        let project_path = project_path.as_ref();
        let mut count = 0;

        // Simple pattern matching for now
        if pattern.ends_with(".md") || pattern == "*.md" {
            // Find all markdown files
            if let Ok(entries) = std::fs::read_dir(project_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map(|e| e == "md").unwrap_or(false) {
                        if self.load_file(&path).is_ok() {
                            count += 1;
                        }
                    }
                }
            }
        }

        Ok(count)
    }

    /// Load documentation files (.md, .txt), returns number of files loaded
    pub fn load_documentation(&mut self, project_path: impl AsRef<Path>) -> PeridotResult<usize> {
        let project_path = project_path.as_ref();
        let mut count = 0;

        if let Ok(entries) = std::fs::read_dir(project_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                let file_type = FileType::from_path(&path);
                if file_type.is_documentation() {
                    if self.load_file(&path).is_ok() {
                        count += 1;
                    }
                }
            }
        }

        Ok(count)
    }

    /// Load code files (.js, .ts, etc.), returns number of files loaded
    pub fn load_code_files(&mut self, project_path: impl AsRef<Path>) -> PeridotResult<usize> {
        let project_path = project_path.as_ref();
        self.load_code_files_recursive(project_path)
    }

    fn load_code_files_recursive(&mut self, dir: &Path) -> PeridotResult<usize> {
        let mut count = 0;
        
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                
                if path.is_dir() {
                    // Skip node_modules and hidden directories
                    if let Some(name) = path.file_name() {
                        let name = name.to_string_lossy();
                        if !name.starts_with('.') && name != "node_modules" && name != "target" {
                            if let Ok(sub_count) = self.load_code_files_recursive(&path) {
                                count += sub_count;
                            }
                        }
                    }
                } else {
                    let file_type = FileType::from_path(&path);
                    if file_type.is_code() {
                        if self.load_file(&path).is_ok() {
                            count += 1;
                        }
                    }
                }
            }
        }

        Ok(count)
    }

    /// Get all loaded files
    pub fn files(&self) -> &[FileInput] {
        &self.loaded_files
    }

    /// Get files of a specific type
    pub fn files_of_type(&self, file_type: FileType) -> Vec<&FileInput> {
        self.loaded_files
            .iter()
            .filter(|f| f.file_type == file_type)
            .collect()
    }

    /// Get total size of loaded files
    pub fn total_size(&self) -> usize {
        self.loaded_files.iter().map(|f| f.size).sum()
    }

    /// Check if any files were truncated
    pub fn has_truncated_files(&self) -> bool {
        self.loaded_files.iter().any(|f| f.truncated)
    }

    /// Generate context string from all files
    pub fn to_context_string(&self) -> String {
        if self.loaded_files.is_empty() {
            return String::new();
        }

        let mut context = String::from("## Loaded Files\n\n");
        
        for file in &self.loaded_files {
            context.push_str(&file.to_context_format());
            context.push('\n');
        }

        context
    }

    /// Clear all loaded files
    pub fn clear(&mut self) {
        self.loaded_files.clear();
    }
}

impl Default for FileInputLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_file_type_detection() {
        assert_eq!(FileType::from_path("test.js"), FileType::JavaScript);
        assert_eq!(FileType::from_path("test.ts"), FileType::TypeScript);
        assert_eq!(FileType::from_path("test.md"), FileType::Markdown);
        assert_eq!(FileType::from_path("test.json"), FileType::Json);
        assert_eq!(FileType::from_path("test.unknown"), FileType::Unknown);
    }

    #[test]
    fn test_file_type_is_code() {
        assert!(FileType::JavaScript.is_code());
        assert!(FileType::TypeScript.is_code());
        assert!(!FileType::Markdown.is_code());
        assert!(!FileType::Text.is_code());
    }

    #[test]
    fn test_file_input_creation() {
        let input = FileInput::new("test.js", "/project/test.js", "console.log('test');");
        
        assert_eq!(input.file_type, FileType::JavaScript);
        assert_eq!(input.line_count, 1);
        assert!(!input.truncated);
    }

    #[test]
    fn test_file_input_context_format() {
        let input = FileInput::new("test.js", "/project/test.js", "console.log('test');");
        let formatted = input.to_context_format();
        
        assert!(formatted.contains("File: test.js"));
        assert!(formatted.contains("javascript"));
        assert!(formatted.contains("console.log"));
    }

    #[test]
    fn test_file_loader() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.js");
        
        let mut file = std::fs::File::create(&file_path).unwrap();
        writeln!(file, "function test() {{}}").unwrap();

        let mut loader = FileInputLoader::new();
        let result = loader.load_file(&file_path);
        
        assert!(result.is_ok());
        assert_eq!(loader.files().len(), 1);
    }

    #[test]
    fn test_file_loader_limits() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("large.js");
        
        // Create a large file
        let mut file = std::fs::File::create(&file_path).unwrap();
        for _ in 0..1000 {
            writeln!(file, "console.log('test');").unwrap();
        }

        let mut loader = FileInputLoader::with_limits(100, 1000); // Small limits
        let result = loader.load_file(&file_path);
        
        assert!(result.is_ok());
        assert!(loader.files()[0].truncated);
    }
}
