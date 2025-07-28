//! Message processing utilities for conversation management.
//!
//! This module provides comprehensive message processing capabilities following the Python reference
//! implementation patterns. It handles message cleanup, validation, and normalization to ensure
//! conversation coherence and robust error handling.
//!
//! Key components:
//! - `clean_orphaned_empty_tool_uses`: Remove incomplete tool uses without corresponding results
//! - `remove_blank_content`: Handle empty text content in messages  
//! - `truncate_tool_results`: Truncate large tool results for context window management
//! - `validate_content`: Validate message structure and content types
//! - `normalize_messages`: Standardize message format across providers

use std::collections::HashSet;
use tracing::{debug, info};

use crate::{
    types::{ContentBlock, MessageRole, Messages, ToolResultContent},
    Result, StoodError,
};

/// Result of a message processing operation
#[derive(Debug, Clone)]
pub struct ProcessingResult {
    /// Whether any changes were made
    pub changes_made: bool,
    /// Number of items processed
    pub items_processed: usize,
    /// Description of the changes made
    pub description: String,
}

impl ProcessingResult {
    /// Create a new processing result
    pub fn new(changes_made: bool, items_processed: usize, description: String) -> Self {
        Self {
            changes_made,
            items_processed,
            description,
        }
    }

    /// Create a result indicating no changes
    pub fn no_changes() -> Self {
        Self {
            changes_made: false,
            items_processed: 0,
            description: "No changes made".to_string(),
        }
    }
}

/// Message processor for handling conversation cleanup and validation
pub struct MessageProcessor;

impl MessageProcessor {
    /// Clean orphaned empty tool uses from messages.
    ///
    /// Removes toolUse entries with empty input that don't have corresponding toolResult blocks.
    /// This follows the Python reference implementation of `clean_orphaned_empty_tool_uses`.
    ///
    /// Algorithm:
    /// 1. Scan assistant messages to identify empty toolUse entries
    /// 2. Scan user messages to identify existing toolResult entries  
    /// 3. Identify orphaned toolUse entries (those without matching toolResult)
    /// 4. Apply fixes in reverse order to avoid index shifting
    ///
    /// # Arguments
    /// * `messages` - The messages to clean
    ///
    /// # Returns
    /// * `ProcessingResult` - Information about the cleanup operation
    pub fn clean_orphaned_empty_tool_uses(messages: &mut Messages) -> ProcessingResult {
        debug!("Starting orphaned tool cleanup");

        let mut empty_tool_uses = Vec::new();
        let mut tool_result_ids = HashSet::new();
        let mut changes_made = false;

        // Step 1: Identify all empty tool uses in assistant messages
        for (msg_idx, message) in messages.messages.iter().enumerate() {
            if message.role != MessageRole::Assistant {
                continue;
            }

            for (content_idx, content) in message.content.iter().enumerate() {
                if let ContentBlock::ToolUse { id, name, input } = content {
                    // Check if tool use is empty (empty object or null)
                    if Self::is_empty_tool_input(input) {
                        empty_tool_uses.push((msg_idx, content_idx, id.clone(), name.clone()));
                        debug!("Found empty tool use: {} ({})", name, id);
                    }
                }
            }
        }

        // Step 2: Identify all existing tool result IDs
        for message in &messages.messages {
            if message.role != MessageRole::User {
                continue;
            }

            for content in &message.content {
                if let ContentBlock::ToolResult { tool_use_id, .. } = content {
                    tool_result_ids.insert(tool_use_id.clone());
                }
            }
        }

        // Step 3: Identify orphaned tool uses (empty tool uses without corresponding results)
        let mut orphaned_tool_uses: Vec<_> = empty_tool_uses
            .into_iter()
            .filter(|(_, _, id, _)| !tool_result_ids.contains(id))
            .collect();

        // Step 4: Apply fixes in reverse order to avoid index shifting
        orphaned_tool_uses.sort_by(|a, b| a.0.cmp(&b.0).reverse().then(a.1.cmp(&b.1).reverse()));

        let orphaned_count = orphaned_tool_uses.len();

        for (msg_idx, content_idx, tool_use_id, tool_name) in orphaned_tool_uses {
            let message = &mut messages.messages[msg_idx];

            if message.content.len() == 1 {
                // If this is the sole content, replace with context message
                let replacement_text = format!(
                    "[Attempted to use {}, but operation was canceled]",
                    tool_name
                );
                message.content[content_idx] = ContentBlock::text(replacement_text);

                info!(
                    "Replaced orphaned tool use '{}' ({}) with context message",
                    tool_name, tool_use_id
                );
            } else {
                // If multiple content items exist, remove the orphaned tool use
                message.content.remove(content_idx);

                info!(
                    "Removed orphaned tool use '{}' ({}) from message",
                    tool_name, tool_use_id
                );
            }

            changes_made = true;
        }

        let items_processed = if changes_made { orphaned_count } else { 0 };

        ProcessingResult::new(
            changes_made,
            items_processed,
            if changes_made {
                format!("Cleaned {} orphaned tool uses", items_processed)
            } else {
                "No orphaned tool uses found".to_string()
            },
        )
    }

