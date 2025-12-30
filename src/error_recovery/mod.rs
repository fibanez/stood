//! Error recovery system for robust agent operation.
//!
//! This module provides comprehensive error recovery capabilities following the Python reference
//! implementation patterns. It handles common failure scenarios like throttling, context window
//! overflow, and network errors with intelligent recovery strategies.
//!
//! Key components:
//! - `RetryConfig`: Configuration for retry behavior with exponential backoff
//! - `ErrorClassifier`: Classify errors as retryable vs non-retryable
//! - `BackoffStrategy`: Different backoff strategies (exponential, fixed, linear)
//! - `ContextRecovery`: Handle context window overflow by truncating messages
//! - `CircuitBreaker`: Prevent cascading failures with circuit breaker pattern

use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

use crate::{types::Messages, Result, StoodError};

/// Configuration for retry behavior and error recovery
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay before first retry
    pub initial_delay: Duration,
    /// Maximum delay between retries (cap for exponential growth)
    pub max_delay: Duration,
    /// Backoff strategy to use
    pub backoff_strategy: BackoffStrategy,
    /// Whether to enable jitter to avoid thundering herd
    pub enable_jitter: bool,
    /// Maximum total time to spend retrying
    pub max_total_duration: Option<Duration>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 6,                       // Python: MAX_ATTEMPTS = 6
            initial_delay: Duration::from_secs(4), // Python: INITIAL_DELAY = 4
            max_delay: Duration::from_secs(240),   // Python: MAX_DELAY = 240 (4 minutes)
            backoff_strategy: BackoffStrategy::Exponential { multiplier: 2.0 },
            enable_jitter: true,
            max_total_duration: Some(Duration::from_secs(600)), // 10 minutes total
        }
    }
}

/// Different backoff strategies for retry delays
#[derive(Debug, Clone)]
pub enum BackoffStrategy {
    /// Fixed delay between retries
    Fixed,
    /// Linear increase in delay
    Linear { increment: Duration },
    /// Exponential backoff with configurable multiplier
    Exponential { multiplier: f64 },
}

/// Classification of errors for retry behavior
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorClassification {
    /// Error is retryable (e.g., throttling, network timeouts)
    Retryable,
    /// Error is not retryable (e.g., invalid input, authentication)
    NonRetryable,
    /// Context window overflow - requires special handling
    ContextOverflow,
}

/// Result of a retry attempt
#[derive(Debug)]
pub struct RetryResult<T> {
    /// The final result (success or failure)
    pub result: Result<T>,
    /// Number of attempts made
    pub attempts_made: u32,
    /// Total time spent retrying
    pub total_duration: Duration,
    /// Whether the maximum attempts were reached
    pub max_attempts_reached: bool,
    /// Whether the maximum duration was reached
    pub max_duration_reached: bool,
}

/// Error classifier that determines retry behavior
pub struct ErrorClassifier;

impl ErrorClassifier {
    /// Classify an error to determine retry behavior
    pub fn classify(error: &StoodError) -> ErrorClassification {
        match error {
            // Retryable errors
            StoodError::ThrottlingError { .. } => ErrorClassification::Retryable,
            StoodError::ServiceUnavailable { .. } => ErrorClassification::Retryable,
            StoodError::NetworkError { .. } => ErrorClassification::Retryable,
            StoodError::TimeoutError { .. } => ErrorClassification::Retryable,

            // Context overflow needs special handling
            StoodError::QuotaExceeded { message }
                if message.contains("context") || message.contains("token") =>
            {
                ErrorClassification::ContextOverflow
            }
            StoodError::InvalidInput { message }
                if message.contains("too long") || message.contains("context") =>
            {
                ErrorClassification::ContextOverflow
            }

            // Enhanced ValidationError handling with message inspection
            StoodError::ValidationError { message } => Self::classify_validation_error(message),

            // Non-retryable errors
            StoodError::InvalidInput { .. } => ErrorClassification::NonRetryable,
            StoodError::ConfigurationError { .. } => ErrorClassification::NonRetryable,
            StoodError::AccessDenied { .. } => ErrorClassification::NonRetryable,
            StoodError::ResourceNotFound { .. } => ErrorClassification::NonRetryable,
            StoodError::SerializationError { .. } => ErrorClassification::NonRetryable,

            // Default to non-retryable for safety
            _ => ErrorClassification::NonRetryable,
        }
    }

    /// Check if an error is retryable
    pub fn is_retryable(error: &StoodError) -> bool {
        matches!(Self::classify(error), ErrorClassification::Retryable)
    }

    /// Check if an error is a context overflow
    pub fn is_context_overflow(error: &StoodError) -> bool {
        matches!(Self::classify(error), ErrorClassification::ContextOverflow)
    }

    /// Classify ValidationError based on message content (matching Python reference)
    fn classify_validation_error(message: &str) -> ErrorClassification {
        // Bedrock-specific context overflow messages (from Python reference)
        const BEDROCK_CONTEXT_OVERFLOW_MESSAGES: &[&str] = &[
            "Input is too long for requested model",
            "input length and `max_tokens` exceed context limit",
            "too many total text bytes",
            "input is too long",
            "input length exceeds context window",
            "input and output tokens exceed your context limit",
        ];

        // Check if this is a context overflow ValidationException
        let message_lower = message.to_lowercase();
        for overflow_message in BEDROCK_CONTEXT_OVERFLOW_MESSAGES {
            if message_lower.contains(&overflow_message.to_lowercase()) {
                return ErrorClassification::ContextOverflow;
            }
        }

        // General validation errors are not retryable
        ErrorClassification::NonRetryable
    }
}

