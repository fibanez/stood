//! Metrics collection trait and implementations for OpenTelemetry integration

use crate::telemetry::metrics::{RequestMetrics, SystemMetrics, TokenMetrics, ToolMetrics};
use opentelemetry::{
    metrics::{Counter, Gauge, Histogram, Meter, UpDownCounter},
    KeyValue,
};
use std::sync::Arc;

/// Core trait for collecting and recording metrics
///
/// Provides a unified interface for recording all types of agent metrics
/// to OpenTelemetry systems including Prometheus, Jaeger, and custom backends.
pub trait MetricsCollector: Send + Sync + std::fmt::Debug {
    /// Record a counter metric (monotonically increasing values)
    fn record_counter(&self, name: &str, value: u64, labels: &[KeyValue]);

    /// Record a histogram metric (latency, duration, size distributions)
    fn record_histogram(&self, name: &str, value: f64, labels: &[KeyValue]);

    /// Record a gauge metric (current state values)
    fn record_gauge(&self, name: &str, value: f64, labels: &[KeyValue]);

    /// Increment a counter by 1
    fn increment(&self, name: &str, labels: &[KeyValue]) {
        self.record_counter(name, 1, labels);
    }

    /// Record comprehensive request metrics
    fn record_request_metrics(&self, metrics: &RequestMetrics) {
        let status = if metrics.success { "success" } else { "error" };
        let labels = &[KeyValue::new("status", status)];

        // Request duration
        self.record_histogram(
            "agent_request_duration",
            metrics.duration.as_secs_f64(),
            labels,
        );

        // Request count
        self.increment("agent_requests_total", labels);

        // Model invocations
        self.record_counter("agent_model_invocations_total", metrics.model_invocations as u64, &[]);

        // Token metrics
        if let Some(ref tokens) = metrics.token_metrics {
            self.record_token_metrics(tokens);
        }

        // Error metrics
        if let Some(ref error_type) = metrics.error_type {
            self.increment("agent_errors_total", &[
                KeyValue::new("error_type", error_type.clone()),
                KeyValue::new("component", "agent"),
            ]);
        }
    }

    /// Record token usage metrics
    fn record_token_metrics(&self, metrics: &TokenMetrics) {
        self.record_counter("agent_tokens_input_total", metrics.input_tokens, &[]);
        self.record_counter("agent_tokens_output_total", metrics.output_tokens, &[]);
        self.record_counter("agent_tokens_total", metrics.total_tokens, &[]);
        self.record_histogram("agent_tokens_per_request", metrics.total_tokens as f64, &[]);
    }

    /// Record tool execution metrics
    fn record_tool_metrics(&self, metrics: &ToolMetrics) {
        let status = if metrics.success { "success" } else { "error" };
        let labels = &[
            KeyValue::new("tool_name", metrics.tool_name.clone()),
            KeyValue::new("status", status),
        ];

        // Tool execution duration
        self.record_histogram(
            "agent_tool_execution_duration",
            metrics.duration.as_secs_f64(),
            labels,
        );

        // Tool call count
        self.increment("agent_tool_calls_total", labels);

        // Retry attempts
        if metrics.retry_attempts > 0 {
            self.record_counter(
                "agent_tool_retry_attempts",
                metrics.retry_attempts as u64,
                &[KeyValue::new("tool_name", metrics.tool_name.clone())],
            );
        }

        // Tool errors
        if let Some(ref error_type) = metrics.error_type {
            self.increment("agent_tool_errors", &[
                KeyValue::new("tool_name", metrics.tool_name.clone()),
                KeyValue::new("error_type", error_type.clone()),
            ]);
        }
    }

    /// Record system resource metrics
    fn record_system_metrics(&self, metrics: &SystemMetrics) {
        self.record_gauge("agent_memory_usage_bytes", metrics.memory_usage_bytes as f64, &[]);
        self.record_gauge("agent_connection_pool_active", metrics.active_connections as f64, &[]);
        self.record_gauge("agent_concurrent_requests", metrics.concurrent_requests as f64, &[]);
        self.record_gauge("agent_thread_pool_utilization", metrics.thread_utilization, &[]);
    }
}

/// OpenTelemetry-based metrics collector implementation
#[derive(Clone, Debug)]
pub struct OtelMetricsCollector {
    meter: Meter,
    
    // Cached instruments for performance
    token_input_counter: Counter<u64>,
    token_output_counter: Counter<u64>,
    token_total_counter: Counter<u64>,
    request_duration_histogram: Histogram<f64>,
    tool_duration_histogram: Histogram<f64>,
    requests_counter: Counter<u64>,
    tool_calls_counter: Counter<u64>,
    errors_counter: Counter<u64>,
    
    memory_gauge: Gauge<f64>,
    connections_gauge: Gauge<f64>,
    concurrent_requests_gauge: UpDownCounter<i64>,
}

