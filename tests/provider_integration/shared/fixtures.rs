//! Test fixtures and data for verification tests
//!
//! This module provides reusable test data, prompts, and expected responses
//! for consistent testing across all providers.

use crate::types::{Message, Messages};
use serde_json::Value;
use std::collections::HashMap;

/// Standard test prompts for different categories
pub struct TestPrompts;

impl TestPrompts {
    /// Simple chat prompt for basic functionality testing
    pub fn simple_chat() -> &'static str {
        "What is 2+2? Answer with just the number."
    }

    /// Multi-turn conversation starter
    pub fn conversation_starter() -> &'static str {
        "My favorite number is 42. Please remember this for our conversation."
    }

    /// Follow-up question for conversation context testing
    pub fn conversation_followup() -> &'static str {
        "What number did I just mention as my favorite?"
    }

    /// Tool usage prompt for calculator
    pub fn calculator_request() -> &'static str {
        "What is 23 * 47? Use the calculator tool to compute this."
    }

    /// Multiple tool usage prompt
    pub fn multiple_tools_request() -> &'static str {
        "Please tell me what time it is, then calculate 12 * 34. Use the appropriate tools."
    }

    /// Complex agentic workflow prompt
    pub fn agentic_workflow() -> &'static str {
        "I need you to help me plan a birthday party. First, check what time it is. Then calculate how many pizza slices we need for 8 people (assuming 3 slices per person). Finally, tell me your recommendations."
    }

    /// Streaming response test prompt
    pub fn streaming_request() -> &'static str {
        "Please write a short story about a robot learning to paint. Make it exactly 3 paragraphs long."
    }

    /// Error handling test prompts
    pub fn invalid_tool_request() -> &'static str {
        "Please use the nonexistent_tool to do something impossible."
    }

    /// Performance test prompt (longer response)
    pub fn performance_test() -> &'static str {
        "Explain the concept of machine learning in detail, covering supervised learning, unsupervised learning, and reinforcement learning. Provide examples for each."
    }
}

/// Expected response patterns for validation
pub struct ExpectedResponses;

impl ExpectedResponses {
    /// Expected content for simple math
    pub fn simple_math_result() -> &'static str {
        "4"
    }

    /// Expected content for calculator tool usage
    pub fn calculator_result() -> &'static str {
        "1081" // 23 * 47
    }

    /// Expected content for multiple calculation
    pub fn multiple_calculation_result() -> &'static str {
        "408" // 12 * 34
    }

    /// Expected content patterns for conversation context
    pub fn conversation_context_patterns() -> Vec<&'static str> {
        vec!["42", "forty-two", "forty two"]
    }

    /// Expected tool names for different test scenarios
    pub fn expected_tools() -> HashMap<&'static str, Vec<&'static str>> {
        let mut tools = HashMap::new();
        tools.insert("calculator", vec!["calculator", "calc", "math"]);
        tools.insert("time", vec!["time", "current_time", "clock"]);
        tools.insert("multiple", vec!["calculator", "time"]);
        tools
    }
}

/// Test data for tool definitions and schemas
pub struct TestToolData;

impl TestToolData {
    /// Sample calculator tool input
    pub fn calculator_input() -> Value {
        serde_json::json!({
            "expression": "23 * 47"
        })
    }

    /// Sample file operation input
    pub fn file_operation_input() -> Value {
        serde_json::json!({
            "operation": "read",
            "path": "/tmp/test.txt"
        })
    }

    /// Sample HTTP request input
    pub fn http_request_input() -> Value {
        serde_json::json!({
            "url": "https://httpbin.org/json",
            "method": "GET"
        })
    }
}

/// Conversation fixtures for testing
pub struct ConversationFixtures;

impl ConversationFixtures {
    /// Create a basic conversation for testing
    pub fn basic_conversation() -> Messages {
        let mut messages = Messages::new();
        messages.add_system_message("You are a helpful assistant.");
        messages.add_user_message("Hello, how are you?");
        messages.add_assistant_message("I'm doing well, thank you! How can I help you today?");
        messages
    }

    /// Create a conversation with tool usage
    pub fn tool_conversation() -> Messages {
        let mut messages = Messages::new();
        messages.add_system_message("You are a helpful assistant with access to tools.");
        messages.add_user_message("What is 15 * 23?");
        // Would normally include assistant's tool use and result here
        messages
    }

    /// Create a long conversation for context testing
    pub fn long_conversation() -> Messages {
        let mut messages = Messages::new();
        messages.add_system_message("You are a helpful assistant.");

        // Add multiple exchanges
        for i in 1..=5 {
            messages.add_user_message(&format!("This is message number {}", i));
            messages.add_assistant_message(&format!("I received your message number {}", i));
        }

        messages
    }
}

/// Performance test data and expectations
pub struct PerformanceFixtures;