/// Retry executor that handles retries with backoff
#[derive(Debug, Clone)]
pub struct RetryExecutor {
    config: RetryConfig,
}

impl Default for RetryExecutor {
    fn default() -> Self {
        Self::new(RetryConfig::default())
    }
}

impl RetryExecutor {
    /// Create a new retry executor with the given configuration
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }

    /// Execute an operation with retry logic
    pub async fn execute<T, F, Fut>(&self, mut operation: F) -> RetryResult<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let start_time = Instant::now();
        let mut attempts_made = 0;
        let mut current_delay = self.config.initial_delay;

        loop {
            attempts_made += 1;

            // Enhanced retry attempt logging
            if attempts_made == 1 {
                debug!(
                    "🚀 Starting operation (attempt 1/{}, max duration: {:?})",
                    self.config.max_attempts, self.config.max_total_duration
                );
            } else {
                info!(
                    "🔄 Retry attempt {}/{} starting...",
                    attempts_made, self.config.max_attempts
                );
            }

            match operation().await {
                Ok(result) => {
                    let total_duration = start_time.elapsed();
                    if attempts_made == 1 {
                        debug!(
                            "✅ Operation succeeded on first attempt in {:.2}s",
                            total_duration.as_secs_f64()
                        );
                    } else {
                        info!(
                            "✅ Operation succeeded after {} retries in {:.2}s total",
                            attempts_made,
                            total_duration.as_secs_f64()
                        );
                    }
                    return RetryResult {
                        result: Ok(result),
                        attempts_made,
                        total_duration: start_time.elapsed(),
                        max_attempts_reached: false,
                        max_duration_reached: false,
                    };
                }
                Err(error) => {
                    let classification = ErrorClassifier::classify(&error);

                    // Check if we should retry
                    if classification != ErrorClassification::Retryable {
                        debug!("Error is not retryable: {:?}", classification);
                        return RetryResult {
                            result: Err(error),
                            attempts_made,
                            total_duration: start_time.elapsed(),
                            max_attempts_reached: false,
                            max_duration_reached: false,
                        };
                    }

                    // Check if we've reached maximum attempts
                    if attempts_made >= self.config.max_attempts {
                        let total_duration = start_time.elapsed();
                        error!(
                            "💥 Maximum retry attempts ({}) reached after {:.2}s - giving up",
                            self.config.max_attempts,
                            total_duration.as_secs_f64()
                        );
                        return RetryResult {
                            result: Err(error),
                            attempts_made,
                            total_duration: start_time.elapsed(),
                            max_attempts_reached: true,
                            max_duration_reached: false,
                        };
                    }

                    // Check if we've reached maximum duration
                    let elapsed = start_time.elapsed();
                    if let Some(max_duration) = self.config.max_total_duration {
                        if elapsed >= max_duration {
                            error!("⏰ Maximum retry duration ({:.1}s) reached after {} attempts - timing out",
                                max_duration.as_secs_f64(), attempts_made);
                            return RetryResult {
                                result: Err(error),
                                attempts_made,
                                total_duration: elapsed,
                                max_attempts_reached: false,
                                max_duration_reached: true,
                            };
                        }
                    }

                    // Calculate delay for next attempt
                    let delay = self.calculate_delay(current_delay, attempts_made);

                    // Enhanced retry logging with user-friendly messages
                    match ErrorClassifier::classify(&error) {
                        ErrorClassification::Retryable => {
                            if attempts_made == 1 {
                                warn!("🔄 First attempt failed, retrying in {:.1}s... (attempt {}/{})",
                                    delay.as_secs_f64(), attempts_made + 1, self.config.max_attempts);
                            } else {
                                warn!(
                                    "⏳ Retry {}/{} failed, trying again in {:.1}s... ({})",
                                    attempts_made,
                                    self.config.max_attempts,
                                    delay.as_secs_f64(),
                                    self.get_user_friendly_error_message(&error)
                                );
                            }
                            debug!("🔍 Retry details - Error: {}", error);
                        }
                        _ => {
                            debug!("❌ Operation failed with non-retryable error: {}", error);
                        }
                    }

                    // Sleep before next retry
                    sleep(delay).await;

                    // Update delay for next iteration
                    current_delay = self.update_delay(current_delay);
                }
            }
        }
    }

    /// Calculate the delay for the next retry attempt
    fn calculate_delay(&self, current_delay: Duration, attempt: u32) -> Duration {
        let base_delay = match self.config.backoff_strategy {
            BackoffStrategy::Fixed => current_delay,
            BackoffStrategy::Linear { increment } => {
                self.config.initial_delay + increment * (attempt - 1)
            }
            BackoffStrategy::Exponential { multiplier } => {
                let delay_secs =
                    self.config.initial_delay.as_secs_f64() * multiplier.powi((attempt - 1) as i32);
                Duration::from_secs_f64(delay_secs)
            }
        };

        // Cap at maximum delay
        let capped_delay = std::cmp::min(base_delay, self.config.max_delay);

        // Add jitter if enabled
        if self.config.enable_jitter {
            self.add_jitter(capped_delay)
        } else {
            capped_delay
        }
    }

    /// Update the current delay for the next iteration
    fn update_delay(&self, current_delay: Duration) -> Duration {
        match self.config.backoff_strategy {
            BackoffStrategy::Fixed => current_delay,
            BackoffStrategy::Linear { .. } => current_delay, // Calculated fresh each time
            BackoffStrategy::Exponential { multiplier } => {
                let new_delay_secs = current_delay.as_secs_f64() * multiplier;
                Duration::from_secs_f64(new_delay_secs)
            }
        }
    }

    /// Add jitter to prevent thundering herd problem
    fn add_jitter(&self, delay: Duration) -> Duration {
        // Add up to 25% jitter
        let jitter_factor = 0.25;
        let max_jitter = delay.as_secs_f64() * jitter_factor;
        let jitter = fastrand::f64() * max_jitter;

        Duration::from_secs_f64(delay.as_secs_f64() + jitter)
    }

    /// Get user-friendly error message for logging
    fn get_user_friendly_error_message(&self, error: &StoodError) -> &'static str {
        match error {
            StoodError::ThrottlingError { .. } => "API rate limit exceeded",
            StoodError::ServiceUnavailable { .. } => "service temporarily unavailable",
            StoodError::NetworkError { .. } => "network connectivity issue",
            StoodError::TimeoutError { .. } => "request timed out",
            StoodError::AccessDenied { .. } => "access denied - check credentials",
            StoodError::ValidationError { .. } => "invalid request format",
            StoodError::ModelError { .. } => "model processing error",
            StoodError::QuotaExceeded { .. } => "quota limit exceeded",
            _ => "temporary service issue",
        }
    }
}

