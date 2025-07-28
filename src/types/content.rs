//! Content block types for messages and responses.

use serde::{Deserialize, Serialize};

/// A block of content within a message
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    /// Plain text content
    Text { text: String },
    /// Tool use request from the assistant
    ToolUse {
        /// Unique identifier for this tool call
        id: String,
        /// Name of the tool to call
        name: String,
        /// Input parameters for the tool
        input: serde_json::Value,
    },
    /// Result of a tool execution
    ToolResult {
        /// ID of the tool call this result corresponds to
        tool_use_id: String,
        /// Content of the tool result
        content: ToolResultContent,
        /// Whether the tool execution was successful
        is_error: bool,
    },
    /// Claude 4 thinking content (from think tool)
    Thinking {
        /// The internal reasoning content
        content: String,
        /// Quality assessment of the reasoning
        #[serde(default)]
        quality: Option<ReasoningQuality>,
        /// Timestamp when thinking occurred
        #[serde(default = "chrono::Utc::now")]
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// AWS Bedrock reasoning content (official API format)
    ReasoningContent {
        /// The reasoning content from the model
        reasoning: ReasoningContentBlock,
    },
}

impl ContentBlock {
    /// Create a new text content block
    pub fn text<S: Into<String>>(text: S) -> Self {
        Self::Text { text: text.into() }
    }

    /// Create a new tool use content block
    pub fn tool_use<S: Into<String>>(id: S, name: S, input: serde_json::Value) -> Self {
        Self::ToolUse {
            id: id.into(),
            name: name.into(),
            input,
        }
    }

    /// Create a new successful tool result content block
    pub fn tool_result_success<S: Into<String>>(
        tool_use_id: S,
        content: ToolResultContent,
    ) -> Self {
        Self::ToolResult {
            tool_use_id: tool_use_id.into(),
            content,
            is_error: false,
        }
    }

    /// Create a new error tool result content block
    pub fn tool_result_error<S: Into<String>>(tool_use_id: S, content: ToolResultContent) -> Self {
        Self::ToolResult {
            tool_use_id: tool_use_id.into(),
            content,
            is_error: true,
        }
    }

    /// Create a new thinking content block
    pub fn thinking<S: Into<String>>(content: S) -> Self {
        Self::Thinking {
            content: content.into(),
            quality: None,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Create a new thinking content block with quality assessment
    pub fn thinking_with_quality<S: Into<String>>(content: S, quality: ReasoningQuality) -> Self {
        Self::Thinking {
            content: content.into(),
            quality: Some(quality),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Get the text content if this is a text block
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text { text } => Some(text),
            _ => None,
        }
    }

    /// Check if this is a tool use block
    pub fn is_tool_use(&self) -> bool {
        matches!(self, Self::ToolUse { .. })
    }

    /// Check if this is a tool result block
    pub fn is_tool_result(&self) -> bool {
        matches!(self, Self::ToolResult { .. })
    }

    /// Check if this is a thinking block
    pub fn is_thinking(&self) -> bool {
        matches!(self, Self::Thinking { .. })
    }

    /// Get tool use details if this is a tool use block
    pub fn as_tool_use(&self) -> Option<(&str, &str, &serde_json::Value)> {
        match self {
            Self::ToolUse { id, name, input } => Some((id, name, input)),
            _ => None,
        }
    }

    /// Get tool result details if this is a tool result block
    pub fn as_tool_result(&self) -> Option<(&str, &ToolResultContent, bool)> {
        match self {
            Self::ToolResult {
                tool_use_id,
                content,
                is_error,
            } => Some((tool_use_id, content, *is_error)),
            _ => None,
        }
    }

    /// Get thinking content if this is a thinking block
    pub fn as_thinking(&self) -> Option<(&str, Option<&ReasoningQuality>)> {
        match self {
            Self::Thinking {
                content, quality, ..
            } => Some((content, quality.as_ref())),
            _ => None,
        }
    }

    /// Check if this is a reasoning content block
    pub fn is_reasoning_content(&self) -> bool {
        matches!(self, Self::ReasoningContent { .. })
    }

    /// Get reasoning content if this is a reasoning content block
    pub fn as_reasoning_content(&self) -> Option<&ReasoningContentBlock> {
        match self {
            Self::ReasoningContent { reasoning } => Some(reasoning),
            _ => None,
        }
    }

    /// Create a new reasoning content block
    pub fn reasoning_content(text: String, signature: Option<String>) -> Self {
        Self::ReasoningContent {
            reasoning: ReasoningContentBlock::new(text, signature),
        }
    }
}

/// Content of a tool execution result
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToolResultContent {
    /// Text result
    Text { text: String },
    /// JSON structured result
    Json { data: serde_json::Value },
    /// Binary data result (base64 encoded)
    Binary { data: String, mime_type: String },
    /// Multiple content blocks
    Multiple { blocks: Vec<ToolResultContent> },
}

impl ToolResultContent {
    /// Create text tool result content
    pub fn text<S: Into<String>>(text: S) -> Self {
        Self::Text { text: text.into() }
    }

    /// Create JSON tool result content
    pub fn json(data: serde_json::Value) -> Self {
        Self::Json { data }
    }

    /// Create binary tool result content
    pub fn binary<S: Into<String>>(data: S, mime_type: S) -> Self {
        Self::Binary {
            data: data.into(),
            mime_type: mime_type.into(),
        }
    }

    /// Create multiple content blocks
    pub fn multiple(blocks: Vec<ToolResultContent>) -> Self {
        Self::Multiple { blocks }
    }

    /// Get text content if available
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text { text } => Some(text),
            _ => None,
        }
    }

