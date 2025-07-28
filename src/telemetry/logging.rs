//! Comprehensive logging and tracing system for performance diagnostics
//!
//! This module provides file-based logging, detailed tracing with exact timings,
//! and comprehensive event loop monitoring to diagnose performance issues.

use crate::StoodError;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, trace, warn};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    fmt::{format::FmtSpan, MakeWriter},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Layer,
};
use serde_json::{Map, Value};
use uuid::Uuid;
use std::io::{self, Write};

/// Custom writer that reorders JSON fields to put source location first
struct SourceLocationFirstWriter<W> {
    inner: W,
}

impl<W> SourceLocationFirstWriter<W> {
    fn new(writer: W) -> Self {
        Self { inner: writer }
    }
}

impl<W: Write> Write for SourceLocationFirstWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // Try to parse as JSON and reorder fields
        if let Ok(s) = std::str::from_utf8(buf) {
            // Only process lines that look like JSON (start with '{' and end with '}')
            let trimmed = s.trim();
            if trimmed.starts_with('{') && trimmed.ends_with('}') {
                if let Ok(mut json) = serde_json::from_str::<Map<String, Value>>(trimmed) {
                    // Create a new ordered map with timestamp first, then source location
                    let mut ordered = Map::new();
                    
                    // Add timestamp first
                    if let Some(timestamp) = json.remove("timestamp") {
                        ordered.insert("timestamp".to_string(), timestamp);
                    }
                    
                    // Add source location fields next (in specific order)
                    if let Some(filename) = json.remove("filename") {
                        ordered.insert("filename".to_string(), filename);
                    }
                    if let Some(line_number) = json.remove("line_number") {
                        ordered.insert("line_number".to_string(), line_number);
                    }
                    if let Some(target) = json.remove("target") {
                        ordered.insert("target".to_string(), target);
                    }
                    
                    // Add level
                    if let Some(level) = json.remove("level") {
                        ordered.insert("level".to_string(), level);
                    }
                    
                    // Add all remaining fields
                    for (key, value) in json {
                        ordered.insert(key, value);
                    }
                    
                    // Write the reordered JSON with newline
                    let reordered_json = serde_json::to_string(&ordered)
                        .unwrap_or_else(|_| trimmed.to_string());
                    let output = format!("{}\n", reordered_json);
                    return self.inner.write(output.as_bytes());
                }
            }
        }
        
        // Fallback to original content if parsing fails
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl<'a, W> MakeWriter<'a> for SourceLocationFirstWriter<W>
where
    W: MakeWriter<'a>,
{
    type Writer = SourceLocationFirstWriter<W::Writer>;

    fn make_writer(&'a self) -> Self::Writer {
        SourceLocationFirstWriter::new(self.inner.make_writer())
    }
}

/// Configuration for the comprehensive logging system
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    /// Base directory for log files
    pub log_dir: PathBuf,
    /// Maximum log file size before rotation (in bytes)
    pub max_file_size: u64,
    /// Number of log files to keep in rotation
    pub max_files: usize,
    /// Log level for file output
    pub file_log_level: String,
    /// Log level for console output (if enabled)
    pub console_log_level: String,
    /// Whether to enable console logging
    pub console_enabled: bool,
    /// Whether to enable JSON format for structured logging
    pub json_format: bool,
    /// Whether to enable detailed performance tracing
    pub enable_performance_tracing: bool,
    /// Whether to enable event loop cycle detection
    pub enable_cycle_detection: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            log_dir: PathBuf::from("."),
            max_file_size: 100 * 1024 * 1024, // 100MB
            max_files: 5,
            file_log_level: "INFO".to_string(),
            console_log_level: "WARN".to_string(),
            console_enabled: true,
            json_format: true,
            enable_performance_tracing: true,
            enable_cycle_detection: true,
        }
    }
}