/// Context recovery handler for dealing with context window overflow
pub struct ContextRecovery;

impl ContextRecovery {
    /// Handle context window overflow by truncating tool results
    pub fn handle_context_overflow(messages: &mut Messages) -> Result<bool> {
        debug!("Handling context window overflow by truncating tool results");

        // Find the last message with tool results
        if let Some(message_index) = Self::find_last_message_with_tool_results(messages) {
            debug!("Found message with tool results at index {}", message_index);

            // Truncate the tool results in this message
            Self::truncate_tool_results(messages, message_index)?;
            Ok(true)
        } else {
            debug!("No tool results found to truncate");
            Ok(false)
        }
    }

    /// Find the index of the last message containing tool results
    fn find_last_message_with_tool_results(messages: &Messages) -> Option<usize> {
        // Iterate backwards through all messages (from newest to oldest)
        for (idx, message) in messages.messages.iter().enumerate().rev() {
            // Check if this message has any content with tool results
            for content in &message.content {
                if let crate::types::ContentBlock::ToolResult { .. } = content {
                    return Some(idx);
                }
            }
        }
        None
    }

    /// Truncate tool results in a message to reduce context size
    fn truncate_tool_results(messages: &mut Messages, message_index: usize) -> Result<()> {
        if message_index >= messages.messages.len() {
            return Err(StoodError::invalid_input("Message index out of bounds"));
        }

        let message = &mut messages.messages[message_index];
        let mut changes_made = false;

        for content in &mut message.content {
            if let crate::types::ContentBlock::ToolResult {
                content, is_error, ..
            } = content
            {
                // Replace large tool results with a truncation message
                *content = crate::types::ToolResultContent::text("The tool result was too large and has been truncated to fit the context window.");
                *is_error = true;
                changes_made = true;
            }
        }

        if changes_made {
            debug!(
                "Truncated tool results in message at index {}",
                message_index
            );
        }

        Ok(())
    }

    /// Remove oldest messages to free up context space
    pub fn remove_oldest_messages(messages: &mut Messages, count: usize) -> usize {
        let original_count = messages.messages.len();
        let to_remove = std::cmp::min(count, original_count.saturating_sub(1)); // Keep at least 1 message

        messages.messages.drain(0..to_remove);

        debug!(
            "Removed {} oldest messages to free context space",
            to_remove
        );
        to_remove
    }

    /// Estimate if messages are likely to overflow context window
    pub fn estimate_context_usage(messages: &Messages) -> usize {
        // Simple estimation based on character count
        // This is a rough approximation - real implementation would use proper token counting
        messages
            .messages
            .iter()
            .map(|msg| {
                msg.content
                    .iter()
                    .map(|content| match content {
                        crate::types::ContentBlock::Text { text } => text.len(),
                        crate::types::ContentBlock::ToolUse { input, .. } => {
                            input.to_string().len() + 100 // Approximate overhead
                        }
                        crate::types::ContentBlock::ToolResult { content, .. } => {
                            // Estimate content size based on type
                            let content_size = match content {
                                crate::types::ToolResultContent::Text { text } => text.len(),
                                crate::types::ToolResultContent::Json { data } => {
                                    data.to_string().len()
                                }
                                crate::types::ToolResultContent::Binary { data, .. } => data.len(),
                                crate::types::ToolResultContent::Multiple { blocks } => {
                                    blocks
                                        .iter()
                                        .map(|b| match b {
                                            crate::types::ToolResultContent::Text { text } => {
                                                text.len()
                                            }
                                            crate::types::ToolResultContent::Json { data } => {
                                                data.to_string().len()
                                            }
                                            crate::types::ToolResultContent::Binary {
                                                data,
                                                ..
                                            } => data.len(),
                                            _ => 100, // Fallback estimate
                                        })
                                        .sum::<usize>()
                                }
                            };
                            content_size + 50 // Approximate overhead
                        }
                        crate::types::ContentBlock::Thinking { content, .. } => content.len(),
                        crate::types::ContentBlock::ReasoningContent { reasoning } => {
                            reasoning.text().len()
                        }
                    })
                    .sum::<usize>()
            })
            .sum()
    }