    /// Remove or replace blank text content in messages.
    ///
    /// Follows the Python reference implementation of `remove_blank_messages_content_text`.
    /// Only processes assistant messages:
    /// - For messages with toolUse content: Removes blank text items entirely
    /// - For messages without toolUse content: Replaces blank text with "[blank text]"
    ///
    /// # Arguments
    /// * `messages` - The messages to process
    ///
    /// # Returns
    /// * `ProcessingResult` - Information about the processing operation
    pub fn remove_blank_content(messages: &mut Messages) -> ProcessingResult {
        debug!("Starting blank content removal");

        let mut changes_made = false;
        let mut items_processed = 0;

        for message in &mut messages.messages {
            // Only process assistant messages
            if message.role != MessageRole::Assistant {
                continue;
            }

            let has_tool_use = message
                .content
                .iter()
                .any(|c| matches!(c, ContentBlock::ToolUse { .. }));
            let mut indices_to_remove = Vec::new();

            for (idx, content) in message.content.iter_mut().enumerate() {
                if let ContentBlock::Text { text } = content {
                    if text.trim().is_empty() {
                        if has_tool_use {
                            // For messages with tool uses, remove blank text entirely
                            indices_to_remove.push(idx);
                            debug!("Marking blank text for removal in message with tool use");
                        } else {
                            // For messages without tool uses, replace with placeholder
                            *text = "[blank text]".to_string();
                            debug!("Replaced blank text with placeholder");
                        }
                        changes_made = true;
                        items_processed += 1;
                    }
                }
            }

            // Remove blank text items in reverse order to avoid index shifting
            for &idx in indices_to_remove.iter().rev() {
                message.content.remove(idx);
            }
        }

        ProcessingResult::new(
            changes_made,
            items_processed,
            if changes_made {
                format!("Processed {} blank content items", items_processed)
            } else {
                "No blank content found".to_string()
            },
        )
    }

    /// Find the index of the last message containing tool results.
    ///
    /// # Arguments
    /// * `messages` - The messages to search
    ///
    /// # Returns
    /// * `Option<usize>` - Index of the last message with tool results, or None
    pub fn find_last_message_with_tool_results(messages: &Messages) -> Option<usize> {
        // Iterate backwards through messages to find the last one with tool results
        for (idx, message) in messages.messages.iter().enumerate().rev() {
            for content in &message.content {
                if matches!(content, ContentBlock::ToolResult { .. }) {
                    return Some(idx);
                }
            }
        }
        None
    }

