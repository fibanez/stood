//! Message types for agent communication.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::content::ContentBlock;

/// Role of a message in the conversation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// Message from the user/human
    User,
    /// Message from the AI assistant
    Assistant,
    /// System message (instructions, context)
    System,
}

/// A single message in a conversation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    /// Unique identifier for this message
    pub id: Uuid,
    /// Role of the message sender
    pub role: MessageRole,
    /// Content blocks that make up this message
    pub content: Vec<ContentBlock>,
    /// Optional metadata associated with the message
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
    /// Timestamp when the message was created
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Message {
    /// Create a new user message with text content
    pub fn user<S: Into<String>>(text: S) -> Self {
        Self::new(MessageRole::User, vec![ContentBlock::text(text)])
    }

    /// Create a new assistant message with text content
    pub fn assistant<S: Into<String>>(text: S) -> Self {
        Self::new(MessageRole::Assistant, vec![ContentBlock::text(text)])
    }

    /// Create a new system message with text content
    pub fn system<S: Into<String>>(text: S) -> Self {
        Self::new(MessageRole::System, vec![ContentBlock::text(text)])
    }

    /// Create a new message with the specified role and content
    pub fn new(role: MessageRole, content: Vec<ContentBlock>) -> Self {
        Self {
            id: Uuid::new_v4(),
            role,
            content,
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Add metadata to this message
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Get the text content of this message (if any)
    pub fn text(&self) -> Option<String> {
        self.content
            .iter()
            .filter_map(|block| {
                if let ContentBlock::Text { text } = block {
                    Some(text.clone())
                } else {
                    None
                }
            })
            .reduce(|acc, text| acc + &text)
    }

    /// Check if this message contains tool use
    pub fn has_tool_use(&self) -> bool {
        self.content
            .iter()
            .any(|block| matches!(block, ContentBlock::ToolUse { .. }))
    }

    /// Check if this message contains tool results
    pub fn has_tool_result(&self) -> bool {
        self.content
            .iter()
            .any(|block| matches!(block, ContentBlock::ToolResult { .. }))
    }

    /// Get all tool use blocks from this message
    pub fn tool_uses(&self) -> Vec<&ContentBlock> {
        self.content
            .iter()
            .filter(|block| matches!(block, ContentBlock::ToolUse { .. }))
            .collect()
    }

    /// Get all tool result blocks from this message
    pub fn tool_results(&self) -> Vec<&ContentBlock> {
        self.content
            .iter()
            .filter(|block| matches!(block, ContentBlock::ToolResult { .. }))
            .collect()
    }
}

/// A collection of messages representing a conversation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Messages {
    /// The list of messages in chronological order
    pub messages: Vec<Message>,
    /// Optional system prompt for the conversation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
}

impl Messages {
    /// Create a new empty message collection
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            system_prompt: None,
        }
    }

    /// Create a new message collection with a system prompt
    pub fn with_system_prompt(system_prompt: String) -> Self {
        Self {
            messages: Vec::new(),
            system_prompt: Some(system_prompt),
        }
    }

    /// Add a message to the collection
    pub fn push(&mut self, message: Message) {
        self.messages.push(message);
    }

    /// Get the number of messages
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    /// Check if the collection is empty
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// Get the last message (most recent)
    pub fn last(&self) -> Option<&Message> {
        self.messages.last()
    }

    /// Get the last assistant message
    pub fn last_assistant_message(&self) -> Option<&Message> {
        self.messages
            .iter()
            .rev()
            .find(|msg| msg.role == MessageRole::Assistant)
    }

    /// Get all messages from a specific role
    pub fn messages_by_role(&self, role: MessageRole) -> Vec<&Message> {
        self.messages
            .iter()
            .filter(|msg| msg.role == role)
            .collect()
    }

    /// Clear all messages
    pub fn clear(&mut self) {
        self.messages.clear();
    }

    /// Truncate to keep only the last N messages
    pub fn truncate_to_last(&mut self, count: usize) {
        if self.messages.len() > count {
            let start = self.messages.len() - count;
            self.messages.drain(0..start);
        }
    }

    /// Convenience method to add a user message with text content
    pub fn add_user_message(&mut self, text: &str) {
        self.push(Message::user(text));
    }

    /// Convenience method to add an assistant message with text content
    pub fn add_assistant_message(&mut self, text: &str) {
        self.push(Message::assistant(text));
    }

    /// Convenience method to add a system message with text content
    pub fn add_system_message(&mut self, text: &str) {
        self.push(Message::system(text));
    }
}

impl std::ops::Deref for Messages {
    type Target = Vec<Message>;

    fn deref(&self) -> &Self::Target {
        &self.messages
    }
}

impl std::ops::DerefMut for Messages {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.messages
    }
}

impl From<Vec<Message>> for Messages {
    fn from(messages: Vec<Message>) -> Self {
        Self {
            messages,
            system_prompt: None,
        }
    }
}

impl IntoIterator for Messages {
    type Item = Message;
    type IntoIter = std::vec::IntoIter<Message>;

    fn into_iter(self) -> Self::IntoIter {
        self.messages.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let msg = Message::user("Hello, world!");
        assert_eq!(msg.role, MessageRole::User);
        assert_eq!(msg.text(), Some("Hello, world!".to_string()));
        assert!(!msg.has_tool_use());
        assert!(!msg.has_tool_result());
    }

    #[test]
    fn test_messages_collection() {
        let mut messages = Messages::new();
        assert!(messages.is_empty());

        messages.push(Message::user("Hello"));
        messages.push(Message::assistant("Hi there!"));

        assert_eq!(messages.len(), 2);
        assert_eq!(
            messages.last().unwrap().text(),
            Some("Hi there!".to_string())
        );
        assert_eq!(
            messages.last_assistant_message().unwrap().text(),
            Some("Hi there!".to_string())
        );
    }

    #[test]
    fn test_message_metadata() {
        let msg = Message::user("Test").with_metadata(
            "key".to_string(),
            serde_json::Value::String("value".to_string()),
        );

        assert!(msg.metadata.contains_key("key"));
    }
}
