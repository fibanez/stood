//! Request Batch Processor
//!
//! This module provides request batching capabilities to optimize API call efficiency
//! by grouping multiple requests together when possible.

use super::PerformanceConfig;
use std::{
    collections::VecDeque,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{
    sync::{Mutex, Notify},
    time::timeout,
};
use tracing::{debug, error, info, warn};

/// A batch of requests to be processed together
#[derive(Debug)]
pub struct RequestBatch<T> {
    pub requests: Vec<T>,
    pub batch_id: usize,
    pub created_at: Instant,
}

impl<T> RequestBatch<T> {
    #[allow(dead_code)]
    fn new(requests: Vec<T>, batch_id: usize) -> Self {
        Self {
            requests,
            batch_id,
            created_at: Instant::now(),
        }
    }

    pub fn size(&self) -> usize {
        self.requests.len()
    }

    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }
}

/// Result of batch processing
#[derive(Debug)]
pub struct BatchResult<T, R> {
    pub batch_id: usize,
    pub results: Vec<Result<R, BatchError>>,
    pub processing_duration: Duration,
    pub original_requests: Vec<T>,
}

/// Errors that can occur during batch processing
#[derive(Debug, thiserror::Error)]
pub enum BatchError {
    #[error("Request timeout")]
    Timeout,
    #[error("Processing error: {0}")]
    ProcessingError(String),
    #[error("Batch size limit exceeded")]
    BatchSizeLimitExceeded,
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Trait for types that can be batched together
pub trait Batchable: Send + 'static {
    type Result: Send + 'static;

    /// Check if this request can be batched with others
    fn can_batch_with(&self, other: &Self) -> bool;

    /// Get the priority of this request (higher numbers = higher priority)
    fn priority(&self) -> u32 {
        0
    }

    /// Get the estimated processing cost of this request
    fn estimated_cost(&self) -> u32 {
        1
    }
}

/// Request batch processor that collects and processes requests in batches
pub struct RequestBatchProcessor {
    config: PerformanceConfig,
    pending_requests: Arc<Mutex<VecDeque<PendingRequest>>>,
    batch_notify: Arc<Notify>,
    next_batch_id: Arc<Mutex<usize>>,
    stats: Arc<Mutex<BatchStats>>,
}

/// A pending request waiting to be batched
struct PendingRequest {
    request: Box<dyn std::any::Any + Send>,
    added_at: Instant,
    #[allow(dead_code)]
    priority: u32,
    #[allow(dead_code)]
    estimated_cost: u32,
}

/// Statistics for batch processing
#[derive(Debug, Clone, Default)]
pub struct BatchStats {
    pub total_requests: usize,
    pub total_batches: usize,
    pub average_batch_size: f64,
    pub total_processing_time: Duration,
    pub timeouts: usize,
    pub errors: usize,
}

impl BatchStats {
    pub fn record_batch(&mut self, batch_size: usize, processing_time: Duration) {
        self.total_requests += batch_size;
        self.total_batches += 1;
        self.total_processing_time += processing_time;
        self.average_batch_size = self.total_requests as f64 / self.total_batches as f64;
    }

    pub fn record_timeout(&mut self) {
        self.timeouts += 1;
    }

    pub fn record_error(&mut self) {
        self.errors += 1;
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_batches == 0 {
            1.0
        } else {
            1.0 - ((self.timeouts + self.errors) as f64 / self.total_batches as f64)
        }
    }
}

impl RequestBatchProcessor {
    /// Create a new batch processor
    pub fn new(config: PerformanceConfig) -> Self {
        Self {
            config,
            pending_requests: Arc::new(Mutex::new(VecDeque::new())),
            batch_notify: Arc::new(Notify::new()),
            next_batch_id: Arc::new(Mutex::new(0)),
            stats: Arc::new(Mutex::new(BatchStats::default())),
        }
    }

    /// Add a request to be batched
    pub async fn add_request<T>(&self, request: T) -> Result<(), BatchError>
    where
        T: Batchable + std::any::Any + Send,
    {
        let pending = PendingRequest {
            request: Box::new(request),
            added_at: Instant::now(),
            priority: 0, // Would extract from T if it implements Batchable
            estimated_cost: 1,
        };

        // Use minimal lock scope for adding requests
        self.pending_requests.lock().await.push_back(pending);

        // Notify batch processor that new requests are available
        self.batch_notify.notify_one();

        debug!("Added request to batch processor queue");
        Ok(())
    }

    /// Process requests in batches
    pub async fn process_batches<T, R, F>(&self, mut processor: F) -> Result<(), BatchError>
    where
        T: Batchable + std::any::Any + Send,
        R: Send + 'static,
        F: FnMut(Vec<T>) -> Result<Vec<R>, BatchError> + Send,
    {
        let mut batch_timer = tokio::time::interval(self.config.batch_timeout);

        loop {
            tokio::select! {
                _ = batch_timer.tick() => {
                    self.flush_pending_batch(&mut processor).await?;
                }
                _ = self.batch_notify.notified() => {
                    if self.should_flush_batch().await {
                        self.flush_pending_batch(&mut processor).await?;
                    }
                }
            }
        }
    }

    /// Check if a batch should be flushed
    async fn should_flush_batch(&self) -> bool {
        let requests = self.pending_requests.lock().await;
        let should_flush = requests.len() >= self.config.max_batch_size
            || requests
                .front()
                .is_some_and(|req| req.added_at.elapsed() >= self.config.batch_timeout);

        if should_flush {
            debug!("Should flush batch: {} requests pending", requests.len());
        }

        should_flush
    }

