//! Event batching system for high-frequency callback optimization
//!
//! This module provides batching capabilities for callback events to improve
//! performance when dealing with high-frequency events like content deltas
//! during streaming.

use super::*;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Notify};
use tokio::time::sleep;

/// Configuration for event batching
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// Maximum number of events in a batch
    pub max_batch_size: usize,
    /// Maximum time to wait before flushing a batch
    pub max_batch_delay: Duration,
    /// Whether to enable batching for content delta events
    pub batch_content_deltas: bool,
    /// Whether to enable batching for tool events
    pub batch_tool_events: bool,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 10,
            max_batch_delay: Duration::from_millis(50), // 50ms max delay
            batch_content_deltas: true,
            batch_tool_events: false, // Tool events are typically less frequent
        }
    }
}

/// A batch of callback events
#[derive(Debug, Clone)]
pub struct EventBatch {
    pub events: Vec<CallbackEvent>,
    pub created_at: Instant,
}

impl Default for EventBatch {
    fn default() -> Self {
        Self {
            events: Vec::new(),
            created_at: Instant::now(),
        }
    }
}

impl EventBatch {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_event(&mut self, event: CallbackEvent) {
        self.events.push(event);
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }
}

/// Batching callback handler that accumulates events and flushes them in batches
pub struct BatchingCallbackHandler {
    inner_handler: Arc<dyn CallbackHandler>,
    config: BatchConfig,
    batch: Arc<Mutex<EventBatch>>,
    flush_notifier: Arc<Notify>,
    _flush_task: tokio::task::JoinHandle<()>,
}

impl BatchingCallbackHandler {
    /// Create a new batching callback handler
    pub fn new(inner_handler: Arc<dyn CallbackHandler>, config: BatchConfig) -> Self {
        let batch = Arc::new(Mutex::new(EventBatch::new()));
        let flush_notifier = Arc::new(Notify::new());

        // Spawn background task for periodic flushing
        let flush_task = Self::spawn_flush_task(
            Arc::clone(&batch),
            Arc::clone(&flush_notifier),
            Arc::clone(&inner_handler),
            config.max_batch_delay,
        );

        Self {
            inner_handler,
            config,
            batch,
            flush_notifier,
            _flush_task: flush_task,
        }
    }

    /// Create a batching handler with default configuration
    pub fn with_defaults(inner_handler: Arc<dyn CallbackHandler>) -> Self {
        Self::new(inner_handler, BatchConfig::default())
    }