impl LoggingConfig {
    /// Create logging configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(log_dir) = std::env::var("STOOD_LOG_DIR") {
            config.log_dir = PathBuf::from(log_dir);
        }

        if let Ok(max_size) = std::env::var("STOOD_LOG_MAX_SIZE") {
            if let Ok(size) = max_size.parse() {
                config.max_file_size = size;
            }
        }

        if let Ok(max_files) = std::env::var("STOOD_LOG_MAX_FILES") {
            if let Ok(files) = max_files.parse() {
                config.max_files = files;
            }
        }

        if let Ok(level) = std::env::var("STOOD_FILE_LOG_LEVEL") {
            config.file_log_level = level;
        }

        if let Ok(level) = std::env::var("STOOD_CONSOLE_LOG_LEVEL") {
            config.console_log_level = level;
        }

        if let Ok(enabled) = std::env::var("STOOD_CONSOLE_LOGGING") {
            config.console_enabled = enabled.parse().unwrap_or(true);
        }

        if let Ok(json) = std::env::var("STOOD_JSON_LOGS") {
            config.json_format = json.parse().unwrap_or(true);
        }

        if let Ok(perf) = std::env::var("STOOD_PERFORMANCE_TRACING") {
            config.enable_performance_tracing = perf.parse().unwrap_or(true);
        }

        if let Ok(cycle) = std::env::var("STOOD_CYCLE_DETECTION") {
            config.enable_cycle_detection = cycle.parse().unwrap_or(true);
        }

        config
    }
}

/// Guard that must be kept alive for the duration of the application
/// to ensure proper log flushing
pub struct LoggingGuard {
    _file_guard: Option<WorkerGuard>,
}

/// Initialize the comprehensive logging system
pub fn init_logging(config: LoggingConfig) -> Result<LoggingGuard, StoodError> {
    // Ensure log directory exists
    std::fs::create_dir_all(&config.log_dir).map_err(|e| {
        StoodError::configuration_error(format!("Failed to create log directory: {}", e))
    })?;

    // File logging layer with JSON format
    let file_appender = tracing_appender::rolling::daily(&config.log_dir, "stood.log");
    let (file_writer, file_guard) = tracing_appender::non_blocking(file_appender);

    // Create filter that excludes key events and other noise
    let base_filter = EnvFilter::try_new(&config.file_log_level)
        .unwrap_or_else(|_| EnvFilter::new("info"));
    
    let file_filter = base_filter
        .add_directive("rustyline=off".parse().unwrap())  // Exclude rustyline key events
        .add_directive("crossterm=off".parse().unwrap())  // Exclude terminal events
        .add_directive("termion=off".parse().unwrap())    // Exclude terminal events
        .add_directive("tui=off".parse().unwrap())        // Exclude TUI events
        .add_directive("winit=off".parse().unwrap())      // Exclude window events
        .add_directive("wgpu=off".parse().unwrap())       // Exclude GPU events
        .add_directive("tokio=warn".parse().unwrap())     // Reduce tokio noise
        .add_directive("hyper=warn".parse().unwrap())     // Reduce HTTP noise
        .add_directive("h2=warn".parse().unwrap())        // Reduce HTTP/2 noise
        .add_directive("reqwest=warn".parse().unwrap())   // Reduce reqwest noise
        .add_directive("aws_smithy=warn".parse().unwrap()) // Reduce AWS SDK noise
        .add_directive("aws_config=warn".parse().unwrap()) // Reduce AWS config noise
        .add_directive("aws_credential_types=warn".parse().unwrap()); // Reduce AWS credential noise

    let subscriber_builder = tracing_subscriber::registry();

    if config.json_format {
        // Use custom writer that reorders JSON fields
        let custom_writer = SourceLocationFirstWriter::new(file_writer);
        
        let file_layer = tracing_subscriber::fmt::layer()
            .with_writer(custom_writer)
            .with_ansi(false)
            .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
            .with_timer(tracing_subscriber::fmt::time::ChronoUtc::new(
                "%Y-%m-%d %H:%M:%S%.3f UTC".to_string(),
            ))
            .with_file(true)      // Include filename
            .with_line_number(true) // Include line number
            .with_target(true)    // Include module path (acts as function context)
            .json()
            .with_current_span(true)
            .with_span_list(true)
            .with_filter(file_filter);

        if config.console_enabled {
            let console_filter = EnvFilter::try_new(&config.console_log_level)
                .unwrap_or_else(|_| EnvFilter::new("warn"))
                .add_directive("rustyline=off".parse().unwrap())  // Exclude rustyline from console too
                .add_directive("crossterm=off".parse().unwrap())  // Exclude terminal events
                .add_directive("tokio=warn".parse().unwrap())     // Reduce tokio noise
                .add_directive("aws_smithy=warn".parse().unwrap()); // Reduce AWS SDK noise
                
            let console_layer = tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr)
                .with_ansi(true)
                .with_span_events(FmtSpan::NONE)
                .with_file(true)      // Include filename in console too
                .with_line_number(true) // Include line number in console too
                .with_target(true)    // Include module path
                .with_filter(console_filter);
            subscriber_builder.with(file_layer).with(console_layer).init();
        } else {
            subscriber_builder.with(file_layer).init();
        }
    } else {
        let file_layer = tracing_subscriber::fmt::layer()
            .with_writer(file_writer)
            .with_ansi(false)
            .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
            .with_timer(tracing_subscriber::fmt::time::ChronoUtc::new(
                "%Y-%m-%d %H:%M:%S%.3f UTC".to_string(),
            ))
            .with_file(true)      // Include filename
            .with_line_number(true) // Include line number
            .with_target(true)    // Include module path
            .with_filter(file_filter);

        if config.console_enabled {
            let console_filter = EnvFilter::try_new(&config.console_log_level)
                .unwrap_or_else(|_| EnvFilter::new("warn"))
                .add_directive("rustyline=off".parse().unwrap())  // Exclude rustyline from console too
                .add_directive("crossterm=off".parse().unwrap())  // Exclude terminal events
                .add_directive("tokio=warn".parse().unwrap())     // Reduce tokio noise
                .add_directive("aws_smithy=warn".parse().unwrap()); // Reduce AWS SDK noise
                
            let console_layer = tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr)
                .with_ansi(true)
                .with_span_events(FmtSpan::NONE)
                .with_file(true)      // Include filename
                .with_line_number(true) // Include line number
                .with_target(true)    // Include module path
                .with_filter(console_filter);
            subscriber_builder.with(file_layer).with(console_layer).init();
        } else {
            subscriber_builder.with(file_layer).init();
        }
    }

    info!(
        log_dir = %config.log_dir.display(),
        json_format = config.json_format,
        performance_tracing = config.enable_performance_tracing,
        cycle_detection = config.enable_cycle_detection,
        "Comprehensive logging system initialized"
    );

    Ok(LoggingGuard {
        _file_guard: Some(file_guard),
    })
}

