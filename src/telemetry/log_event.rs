//! OTEL Log Event structures for CloudWatch GenAI Evaluations
//!
//! This module provides log event data structures that conform to the OpenTelemetry
//! log data model and are compatible with AWS Bedrock AgentCore Evaluations.
//!
//! The evaluate API requires log events with `body.input.messages` and `body.output.messages`
//! to run evaluations like Correctness, Conciseness, Helpfulness, etc.
//!
//! ## Format
//!
//! We use the LangChain telemetry format (`opentelemetry.instrumentation.langchain`)
//! which is reliably parsed by the AgentCore Evaluations API. The body uses nested
//! JSON with `kwargs.content` structure per LangChain conventions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// LangChain message format helpers
///
/// The AgentCore Evaluations API expects messages in LangChain format with
/// nested JSON containing `kwargs.content` structure.
mod langchain_format {
    use serde_json::json;

    /// Format a user message in LangChain format
    pub fn format_user_message(content: &str) -> String {
        let message = json!({
            "inputs": {
                "messages": [{
                    "lc": 1,
                    "type": "constructor",
                    "id": ["langchain", "schema", "messages", "HumanMessage"],
                    "kwargs": {
                        "content": content,
                        "type": "human"
                    }
                }],
                "next_step": "reason"
            },
            "tags": []
        });
        serde_json::to_string(&message).unwrap_or_else(|_| content.to_string())
    }

    /// Format an assistant message in LangChain format
    pub fn format_assistant_message(content: &str) -> String {
        let message = json!({
            "outputs": {
                "messages": [{
                    "lc": 1,
                    "type": "constructor",
                    "id": ["langchain", "schema", "messages", "AIMessage"],
                    "kwargs": {
                        "content": content,
                        "type": "ai"
                    }
                }],
                "next_step": "end"
            },
            "kwargs": {}
        });
        serde_json::to_string(&message).unwrap_or_else(|_| content.to_string())
    }

    /// Format a tool message in LangChain format
    ///
    /// Uses the proper LangChain ToolMessage schema so evaluators can recognize
    /// tool outputs in the conversation history.
    pub fn format_tool_message(tool_name: &str, content: &str) -> String {
        let message = json!({
            "outputs": {
                "messages": [{
                    "lc": 1,
                    "type": "constructor",
                    "id": ["langchain", "schema", "messages", "ToolMessage"],
                    "kwargs": {
                        "content": content,
                        "name": tool_name,
                        "type": "tool"
                    }
                }],
                "next_step": "continue"
            },
            "kwargs": {}
        });
        serde_json::to_string(&message).unwrap_or_else(|_| content.to_string())
    }

    /// Format a system message in LangChain format
    pub fn format_system_message(content: &str) -> String {
        let message = json!({
            "inputs": {
                "messages": [{
                    "lc": 1,
                    "type": "constructor",
                    "id": ["langchain", "schema", "messages", "SystemMessage"],
                    "kwargs": {
                        "content": content,
                        "type": "system"
                    }
                }],
                "next_step": "setup"
            },
            "tags": []
        });
        serde_json::to_string(&message).unwrap_or_else(|_| content.to_string())
    }
}

/// OTEL Log Event for AgentCore Evaluations
///
/// This structure represents a log record that captures the input/output
/// of an agent interaction. It must be linked to a span via traceId and spanId.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEvent {
    /// Trace ID linking this event to its parent trace (32 hex chars)
    pub trace_id: String,

    /// Span ID linking this event to a specific span (16 hex chars)
    pub span_id: String,

    /// Instrumentation scope (opentelemetry.instrumentation.langchain for AgentCore Evaluations)
    pub scope: LogScope,

    /// Event timestamp in nanoseconds since Unix epoch
    pub time_unix_nano: u64,

    /// Observed timestamp in nanoseconds since Unix epoch
    pub observed_time_unix_nano: u64,

    /// Severity level (9 = INFO)
    pub severity_number: u8,

    /// Severity text (empty string for INFO in Strands format)
    #[serde(default)]
    pub severity_text: String,

    /// Event body containing input/output messages
    pub body: LogEventBody,

    /// Event attributes including event.name and session.id
    pub attributes: HashMap<String, String>,

    /// Flags (1 for sampled traces)
    #[serde(default = "default_flags")]
    pub flags: u8,

    /// Resource attributes (optional, included for completeness)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource: Option<LogResource>,
}

fn default_flags() -> u8 {
    1
}

/// Instrumentation scope for log events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogScope {
    /// Scope name - "opentelemetry.instrumentation.langchain" for AgentCore Evaluations
    pub name: String,

    /// Scope version
    #[serde(default)]
    pub version: String,
}

/// The scope name used for LangChain format (required for AgentCore Evaluations)
pub const LANGCHAIN_SCOPE: &str = "opentelemetry.instrumentation.langchain";