    /// Spawn background task for periodic batch flushing
    fn spawn_flush_task(
        batch: Arc<Mutex<EventBatch>>,
        flush_notifier: Arc<Notify>,
        handler: Arc<dyn CallbackHandler>,
        max_delay: Duration,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            loop {
                // Wait for either a notification or timeout
                tokio::select! {
                    _ = flush_notifier.notified() => {
                        // Immediate flush requested
                    }
                    _ = sleep(max_delay) => {
                        // Timeout-based flush
                    }
                }

                // Flush the batch
                let events_to_flush = {
                    let mut batch_guard = batch.lock().await;
                    if batch_guard.is_empty() {
                        continue;
                    }

                    let events = batch_guard.events.clone();
                    *batch_guard = EventBatch::new(); // Reset batch
                    events
                };

                // Send all events to the inner handler
                for event in events_to_flush {
                    if let Err(e) = handler.handle_event(event).await {
                        tracing::warn!("Batch flush failed for event: {}", e);
                    }
                }
            }
        })
    }

    /// Determine if an event should be batched
    fn should_batch_event(&self, event: &CallbackEvent) -> bool {
        match event {
            CallbackEvent::ContentDelta { .. } => self.config.batch_content_deltas,
            CallbackEvent::ToolStart { .. } | CallbackEvent::ToolComplete { .. } => {
                self.config.batch_tool_events
            }
            // Don't batch critical events like errors or completion
            CallbackEvent::Error { .. }
            | CallbackEvent::EventLoopComplete { .. }
            | CallbackEvent::EventLoopStart { .. } => false,
            // Default to not batching for other events
            _ => false,
        }
    }

    /// Force flush the current batch
    pub async fn flush(&self) -> Result<(), CallbackError> {
        let events_to_flush = {
            let mut batch_guard = self.batch.lock().await;
            if batch_guard.is_empty() {
                return Ok(());
            }

            let events = batch_guard.events.clone();
            *batch_guard = EventBatch::new();
            events
        };

        // Send all events to the inner handler
        for event in events_to_flush {
            self.inner_handler.handle_event(event).await?;
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl CallbackHandler for BatchingCallbackHandler {
    async fn handle_event(&self, event: CallbackEvent) -> Result<(), CallbackError> {
        if self.should_batch_event(&event) {
            let should_flush = {
                let mut batch_guard = self.batch.lock().await;
                batch_guard.add_event(event);

                // Check if we should flush based on size or age
                batch_guard.len() >= self.config.max_batch_size
                    || batch_guard.age() >= self.config.max_batch_delay
            };

            if should_flush {
                self.flush_notifier.notify_one();
            }

            Ok(())
        } else {
            // Send non-batchable events immediately
            self.inner_handler.handle_event(event).await
        }
    }
}

impl Drop for BatchingCallbackHandler {
    fn drop(&mut self) {
        // Abort the flush task when the handler is dropped
        self._flush_task.abort();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Test callback handler that counts events
    #[derive(Debug)]
    struct CountingCallbackHandler {
        count: Arc<AtomicUsize>,
    }

    impl CountingCallbackHandler {
        fn new() -> Self {
            Self {
                count: Arc::new(AtomicUsize::new(0)),
            }
        }

        fn get_count(&self) -> usize {
            self.count.load(Ordering::Relaxed)
        }
    }

    #[async_trait::async_trait]
    impl CallbackHandler for CountingCallbackHandler {
        async fn handle_event(&self, _event: CallbackEvent) -> Result<(), CallbackError> {
            self.count.fetch_add(1, Ordering::Relaxed);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_batching_content_deltas() {
        let inner = Arc::new(CountingCallbackHandler::new());
        let config = BatchConfig {
            max_batch_size: 3,
            max_batch_delay: Duration::from_millis(100),
            batch_content_deltas: true,
            batch_tool_events: false,
        };

        let batching_handler =
            BatchingCallbackHandler::new(inner.clone() as Arc<dyn CallbackHandler>, config);

        // Send some content delta events
        for i in 0..5 {
            let event = CallbackEvent::ContentDelta {
                delta: format!("chunk {}", i),
                complete: false,
                reasoning: false,
            };
            batching_handler.handle_event(event).await.unwrap();
        }

        // Should have triggered a batch flush after 3 events
        // Wait a bit for processing
        tokio::time::sleep(Duration::from_millis(10)).await;
        assert!(inner.get_count() >= 3);

        // Flush remaining events
        batching_handler.flush().await.unwrap();
        tokio::time::sleep(Duration::from_millis(10)).await;
        assert_eq!(inner.get_count(), 5);
    }

    #[tokio::test]
    async fn test_non_batchable_events() {
        let inner = Arc::new(CountingCallbackHandler::new());
        let config = BatchConfig::default();
        let batching_handler =
            BatchingCallbackHandler::new(inner.clone() as Arc<dyn CallbackHandler>, config);

        // Send an error event (should not be batched)
        let event = CallbackEvent::Error {
            error: crate::StoodError::model_error("test error".to_string()),
            context: "test".to_string(),
        };
        batching_handler.handle_event(event).await.unwrap();

        // Should be processed immediately
        tokio::time::sleep(Duration::from_millis(10)).await;
        assert_eq!(inner.get_count(), 1);
    }
}