    /// Handle context overflow specifically from ValidationException
    /// Follows Python reference pattern: tool result truncation + oldest message removal
    pub fn handle_validation_context_overflow(messages: &mut Messages) -> Result<bool> {
        tracing::warn!("🔄 Handling ValidationException context overflow");

        // Log conversation state before recovery
        let original_message_count = messages.messages.len();
        let total_chars = messages
            .messages
            .iter()
            .map(|m| {
                m.content
                    .iter()
                    .map(|c| match c {
                        crate::types::ContentBlock::Text { text } => text.len(),
                        crate::types::ContentBlock::ToolResult { content, .. } => {
                            content.to_display_string().len()
                        }
                        _ => 100, // Estimate for other content types
                    })
                    .sum::<usize>()
            })
            .sum::<usize>();

        tracing::debug!(
            "📊 Pre-recovery state: {} messages, ~{} total characters",
            original_message_count,
            total_chars
        );

        // Step 1: Try tool result truncation first (matches Python priority)
        if Self::truncate_enhanced_tool_results(messages)? {
            let post_truncation_chars = messages
                .messages
                .iter()
                .map(|m| {
                    m.content
                        .iter()
                        .map(|c| match c {
                            crate::types::ContentBlock::Text { text } => text.len(),
                            crate::types::ContentBlock::ToolResult { content, .. } => {
                                content.to_display_string().len()
                            }
                            _ => 100,
                        })
                        .sum::<usize>()
                })
                .sum::<usize>();

            tracing::info!("✂️ Truncated tool results to recover from ValidationException");
            tracing::debug!(
                "📉 Character reduction: {} → {} ({:.1}% reduction)",
                total_chars,
                post_truncation_chars,
                (total_chars as f64 - post_truncation_chars as f64) / total_chars as f64 * 100.0
            );
            return Ok(true);
        }

        // Step 2: Remove oldest messages (sliding window approach)
        let removed = Self::remove_oldest_messages(messages, 3); // Remove 3 oldest
        if removed > 0 {
            let post_removal_chars = messages
                .messages
                .iter()
                .map(|m| {
                    m.content
                        .iter()
                        .map(|c| match c {
                            crate::types::ContentBlock::Text { text } => text.len(),
                            crate::types::ContentBlock::ToolResult { content, .. } => {
                                content.to_display_string().len()
                            }
                            _ => 100,
                        })
                        .sum::<usize>()
                })
                .sum::<usize>();

            tracing::info!(
                "🗑️ Removed {} oldest messages to recover from ValidationException",
                removed
            );
            tracing::debug!("📉 Message reduction: {} → {} messages, {} → {} characters ({:.1}% char reduction)",
                original_message_count, messages.messages.len(),
                total_chars, post_removal_chars,
                (total_chars as f64 - post_removal_chars as f64) / total_chars as f64 * 100.0);
            return Ok(true);
        }

        tracing::error!("❌ Unable to recover from ValidationException context overflow");
        tracing::debug!(
            "💔 Recovery failed - no tool results to truncate and no messages to remove"
        );
        Ok(false)
    }

    /// Enhanced tool result truncation that preserves more structure than the basic version
    /// Matches Python reference: find_last_message_with_tool_results + truncate_tool_results
    fn truncate_enhanced_tool_results(messages: &mut Messages) -> Result<bool> {
        // Find the last message with tool results
        let last_tool_result_index = messages.messages.iter().rposition(|msg| {
            msg.content
                .iter()
                .any(|content| matches!(content, crate::types::ContentBlock::ToolResult { .. }))
        });

        if let Some(index) = last_tool_result_index {
            let message = &mut messages.messages[index];
            let original_count = message.content.len();

            // Truncate tool result content to first 1000 characters
            for content in &mut message.content {
                if let crate::types::ContentBlock::ToolResult {
                    content: tool_content,
                    ..
                } = content
                {
                    *tool_content = Self::truncate_tool_result_content(tool_content, 1000);
                }
            }

            debug!(
                "Truncated tool results in message {} (had {} content blocks)",
                index, original_count
            );
            return Ok(true);
        }

        Ok(false)
    }

