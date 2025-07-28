//! Retry utilities for provider-level resilience
//!
//! This module provides configurable retry logic with exponential backoff
//! to handle temporary failures like model loading delays in LM Studio.

use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use tokio::time::sleep;
use crate::llm::traits::LlmError;

/// Configuration for retry behavior with exponential backoff
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts (not including the initial attempt)
    pub max_attempts: u32,
    /// Initial delay before the first retry
    pub initial_delay: Duration,
    /// Maximum delay between retries (caps exponential growth)
    pub max_delay: Duration,
    /// Multiplier for exponential backoff (e.g., 2.0 doubles delay each time)
    pub backoff_multiplier: f64,
    /// Whether to add random jitter to prevent thundering herd
    pub jitter: bool,
}

impl RetryConfig {
    /// Create a sensible default retry configuration for LM Studio
    pub fn lm_studio_default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(1000),  // Start with 1 second
            max_delay: Duration::from_secs(30),          // Cap at 30 seconds
            backoff_multiplier: 2.0,                     // Double each time
            jitter: true,                                // Add randomness
        }
    }

    /// Create a more aggressive retry configuration for slow model loading
    pub fn lm_studio_aggressive() -> Self {
        Self {
            max_attempts: 5,
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 1.5,
            jitter: true,
        }
    }

    /// Create a conservative retry configuration for production
    pub fn lm_studio_conservative() -> Self {
        Self {
            max_attempts: 2,
            initial_delay: Duration::from_millis(2000),
            max_delay: Duration::from_secs(15),
            backoff_multiplier: 2.0,
            jitter: false,
        }
    }

    /// Disable retries (for testing or when retries are undesired)
    pub fn disabled() -> Self {
        Self {
            max_attempts: 0,
            initial_delay: Duration::from_millis(0),
            max_delay: Duration::from_millis(0),
            backoff_multiplier: 1.0,
            jitter: false,
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self::lm_studio_default()
    }
}

/// Decision about whether to retry after an error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryDecision {
    /// Retry the operation after waiting
    Retry,
    /// Fail immediately without retrying
    FailImmediately,
}

/// Type alias for boxed future to simplify retry function signatures
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Calculate delay for exponential backoff with optional jitter
pub fn calculate_backoff_delay(attempt: u32, config: &RetryConfig) -> Duration {
    let base_delay = config.initial_delay.as_millis() as f64;
    let multiplier = config.backoff_multiplier.powi(attempt as i32);
    let delay_ms = (base_delay * multiplier) as u64;
    
    let delay = Duration::from_millis(delay_ms).min(config.max_delay);
    
    if config.jitter {
        add_jitter(delay)
    } else {
        delay
    }
}

/// Add random jitter to delay to prevent thundering herd effect
fn add_jitter(delay: Duration) -> Duration {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    // Simple pseudo-random jitter based on current timestamp
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    
    let mut hasher = DefaultHasher::new();
    now.hash(&mut hasher);
    let hash = hasher.finish();
    
    // Add ±25% jitter
    let jitter_factor = 0.75 + 0.5 * ((hash % 1000) as f64 / 1000.0);
    let jittered_ms = (delay.as_millis() as f64 * jitter_factor) as u64;
    
    Duration::from_millis(jittered_ms)
}

/// Determine if an LlmError should trigger a retry
pub fn should_retry_llm_error(error: &LlmError) -> RetryDecision {
    match error {
        // Network and connection errors - often transient
        LlmError::NetworkError { .. } => RetryDecision::Retry,
        
        // Provider errors that might be due to model loading
        LlmError::ProviderError { message, .. } => {
            let msg = message.to_lowercase();
            
            // Retry on connection-related errors
            if msg.contains("connection refused") 
                || msg.contains("connection reset")
                || msg.contains("timeout")
                || msg.contains("service unavailable")
                || msg.contains("bad gateway")
                || msg.contains("502")
                || msg.contains("503") {
                RetryDecision::Retry
            } else {
                RetryDecision::FailImmediately
            }
        },
        
        // Rate limiting - should respect retry_after but can retry
        LlmError::RateLimitError { .. } => RetryDecision::Retry,
        
        // These errors are not transient - don't retry
        LlmError::ConfigurationError { .. } => RetryDecision::FailImmediately,
        LlmError::AuthenticationError { .. } => RetryDecision::FailImmediately,
        LlmError::ModelNotFound { .. } => RetryDecision::FailImmediately,
        LlmError::SerializationError { .. } => RetryDecision::FailImmediately,
        LlmError::UnsupportedFeature { .. } => RetryDecision::FailImmediately,
    }
}