impl OtelMetricsCollector {
    /// Create a new OpenTelemetry metrics collector
    pub fn new(meter: Meter) -> crate::Result<Self> {
        // Create all metric instruments
        let token_input_counter = meter
            .u64_counter("agent_tokens_input_total")
            .with_description("Total input tokens consumed by the agent")
            .with_unit("tokens")
            .init();

        let token_output_counter = meter
            .u64_counter("agent_tokens_output_total") 
            .with_description("Total output tokens generated by the agent")
            .with_unit("tokens")
            .init();

        let token_total_counter = meter
            .u64_counter("agent_tokens_total")
            .with_description("Total tokens (input + output) processed")
            .with_unit("tokens")
            .init();

        let request_duration_histogram = meter
            .f64_histogram("agent_request_duration")
            .with_description("Agent request processing duration")
            .with_unit("s")
            .init();

        let tool_duration_histogram = meter
            .f64_histogram("agent_tool_execution_duration")
            .with_description("Tool execution duration")
            .with_unit("s")
            .init();

        let requests_counter = meter
            .u64_counter("agent_requests_total")
            .with_description("Total agent requests processed")
            .init();

        let tool_calls_counter = meter
            .u64_counter("agent_tool_calls_total")
            .with_description("Total tool calls executed")
            .init();

        let errors_counter = meter
            .u64_counter("agent_errors_total")
            .with_description("Total errors encountered")
            .init();

        let memory_gauge = meter
            .f64_gauge("agent_memory_usage_bytes")
            .with_description("Current memory usage in bytes")
            .with_unit("bytes")
            .init();

        let connections_gauge = meter
            .f64_gauge("agent_connection_pool_active")
            .with_description("Active connections in pool")
            .init();

        let concurrent_requests_gauge = meter
            .i64_up_down_counter("agent_concurrent_requests")
            .with_description("Current number of concurrent requests")
            .init();

        Ok(Self {
            meter,
            token_input_counter,
            token_output_counter,
            token_total_counter,
            request_duration_histogram,
            tool_duration_histogram,
            requests_counter,
            tool_calls_counter,
            errors_counter,
            memory_gauge,
            connections_gauge,
            concurrent_requests_gauge,
        })
    }
}

impl MetricsCollector for OtelMetricsCollector {
    fn record_counter(&self, name: &str, value: u64, labels: &[KeyValue]) {
        match name {
            "agent_tokens_input_total" => self.token_input_counter.add(value, labels),
            "agent_tokens_output_total" => self.token_output_counter.add(value, labels),
            "agent_tokens_total" => self.token_total_counter.add(value, labels),
            "agent_requests_total" => self.requests_counter.add(value, labels),
            "agent_tool_calls_total" => self.tool_calls_counter.add(value, labels),
            "agent_errors_total" => self.errors_counter.add(value, labels),
            _ => {
                // Create ad-hoc counter for unknown metrics
                let counter = self.meter.u64_counter(name.to_string()).init();
                counter.add(value, labels);
            }
        }
    }

    fn record_histogram(&self, name: &str, value: f64, labels: &[KeyValue]) {
        match name {
            "agent_request_duration" => self.request_duration_histogram.record(value, labels),
            "agent_tool_execution_duration" => self.tool_duration_histogram.record(value, labels),
            "agent_tokens_per_request" => {
                // Create histogram for token distribution
                let histogram = self.meter.f64_histogram("agent_tokens_per_request").init();
                histogram.record(value, labels);
            }
            _ => {
                // Create ad-hoc histogram for unknown metrics
                let histogram = self.meter.f64_histogram(name.to_string()).init();
                histogram.record(value, labels);
            }
        }
    }

    fn record_gauge(&self, name: &str, value: f64, labels: &[KeyValue]) {
        match name {
            "agent_memory_usage_bytes" => self.memory_gauge.record(value, labels),
            "agent_connection_pool_active" => self.connections_gauge.record(value, labels),
            "agent_concurrent_requests" => self.concurrent_requests_gauge.add(value as i64, labels),
            _ => {
                // Create ad-hoc gauge for unknown metrics
                let gauge = self.meter.f64_gauge(name.to_string()).init();
                gauge.record(value, labels);
            }
        }
    }
}

/// No-op metrics collector for when telemetry is disabled
#[derive(Default, Clone, Debug)]
pub struct NoOpMetricsCollector;

impl MetricsCollector for NoOpMetricsCollector {
    fn record_counter(&self, _name: &str, _value: u64, _labels: &[KeyValue]) {}
    fn record_histogram(&self, _name: &str, _value: f64, _labels: &[KeyValue]) {}
    fn record_gauge(&self, _name: &str, _value: f64, _labels: &[KeyValue]) {}
}

/// Shared metrics collector instance
pub type SharedMetricsCollector = Arc<dyn MetricsCollector>;