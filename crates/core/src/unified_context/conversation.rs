//! Conversation Memory
//!
//! Manages short-term conversation history for the agent.
//! Provides storage, retrieval, and summarization of conversation turns.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::SystemTime;

/// A single turn in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationTurn {
    /// Unique identifier for this turn
    pub id: String,
    /// Timestamp when the turn occurred
    pub timestamp: SystemTime,
    /// User's input
    pub user_input: String,
    /// Agent's response
    pub agent_response: String,
    /// Tools used during this turn
    pub tools_used: Vec<String>,
    /// Any files that were modified
    pub files_modified: Vec<String>,
    /// Context metadata
    pub metadata: std::collections::HashMap<String, String>,
}

/// Short-term conversation memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMemory {
    /// Maximum number of turns to remember
    max_turns: usize,
    /// The conversation history (oldest first)
    turns: VecDeque<ConversationTurn>,
    /// Session identifier
    session_id: String,
    /// When the conversation started
    started_at: SystemTime,
}

impl ConversationMemory {
    /// Create a new conversation memory with default capacity
    pub fn new() -> Self {
        Self::with_capacity(10)
    }

    /// Create with specific capacity
    pub fn with_capacity(max_turns: usize) -> Self {
        Self {
            max_turns,
            turns: VecDeque::with_capacity(max_turns),
            session_id: Self::generate_session_id(),
            started_at: SystemTime::now(),
        }
    }

    /// Add a new turn to the conversation
    pub fn add_turn(&mut self, user_input: impl Into<String>, agent_response: impl Into<String>) -> &ConversationTurn {
        let turn = ConversationTurn {
            id: format!("turn_{}", self.turns.len()),
            timestamp: SystemTime::now(),
            user_input: user_input.into(),
            agent_response: agent_response.into(),
            tools_used: Vec::new(),
            files_modified: Vec::new(),
            metadata: std::collections::HashMap::new(),
        };

        // Add to front (newest first for iteration)
        self.turns.push_front(turn);

        // Trim if exceeding capacity
        while self.turns.len() > self.max_turns {
            self.turns.pop_back();
        }

        // Return reference to the added turn
        self.turns.front().unwrap()
    }

    /// Add a turn with tool usage
    pub fn add_turn_with_tools(
        &mut self,
        user_input: impl Into<String>,
        agent_response: impl Into<String>,
        tools_used: Vec<String>,
        files_modified: Vec<String>,
    ) -> &ConversationTurn {
        let user_input = user_input.into();
        let agent_response = agent_response.into();
        
        let turn = ConversationTurn {
            id: format!("turn_{}", self.turns.len()),
            timestamp: SystemTime::now(),
            user_input,
            agent_response,
            tools_used,
            files_modified,
            metadata: std::collections::HashMap::new(),
        };

        self.turns.push_front(turn);

        while self.turns.len() > self.max_turns {
            self.turns.pop_back();
        }

        self.turns.front().unwrap()
    }

    /// Get recent turns (newest first)
    pub fn recent_turns(&self, count: usize) -> Vec<&ConversationTurn> {
        self.turns.iter().take(count).collect()
    }

    /// Get all turns (newest first)
    pub fn all_turns(&self) -> &VecDeque<ConversationTurn> {
        &self.turns
    }

    /// Get the last turn
    pub fn last_turn(&self) -> Option<&ConversationTurn> {
        self.turns.front()
    }

    /// Get turn count
    pub fn len(&self) -> usize {
        self.turns.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.turns.is_empty()
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.turns.clear();
        self.started_at = SystemTime::now();
    }

    /// Generate a summary of the conversation
    pub fn summary(&self) -> String {
        if self.turns.is_empty() {
            return "No previous conversation.".to_string();
        }

        let mut summary = format!("Conversation ({} turns):\n", self.turns.len());
        
        for (i, turn) in self.turns.iter().enumerate().take(5) {
            summary.push_str(&format!(
                "  {}. User: {}\n     Agent: {}\n",
                i + 1,
                Self::truncate(&turn.user_input, 50),
                Self::truncate(&turn.agent_response, 50)
            ));
        }

        if self.turns.len() > 5 {
            summary.push_str(&format!("  ... and {} more turns\n", self.turns.len() - 5));
        }

        summary
    }