/// Performance tracer for detecting bottlenecks and loops
#[derive(Debug, Clone)]
pub struct PerformanceTracer {
    session_id: Uuid,
    model_interaction_counter: Arc<AtomicU64>,
    last_cycle_times: Arc<std::sync::Mutex<std::collections::VecDeque<(Instant, String)>>>,
    operation_stack: Arc<std::sync::Mutex<Vec<(String, Instant)>>>,
}

impl PerformanceTracer {
    /// Create a new performance tracer
    pub fn new() -> Self {
        let session_id = Uuid::new_v4();
        info!(session_id = %session_id, "Performance tracing session started");

        Self {
            session_id,
            model_interaction_counter: Arc::new(AtomicU64::new(0)),
            last_cycle_times: Arc::new(std::sync::Mutex::new(
                std::collections::VecDeque::with_capacity(100),
            )),
            operation_stack: Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    /// Start tracking an operation
    pub fn start_operation(&self, operation: &str) -> OperationGuard {
        let start_time = Instant::now();
        let operation_id = Uuid::new_v4();

        // Push to operation stack
        if let Ok(mut stack) = self.operation_stack.lock() {
            stack.push((operation.to_string(), start_time));
        }

        debug!(
            operation_id = %operation_id,
            operation = %operation,
            start_time = ?start_time,
            stack_depth = self.get_stack_depth(),
            "Operation started"
        );

        OperationGuard {
            tracer: self.clone(),
            operation: operation.to_string(),
            operation_id,
            start_time,
        }
    }

    /// Record a model interaction cycle
    pub fn record_cycle(&self, cycle_type: &str, duration: Duration) {
        let model_interaction_num = self.model_interaction_counter.fetch_add(1, Ordering::SeqCst);
        let now = Instant::now();

        // Add to recent cycles for loop detection
        if let Ok(mut cycles) = self.last_cycle_times.lock() {
            cycles.push_back((now, cycle_type.to_string()));
            if cycles.len() > 100 {
                cycles.pop_front();
            }

            // Check for potential infinite loops
            self.check_for_loops(&cycles, cycle_type);
        }

        info!(
            model_interaction_number = model_interaction_num,
            cycle_type = %cycle_type,
            duration_ms = duration.as_millis(),
            duration_micros = duration.as_micros(),
            "Model interaction cycle completed"
        );

        // Log slow cycles
        if duration > Duration::from_millis(1000) {
            warn!(
                model_interaction_number = model_interaction_num,
                cycle_type = %cycle_type,
                duration_ms = duration.as_millis(),
                "Slow model interaction cycle detected"
            );
        }

        // Log extremely slow cycles
        if duration > Duration::from_millis(5000) {
            error!(
                model_interaction_number = model_interaction_num,
                cycle_type = %cycle_type,
                duration_ms = duration.as_millis(),
                "Extremely slow model interaction cycle detected - possible hang"
            );
        }
    }

    /// Record waiting for a specific resource or condition
    pub fn record_waiting(&self, waiting_for: &str, duration: Duration) {
        debug!(
            waiting_for = %waiting_for,
            duration_ms = duration.as_millis(),
            duration_micros = duration.as_micros(),
            stack_depth = self.get_stack_depth(),
            "Waiting for resource/condition"
        );

        // Log long waits
        if duration > Duration::from_millis(500) {
            warn!(
                waiting_for = %waiting_for,
                duration_ms = duration.as_millis(),
                "Long wait detected"
            );
        }

        // Log extremely long waits
        if duration > Duration::from_millis(2000) {
            error!(
                waiting_for = %waiting_for,
                duration_ms = duration.as_millis(),
                "Extremely long wait detected - possible deadlock"
            );
        }
    }

    /// Record a blocking operation
    pub fn record_blocking(&self, operation: &str, duration: Duration) {
        warn!(
            operation = %operation,
            duration_ms = duration.as_millis(),
            duration_micros = duration.as_micros(),
            stack_depth = self.get_stack_depth(),
            "Blocking operation detected"
        );

        // Log critical blocking operations
        if duration > Duration::from_millis(1000) {
            error!(
                operation = %operation,
                duration_ms = duration.as_millis(),
                "Critical blocking operation - significantly impacting performance"
            );
        }
    }

    /// Check for potential infinite loops in cycles
    fn check_for_loops(&self, cycles: &std::collections::VecDeque<(Instant, String)>, current_type: &str) {
        if cycles.len() < 10 {
            return;
        }

        // Check if the last 5 cycles are all the same type within a short time window
        let recent_cycles: Vec<_> = cycles.iter().rev().take(5).collect();
        let all_same_type = recent_cycles
            .iter()
            .all(|(_, cycle_type)| cycle_type == current_type);

        if all_same_type {
            let time_span = recent_cycles[0].0.duration_since(recent_cycles[4].0);
            if time_span < Duration::from_millis(100) {
                error!(
                    cycle_type = %current_type,
                    time_span_ms = time_span.as_millis(),
                    "Potential infinite loop detected - same cycle type repeated rapidly"
                );
            }
        }

        // Check for rapid cycling between types
        let type_changes = cycles
            .iter()
            .rev()
            .take(10)
            .collect::<Vec<_>>()
            .windows(2)
            .filter(|pair| pair[0].1 != pair[1].1)
            .count();

        if type_changes > 8 {
            warn!(
                type_changes = type_changes,
                "Rapid cycle type changes detected - possible event loop instability"
            );
        }
    }

    /// Get current operation stack depth
    fn get_stack_depth(&self) -> usize {
        self.operation_stack.lock().map(|stack| stack.len()).unwrap_or(0)
    }

    /// Get session statistics
    pub fn get_session_stats(&self) -> SessionStats {
        let model_interaction_count = self.model_interaction_counter.load(Ordering::SeqCst);
        let stack_depth = self.get_stack_depth();

        SessionStats {
            session_id: self.session_id,
            total_cycles: model_interaction_count,
            current_stack_depth: stack_depth,
        }
    }
}

impl Default for PerformanceTracer {
    fn default() -> Self {
        Self::new()
    }
}

/// Guard for tracking operation duration
pub struct OperationGuard {
    tracer: PerformanceTracer,
    operation: String,
    operation_id: Uuid,
    start_time: Instant,
}

impl OperationGuard {
    /// Add context information to the operation
    pub fn add_context(&self, key: &str, value: &str) {
        trace!(
            operation_id = %self.operation_id,
            operation = %self.operation,
            context_key = %key,
            context_value = %value,
            "Operation context added"
        );
    }

    /// Record a checkpoint within the operation
    pub fn checkpoint(&self, checkpoint: &str) {
        let elapsed = self.start_time.elapsed();
        debug!(
            operation_id = %self.operation_id,
            operation = %self.operation,
            checkpoint = %checkpoint,
            elapsed_ms = elapsed.as_millis(),
            elapsed_micros = elapsed.as_micros(),
            "Operation checkpoint"
        );
    }

    /// Record waiting within the operation
    pub fn record_wait(&self, waiting_for: &str, duration: Duration) {
        self.tracer.record_waiting(waiting_for, duration);
    }

    /// Record blocking within the operation
    pub fn record_blocking(&self, description: &str, duration: Duration) {
        self.tracer.record_blocking(description, duration);
    }
}

impl Drop for OperationGuard {
    fn drop(&mut self) {
        let duration = self.start_time.elapsed();

        // Pop from operation stack
        if let Ok(mut stack) = self.tracer.operation_stack.lock() {
            if let Some((op, _)) = stack.pop() {
                if op != self.operation {
                    warn!(
                        expected = %self.operation,
                        actual = %op,
                        "Operation stack mismatch detected"
                    );
                }
            }
        }

        debug!(
            operation_id = %self.operation_id,
            operation = %self.operation,
            duration_ms = duration.as_millis(),
            duration_micros = duration.as_micros(),
            "Operation completed"
        );

        // Log slow operations
        if duration > Duration::from_millis(100) {
            warn!(
                operation_id = %self.operation_id,
                operation = %self.operation,
                duration_ms = duration.as_millis(),
                "Slow operation detected"
            );
        }

        // Log extremely slow operations
        if duration > Duration::from_millis(1000) {
            warn!(
                operation_id = %self.operation_id,
                operation = %self.operation,
                duration_ms = duration.as_millis(),
                "Extremely slow operation detected"
            );
        }
    }
}

/// Session statistics for monitoring
#[derive(Debug, Clone)]
pub struct SessionStats {
    pub session_id: Uuid,
    /// Total number of model interaction cycles completed
    pub total_cycles: u64,
    pub current_stack_depth: usize,
}

/// Macro for instrumenting functions with performance tracing
#[macro_export]
macro_rules! trace_performance {
    ($tracer:expr, $operation:expr, $block:block) => {{
        let _guard = $tracer.start_operation($operation);
        $block
    }};
}

/// Macro for timing and logging wait operations
#[macro_export]
macro_rules! trace_wait {
    ($tracer:expr, $waiting_for:expr, $block:block) => {{
        let start = std::time::Instant::now();
        let result = $block;
        let duration = start.elapsed();
        $tracer.record_waiting($waiting_for, duration);
        result
    }};
}

/// Macro for timing and logging blocking operations
#[macro_export]
macro_rules! trace_blocking {
    ($tracer:expr, $operation:expr, $block:block) => {{
        let start = std::time::Instant::now();
        let result = $block;
        let duration = start.elapsed();
        $tracer.record_blocking($operation, duration);
        result
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_logging_config_default() {
        let config = LoggingConfig::default();
        assert_eq!(config.log_dir, PathBuf::from("."));
        assert_eq!(config.max_file_size, 100 * 1024 * 1024);
        assert_eq!(config.max_files, 5);
        assert!(config.console_enabled);
        assert!(config.json_format);
    }

    #[test]
    fn test_performance_tracer() {
        let tracer = PerformanceTracer::new();
        
        {
            let _guard = tracer.start_operation("test_operation");
            thread::sleep(Duration::from_millis(10));
        }

        tracer.record_cycle("test_cycle", Duration::from_millis(50));
        tracer.record_waiting("test_resource", Duration::from_millis(25));

        let stats = tracer.get_session_stats();
        assert_eq!(stats.total_cycles, 1);
        assert_eq!(stats.current_stack_depth, 0);
    }

    #[test]
    fn test_operation_guard() {
        let tracer = PerformanceTracer::new();
        
        let guard = tracer.start_operation("test_operation");
        guard.add_context("key", "value");
        guard.checkpoint("midpoint");
        guard.record_wait("network", Duration::from_millis(5));
        guard.record_blocking("disk_io", Duration::from_millis(2));
        
        // Guard will be dropped here, triggering the completion log
    }
}