impl Default for LogScope {
    fn default() -> Self {
        Self {
            name: LANGCHAIN_SCOPE.to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

/// Log event body containing input and output messages
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LogEventBody {
    /// Input messages (user prompt, system prompt)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<MessageList>,

    /// Output messages (assistant response)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<MessageList>,
}

/// List of messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageList {
    /// Messages in this list
    pub messages: Vec<Message>,
}

/// A single message in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Role: "system", "user", or "assistant"
    pub role: String,

    /// Message content
    pub content: String,
}

/// Resource attributes for log events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogResource {
    /// Resource attributes
    pub attributes: HashMap<String, String>,
}

/// Event names for LangChain format
///
/// For LangChain format, the event.name should match the scope name.
/// This is what the AgentCore Evaluations API expects.
pub mod event_names {
    use super::LANGCHAIN_SCOPE;

    /// Event name for LangChain format (matches scope name)
    pub const LANGCHAIN: &str = LANGCHAIN_SCOPE;

    // Legacy GenAI semantic convention names (kept for reference)
    // These are NOT used by LangChain format but may be useful for other integrations
    #[allow(dead_code)]
    pub const CONTENT_COMPLETION: &str = "gen_ai.content.completion";
    #[allow(dead_code)]
    pub const CHOICE: &str = "gen_ai.choice";
    #[allow(dead_code)]
    pub const TOOL_MESSAGE: &str = "gen_ai.tool.message";
}

impl LogEvent {
    /// Create a new log event linked to a span
    ///
    /// Uses the LangChain scope name as the event.name (required by AgentCore Evaluations).
    pub fn new(trace_id: impl Into<String>, span_id: impl Into<String>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        let mut attributes = HashMap::new();
        // Use LangChain scope name as event.name (required for AgentCore Evaluations)
        attributes.insert("event.name".to_string(), LANGCHAIN_SCOPE.to_string());

        Self {
            trace_id: trace_id.into(),
            span_id: span_id.into(),
            scope: LogScope::default(),
            time_unix_nano: now,
            observed_time_unix_nano: now,
            severity_number: 9, // INFO
            severity_text: String::new(),
            body: LogEventBody::default(),
            attributes,
            flags: 1,
            resource: None,
        }
    }

    /// Set the event name attribute
    pub fn with_event_name(mut self, event_name: &str) -> Self {
        self.attributes
            .insert("event.name".to_string(), event_name.to_string());
        self
    }

    /// Set input messages (system prompt + user prompt)
    pub fn with_input_messages(mut self, messages: Vec<Message>) -> Self {
        self.body.input = Some(MessageList { messages });
        self
    }

    /// Set output messages (assistant response)
    pub fn with_output_messages(mut self, messages: Vec<Message>) -> Self {
        self.body.output = Some(MessageList { messages });
        self
    }