/// Execute an operation with retry logic and exponential backoff
pub async fn retry_with_backoff<F, T, E>(
    mut operation: F,
    config: &RetryConfig,
    should_retry: impl Fn(&E) -> RetryDecision,
) -> Result<T, E>
where
    F: FnMut() -> BoxFuture<'static, Result<T, E>>,
{
    let mut _last_error: Option<E> = None;
    
    // Initial attempt (attempt 0)
    match operation().await {
        Ok(result) => return Ok(result),
        Err(error) => {
            if config.max_attempts == 0 || should_retry(&error) == RetryDecision::FailImmediately {
                return Err(error);
            }
            _last_error = Some(error);
        }
    }
    
    // Retry attempts (attempts 1 through max_attempts)
    for attempt in 1..=config.max_attempts {
        let delay = calculate_backoff_delay(attempt - 1, config);
        
        tracing::debug!(
            "🔄 Retrying operation after {} ms (attempt {}/{})",
            delay.as_millis(),
            attempt,
            config.max_attempts
        );
        
        sleep(delay).await;
        
        match operation().await {
            Ok(result) => {
                tracing::info!("✅ Operation succeeded on retry attempt {}", attempt);
                return Ok(result);
            }
            Err(error) => {
                if should_retry(&error) == RetryDecision::FailImmediately {
                    tracing::warn!("❌ Operation failed with non-retryable error on attempt {}", attempt);
                    return Err(error);
                }
                
                if attempt == config.max_attempts {
                    tracing::error!("❌ Operation failed after {} retry attempts", config.max_attempts);
                    return Err(error);
                }
                
                _last_error = Some(error);
            }
        }
    }
    
    // This should never be reached, but handle it gracefully
    Err(_last_error.unwrap())
}

/// Convenience function for retrying LlmError operations
pub async fn retry_llm_operation<F, T>(
    operation: F,
    config: &RetryConfig,
) -> Result<T, LlmError>
where
    F: FnMut() -> BoxFuture<'static, Result<T, LlmError>>,
{
    retry_with_backoff(operation, config, should_retry_llm_error).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_retry_config_defaults() {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.initial_delay, Duration::from_millis(1000));
        assert_eq!(config.backoff_multiplier, 2.0);
        assert!(config.jitter);
    }

    #[test]
    fn test_backoff_calculation() {
        let config = RetryConfig {
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
            jitter: false,
            max_attempts: 5,
        };

        assert_eq!(calculate_backoff_delay(0, &config), Duration::from_millis(100));
        assert_eq!(calculate_backoff_delay(1, &config), Duration::from_millis(200));
        assert_eq!(calculate_backoff_delay(2, &config), Duration::from_millis(400));
    }

    #[test]
    fn test_max_delay_cap() {
        let config = RetryConfig {
            initial_delay: Duration::from_millis(1000),
            max_delay: Duration::from_millis(2000),
            backoff_multiplier: 10.0,
            jitter: false,
            max_attempts: 5,
        };

        // Should be capped at max_delay
        let delay = calculate_backoff_delay(5, &config);
        assert!(delay <= config.max_delay);
    }

    #[tokio::test]
    async fn test_successful_operation_no_retry() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();
        
        let config = RetryConfig::default();
        
        let result = retry_with_backoff(
            move || {
                let counter = counter_clone.clone();
                Box::pin(async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Ok::<i32, &'static str>(42)
                })
            },
            &config,
            |_| RetryDecision::Retry,
        ).await;
        
        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 1); // Only called once
    }

    #[tokio::test]
    async fn test_retry_with_eventual_success() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();
        
        let config = RetryConfig {
            max_attempts: 3,
            initial_delay: Duration::from_millis(1), // Fast for testing
            max_delay: Duration::from_millis(10),
            backoff_multiplier: 2.0,
            jitter: false,
        };
        
        let result = retry_with_backoff(
            move || {
                let counter = counter_clone.clone();
                Box::pin(async move {
                    let count = counter.fetch_add(1, Ordering::SeqCst);
                    if count < 2 {
                        Err("temporary failure")
                    } else {
                        Ok(42)
                    }
                })
            },
            &config,
            |_| RetryDecision::Retry,
        ).await;
        
        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 3); // Called 3 times total
    }

    #[tokio::test]
    async fn test_max_attempts_exhausted() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();
        
        let config = RetryConfig {
            max_attempts: 2,
            initial_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(10),
            backoff_multiplier: 2.0,
            jitter: false,
        };
        
        let result = retry_with_backoff(
            move || {
                let counter = counter_clone.clone();
                Box::pin(async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Err::<i32, &'static str>("persistent failure")
                })
            },
            &config,
            |_| RetryDecision::Retry,
        ).await;
        
        assert!(result.is_err());
        assert_eq!(counter.load(Ordering::SeqCst), 3); // Initial + 2 retries
    }

    #[tokio::test]
    async fn test_non_retryable_error() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();
        
        let config = RetryConfig::default();
        
        let result = retry_with_backoff(
            move || {
                let counter = counter_clone.clone();
                Box::pin(async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Err::<i32, &'static str>("non-retryable error")
                })
            },
            &config,
            |_| RetryDecision::FailImmediately,
        ).await;
        
        assert!(result.is_err());
        assert_eq!(counter.load(Ordering::SeqCst), 1); // Only called once
    }
}