    /// Get JSON data if available
    pub fn as_json(&self) -> Option<&serde_json::Value> {
        match self {
            Self::Json { data } => Some(data),
            _ => None,
        }
    }

    /// Convert to a display string
    pub fn to_display_string(&self) -> String {
        match self {
            Self::Text { text } => text.clone(),
            Self::Json { data } => {
                serde_json::to_string_pretty(data).unwrap_or_else(|_| "Invalid JSON".to_string())
            }
            Self::Binary { mime_type, .. } => format!("[Binary data: {}]", mime_type),
            Self::Multiple { blocks } => blocks
                .iter()
                .map(|block| block.to_display_string())
                .collect::<Vec<_>>()
                .join("\n"),
        }
    }
}

/// Quality assessment of reasoning content from Claude 4's think tool
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningQuality {
    /// High quality reasoning with clear logic
    High,
    /// Medium quality reasoning with some gaps
    Medium,
    /// Low quality reasoning with significant issues
    Low,
    /// Reasoning quality could not be assessed
    Unknown,
}

impl Default for ReasoningQuality {
    fn default() -> Self {
        Self::Unknown
    }
}

/// AWS Bedrock reasoning content structure (matches official API)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReasoningContentBlock {
    /// The reasoning text content
    pub reasoning_text: ReasoningText,
}

/// Reasoning text with signature for verification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReasoningText {
    /// The actual reasoning text from the model
    pub text: String,
    /// Signature for verification (optional)
    pub signature: Option<String>,
}

impl ReasoningContentBlock {
    /// Create new reasoning content
    pub fn new(text: String, signature: Option<String>) -> Self {
        Self {
            reasoning_text: ReasoningText { text, signature },
        }
    }
    
    /// Get the reasoning text
    pub fn text(&self) -> &str {
        &self.reasoning_text.text
    }
    
    /// Get the signature if present
    pub fn signature(&self) -> Option<&str> {
        self.reasoning_text.signature.as_deref()
    }
}

/// Thinking summary for conversation management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingSummary {
    /// Number of thinking blocks
    pub count: usize,
    /// Total length of thinking content
    pub total_length: usize,
    /// Average reasoning quality
    pub average_quality: Option<ReasoningQuality>,
    /// Key insights from the thinking
    pub key_insights: Vec<String>,
}

impl ThinkingSummary {
    /// Create a new thinking summary
    pub fn new() -> Self {
        Self {
            count: 0,
            total_length: 0,
            average_quality: None,
            key_insights: Vec::new(),
        }
    }

    /// Add a thinking content block to the summary
    pub fn add_thinking(&mut self, content: &str, _quality: Option<&ReasoningQuality>) {
        self.count += 1;
        self.total_length += content.len();

        // Simple key insight extraction (first sentence or first 100 chars)
        if let Some(first_sentence) = content.split('.').next() {
            if !first_sentence.is_empty() && first_sentence.len() <= 100 {
                self.key_insights.push(first_sentence.trim().to_string());
            }
        }
    }
}

impl Default for ThinkingSummary {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_block_creation() {
        let text_block = ContentBlock::text("Hello");
        assert_eq!(text_block.as_text(), Some("Hello"));
        assert!(!text_block.is_tool_use());

        let tool_block = ContentBlock::tool_use(
            "call_1",
            "calculator",
            serde_json::json!({"expression": "2+2"}),
        );
        assert!(tool_block.is_tool_use());
        assert_eq!(
            tool_block.as_tool_use(),
            Some((
                "call_1",
                "calculator",
                &serde_json::json!({"expression": "2+2"})
            ))
        );

        let thinking_block = ContentBlock::thinking("Let me think about this...");
        assert!(thinking_block.is_thinking());
        assert_eq!(
            thinking_block.as_thinking(),
            Some(("Let me think about this...", None))
        );
    }

    #[test]
    fn test_tool_result_content() {
        let text_result = ToolResultContent::text("Result: 4");
        assert_eq!(text_result.as_text(), Some("Result: 4"));
        assert_eq!(text_result.to_display_string(), "Result: 4");

        let json_result = ToolResultContent::json(serde_json::json!({"answer": 4}));
        assert_eq!(
            json_result.as_json(),
            Some(&serde_json::json!({"answer": 4}))
        );
    }

    #[test]
    fn test_thinking_summary() {
        let mut summary = ThinkingSummary::new();
        summary.add_thinking(
            "This is a test. More content here.",
            Some(&ReasoningQuality::High),
        );

        assert_eq!(summary.count, 1);
        assert!(summary.total_length > 0);
        assert_eq!(summary.key_insights.len(), 1);
        assert_eq!(summary.key_insights[0], "This is a test");
    }
}
