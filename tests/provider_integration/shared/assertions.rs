//! Verification assertions for testing LLM Client functionality
//!
//! This module provides helper functions for making assertions about
//! test results and validating behavior across different providers.

use super::*;
use std::time::Duration;

/// Assert that a verification result is successful
pub fn assert_success(result: &VerificationResult) {
    if !result.success {
        panic!(
            "Test '{}' failed for provider {:?} with model '{}': {}",
            result.test_name,
            result.provider,
            result.model_id,
            result.error.as_deref().unwrap_or("Unknown error")
        );
    }
}

/// Assert that a response contains expected content
pub fn assert_response_contains(response: &str, expected: &str, context: &str) {
    if !response.contains(expected) {
        panic!(
            "Response assertion failed in {}: Expected '{}' in response '{}'",
            context, expected, response
        );
    }
}

/// Assert that response time is within acceptable limits
pub fn assert_response_time(duration: Duration, max_seconds: f64, context: &str) {
    let actual_seconds = duration.as_secs_f64();
    if actual_seconds > max_seconds {
        panic!(
            "Response time assertion failed in {}: Expected <= {:.2}s, got {:.2}s",
            context, max_seconds, actual_seconds
        );
    }
}

/// Assert that tool usage is correct
pub fn assert_tool_usage(tools_called: &[String], expected_tool: &str, context: &str) {
    if tools_called.is_empty() {
        panic!(
            "Tool usage assertion failed in {}: No tools were used",
            context
        );
    }

    let tool_used = tools_called
        .iter()
        .any(|tool_name| tool_name == expected_tool || tool_name.contains(expected_tool));

    if !tool_used {
        panic!(
            "Tool usage assertion failed in {}: Expected '{}' tool, but used: {:?}",
            context, expected_tool, tools_called
        );
    }
}

/// Assert that conversation history is maintained correctly
pub fn assert_conversation_length(
    conversation: &crate::types::Messages,
    min_messages: usize,
    context: &str,
) {
    let count = conversation.len();
    if count < min_messages {
        panic!(
            "Conversation length assertion failed in {}: Expected >= {} messages, got {}",
            context, min_messages, count
        );
    }
}

/// Assert that a specific message type exists in conversation
pub fn assert_message_type_exists(
    conversation: &crate::types::Messages,
    role: crate::types::MessageRole,
    context: &str,
) {
    let has_role = conversation.messages.iter().any(|msg| msg.role == role);

    if !has_role {
        panic!(
            "Message type assertion failed in {}: Expected message with role '{:?}' not found",
            context, role
        );
    }
}

/// Assert that metadata contains expected key-value pairs
pub fn assert_metadata_contains(
    metadata: &std::collections::HashMap<String, serde_json::Value>,
    key: &str,
    context: &str,
) {
    if !metadata.contains_key(key) {
        panic!(
            "Metadata assertion failed in {}: Expected key '{}' not found in metadata",
            context, key
        );
    }
}

/// Assert that a numeric value is within expected range
pub fn assert_numeric_range(actual: f64, min: f64, max: f64, context: &str) {
    if actual < min || actual > max {
        panic!(
            "Numeric range assertion failed in {}: Expected {:.2} to be between {:.2} and {:.2}",
            context, actual, min, max
        );
    }
}

/// Assert that provider supports required features
pub fn assert_provider_supports_features(
    provider: ProviderType,
    features: &[ProviderFeature],
    context: &str,
) {
    for feature in features {
        if !provider_supports_feature(provider, feature) {
            panic!(
                "Provider feature assertion failed in {}: Provider {:?} does not support {:?}",
                context, provider, feature
            );
        }
    }
}

/// Helper function to check if provider supports a feature
fn provider_supports_feature(provider: ProviderType, feature: &ProviderFeature) -> bool {
    match (provider, feature) {
        // All providers support basic chat
        (_, ProviderFeature::BasicChat) => true,

        // Tool calling support
        (ProviderType::Bedrock, ProviderFeature::ToolCalling) => true,
        (ProviderType::LmStudio, ProviderFeature::ToolCalling) => true,
        (ProviderType::Anthropic, ProviderFeature::ToolCalling) => true,

        // Streaming support
        (ProviderType::Bedrock, ProviderFeature::Streaming) => true,
        (ProviderType::LmStudio, ProviderFeature::Streaming) => true,
        (ProviderType::Anthropic, ProviderFeature::Streaming) => false, // TODO

        // Advanced features
        (ProviderType::Anthropic, ProviderFeature::ThinkingMode) => false, // TODO
        (ProviderType::Bedrock, ProviderFeature::Vision) => true,
        (ProviderType::Anthropic, ProviderFeature::Vision) => false, // TODO

        // Default: assume not supported
        _ => false,
    }
}

/// Validate test configuration for consistency
pub fn validate_test_config(config: &TestConfig) -> Result<(), String> {
    // Check timeout is reasonable
    if config.timeout.as_secs() < 5 {
        return Err("Timeout too short (minimum 5 seconds)".to_string());
    }

    if config.timeout.as_secs() > 300 {
        return Err("Timeout too long (maximum 300 seconds)".to_string());
    }

    // Check retry count is reasonable
    if config.max_retries > 10 {
        return Err("Too many retries (maximum 10)".to_string());
    }

    // Check parallel tool count
    if config.max_parallel_tools == 0 {
        return Err("max_parallel_tools must be at least 1".to_string());
    }

    if config.max_parallel_tools > 10 {
        return Err("max_parallel_tools too high (maximum 10)".to_string());
    }

    Ok(())
}

/// Compare results across different providers for the same test
pub fn compare_cross_provider_results(
    results: &std::collections::HashMap<ProviderType, Vec<VerificationResult>>,
    test_name: &str,
) -> String {
    let mut comparison = String::new();
    comparison.push_str(&format!("Cross-Provider Comparison for '{}'\n", test_name));
    comparison.push_str("=".repeat(50).as_str());
    comparison.push('\n');

    let mut test_results = Vec::new();

    for (provider, provider_results) in results {
        if let Some(result) = provider_results.iter().find(|r| r.test_name == test_name) {
            test_results.push((provider, result));
        }
    }

    if test_results.is_empty() {
        comparison.push_str("No results found for this test\n");
        return comparison;
    }

    // Success rates
    comparison.push_str("\nSuccess Status:\n");
    for (provider, result) in &test_results {
        let status = if result.success {
            "✅ PASS"
        } else {
            "❌ FAIL"
        };
        comparison.push_str(&format!("  {:?}: {}\n", provider, status));
    }

    // Response times
    comparison.push_str("\nResponse Times:\n");
    let mut times: Vec<_> = test_results
        .iter()
        .map(|(provider, result)| (provider, result.duration.as_millis()))
        .collect();
    times.sort_by_key(|(_, time)| *time);

    for (provider, time) in times {
        comparison.push_str(&format!("  {:?}: {}ms\n", provider, time));
    }

    // Errors (if any)
    let failed_results: Vec<_> = test_results
        .iter()
        .filter(|(_, result)| !result.success)
        .collect();

    if !failed_results.is_empty() {
        comparison.push_str("\nFailure Details:\n");
        for (provider, result) in failed_results {
            comparison.push_str(&format!(
                "  {:?}: {}\n",
                provider,
                result.error.as_deref().unwrap_or("Unknown error")
            ));
        }
    }

    comparison
}