    /// Truncate tool results in a message to reduce context size.
    ///
    /// Replaces tool result content with an error message indicating truncation.
    /// This is used for context window overflow recovery.
    ///
    /// # Arguments
    /// * `messages` - The messages to modify
    /// * `message_index` - Index of the message containing tool results to truncate
    ///
    /// # Returns
    /// * `ProcessingResult` - Information about the truncation operation
    pub fn truncate_tool_results(
        messages: &mut Messages,
        message_index: usize,
    ) -> Result<ProcessingResult> {
        if message_index >= messages.messages.len() {
            return Err(StoodError::invalid_input("Message index out of bounds"));
        }

        let message = &mut messages.messages[message_index];
        let mut changes_made = false;
        let mut items_processed = 0;

        for content in &mut message.content {
            if let ContentBlock::ToolResult {
                content, is_error, ..
            } = content
            {
                // Replace with truncation message
                *content = ToolResultContent::text("The tool result was too large and has been truncated to fit the context window.");
                *is_error = true;
                changes_made = true;
                items_processed += 1;
            }
        }

        if changes_made {
            info!(
                "Truncated {} tool results in message at index {}",
                items_processed, message_index
            );
        }

        Ok(ProcessingResult::new(
            changes_made,
            items_processed,
            if changes_made {
                format!("Truncated {} tool results", items_processed)
            } else {
                "No tool results to truncate".to_string()
            },
        ))
    }

    /// Validate message content structure and types.
    ///
    /// Performs comprehensive validation of message structure following patterns
    /// from the Python reference implementation.
    ///
    /// # Arguments
    /// * `messages` - The messages to validate
    ///
    /// # Returns
    /// * `Result<ProcessingResult>` - Validation results or error
    pub fn validate_content(messages: &Messages) -> Result<ProcessingResult> {
        debug!("Starting content validation");

        let mut validation_errors = Vec::new();
        let mut items_processed = 0;

        for (msg_idx, message) in messages.messages.iter().enumerate() {
            items_processed += 1;

            // Validate message role
            if !Self::is_valid_message_role(&message.role) {
                validation_errors.push(format!(
                    "Message {} has invalid role: {:?}",
                    msg_idx, message.role
                ));
            }

            // Validate content blocks
            if message.content.is_empty() {
                validation_errors.push(format!("Message {} has no content blocks", msg_idx));
            }

            for (content_idx, content) in message.content.iter().enumerate() {
                match content {
                    ContentBlock::Text { text } => {
                        if text.is_empty() {
                            validation_errors.push(format!(
                                "Message {} content {} has empty text",
                                msg_idx, content_idx
                            ));
                        }
                    }
                    ContentBlock::ToolUse { id, name, input: _ } => {
                        if id.is_empty() {
                            validation_errors.push(format!(
                                "Message {} content {} has empty tool use ID",
                                msg_idx, content_idx
                            ));
                        }
                        if let Err(e) = Self::validate_tool_name(name) {
                            validation_errors.push(format!(
                                "Message {} content {} has invalid tool name '{}': {}",
                                msg_idx, content_idx, name, e
                            ));
                        }
                    }
                    ContentBlock::ToolResult { tool_use_id, .. } => {
                        if tool_use_id.is_empty() {
                            validation_errors.push(format!(
                                "Message {} content {} has empty tool result ID",
                                msg_idx, content_idx
                            ));
                        }
                    }
                    ContentBlock::Thinking { content, .. } => {
                        if content.is_empty() {
                            validation_errors.push(format!(
                                "Message {} content {} has empty thinking content",
                                msg_idx, content_idx
                            ));
                        }
                    }
                    ContentBlock::ReasoningContent { reasoning } => {
                        if reasoning.text().is_empty() {
                            validation_errors.push(format!(
                                "Message {} content {} has empty reasoning content",
                                msg_idx, content_idx
                            ));
                        }
                    }
                }
            }
        }

        if !validation_errors.is_empty() {
            let error_message = format!(
                "Content validation failed with {} errors:\n{}",
                validation_errors.len(),
                validation_errors.join("\n")
            );
            return Err(StoodError::invalid_input(error_message));
        }

        Ok(ProcessingResult::new(
            false, // No changes made during validation
            items_processed,
            format!("Validated {} messages successfully", items_processed),
        ))
    }

