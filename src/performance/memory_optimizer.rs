//! Memory Optimization Module
//!
//! This module provides memory optimization strategies for the Stood agent library,
//! including conversation context management and resource cleanup.

use super::PerformanceConfig;
use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use tracing::debug;

/// Memory optimization strategies
#[derive(Debug, Clone, Copy)]
pub enum OptimizationStrategy {
    /// Sliding window approach - keep only recent messages
    SlidingWindow,
    /// Importance-based filtering - keep important messages
    ImportanceBased,
    /// Hybrid approach combining multiple strategies
    Hybrid,
}

/// Message importance scoring
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MessageImportance {
    Critical = 5,
    High = 4,
    Medium = 3,
    Low = 2,
    Minimal = 1,
}

/// A message in the conversation context
#[derive(Debug, Clone)]
pub struct ContextMessage {
    pub id: String,
    pub content: String,
    pub role: String,
    pub timestamp: Instant,
    pub importance: MessageImportance,
    pub size_bytes: usize,
    pub is_tool_related: bool,
}

impl ContextMessage {
    pub fn new(id: String, content: String, role: String) -> Self {
        let size_bytes = content.len() + role.len() + id.len();
        Self {
            id,
            content,
            role,
            timestamp: Instant::now(),
            importance: MessageImportance::Medium,
            size_bytes,
            is_tool_related: false,
        }
    }

    pub fn with_importance(mut self, importance: MessageImportance) -> Self {
        self.importance = importance;
        self
    }

    pub fn with_tool_relation(mut self, is_tool_related: bool) -> Self {
        self.is_tool_related = is_tool_related;
        self
    }

    pub fn age(&self) -> Duration {
        self.timestamp.elapsed()
    }

    pub fn importance_score(&self) -> f64 {
        let base_score = self.importance as u8 as f64;
        let age_factor = (-(self.age().as_secs() as f64) / 3600.0).exp(); // Decay over hours
        let tool_bonus = if self.is_tool_related { 1.5 } else { 1.0 };

        base_score * age_factor * tool_bonus
    }
}

/// Conversation context manager for memory optimization
pub struct ConversationContext {
    messages: VecDeque<ContextMessage>,
    max_size_bytes: usize,
    max_messages: usize,
    current_size_bytes: usize,
    strategy: OptimizationStrategy,
}

impl ConversationContext {
    pub fn new(max_size_bytes: usize, max_messages: usize, strategy: OptimizationStrategy) -> Self {
        Self {
            messages: VecDeque::new(),
            max_size_bytes,
            max_messages,
            current_size_bytes: 0,
            strategy,
        }
    }

    /// Add a message to the context
    pub fn add_message(&mut self, message: ContextMessage) {
        self.current_size_bytes += message.size_bytes;
        self.messages.push_back(message);

        // Optimize if necessary
        if self.needs_optimization() {
            self.optimize();
        }
    }

    /// Check if optimization is needed
    pub fn needs_optimization(&self) -> bool {
        self.current_size_bytes > self.max_size_bytes || self.messages.len() > self.max_messages
    }

    /// Optimize the conversation context
    pub fn optimize(&mut self) -> usize {
        let initial_size = self.current_size_bytes;

        match self.strategy {
            OptimizationStrategy::SlidingWindow => self.sliding_window_optimize(),
            OptimizationStrategy::ImportanceBased => self.importance_based_optimize(),
            OptimizationStrategy::Hybrid => self.hybrid_optimize(),
        }

        let freed = initial_size.saturating_sub(self.current_size_bytes);
        if freed > 0 {
            debug!(
                "Conversation optimization freed {} bytes using {:?} strategy",
                freed, self.strategy
            );
        }

        freed
    }

    /// Sliding window optimization - remove oldest messages
    fn sliding_window_optimize(&mut self) {
        while (self.current_size_bytes > self.max_size_bytes
            || self.messages.len() > self.max_messages)
            && !self.messages.is_empty()
        {
            if let Some(message) = self.messages.pop_front() {
                self.current_size_bytes =
                    self.current_size_bytes.saturating_sub(message.size_bytes);
            }
        }
    }

    /// Importance-based optimization - remove least important messages
    fn importance_based_optimize(&mut self) {
        let mut messages: Vec<_> = self.messages.drain(..).collect();

        // Sort by importance score (ascending, so lowest scores are first)
        messages.sort_by(|a, b| {
            a.importance_score()
                .partial_cmp(&b.importance_score())
                .unwrap()
        });

        self.current_size_bytes = 0;
        self.messages.clear();

        // Add back messages starting with highest importance
        for message in messages.into_iter().rev() {
            if self.current_size_bytes + message.size_bytes <= self.max_size_bytes
                && self.messages.len() < self.max_messages
            {
                self.current_size_bytes += message.size_bytes;
                self.messages.push_back(message);
            }
        }
    }