impl PerformanceFixtures {
    /// Expected response time ranges for different operations (in seconds)
    pub fn response_time_expectations() -> HashMap<&'static str, (f64, f64)> {
        let mut expectations = HashMap::new();
        expectations.insert("simple_chat", (0.5, 10.0)); // 0.5s to 10s
        expectations.insert("tool_usage", (1.0, 15.0)); // 1s to 15s
        expectations.insert("multiple_tools", (2.0, 30.0)); // 2s to 30s
        expectations.insert("streaming", (1.0, 20.0)); // 1s to 20s
        expectations.insert("agentic_workflow", (5.0, 60.0)); // 5s to 60s
        expectations
    }

    /// Expected token usage ranges (approximate)
    pub fn token_usage_expectations() -> HashMap<&'static str, (usize, usize)> {
        let mut expectations = HashMap::new();
        expectations.insert("simple_chat", (10, 100)); // 10 to 100 tokens
        expectations.insert("tool_usage", (50, 300)); // 50 to 300 tokens
        expectations.insert("performance_test", (200, 1000)); // 200 to 1000 tokens
        expectations
    }
}

/// Error test scenarios
pub struct ErrorScenarios;

impl ErrorScenarios {
    /// Network-related error scenarios
    pub fn network_errors() -> Vec<(&'static str, &'static str)> {
        vec![
            ("connection_refused", "Simulate connection refused"),
            ("timeout", "Simulate request timeout"),
            ("dns_failure", "Simulate DNS resolution failure"),
        ]
    }

    /// Model-related error scenarios
    pub fn model_errors() -> Vec<(&'static str, &'static str)> {
        vec![
            ("invalid_model", "Use non-existent model ID"),
            ("context_overflow", "Send message exceeding context window"),
            ("rate_limit", "Exceed rate limiting"),
        ]
    }

    /// Tool-related error scenarios
    pub fn tool_errors() -> Vec<(&'static str, &'static str)> {
        vec![
            ("invalid_tool", "Call non-existent tool"),
            ("invalid_params", "Call tool with invalid parameters"),
            ("tool_failure", "Tool execution fails"),
        ]
    }
}

/// Test configuration templates
pub struct ConfigTemplates;

impl ConfigTemplates {
    /// Fast test configuration for quick feedback
    pub fn fast_config() -> HashMap<&'static str, Value> {
        let mut config = HashMap::new();
        config.insert("timeout_seconds", Value::Number(10.into()));
        config.insert("max_retries", Value::Number(1.into()));
        config.insert("max_parallel_tools", Value::Number(2.into()));
        config
    }

    /// Thorough test configuration for comprehensive testing
    pub fn thorough_config() -> HashMap<&'static str, Value> {
        let mut config = HashMap::new();
        config.insert("timeout_seconds", Value::Number(60.into()));
        config.insert("max_retries", Value::Number(3.into()));
        config.insert("max_parallel_tools", Value::Number(4.into()));
        config
    }

    /// Performance test configuration
    pub fn performance_config() -> HashMap<&'static str, Value> {
        let mut config = HashMap::new();
        config.insert("timeout_seconds", Value::Number(120.into()));
        config.insert("max_retries", Value::Number(1.into()));
        config.insert("max_parallel_tools", Value::Number(8.into()));
        config.insert("enable_telemetry", Value::Bool(true));
        config
    }
}

/// Helper functions for creating test data
pub struct TestDataHelpers;

impl TestDataHelpers {
    /// Create a test message with specific role and content
    pub fn create_message(role: &str, content: &str) -> Message {
        match role {
            "user" => Message::user(content),
            "assistant" => Message::assistant(content),
            "system" => Message::system(content),
            _ => Message::user(content), // Default to user
        }
    }

    /// Create a conversation from a list of (role, content) pairs
    pub fn create_conversation(exchanges: Vec<(&str, &str)>) -> Messages {
        let mut messages = Messages::new();

        for (role, content) in exchanges {
            match role {
                "user" => messages.add_user_message(content),
                "assistant" => messages.add_assistant_message(content),
                "system" => messages.add_system_message(content),
                _ => messages.add_user_message(content),
            }
        }

        messages
    }

    /// Generate random test data for stress testing
    pub fn generate_large_prompt(size_chars: usize) -> String {
        let base_text =
            "This is a test prompt that will be repeated many times to create a large input. ";
        let repetitions = (size_chars / base_text.len()) + 1;
        base_text.repeat(repetitions)[..size_chars].to_string()
    }

    /// Create metadata for test tracking
    pub fn create_test_metadata(
        test_name: &str,
        provider: &str,
        model: &str,
    ) -> HashMap<String, Value> {
        let mut metadata = HashMap::new();
        metadata.insert(
            "test_name".to_string(),
            Value::String(test_name.to_string()),
        );
        metadata.insert("provider".to_string(), Value::String(provider.to_string()));
        metadata.insert("model".to_string(), Value::String(model.to_string()));
        metadata.insert(
            "timestamp".to_string(),
            Value::String(chrono::Utc::now().to_rfc3339()),
        );
        metadata
    }
}