    /// Normalize message format for consistency across providers.
    ///
    /// Ensures messages follow a standardized format with consistent structure
    /// and content organization.
    ///
    /// # Arguments
    /// * `messages` - The messages to normalize
    ///
    /// # Returns
    /// * `ProcessingResult` - Information about the normalization operation
    pub fn normalize_messages(messages: &mut Messages) -> ProcessingResult {
        debug!("Starting message normalization");

        let mut changes_made = false;
        let mut items_processed = 0;

        for message in &mut messages.messages {
            items_processed += 1;

            // Ensure content blocks are in a canonical order:
            // 1. Text content first
            // 2. Tool uses second
            // 3. Tool results third
            // 4. Thinking content last
            let mut text_blocks = Vec::new();
            let mut tool_use_blocks = Vec::new();
            let mut tool_result_blocks = Vec::new();
            let mut thinking_blocks = Vec::new();

            for content in message.content.drain(..) {
                match content {
                    ContentBlock::Text { .. } => text_blocks.push(content),
                    ContentBlock::ToolUse { .. } => tool_use_blocks.push(content),
                    ContentBlock::ToolResult { .. } => tool_result_blocks.push(content),
                    ContentBlock::Thinking { .. } => thinking_blocks.push(content),
                    ContentBlock::ReasoningContent { .. } => thinking_blocks.push(content),
                }
            }

            // Check if reordering is needed
            let original_order = message.content.clone();

            // Rebuild in canonical order
            message.content.extend(text_blocks);
            message.content.extend(tool_use_blocks);
            message.content.extend(tool_result_blocks);
            message.content.extend(thinking_blocks);

            // Check if order changed
            if original_order != message.content {
                changes_made = true;
                debug!("Reordered content blocks in message");
            }
        }

        ProcessingResult::new(
            changes_made,
            items_processed,
            if changes_made {
                format!("Normalized {} messages", items_processed)
            } else {
                "All messages already normalized".to_string()
            },
        )
    }

    /// Check if tool input is considered empty.
    ///
    /// # Arguments
    /// * `input` - The tool input to check
    ///
    /// # Returns
    /// * `bool` - True if input is empty
    fn is_empty_tool_input(input: &serde_json::Value) -> bool {
        match input {
            serde_json::Value::Null => true,
            serde_json::Value::Object(obj) => obj.is_empty(),
            _ => false,
        }
    }

    /// Validate if a message role is valid.
    ///
    /// # Arguments
    /// * `role` - The message role to validate
    ///
    /// # Returns
    /// * `bool` - True if role is valid
    fn is_valid_message_role(role: &MessageRole) -> bool {
        matches!(
            role,
            MessageRole::User | MessageRole::Assistant | MessageRole::System
        )
    }

    /// Validate tool name according to rules from Python reference.
    ///
    /// Tool names must:
    /// - Match pattern ^[a-zA-Z][a-zA-Z0-9_]*$
    /// - Be ≤64 characters
    ///
    /// # Arguments
    /// * `name` - The tool name to validate
    ///
    /// # Returns
    /// * `Result<()>` - Ok if valid, Error if invalid
    fn validate_tool_name(name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(StoodError::invalid_input("Tool name cannot be empty"));
        }

        if name.len() > 64 {
            return Err(StoodError::invalid_input(
                "Tool name cannot exceed 64 characters",
            ));
        }

        // Check pattern: must start with letter, then letters/digits/underscores
        let mut chars = name.chars();

        if let Some(first_char) = chars.next() {
            if !first_char.is_ascii_alphabetic() {
                return Err(StoodError::invalid_input(
                    "Tool name must start with a letter",
                ));
            }
        }