    /// Get formatted conversation for LLM context
    pub fn to_context_string(&self, max_turns: usize) -> String {
        if self.turns.is_empty() {
            return String::new();
        }

        let mut context = String::new();
        context.push_str("Previous conversation:\n\n");

        // Get recent turns in chronological order (reverse from newest-first)
        let turns: Vec<_> = self.turns.iter().take(max_turns).rev().collect();

        for turn in turns {
            context.push_str(&format!("User: {}\n", turn.user_input));
            context.push_str(&format!("Assistant: {}\n\n", turn.agent_response));
        }

        context
    }

    /// Get files that have been modified in this conversation
    pub fn modified_files(&self) -> Vec<&str> {
        let mut files = std::collections::HashSet::new();
        for turn in &self.turns {
            for file in &turn.files_modified {
                files.insert(file.as_str());
            }
        }
        files.into_iter().collect()
    }

    /// Get tools that have been used
    pub fn tools_used(&self) -> Vec<&str> {
        let mut tools = std::collections::HashSet::new();
        for turn in &self.turns {
            for tool in &turn.tools_used {
                tools.insert(tool.as_str());
            }
        }
        tools.into_iter().collect()
    }

    /// Get session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Generate a unique session ID
    fn generate_session_id() -> String {
        format!("session_{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs())
    }

    /// Truncate string for display
    fn truncate(s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else {
            format!("{}...", &s[..max_len])
        }
    }
}

impl Default for ConversationMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl ConversationTurn {
    /// Create a new conversation turn
    pub fn new(user_input: impl Into<String>, agent_response: impl Into<String>) -> Self {
        Self {
            id: String::new(),
            timestamp: SystemTime::now(),
            user_input: user_input.into(),
            agent_response: agent_response.into(),
            tools_used: Vec::new(),
            files_modified: Vec::new(),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Add tool usage
    pub fn with_tool(mut self, tool: impl Into<String>) -> Self {
        self.tools_used.push(tool.into());
        self
    }

    /// Add file modification
    pub fn with_file(mut self, file: impl Into<String>) -> Self {
        self.files_modified.push(file.into());
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversation_memory_creation() {
        let memory = ConversationMemory::new();
        assert!(memory.is_empty());
        assert_eq!(memory.len(), 0);
    }

    #[test]
    fn test_add_turn() {
        let mut memory = ConversationMemory::new();
        memory.add_turn("Hello", "Hi there!");
        
        assert_eq!(memory.len(), 1);
        assert!(!memory.is_empty());
        
        let last = memory.last_turn().unwrap();
        assert_eq!(last.user_input, "Hello");
        assert_eq!(last.agent_response, "Hi there!");
    }

    #[test]
    fn test_max_capacity() {
        let mut memory = ConversationMemory::with_capacity(3);
        
        memory.add_turn("Message 1", "Response 1");
        memory.add_turn("Message 2", "Response 2");
        memory.add_turn("Message 3", "Response 3");
        memory.add_turn("Message 4", "Response 4");
        
        assert_eq!(memory.len(), 3);
        
        // Should have dropped the oldest (Message 1)
        let turns: Vec<_> = memory.all_turns().iter().collect();
        assert!(!turns.iter().any(|t| t.user_input == "Message 1"));
    }

    #[test]
    fn test_summary() {
        let mut memory = ConversationMemory::new();
        memory.add_turn("Create a game", "I'll help you create a game");
        memory.add_turn("Make it a platformer", "Sure, I'll create a platformer");
        
        let summary = memory.summary();
        assert!(summary.contains("2 turns"));
        assert!(summary.contains("Create a game"));
    }

    #[test]
    fn test_context_string() {
        let mut memory = ConversationMemory::new();
        memory.add_turn("Hello", "Hi!");
        
        let context = memory.to_context_string(5);
        assert!(context.contains("User: Hello"));
        assert!(context.contains("Assistant: Hi!"));
    }

    #[test]
    fn test_modified_files_tracking() {
        let mut memory = ConversationMemory::new();
        memory.add_turn_with_tools(
            "Fix the bug",
            "Fixed!",
            vec!["modify_code".to_string()],
            vec!["src/player.js".to_string()],
        );
        
        let files = memory.modified_files();
        assert!(files.contains(&"src/player.js"));
    }
}