    /// Hybrid optimization - combine strategies
    fn hybrid_optimize(&mut self) {
        // First, try importance-based filtering
        self.importance_based_optimize();

        // If still over limits, apply sliding window
        if self.needs_optimization() {
            self.sliding_window_optimize();
        }
    }

    /// Get current memory usage in bytes
    pub fn memory_usage(&self) -> usize {
        self.current_size_bytes
    }

    /// Get number of messages
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    /// Get utilization percentage
    pub fn utilization_percent(&self) -> f64 {
        (self.current_size_bytes as f64 / self.max_size_bytes as f64) * 100.0
    }
}

/// Memory optimizer for the entire application
pub struct MemoryOptimizer {
    #[allow(dead_code)]
    config: PerformanceConfig,
    conversations: Arc<RwLock<HashMap<String, ConversationContext>>>,
    memory_pools: Arc<RwLock<Vec<MemoryPool>>>,
    last_optimization: Instant,
    total_freed: usize,
}

/// Memory pool for reusable buffers
pub struct MemoryPool {
    name: String,
    buffers: VecDeque<Vec<u8>>,
    max_buffers: usize,
    buffer_size: usize,
    hits: usize,
    misses: usize,
}

impl MemoryPool {
    pub fn new(name: String, max_buffers: usize, buffer_size: usize) -> Self {
        Self {
            name,
            buffers: VecDeque::new(),
            max_buffers,
            buffer_size,
            hits: 0,
            misses: 0,
        }
    }

    /// Get a buffer from the pool
    pub fn get_buffer(&mut self) -> Vec<u8> {
        if let Some(mut buffer) = self.buffers.pop_front() {
            buffer.clear();
            buffer.reserve(self.buffer_size);
            self.hits += 1;
            buffer
        } else {
            self.misses += 1;
            Vec::with_capacity(self.buffer_size)
        }
    }

    /// Return a buffer to the pool
    pub fn return_buffer(&mut self, buffer: Vec<u8>) {
        if self.buffers.len() < self.max_buffers && buffer.capacity() >= self.buffer_size {
            self.buffers.push_back(buffer);
        }
    }

    /// Get hit rate
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    /// Get pool stats
    pub fn stats(&self) -> MemoryPoolStats {
        MemoryPoolStats {
            name: self.name.clone(),
            available_buffers: self.buffers.len(),
            max_buffers: self.max_buffers,
            buffer_size: self.buffer_size,
            hit_rate: self.hit_rate(),
            total_requests: self.hits + self.misses,
        }
    }
}

/// Memory pool statistics
#[derive(Debug, Clone)]
pub struct MemoryPoolStats {
    pub name: String,
    pub available_buffers: usize,
    pub max_buffers: usize,
    pub buffer_size: usize,
    pub hit_rate: f64,
    pub total_requests: usize,
}

impl MemoryOptimizer {
    pub fn new(config: PerformanceConfig) -> Self {
        let optimizer = Self {
            config,
            conversations: Arc::new(RwLock::new(HashMap::new())),
            memory_pools: Arc::new(RwLock::new(Vec::new())),
            last_optimization: Instant::now(),
            total_freed: 0,
        };

        // Initialize default memory pools in background
        let pools = optimizer.memory_pools.clone();
        tokio::spawn(async move {
            let mut pools = pools.write().await;

            // Small buffer pool (1KB)
            pools.push(MemoryPool::new("small".to_string(), 50, 1024));
            // Medium buffer pool (32KB)
            pools.push(MemoryPool::new("medium".to_string(), 20, 32 * 1024));
            // Large buffer pool (512KB)
            pools.push(MemoryPool::new("large".to_string(), 5, 512 * 1024));
        });

        optimizer
    }

