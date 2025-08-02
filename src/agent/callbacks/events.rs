//! Event types for the callback system.
//!
//! This module defines all the events that can be emitted during agent execution,
//! providing comprehensive coverage of the agentic workflow.

use uuid::Uuid;
use std::time::Duration;
use serde_json::Value;
use crate::agent::event_loop::{EventLoopConfig, EventLoopResult};
use crate::types::{Messages, StopReason};
use crate::llm::traits::ProviderType;
use crate::error::StoodError;
use chrono::{DateTime, Utc};

/// Comprehensive event types covering all Python callback scenarios
#[derive(Debug, Clone)]
pub enum CallbackEvent {
    // Initialization Events (matches Python's init_event_loop=True)
    EventLoopStart {
        loop_id: Uuid,
        prompt: String,
        config: EventLoopConfig,
    },
    CycleStart {
        cycle_id: Uuid,
        cycle_number: u32,
    },
    
    // Model Events (matches Python's model interaction patterns)
    ModelStart {
        provider: ProviderType,
        model_id: String,
        messages: Messages,
        tools_available: usize,
        /// Raw JSON request sent to provider API (if capture enabled)
        raw_request_json: Option<String>,
    },
    ModelComplete {
        response: String,
        stop_reason: StopReason,
        duration: Duration,
        tokens: Option<TokenUsage>,
        /// Complete raw response data from provider API (if capture enabled)
        raw_response_data: Option<RawResponseData>,
    },
    
    // Streaming Events (matches Python's data/delta/reasoning kwargs)
    ContentDelta {
        delta: String,
        complete: bool,
        reasoning: bool, // Matches Python's reasoningText kwarg
    },
    
    // Tool Events (matches Python's current_tool_use kwarg)
    ToolStart {
        tool_name: String,
        tool_use_id: String,
        input: Value,
    },
    ToolComplete {
        tool_name: String,
        tool_use_id: String,
        output: Option<Value>,
        error: Option<String>,
        duration: Duration,
    },
    
    // Parallel Execution Events
    ParallelStart {
        tool_count: usize,
        max_parallel: usize,
    },
    ParallelProgress {
        completed: usize,
        total: usize,
        running: usize,
    },
    ParallelComplete {
        total_duration: Duration,
        success_count: usize,
        failure_count: usize,
    },
    
    // Completion Events (matches Python's complete=True)
    EventLoopComplete {
        result: EventLoopResult,
        total_duration: Duration,
    },
    
    // Error Events (matches Python's force_stop=True)
    Error {
        error: StoodError,
        context: String,
    },
    
    // Evaluation Events
    EvaluationStart {
        strategy: String,
        prompt: String,
        cycle_number: u32,
    },
    EvaluationComplete {
        strategy: String,
        decision: bool, // true = continue, false = stop
        reasoning: String,
        duration: Duration,
    },
}

/// Tool-specific events for easier handling
#[derive(Debug, Clone)]
pub enum ToolEvent {
    Started { name: String, input: Value },
    Completed { name: String, output: Option<Value>, duration: Duration },
    Failed { name: String, error: String, duration: Duration },
}

/// Token usage information (matches Python callback patterns)
#[derive(Debug, Clone)]
pub struct TokenUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
}

/// Container for all raw response information from provider APIs
#[derive(Debug, Clone)]
pub struct RawResponseData {
    /// Type of response (streaming vs non-streaming)
    pub response_type: ResponseType,
    /// Raw JSON for non-streaming responses
    pub non_streaming_json: Option<String>,
    /// All SSE events for streaming responses
    pub streaming_events: Option<Vec<SSEEvent>>,
    /// Provider-specific metadata
    pub raw_metadata: std::collections::HashMap<String, Value>,
}

/// Type of response from provider
#[derive(Debug, Clone)]
pub enum ResponseType {
    NonStreaming,
    Streaming,
}

/// Single Server-Sent Event from streaming responses
#[derive(Debug, Clone)]
pub struct SSEEvent {
    /// When this event was received
    pub timestamp: DateTime<Utc>,
    /// Raw JSON content of the SSE chunk
    pub raw_json: String,
    /// SSE event type if available (e.g., "message_start", "content_block_delta")
    pub event_type: Option<String>,
}