    /// Flush pending requests as a batch
    async fn flush_pending_batch<T, R, F>(&self, processor: &mut F) -> Result<(), BatchError>
    where
        T: Batchable + std::any::Any + Send,
        R: Send + 'static,
        F: FnMut(Vec<T>) -> Result<Vec<R>, BatchError>,
    {
        // Extract requests with minimal lock time and defer type casting
        let raw_requests = {
            let mut pending = self.pending_requests.lock().await;
            if pending.is_empty() {
                return Ok(());
            }

            // Extract up to max_batch_size requests quickly
            let batch_size = pending.len().min(self.config.max_batch_size);
            let mut extracted = Vec::with_capacity(batch_size);

            for _ in 0..batch_size {
                if let Some(pending_req) = pending.pop_front() {
                    extracted.push(pending_req);
                }
            }

            extracted
        };

        // Perform type casting outside of lock to reduce contention
        let mut requests = Vec::with_capacity(raw_requests.len());
        for pending_req in raw_requests {
            if let Ok(request) = pending_req.request.downcast::<T>() {
                requests.push(*request);
            } else {
                warn!("Failed to downcast request to expected type");
            }
        }

        if requests.is_empty() {
            return Ok(());
        }

        let batch_id = {
            let mut id = self.next_batch_id.lock().await;
            *id += 1;
            *id
        };

        let batch_size = requests.len();
        debug!("Processing batch {} with {} requests", batch_id, batch_size);

        let start_time = Instant::now();

        // Process the batch
        let result = timeout(
            Duration::from_secs(30), // Processing timeout
            async { processor(requests) },
        )
        .await;

        let processing_time = start_time.elapsed();

        // Update statistics
        {
            let mut stats = self.stats.lock().await;
            match result {
                Ok(Ok(_)) => {
                    stats.record_batch(batch_size, processing_time);
                    info!(
                        "Successfully processed batch {} ({} requests) in {:?}",
                        batch_id, batch_size, processing_time
                    );
                }
                Ok(Err(_)) => {
                    stats.record_error();
                    error!("Error processing batch {}", batch_id);
                }
                Err(_) => {
                    stats.record_timeout();
                    error!("Timeout processing batch {}", batch_id);
                }
            }
        }

        result.map_err(|_| BatchError::Timeout)??;
        Ok(())
    }

    /// Get current batch processing statistics
    pub async fn stats(&self) -> BatchStats {
        let stats = self.stats.lock().await;
        stats.clone()
    }

    /// Get the number of pending requests
    pub async fn pending_count(&self) -> usize {
        let requests = self.pending_requests.lock().await;
        requests.len()
    }

    /// Clear all pending requests
    pub async fn clear_pending(&self) -> usize {
        let mut requests = self.pending_requests.lock().await;
        let count = requests.len();
        requests.clear();
        count
    }
}

/// Specialized batch processor for common request types
pub struct ToolExecutionBatcher {
    processor: RequestBatchProcessor,
}

impl ToolExecutionBatcher {
    pub fn new(config: PerformanceConfig) -> Self {
        Self {
            processor: RequestBatchProcessor::new(config),
        }
    }

    /// Add a tool execution request to be batched
    pub async fn add_tool_request<T>(&self, request: T) -> Result<(), BatchError>
    where
        T: Batchable + std::any::Any + Send,
    {
        self.processor.add_request(request).await
    }

    /// Process tool execution batches
    pub async fn process_tool_batches<T, R, F>(&self, processor: F) -> Result<(), BatchError>
    where
        T: Batchable + std::any::Any + Send,
        R: Send + 'static,
        F: FnMut(Vec<T>) -> Result<Vec<R>, BatchError> + Send,
    {
        self.processor.process_batches(processor).await
    }

    /// Get statistics
    pub async fn stats(&self) -> BatchStats {
        self.processor.stats().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct TestRequest {
        id: u32,
        #[allow(dead_code)]
        data: String,
    }

    impl Batchable for TestRequest {
        type Result = String;

        fn can_batch_with(&self, _other: &Self) -> bool {
            true
        }

        fn priority(&self) -> u32 {
            self.id
        }
    }

    #[tokio::test]
    async fn test_batch_processor_creation() {
        let config = PerformanceConfig::default();
        let processor = RequestBatchProcessor::new(config);

        assert_eq!(processor.pending_count().await, 0);
    }

    #[tokio::test]
    async fn test_add_request() {
        let config = PerformanceConfig::default();
        let processor = RequestBatchProcessor::new(config);

        let request = TestRequest {
            id: 1,
            data: "test".to_string(),
        };

        let result = processor.add_request(request).await;
        assert!(result.is_ok());
        assert_eq!(processor.pending_count().await, 1);
    }

    #[tokio::test]
    async fn test_batch_stats() {
        let mut stats = BatchStats::default();
        stats.record_batch(5, Duration::from_millis(100));

        assert_eq!(stats.total_requests, 5);
        assert_eq!(stats.total_batches, 1);
        assert_eq!(stats.average_batch_size, 5.0);
        assert_eq!(stats.success_rate(), 1.0);
    }

    #[tokio::test]
    async fn test_tool_execution_batcher() {
        let config = PerformanceConfig::default();
        let batcher = ToolExecutionBatcher::new(config);

        let request = TestRequest {
            id: 1,
            data: "tool test".to_string(),
        };

        let result = batcher.add_tool_request(request).await;
        assert!(result.is_ok());
    }
}