    /// Optimize memory usage across all components
    pub async fn optimize(&mut self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let start_time = Instant::now();
        let mut total_freed = 0;

        // Optimize conversations with reduced lock contention
        let conversations_to_optimize = {
            let conversations = self.conversations.read().await;
            conversations
                .iter()
                .filter_map(|(id, context)| {
                    if context.needs_optimization() {
                        Some(id.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        };

        // Optimize each conversation individually with short write locks
        for conversation_id in conversations_to_optimize {
            if let Some(mut context) = {
                let mut conversations = self.conversations.write().await;
                conversations.remove(&conversation_id)
            } {
                // Optimize outside of lock
                let freed = context.optimize();
                total_freed += freed;
                debug!(
                    "Optimized conversation {}: freed {} bytes",
                    conversation_id, freed
                );

                // Put back if still has content
                if context.message_count() > 0 {
                    let mut conversations = self.conversations.write().await;
                    conversations.insert(conversation_id, context);
                }
            }
        }

        // Clean up empty conversations in a single pass
        {
            let mut conversations = self.conversations.write().await;
            let initial_count = conversations.len();
            conversations.retain(|_, context| context.message_count() > 0);
            let removed = initial_count - conversations.len();

            if removed > 0 {
                debug!("Removed {} empty conversations", removed);
            }
        }

        // Optimize memory pools
        {
            let mut pools = self.memory_pools.write().await;
            for pool in pools.iter_mut() {
                // Remove excess buffers if pool is over capacity
                while pool.buffers.len() > pool.max_buffers {
                    if let Some(buffer) = pool.buffers.pop_back() {
                        total_freed += buffer.capacity();
                    }
                }
            }
        }

        self.last_optimization = Instant::now();
        self.total_freed += total_freed;

        let optimization_time = start_time.elapsed();
        debug!(
            "Memory optimization completed in {:?}, freed {} bytes",
            optimization_time, total_freed
        );

        Ok(total_freed)
    }

    /// Add a conversation context
    pub async fn add_conversation(&self, id: String, context: ConversationContext) {
        let mut conversations = self.conversations.write().await;
        conversations.insert(id, context);
    }

    /// Get memory usage statistics
    pub async fn memory_stats(&self) -> MemoryStats {
        let conversations = self.conversations.read().await;
        let pools = self.memory_pools.read().await;

        let total_conversation_memory: usize =
            conversations.values().map(|c| c.memory_usage()).sum();

        let total_conversations = conversations.len();
        let pool_stats: Vec<_> = pools.iter().map(|p| p.stats()).collect();

        MemoryStats {
            total_conversation_memory,
            total_conversations,
            total_freed: self.total_freed,
            last_optimization: self.last_optimization,
            pool_stats,
        }
    }

    /// Get a buffer from the appropriate memory pool
    pub async fn get_buffer(&self, size_hint: usize) -> Vec<u8> {
        let mut pools = self.memory_pools.write().await;

        // Find the appropriate pool based on size hint
        for pool in pools.iter_mut() {
            if size_hint <= pool.buffer_size {
                return pool.get_buffer();
            }
        }

        // If no suitable pool found, allocate directly
        Vec::with_capacity(size_hint)
    }

    /// Return a buffer to the appropriate memory pool
    pub async fn return_buffer(&self, buffer: Vec<u8>) {
        let mut pools = self.memory_pools.write().await;
        let capacity = buffer.capacity();

        // Find the appropriate pool
        for pool in pools.iter_mut() {
            if capacity >= pool.buffer_size && capacity < pool.buffer_size * 2 {
                pool.return_buffer(buffer);
                return;
            }
        }

        // Buffer doesn't fit any pool, let it be dropped
    }

    /// Force garbage collection (platform-specific)
    pub async fn force_gc(&self) -> usize {
        // In Rust, we don't have explicit GC, but we can trigger drop operations
        // This is more of a placeholder for potential future optimizations
        0
    }
}

/// Overall memory statistics
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub total_conversation_memory: usize,
    pub total_conversations: usize,
    pub total_freed: usize,
    pub last_optimization: Instant,
    pub pool_stats: Vec<MemoryPoolStats>,
}

impl MemoryStats {
    pub fn average_conversation_size(&self) -> f64 {
        if self.total_conversations == 0 {
            0.0
        } else {
            self.total_conversation_memory as f64 / self.total_conversations as f64
        }
    }

    pub fn time_since_optimization(&self) -> Duration {
        self.last_optimization.elapsed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_message_creation() {
        let message = ContextMessage::new(
            "test-id".to_string(),
            "Hello world".to_string(),
            "user".to_string(),
        );

        assert_eq!(message.role, "user");
        assert_eq!(message.content, "Hello world");
        assert!(message.size_bytes > 0);
    }

    #[test]
    fn test_conversation_context() {
        let mut context = ConversationContext::new(1000, 10, OptimizationStrategy::SlidingWindow);

        let message = ContextMessage::new(
            "1".to_string(),
            "Test message".to_string(),
            "user".to_string(),
        );

        context.add_message(message);
        assert_eq!(context.message_count(), 1);
        assert!(context.memory_usage() > 0);
    }

    #[test]
    fn test_memory_pool() {
        let mut pool = MemoryPool::new("test".to_string(), 5, 1024);

        let buffer = pool.get_buffer();
        assert!(buffer.capacity() >= 1024);

        pool.return_buffer(buffer);
        assert_eq!(pool.buffers.len(), 1);
    }

    #[tokio::test]
    async fn test_memory_optimizer() {
        let config = PerformanceConfig::default();
        let mut optimizer = MemoryOptimizer::new(config);

        let result = optimizer.optimize().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_importance_scoring() {
        let message = ContextMessage::new(
            "test".to_string(),
            "content".to_string(),
            "user".to_string(),
        )
        .with_importance(MessageImportance::High)
        .with_tool_relation(true);

        let score = message.importance_score();
        assert!(score > 0.0);
    }
}