    /// Truncate tool result content while preserving structure
    fn truncate_tool_result_content(
        content: &crate::types::ToolResultContent,
        max_chars: usize,
    ) -> crate::types::ToolResultContent {
        match content {
            crate::types::ToolResultContent::Text { text } => {
                if text.len() > max_chars {
                    crate::types::ToolResultContent::text(format!(
                        "{}...[truncated for context window]",
                        &text[..max_chars]
                    ))
                } else {
                    content.clone()
                }
            }
            crate::types::ToolResultContent::Json { data } => {
                let text = data.to_string();
                if text.len() > max_chars {
                    crate::types::ToolResultContent::text(format!(
                        "{}...[truncated JSON for context window]",
                        &text[..max_chars]
                    ))
                } else {
                    content.clone()
                }
            }
            crate::types::ToolResultContent::Binary { data, mime_type } => {
                if data.len() > max_chars {
                    crate::types::ToolResultContent::text(format!(
                        "[Binary data ({}) truncated for context window: {} bytes]",
                        mime_type,
                        data.len()
                    ))
                } else {
                    content.clone()
                }
            }
            crate::types::ToolResultContent::Multiple { blocks } => {
                let truncated_blocks: Vec<_> = blocks
                    .iter()
                    .take(3) // Keep only first 3 blocks
                    .map(|block| Self::truncate_tool_result_content(block, max_chars / 3))
                    .collect();

                if blocks.len() > 3 {
                    let mut final_blocks = truncated_blocks;
                    final_blocks.push(crate::types::ToolResultContent::text(format!(
                        "...[{} more blocks truncated for context window]",
                        blocks.len() - 3
                    )));
                    crate::types::ToolResultContent::multiple(final_blocks)
                } else {
                    crate::types::ToolResultContent::multiple(truncated_blocks)
                }
            }
        }
    }
}

/// Circuit breaker to prevent cascading failures
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    /// Failure threshold before opening circuit
    pub failure_threshold: u32,
    /// Time to wait before attempting to close circuit
    pub recovery_timeout: Duration,
    /// Current state
    state: CircuitBreakerState,
    /// Failure count in current window
    failure_count: u32,
    /// Time when circuit was opened
    opened_at: Option<Instant>,
}