    /// Set session ID attribute
    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.attributes
            .insert("session.id".to_string(), session_id.into());
        self
    }

    /// Set resource attributes
    pub fn with_resource(mut self, service_name: &str, service_version: &str) -> Self {
        let mut attrs = HashMap::new();
        attrs.insert("aws.service.type".to_string(), "gen_ai_agent".to_string());
        attrs.insert("service.name".to_string(), service_name.to_string());
        attrs.insert("service.version".to_string(), service_version.to_string());
        self.resource = Some(LogResource { attributes: attrs });
        self
    }

    /// Add a custom attribute
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }

    /// Create a log event for an agent invocation
    ///
    /// This captures the full conversation context for evaluation using LangChain format.
    /// The body uses nested JSON with `kwargs.content` structure.
    pub fn for_agent_invocation(
        trace_id: impl Into<String>,
        span_id: impl Into<String>,
        session_id: impl Into<String>,
        system_prompt: Option<&str>,
        user_prompt: &str,
        assistant_response: &str,
    ) -> Self {
        // Build input message in LangChain format
        // If there's a system prompt, include it in the user message context
        let user_content = if let Some(system) = system_prompt {
            format!("{}\n\n{}", system, user_prompt)
        } else {
            user_prompt.to_string()
        };

        let input_messages = vec![Message {
            role: "user".to_string(),
            content: langchain_format::format_user_message(&user_content),
        }];

        let output_messages = vec![Message {
            role: "assistant".to_string(),
            content: langchain_format::format_assistant_message(assistant_response),
        }];

        Self::new(trace_id, span_id)
            .with_input_messages(input_messages)
            .with_output_messages(output_messages)
            .with_session_id(session_id)
    }

    /// Create a log event for an agent invocation with tool results
    ///
    /// This captures the full conversation including tool executions for evaluation.
    /// The Faithfulness evaluator requires tool outputs in the conversation history
    /// to verify the assistant's response is grounded in the tool results.
    ///
    /// Tool results are included in the user message content as context, maintaining
    /// the single user message structure that the evaluator expects.
    ///
    /// # Arguments
    /// * `tool_results` - Vec of (tool_name, tool_input, tool_output) tuples
    pub fn for_agent_invocation_with_tools(
        trace_id: impl Into<String>,
        span_id: impl Into<String>,
        session_id: impl Into<String>,
        system_prompt: Option<&str>,
        user_prompt: &str,
        tool_results: &[(String, String, String)],
        assistant_response: &str,
    ) -> Self {
        // Build the user message content with tool results appended as context
        // This maintains the single user message structure expected by evaluators
        let mut user_content = if let Some(system) = system_prompt {
            format!("{}\n\n{}", system, user_prompt)
        } else {
            user_prompt.to_string()
        };

        // Append tool results as context for Faithfulness evaluation
        if !tool_results.is_empty() {
            user_content.push_str("\n\n--- Tool Execution Results ---\n");
            for (tool_name, tool_input, tool_output) in tool_results {
                user_content.push_str(&format!(
                    "\nTool: {}\nInput: {}\nOutput: {}\n",
                    tool_name, tool_input, tool_output
                ));
            }
        }

        let input_messages = vec![Message {
            role: "user".to_string(),
            content: langchain_format::format_user_message(&user_content),
        }];

        // Output: final assistant response
        let output_messages = vec![Message {
            role: "assistant".to_string(),
            content: langchain_format::format_assistant_message(assistant_response),
        }];

        Self::new(trace_id, span_id)
            .with_input_messages(input_messages)
            .with_output_messages(output_messages)
            .with_session_id(session_id)
    }

    /// Create a log event for a tool execution
    ///
    /// This captures the tool input and output for AgentCore Evaluations using LangChain format.
    /// Required for Builtin.ToolSelectionAccuracy and other tool-related evaluators.
    pub fn for_tool_execution(
        trace_id: impl Into<String>,
        span_id: impl Into<String>,
        session_id: impl Into<String>,
        tool_name: &str,
        tool_input: &str,
        tool_output: &str,
    ) -> Self {
        // Format tool input as a user message with tool call info
        let input_content = format!("Tool call: {} with input: {}", tool_name, tool_input);
        let input_messages = vec![Message {
            role: "user".to_string(),
            content: langchain_format::format_user_message(&input_content),
        }];

        // Format tool output in LangChain format
        let output_messages = vec![Message {
            role: "assistant".to_string(),
            content: langchain_format::format_tool_message(tool_name, tool_output),
        }];

        Self::new(trace_id, span_id)
            .with_input_messages(input_messages)
            .with_output_messages(output_messages)
            .with_session_id(session_id)
            .with_attribute("gen_ai.tool.name", tool_name)
    }

    /// Create a log event for a chat/model completion
    ///
    /// This captures the model invocation input and output for AgentCore Evaluations.
    /// Uses LangChain format with nested JSON structure.
    pub fn for_chat_completion(
        trace_id: impl Into<String>,
        span_id: impl Into<String>,
        session_id: impl Into<String>,
        model: &str,
        user_input: &str,
        assistant_output: &str,
    ) -> Self {
        let input_messages = vec![Message {
            role: "user".to_string(),
            content: langchain_format::format_user_message(user_input),
        }];

        let output_messages = vec![Message {
            role: "assistant".to_string(),
            content: langchain_format::format_assistant_message(assistant_output),
        }];

        Self::new(trace_id, span_id)
            .with_input_messages(input_messages)
            .with_output_messages(output_messages)
            .with_session_id(session_id)
            .with_attribute("gen_ai.request.model", model)
    }
}

impl Message {
    /// Create a new message
    pub fn new(role: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            content: content.into(),
        }
    }

    /// Create a system message
    pub fn system(content: impl Into<String>) -> Self {
        Self::new("system", content)
    }

    /// Create a user message
    pub fn user(content: impl Into<String>) -> Self {
        Self::new("user", content)
    }

    /// Create an assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        Self::new("assistant", content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_event_serialization() {
        let event = LogEvent::for_agent_invocation(
            "abc123def456",
            "span123",
            "session-001",
            Some("You are a helpful assistant."),
            "What is 2+2?",
            "2+2 equals 4.",
        );

        let json = serde_json::to_string_pretty(&event).unwrap();
        println!("{}", json);

        // Verify key fields (pretty-printed JSON has spaces after colons)
        assert!(json.contains("\"traceId\": \"abc123def456\""));
        assert!(json.contains("\"spanId\": \"span123\""));
        assert!(json.contains("\"name\": \"opentelemetry.instrumentation.langchain\""));
        // With LangChain format, input is a single user message with content as nested JSON
        assert!(json.contains("\"role\": \"user\""));
        assert!(json.contains("\"role\": \"assistant\""));
        assert!(json.contains("\"session.id\": \"session-001\""));
    }

    #[test]
    fn test_log_event_without_system_prompt() {
        let event = LogEvent::for_agent_invocation(
            "trace1",
            "span1",
            "session1",
            None,
            "Hello!",
            "Hi there!",
        );

        let body = &event.body;
        let input = body.input.as_ref().unwrap();
        assert_eq!(input.messages.len(), 1); // Only user message
        assert_eq!(input.messages[0].role, "user");
    }

    #[test]
    fn test_message_helpers() {
        let system = Message::system("You are helpful.");
        let user = Message::user("Hello");
        let assistant = Message::assistant("Hi!");

        assert_eq!(system.role, "system");
        assert_eq!(user.role, "user");
        assert_eq!(assistant.role, "assistant");
    }

    #[test]
    fn test_default_scope() {
        let scope = LogScope::default();
        assert_eq!(scope.name, LANGCHAIN_SCOPE);
    }
}