        for ch in chars {
            if !ch.is_ascii_alphanumeric() && ch != '_' {
                return Err(StoodError::invalid_input(
                    "Tool name can only contain letters, digits, and underscores",
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ContentBlock, Message, MessageRole, Messages};
    use serde_json::json;

    #[test]
    fn test_clean_orphaned_empty_tool_uses_no_orphans() {
        let mut messages = Messages::new();

        // Add a normal tool use with result
        messages.push(Message::new(
            MessageRole::Assistant,
            vec![ContentBlock::ToolUse {
                id: "tool_1".to_string(),
                name: "calculator".to_string(),
                input: json!({"expression": "2+2"}),
            }],
        ));

        messages.push(Message::new(
            MessageRole::User,
            vec![ContentBlock::ToolResult {
                tool_use_id: "tool_1".to_string(),
                content: crate::types::ToolResultContent::text("4"),
                is_error: false,
            }],
        ));

        let result = MessageProcessor::clean_orphaned_empty_tool_uses(&mut messages);

        assert!(!result.changes_made);
        assert_eq!(result.items_processed, 0);
        assert_eq!(messages.messages.len(), 2);
    }

    #[test]
    fn test_clean_orphaned_empty_tool_uses_with_orphans() {
        let mut messages = Messages::new();

        // Add an orphaned empty tool use (sole content)
        messages.push(Message::new(
            MessageRole::Assistant,
            vec![ContentBlock::ToolUse {
                id: "orphan_1".to_string(),
                name: "calculator".to_string(),
                input: json!({}), // Empty input
            }],
        ));

        // Add an orphaned empty tool use with other content
        messages.push(Message::new(
            MessageRole::Assistant,
            vec![
                ContentBlock::text("Some text"),
                ContentBlock::ToolUse {
                    id: "orphan_2".to_string(),
                    name: "file_reader".to_string(),
                    input: serde_json::Value::Null, // Null input
                },
            ],
        ));

        let result = MessageProcessor::clean_orphaned_empty_tool_uses(&mut messages);

        assert!(result.changes_made);
        assert_eq!(result.items_processed, 2);

        // First message should have context message replacement
        if let ContentBlock::Text { text } = &messages.messages[0].content[0] {
            assert!(text.contains("Attempted to use calculator"));
        } else {
            panic!("Expected text content");
        }

        // Second message should have the orphaned tool use removed
        assert_eq!(messages.messages[1].content.len(), 1);
        assert!(matches!(
            messages.messages[1].content[0],
            ContentBlock::Text { .. }
        ));
    }

    #[test]
    fn test_remove_blank_content_assistant_with_tools() {
        let mut messages = Messages::new();

        messages.push(Message::new(
            MessageRole::Assistant,
            vec![
                ContentBlock::text("   "), // Blank text
                ContentBlock::ToolUse {
                    id: "tool_1".to_string(),
                    name: "calculator".to_string(),
                    input: json!({"expression": "2+2"}),
                },
                ContentBlock::text("Valid text"),
            ],
        ));

        let result = MessageProcessor::remove_blank_content(&mut messages);

        assert!(result.changes_made);
        assert_eq!(result.items_processed, 1);

        // Should have removed the blank text, leaving tool use and valid text
        assert_eq!(messages.messages[0].content.len(), 2);
        assert!(matches!(
            messages.messages[0].content[0],
            ContentBlock::ToolUse { .. }
        ));
        assert!(matches!(
            messages.messages[0].content[1],
            ContentBlock::Text { .. }
        ));
    }

    #[test]
    fn test_remove_blank_content_assistant_without_tools() {
        let mut messages = Messages::new();

        messages.push(Message::new(
            MessageRole::Assistant,
            vec![ContentBlock::text("   ")], // Blank text, no tools
        ));

        let result = MessageProcessor::remove_blank_content(&mut messages);

        assert!(result.changes_made);
        assert_eq!(result.items_processed, 1);

        // Should have replaced blank text with placeholder
        if let ContentBlock::Text { text } = &messages.messages[0].content[0] {
            assert_eq!(text, "[blank text]");
        } else {
            panic!("Expected text content");
        }
    }

    #[test]
    fn test_find_last_message_with_tool_results() {
        let mut messages = Messages::new();

        messages.push(Message::new(
            MessageRole::User,
            vec![ContentBlock::text("Hello")],
        ));

        messages.push(Message::new(
            MessageRole::User,
            vec![ContentBlock::ToolResult {
                tool_use_id: "tool_1".to_string(),
                content: crate::types::ToolResultContent::text("result 1"),
                is_error: false,
            }],
        ));

        messages.push(Message::new(
            MessageRole::User,
            vec![ContentBlock::ToolResult {
                tool_use_id: "tool_2".to_string(),
                content: crate::types::ToolResultContent::text("result 2"),
                is_error: false,
            }],
        ));

        let result = MessageProcessor::find_last_message_with_tool_results(&messages);
        assert_eq!(result, Some(2)); // Last message with tool results
    }

    #[test]
    fn test_truncate_tool_results() {
        let mut messages = Messages::new();

        messages.push(Message::new(
            MessageRole::User,
            vec![
                ContentBlock::text("Some text"),
                ContentBlock::ToolResult {
                    tool_use_id: "tool_1".to_string(),
                    content: crate::types::ToolResultContent::text("Large result content"),
                    is_error: false,
                },
            ],
        ));

        let result = MessageProcessor::truncate_tool_results(&mut messages, 0).unwrap();

        assert!(result.changes_made);
        assert_eq!(result.items_processed, 1);

        // Check that tool result was truncated
        if let ContentBlock::ToolResult {
            content, is_error, ..
        } = &messages.messages[0].content[1]
        {
            if let crate::types::ToolResultContent::Text { text } = content {
                assert!(text.contains("truncated"));
            }
            assert!(*is_error);
        } else {
            panic!("Expected tool result content");
        }
    }

    #[test]
    fn test_validate_content_success() {
        let mut messages = Messages::new();

        messages.push(Message::new(
            MessageRole::User,
            vec![ContentBlock::text("Hello")],
        ));

        messages.push(Message::new(
            MessageRole::Assistant,
            vec![ContentBlock::ToolUse {
                id: "tool_1".to_string(),
                name: "valid_tool_name".to_string(),
                input: json!({"param": "value"}),
            }],
        ));

        let result = MessageProcessor::validate_content(&messages).unwrap();

        assert!(!result.changes_made);
        assert_eq!(result.items_processed, 2);
    }

    #[test]
    fn test_validate_content_invalid_tool_name() {
        let mut messages = Messages::new();

        messages.push(Message::new(
            MessageRole::Assistant,
            vec![ContentBlock::ToolUse {
                id: "tool_1".to_string(),
                name: "123invalid".to_string(), // Invalid: starts with number
                input: json!({"param": "value"}),
            }],
        ));

        let result = MessageProcessor::validate_content(&messages);
        assert!(result.is_err());
    }

    #[test]
    fn test_normalize_messages() {
        let mut messages = Messages::new();

        // Create a message with content in non-canonical order
        messages.push(Message::new(
            MessageRole::Assistant,
            vec![
                ContentBlock::ToolResult {
                    tool_use_id: "tool_1".to_string(),
                    content: crate::types::ToolResultContent::text("result"),
                    is_error: false,
                },
                ContentBlock::text("Hello"),
                ContentBlock::ToolUse {
                    id: "tool_1".to_string(),
                    name: "calculator".to_string(),
                    input: json!({"expr": "2+2"}),
                },
            ],
        ));

        let result = MessageProcessor::normalize_messages(&mut messages);

        assert!(result.changes_made);
        assert_eq!(result.items_processed, 1);

        // Check that content is now in canonical order: text, tool_use, tool_result
        assert!(matches!(
            messages.messages[0].content[0],
            ContentBlock::Text { .. }
        ));
        assert!(matches!(
            messages.messages[0].content[1],
            ContentBlock::ToolUse { .. }
        ));
        assert!(matches!(
            messages.messages[0].content[2],
            ContentBlock::ToolResult { .. }
        ));
    }

    #[test]
    fn test_validate_tool_name() {
        // Valid names
        assert!(MessageProcessor::validate_tool_name("calculator").is_ok());
        assert!(MessageProcessor::validate_tool_name("file_reader").is_ok());
        assert!(MessageProcessor::validate_tool_name("HTTP_Client").is_ok());
        assert!(MessageProcessor::validate_tool_name("a").is_ok());

        // Invalid names
        assert!(MessageProcessor::validate_tool_name("").is_err()); // Empty
        assert!(MessageProcessor::validate_tool_name("123invalid").is_err()); // Starts with number
        assert!(MessageProcessor::validate_tool_name("invalid-name").is_err()); // Contains hyphen
        assert!(MessageProcessor::validate_tool_name("invalid.name").is_err()); // Contains dot
        assert!(MessageProcessor::validate_tool_name(&"a".repeat(65)).is_err());
        // Too long
    }
}