#[derive(Debug, Clone, PartialEq)]
enum CircuitBreakerState {
    Closed,   // Normal operation
    Open,     // Failing, rejecting requests
    HalfOpen, // Testing if service recovered
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new(5, Duration::from_secs(60)) // 5 failures, 1 minute recovery
    }
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(failure_threshold: u32, recovery_timeout: Duration) -> Self {
        Self {
            failure_threshold,
            recovery_timeout,
            state: CircuitBreakerState::Closed,
            failure_count: 0,
            opened_at: None,
        }
    }

    /// Check if a request should be allowed through
    pub fn should_allow_request(&mut self) -> bool {
        match self.state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                // Check if recovery timeout has passed
                if let Some(opened_at) = self.opened_at {
                    if opened_at.elapsed() >= self.recovery_timeout {
                        debug!("Circuit breaker entering half-open state");
                        self.state = CircuitBreakerState::HalfOpen;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitBreakerState::HalfOpen => true, // Allow one test request
        }
    }

    /// Record a successful operation
    pub fn record_success(&mut self) {
        match self.state {
            CircuitBreakerState::Closed => {
                self.failure_count = 0; // Reset failure count
            }
            CircuitBreakerState::HalfOpen => {
                debug!("Circuit breaker closing after successful test");
                self.state = CircuitBreakerState::Closed;
                self.failure_count = 0;
                self.opened_at = None;
            }
            CircuitBreakerState::Open => {
                // Should not happen if should_allow_request is used correctly
            }
        }
    }

    /// Record a failed operation
    pub fn record_failure(&mut self) {
        self.failure_count += 1;

        match self.state {
            CircuitBreakerState::Closed => {
                if self.failure_count >= self.failure_threshold {
                    debug!(
                        "Circuit breaker opening due to {} failures",
                        self.failure_count
                    );
                    self.state = CircuitBreakerState::Open;
                    self.opened_at = Some(Instant::now());
                }
            }
            CircuitBreakerState::HalfOpen => {
                debug!("Circuit breaker opening again after failed test");
                self.state = CircuitBreakerState::Open;
                self.opened_at = Some(Instant::now());
            }
            CircuitBreakerState::Open => {
                // Already open, no action needed
            }
        }
    }

    /// Get current state information
    pub fn state(&self) -> &str {
        match self.state {
            CircuitBreakerState::Closed => "closed",
            CircuitBreakerState::Open => "open",
            CircuitBreakerState::HalfOpen => "half-open",
        }
    }

    /// Get current failure count
    pub fn failure_count(&self) -> u32 {
        self.failure_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, 6);
        assert_eq!(config.initial_delay, Duration::from_secs(4));
        assert_eq!(config.max_delay, Duration::from_secs(240));
        assert!(config.enable_jitter);
        assert!(matches!(
            config.backoff_strategy,
            BackoffStrategy::Exponential { .. }
        ));
    }

    #[test]
    fn test_error_classification() {
        // Retryable errors
        assert_eq!(
            ErrorClassifier::classify(&StoodError::ThrottlingError {
                message: "throttled".to_string()
            }),
            ErrorClassification::Retryable
        );
        assert_eq!(
            ErrorClassifier::classify(&StoodError::ServiceUnavailable {
                message: "unavailable".to_string()
            }),
            ErrorClassification::Retryable
        );
        assert_eq!(
            ErrorClassifier::classify(&StoodError::NetworkError {
                message: "network".to_string()
            }),
            ErrorClassification::Retryable
        );

        // Context overflow
        assert_eq!(
            ErrorClassifier::classify(&StoodError::QuotaExceeded {
                message: "context limit exceeded".to_string()
            }),
            ErrorClassification::ContextOverflow
        );
        assert_eq!(
            ErrorClassifier::classify(&StoodError::InvalidInput {
                message: "input too long".to_string()
            }),
            ErrorClassification::ContextOverflow
        );

        // Non-retryable errors
        assert_eq!(
            ErrorClassifier::classify(&StoodError::ConfigurationError {
                message: "bad config".to_string()
            }),
            ErrorClassification::NonRetryable
        );
        assert_eq!(
            ErrorClassifier::classify(&StoodError::AccessDenied {
                message: "denied".to_string()
            }),
            ErrorClassification::NonRetryable
        );
    }

    #[test]
    fn test_validation_exception_context_overflow_classification() {
        // Test Bedrock context overflow messages (should be retryable with context recovery)
        let error1 = StoodError::validation_error("Input is too long for requested model");
        assert_eq!(
            ErrorClassifier::classify(&error1),
            ErrorClassification::ContextOverflow
        );

        let error2 =
            StoodError::validation_error("input length and `max_tokens` exceed context limit");
        assert_eq!(
            ErrorClassifier::classify(&error2),
            ErrorClassification::ContextOverflow
        );

        let error3 = StoodError::validation_error("too many total text bytes");
        assert_eq!(
            ErrorClassifier::classify(&error3),
            ErrorClassification::ContextOverflow
        );

        let error4 = StoodError::validation_error("input is too long");
        assert_eq!(
            ErrorClassifier::classify(&error4),
            ErrorClassification::ContextOverflow
        );

        let error5 = StoodError::validation_error("input length exceeds context window");
        assert_eq!(
            ErrorClassifier::classify(&error5),
            ErrorClassification::ContextOverflow
        );

        let error6 =
            StoodError::validation_error("input and output tokens exceed your context limit");
        assert_eq!(
            ErrorClassifier::classify(&error6),
            ErrorClassification::ContextOverflow
        );

        // Test case insensitivity
        let error7 = StoodError::validation_error("INPUT IS TOO LONG FOR REQUESTED MODEL");
        assert_eq!(
            ErrorClassifier::classify(&error7),
            ErrorClassification::ContextOverflow
        );

        // Test non-context validation errors (should be non-retryable)
        let error8 = StoodError::validation_error("Invalid parameter value");
        assert_eq!(
            ErrorClassifier::classify(&error8),
            ErrorClassification::NonRetryable
        );

        let error9 = StoodError::validation_error("Malformed request");
        assert_eq!(
            ErrorClassifier::classify(&error9),
            ErrorClassification::NonRetryable
        );

        let error10 = StoodError::validation_error("Missing required field");
        assert_eq!(
            ErrorClassifier::classify(&error10),
            ErrorClassification::NonRetryable
        );
    }

    #[test]
    fn test_error_classifier_helpers() {
        let retryable_error = StoodError::ThrottlingError {
            message: "throttled".to_string(),
        };
        let context_error = StoodError::InvalidInput {
            message: "context too long".to_string(),
        };
        let non_retryable_error = StoodError::AccessDenied {
            message: "denied".to_string(),
        };

        assert!(ErrorClassifier::is_retryable(&retryable_error));
        assert!(!ErrorClassifier::is_retryable(&context_error));
        assert!(!ErrorClassifier::is_retryable(&non_retryable_error));

        assert!(!ErrorClassifier::is_context_overflow(&retryable_error));
        assert!(ErrorClassifier::is_context_overflow(&context_error));
        assert!(!ErrorClassifier::is_context_overflow(&non_retryable_error));
    }

    #[tokio::test]
    async fn test_retry_executor_success() {
        let config = RetryConfig {
            max_attempts: 3,
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(100),
            backoff_strategy: BackoffStrategy::Fixed,
            enable_jitter: false,
            max_total_duration: None,
        };

        let executor = RetryExecutor::new(config);
        let call_count = std::sync::Arc::new(std::sync::Mutex::new(0));
        let call_count_clone = call_count.clone();

        let result = executor
            .execute(move || {
                let call_count = call_count_clone.clone();
                async move {
                    *call_count.lock().unwrap() += 1;
                    Ok::<i32, StoodError>(42)
                }
            })
            .await;

        assert!(result.result.is_ok());
        assert_eq!(result.result.unwrap(), 42);
        assert_eq!(result.attempts_made, 1);
        assert_eq!(*call_count.lock().unwrap(), 1);
        assert!(!result.max_attempts_reached);
    }

    #[tokio::test]
    async fn test_retry_executor_retryable_error() {
        let config = RetryConfig {
            max_attempts: 3,
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(100),
            backoff_strategy: BackoffStrategy::Fixed,
            enable_jitter: false,
            max_total_duration: None,
        };

        let executor = RetryExecutor::new(config);
        let call_count = std::sync::Arc::new(std::sync::Mutex::new(0));
        let call_count_clone = call_count.clone();

        let result = executor
            .execute(move || {
                let call_count = call_count_clone.clone();
                async move {
                    let mut count = call_count.lock().unwrap();
                    *count += 1;
                    let current_count = *count;
                    drop(count);

                    if current_count < 3 {
                        Err(StoodError::ThrottlingError {
                            message: "throttled".to_string(),
                        })
                    } else {
                        Ok::<i32, StoodError>(42)
                    }
                }
            })
            .await;

        assert!(result.result.is_ok());
        assert_eq!(result.result.unwrap(), 42);
        assert_eq!(result.attempts_made, 3);
        assert_eq!(*call_count.lock().unwrap(), 3);
        assert!(!result.max_attempts_reached);
    }

    #[tokio::test]
    async fn test_retry_executor_non_retryable_error() {
        let config = RetryConfig {
            max_attempts: 3,
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(100),
            backoff_strategy: BackoffStrategy::Fixed,
            enable_jitter: false,
            max_total_duration: None,
        };

        let executor = RetryExecutor::new(config);
        let call_count = std::sync::Arc::new(std::sync::Mutex::new(0));
        let call_count_clone = call_count.clone();

        let result = executor
            .execute(move || {
                let call_count = call_count_clone.clone();
                async move {
                    *call_count.lock().unwrap() += 1;
                    Err::<i32, StoodError>(StoodError::AccessDenied {
                        message: "denied".to_string(),
                    })
                }
            })
            .await;

        assert!(result.result.is_err());
        assert_eq!(result.attempts_made, 1);
        assert_eq!(*call_count.lock().unwrap(), 1);
        assert!(!result.max_attempts_reached);
    }

    #[tokio::test]
    async fn test_retry_executor_max_attempts() {
        let config = RetryConfig {
            max_attempts: 2,
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(100),
            backoff_strategy: BackoffStrategy::Fixed,
            enable_jitter: false,
            max_total_duration: None,
        };

        let executor = RetryExecutor::new(config);
        let call_count = std::sync::Arc::new(std::sync::Mutex::new(0));
        let call_count_clone = call_count.clone();

        let result = executor
            .execute(move || {
                let call_count = call_count_clone.clone();
                async move {
                    *call_count.lock().unwrap() += 1;
                    Err::<i32, StoodError>(StoodError::ThrottlingError {
                        message: "throttled".to_string(),
                    })
                }
            })
            .await;

        assert!(result.result.is_err());
        assert_eq!(result.attempts_made, 2);
        assert_eq!(*call_count.lock().unwrap(), 2);
        assert!(result.max_attempts_reached);
    }

    #[test]
    fn test_context_recovery_find_tool_results() {
        use crate::types::{ContentBlock, Message, MessageRole, Messages, ToolResultContent};

        let mut messages = Messages::new();

        // Add message without tool results
        messages.push(Message::new(
            MessageRole::User,
            vec![ContentBlock::text("Hello")],
        ));

        // Add message with tool results
        messages.push(Message::new(
            MessageRole::User,
            vec![
                ContentBlock::text("Some text"),
                ContentBlock::ToolResult {
                    tool_use_id: "tool_123".to_string(),
                    content: ToolResultContent::text("result"),
                    is_error: false,
                },
            ],
        ));

        let index = ContextRecovery::find_last_message_with_tool_results(&messages);
        assert_eq!(index, Some(1));
    }

    #[test]
    fn test_context_recovery_no_tool_results() {
        use crate::types::{ContentBlock, Message, MessageRole, Messages};

        let mut messages = Messages::new();
        messages.push(Message::new(
            MessageRole::User,
            vec![ContentBlock::text("Hello")],
        ));

        let index = ContextRecovery::find_last_message_with_tool_results(&messages);
        assert_eq!(index, None);
    }

    #[test]
    fn test_context_recovery_truncate_tool_results() {
        use crate::types::{ContentBlock, Message, MessageRole, Messages, ToolResultContent};

        let mut messages = Messages::new();
        messages.push(Message::new(
            MessageRole::User,
            vec![
                ContentBlock::text("Some text"),
                ContentBlock::ToolResult {
                    tool_use_id: "tool_123".to_string(),
                    content: ToolResultContent::text("Large result that needs truncation"),
                    is_error: false,
                },
            ],
        ));

        let result = ContextRecovery::truncate_tool_results(&mut messages, 0);
        assert!(result.is_ok());

        // Check that tool result was truncated
        if let ContentBlock::ToolResult {
            content, is_error, ..
        } = &messages.messages[0].content[1]
        {
            if let ToolResultContent::Text { text } = content {
                assert!(text.contains("truncated"));
            } else {
                panic!("Expected text tool result content");
            }
            assert!(*is_error);
        } else {
            panic!("Expected tool result content block");
        }
    }

    #[test]
    fn test_context_recovery_estimate_usage() {
        use crate::types::{ContentBlock, Message, MessageRole, Messages};

        let mut messages = Messages::new();
        messages.push(Message::new(
            MessageRole::User,
            vec![ContentBlock::text("Hello world")], // 11 characters
        ));
        messages.push(Message::new(
            MessageRole::Assistant,
            vec![ContentBlock::text("Hi there")], // 8 characters
        ));

        let usage = ContextRecovery::estimate_context_usage(&messages);
        assert_eq!(usage, 19); // 11 + 8
    }

    #[test]
    fn test_circuit_breaker_default() {
        let mut breaker = CircuitBreaker::default();
        assert_eq!(breaker.state(), "closed");
        assert_eq!(breaker.failure_count(), 0);
        assert!(breaker.should_allow_request());
    }

    #[test]
    fn test_circuit_breaker_failure_threshold() {
        let mut breaker = CircuitBreaker::new(2, Duration::from_secs(10));

        // Should be closed initially
        assert_eq!(breaker.state(), "closed");
        assert!(breaker.should_allow_request());

        // Record first failure
        breaker.record_failure();
        assert_eq!(breaker.state(), "closed");
        assert_eq!(breaker.failure_count(), 1);
        assert!(breaker.should_allow_request());

        // Record second failure - should open circuit
        breaker.record_failure();
        assert_eq!(breaker.state(), "open");
        assert_eq!(breaker.failure_count(), 2);
        assert!(!breaker.should_allow_request());
    }

    #[test]
    fn test_circuit_breaker_success_reset() {
        let mut breaker = CircuitBreaker::new(2, Duration::from_secs(10));

        // Record failure
        breaker.record_failure();
        assert_eq!(breaker.failure_count(), 1);

        // Record success - should reset failure count
        breaker.record_success();
        assert_eq!(breaker.failure_count(), 0);
        assert_eq!(breaker.state(), "closed");
    }

    #[tokio::test]
    async fn test_circuit_breaker_recovery() {
        let mut breaker = CircuitBreaker::new(1, Duration::from_millis(50));

        // Trigger circuit open
        breaker.record_failure();
        assert_eq!(breaker.state(), "open");
        assert!(!breaker.should_allow_request());

        // Wait for recovery timeout
        sleep(Duration::from_millis(60)).await;

        // Should enter half-open state
        assert!(breaker.should_allow_request());
        assert_eq!(breaker.state(), "half-open");

        // Success should close circuit
        breaker.record_success();
        assert_eq!(breaker.state(), "closed");
        assert!(breaker.should_allow_request());
    }

    #[tokio::test]
    async fn test_validation_context_recovery() {
        use crate::types::{ContentBlock, Message, MessageRole, Messages, ToolResultContent};

        // Create messages with tool results that would cause context overflow
        let mut messages = Messages::new();

        // Add a user message
        let user_message = Message::new(
            MessageRole::User,
            vec![ContentBlock::text("Calculate something for me")],
        );
        messages.push(user_message);

        // Add an assistant message with tool use
        let assistant_message = Message::new(
            MessageRole::Assistant,
            vec![ContentBlock::tool_use(
                "tool_123".to_string(),
                "calculator".to_string(),
                serde_json::json!({"expression": "2 + 2"}),
            )],
        );
        messages.push(assistant_message);

        // Add a large tool result that would cause context overflow
        let large_result = "x".repeat(5000); // Very large result
        let tool_result_message = Message::new(
            MessageRole::User,
            vec![ContentBlock::tool_result_success(
                "tool_123".to_string(),
                ToolResultContent::text(large_result),
            )],
        );
        messages.push(tool_result_message);

        let original_message_count = messages.messages.len();
        assert_eq!(original_message_count, 3);

        // Test ValidationException context recovery
        let result = ContextRecovery::handle_validation_context_overflow(&mut messages);
        assert!(result.is_ok());
        assert!(result.unwrap()); // Should return true indicating successful recovery

        // Check that tool result was truncated
        let tool_result_message = &messages.messages[2];
        if let ContentBlock::ToolResult { content, .. } = &tool_result_message.content[0] {
            let content_text = content.to_display_string();
            assert!(content_text.contains("truncated for context window"));
            assert!(content_text.len() < 5000); // Should be much smaller now
        } else {
            panic!("Expected tool result content block");
        }

        // Messages should still be there, just with truncated content
        assert_eq!(messages.messages.len(), 3);
    }

    #[tokio::test]
    async fn test_validation_context_recovery_no_tool_results() {
        use crate::types::{ContentBlock, Message, MessageRole, Messages};

        // Create messages without tool results
        let mut messages = Messages::new();

        for i in 0..5 {
            let message = Message::new(
                MessageRole::User,
                vec![ContentBlock::text(format!(
                    "Message {} with lots of text",
                    i
                ))],
            );
            messages.push(message);
        }

        let original_count = messages.messages.len();
        assert_eq!(original_count, 5);

        // Test ValidationException context recovery
        let result = ContextRecovery::handle_validation_context_overflow(&mut messages);
        assert!(result.is_ok());
        assert!(result.unwrap()); // Should return true - removed oldest messages

        // Should have removed 3 oldest messages
        assert_eq!(messages.messages.len(), 2);

        // Remaining messages should be the newest ones
        if let ContentBlock::Text { text } = &messages.messages[0].content[0] {
            assert!(text.contains("Message 3"));
        }
        if let ContentBlock::Text { text } = &messages.messages[1].content[0] {
            assert!(text.contains("Message 4"));
        }
    }
}
