//! OpenTelemetry semantic conventions for Stood agent metrics
//!
//! This module defines standardized metric names and attributes following
//! OpenTelemetry semantic conventions for AI/ML and agent workloads.

/// Core Agent Performance Metrics
pub mod agent {
    // Token Usage Metrics
    pub const TOKENS_INPUT_TOTAL: &str = "agent.tokens.input.total";
    pub const TOKENS_OUTPUT_TOTAL: &str = "agent.tokens.output.total"; 
    pub const TOKENS_TOTAL: &str = "agent.tokens.total";
    pub const TOKENS_PER_REQUEST: &str = "agent.tokens.per_request";
    pub const TOKEN_COST_ESTIMATE: &str = "agent.tokens.cost_estimate";

    // Request & Invocation Metrics
    pub const REQUESTS_TOTAL: &str = "agent.requests.total";
    pub const MODEL_INVOCATIONS_TOTAL: &str = "agent.model.invocations.total";
    pub const CYCLES_TOTAL: &str = "agent.cycles.total";
    pub const REQUEST_DURATION: &str = "agent.request.duration";
    pub const MODEL_INVOCATION_DURATION: &str = "agent.model.invocation.duration";
}

/// Tool Execution Metrics
pub mod tool {
    // Tool Performance
    pub const CALLS_TOTAL: &str = "agent.tool.calls.total";
    pub const EXECUTION_DURATION: &str = "agent.tool.execution.duration";
    pub const PARALLEL_EXECUTIONS: &str = "agent.tool.parallel_executions";
    pub const QUEUE_DEPTH: &str = "agent.tool.queue_depth";
    pub const TIMEOUT_RATE: &str = "agent.tool.timeout_rate";

    // Tool Success/Failure Tracking
    pub const SUCCESS_RATE: &str = "agent.tool.success_rate";
    pub const RETRY_ATTEMPTS: &str = "agent.tool.retry_attempts";
    pub const VALIDATION_FAILURES: &str = "agent.tool.validation_failures";
}

/// Agentic Reasoning Metrics
pub mod reasoning {
    // Decision Making & Flow Control
    pub const REASONING_CYCLES_PER_REQUEST: &str = "agent.reasoning.cycles_per_request";
    pub const CONVERSATION_TURNS: &str = "agent.conversation.turns";
    pub const CONTEXT_LENGTH: &str = "agent.context.length";
    pub const PLANNING_DURATION: &str = "agent.planning.duration";
    pub const REFLECTION_CYCLES: &str = "agent.reflection.cycles";

    // Model Selection & Strategy
    pub const MODEL_SWITCHES: &str = "agent.model.switches";
    pub const TEMPERATURE_ADJUSTMENTS: &str = "agent.temperature.adjustments";
    pub const MAX_TOKENS_HIT: &str = "agent.max_tokens.hit";
    pub const STREAMING_VS_BATCH: &str = "agent.streaming_vs_batch";
}

/// System Resource Metrics
pub mod system {
    // Memory & Performance
    pub const MEMORY_USAGE_BYTES: &str = "agent.memory.usage_bytes";
    pub const CONNECTION_POOL_ACTIVE: &str = "agent.connection_pool.active";
    pub const CONNECTION_POOL_IDLE: &str = "agent.connection_pool.idle";
    pub const BATCH_PROCESSING_EFFICIENCY: &str = "agent.batch_processing.efficiency";

    // Concurrency & Threading
    pub const CONCURRENT_REQUESTS: &str = "agent.concurrent_requests";
    pub const THREAD_POOL_UTILIZATION: &str = "agent.thread_pool.utilization";
    pub const ASYNC_TASKS_ACTIVE: &str = "agent.async_tasks.active";
}

/// Quality & Reliability Metrics
pub mod quality {
    // Error Tracking
    pub const ERRORS_TOTAL: &str = "agent.errors.total";
    pub const MODEL_ERRORS: &str = "agent.model.errors";
    pub const TOOL_ERRORS: &str = "agent.tool.errors";
    pub const TIMEOUT_ERRORS: &str = "agent.timeout.errors";
    pub const RATE_LIMIT_HITS: &str = "agent.rate_limit.hits";

    // Health & Availability
    pub const HEALTH_SCORE: &str = "agent.health.score";
    pub const UPTIME_SECONDS: &str = "agent.uptime.seconds";
    pub const CIRCUIT_BREAKER_STATE: &str = "agent.circuit_breaker.state";
    pub const DEGRADED_PERFORMANCE_EVENTS: &str = "agent.degraded_performance.events";
}

/// Business & Usage Metrics
pub mod business {
    // User Experience
    pub const RESPONSE_QUALITY_SCORE: &str = "agent.response.quality_score";
    pub const USER_SATISFACTION: &str = "agent.user.satisfaction";
    pub const TASK_COMPLETION_RATE: &str = "agent.task.completion_rate";
    pub const RESPONSE_RELEVANCE: &str = "agent.response.relevance";

    // Cost & Efficiency
    pub const COST_PER_REQUEST: &str = "agent.cost.per_request";
    pub const TOKENS_PER_DOLLAR: &str = "agent.tokens.per_dollar";
    pub const RESOURCE_UTILIZATION: &str = "agent.resource.utilization";
}

/// Standard metric attributes and labels
pub mod attributes {
    pub const STATUS: &str = "status";
    pub const ERROR_TYPE: &str = "error_type";
    pub const COMPONENT: &str = "component";
    pub const TOOL_NAME: &str = "tool_name";
    pub const MODEL_NAME: &str = "model_name";
    pub const REQUEST_ID: &str = "request_id";
    pub const AGENT_ID: &str = "agent_id";
    pub const USER_ID: &str = "user_id";
    pub const SESSION_ID: &str = "session_id";
    pub const OPERATION: &str = "operation";

    // Status values
    pub const STATUS_SUCCESS: &str = "success";
    pub const STATUS_ERROR: &str = "error";
    pub const STATUS_TIMEOUT: &str = "timeout";
    pub const STATUS_RETRYING: &str = "retrying";

    // Component values
    pub const COMPONENT_AGENT: &str = "agent";
    pub const COMPONENT_TOOL: &str = "tool";
    pub const COMPONENT_MODEL: &str = "model";
    pub const COMPONENT_SYSTEM: &str = "system";

    // Error type values
    pub const ERROR_MODEL: &str = "model_error";
    pub const ERROR_TOOL: &str = "tool_error";
    pub const ERROR_CONFIGURATION: &str = "configuration_error";
    pub const ERROR_INVALID_INPUT: &str = "invalid_input";
    pub const ERROR_RATE_LIMIT: &str = "rate_limit";
    pub const ERROR_TIMEOUT: &str = "timeout";
    pub const ERROR_AUTH: &str = "auth_error";
    pub const ERROR_NETWORK: &str = "network_error";
}

/// Metric units following OpenTelemetry conventions
pub mod units {
    pub const TOKENS: &str = "tokens";
    pub const SECONDS: &str = "s";
    pub const MILLISECONDS: &str = "ms";
    pub const BYTES: &str = "bytes";
    pub const MEGABYTES: &str = "MB";
    pub const PERCENT: &str = "%";
    pub const RATIO: &str = "1";
    pub const COUNT: &str = "1";
    pub const DOLLARS: &str = "USD";
}