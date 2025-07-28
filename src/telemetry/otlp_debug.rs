//! OTLP Debug Logging System
//!
//! This module provides comprehensive logging for all OpenTelemetry exports
//! to help debug telemetry data flow issues. All OTLP exports are logged to
//! ~/.local/share/stood-telemetry/logs/ with detailed payload information.

use chrono::Utc;
use serde_json::json;
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use tracing::{error, info, warn};

/// OTLP Debug Logger for tracking all telemetry exports
pub struct OtlpDebugLogger {
    log_file_path: PathBuf,
    enabled: bool,
}

impl OtlpDebugLogger {
    /// Initialize the OTLP debug logger
    pub fn new() -> Self {
        let log_dir = Self::get_log_directory();
        let log_file_path = log_dir.join("otlp_exports.jsonl");
        
        // Create log directory if it doesn't exist
        let enabled = match create_dir_all(&log_dir) {
            Ok(_) => {
                info!("📁 OTLP debug logging enabled: {:?}", log_file_path);
                true
            }
            Err(e) => {
                warn!("❌ Failed to create OTLP log directory: {}", e);
                false
            }
        };

        Self {
            log_file_path,
            enabled,
        }
    }

    /// Get the log directory path (~/.local/share/stood-telemetry/logs/)
    fn get_log_directory() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(home).join(".local/share/stood-telemetry/logs")
    }

    /// Log an OTLP export attempt with full details
    pub fn log_export(
        &self,
        module: &str,
        export_type: OtlpExportType,
        endpoint: &str,
        payload_summary: &str,
        payload_size_bytes: usize,
        result: &Result<(), String>,
    ) {
        if !self.enabled {
            return;
        }

        let timestamp = Utc::now();
        let log_entry = json!({
            "timestamp": timestamp.to_rfc3339(),
            "module": module,
            "export_type": export_type.as_str(),
            "endpoint": endpoint,
            "payload_summary": payload_summary,
            "payload_size_bytes": payload_size_bytes,
            "success": result.is_ok(),
            "error": result.as_ref().err(),
            "thread_id": format!("{:?}", std::thread::current().id()),
        });

        // Write to log file
        if let Err(e) = self.write_log_entry(&log_entry) {
            error!("Failed to write OTLP debug log: {}", e);
        }

        // Also log to tracing for immediate visibility
        match result {
            Ok(_) => {
                info!(
                    "📤 OTLP Export [{}]: {} to {} ({} bytes) - SUCCESS",
                    module, export_type.as_str(), endpoint, payload_size_bytes
                );
            }
            Err(e) => {
                // Check if telemetry was explicitly requested via environment variable
                let explicit_telemetry = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").is_ok() 
                    || std::env::var("OTEL_ENABLED").map(|v| v.to_lowercase()) == Ok("true".to_string());
                
                if explicit_telemetry {
                    error!(
                        "📤 OTLP Export [{}]: {} to {} ({} bytes) - FAILED: {}",
                        module, export_type.as_str(), endpoint, payload_size_bytes, e
                    );
                } else {
                    warn!(
                        "📤 OTLP Export [{}]: {} to {} ({} bytes) - FAILED: {}",
                        module, export_type.as_str(), endpoint, payload_size_bytes, e
                    );
                }
            }
        }
    }

    /// Log telemetry initialization attempts
    pub fn log_telemetry_init(
        &self,
        module: &str,
        config: &crate::telemetry::TelemetryConfig,
        result: &Result<bool, String>,
    ) {
        if !self.enabled {
            return;
        }

        let timestamp = Utc::now();
        let log_entry = json!({
            "timestamp": timestamp.to_rfc3339(),
            "event_type": "telemetry_init",
            "module": module,
            "config": {
                "enabled": config.enabled,
                "otlp_endpoint": config.otlp_endpoint,
                "console_export": config.console_export,
                "service_name": config.service_name,
                "batch_processor": config.enable_batch_processor,
            },
            "success": result.is_ok(),
            "tracer_created": result.as_ref().unwrap_or(&false),
            "error": result.as_ref().err(),
        });

        if let Err(e) = self.write_log_entry(&log_entry) {
            error!("Failed to write telemetry init log: {}", e);
        }

        match result {
            Ok(true) => {
                info!("🚀 Telemetry Init [{}]: SUCCESS - Tracer created", module);
            }
            Ok(false) => {
                warn!("⚠️ Telemetry Init [{}]: SUCCESS - But no tracer (disabled)", module);
            }
            Err(e) => {
                error!("❌ Telemetry Init [{}]: FAILED - {}", module, e);
            }
        }
    }

    /// Log span creation and operations
    pub fn log_span_operation(
        &self,
        module: &str,
        operation: SpanOperation,
        span_name: &str,
        attributes: Option<&serde_json::Value>,
    ) {
        if !self.enabled {
            return;
        }

        let timestamp = Utc::now();
        let log_entry = json!({
            "timestamp": timestamp.to_rfc3339(),
            "event_type": "span_operation",
            "module": module,
            "operation": operation.as_str(),
            "span_name": span_name,
            "attributes": attributes,
        });

        if let Err(e) = self.write_log_entry(&log_entry) {
            error!("Failed to write span operation log: {}", e);
        }

        info!(
            "🔄 Span [{}]: {} - {}",
            module, operation.as_str(), span_name
        );
    }

    /// Write a log entry to the file
    fn write_log_entry(&self, entry: &serde_json::Value) -> Result<(), std::io::Error> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file_path)?;

        writeln!(file, "{}", entry)?;
        file.flush()?;
        Ok(())
    }

    /// Get the current log file path for external access
    pub fn log_file_path(&self) -> &PathBuf {
        &self.log_file_path
    }
}

/// Types of OTLP exports
#[derive(Debug, Clone)]
pub enum OtlpExportType {
    Traces,
    Metrics,
    Logs,
}

impl OtlpExportType {
    fn as_str(&self) -> &'static str {
        match self {
            OtlpExportType::Traces => "traces",
            OtlpExportType::Metrics => "metrics", 
            OtlpExportType::Logs => "logs",
        }
    }
}

/// Span operations being logged
#[derive(Debug, Clone)]
pub enum SpanOperation {
    Start,
    SetAttribute,
    AddEvent,
    SetStatus,
    Finish,
}

impl SpanOperation {
    fn as_str(&self) -> &'static str {
        match self {
            SpanOperation::Start => "start",
            SpanOperation::SetAttribute => "set_attribute",
            SpanOperation::AddEvent => "add_event",
            SpanOperation::SetStatus => "set_status",
            SpanOperation::Finish => "finish",
        }
    }
}

/// Global OTLP debug logger instance
static OTLP_LOGGER: std::sync::LazyLock<Mutex<OtlpDebugLogger>> = std::sync::LazyLock::new(|| {
    Mutex::new(OtlpDebugLogger::new())
});

/// Log an OTLP export globally
pub fn log_otlp_export(
    module: &str,
    export_type: OtlpExportType,
    endpoint: &str,
    payload_summary: &str,
    payload_size_bytes: usize,
    result: &Result<(), String>,
) {
    if let Ok(logger) = OTLP_LOGGER.lock() {
        logger.log_export(module, export_type, endpoint, payload_summary, payload_size_bytes, result);
    }
}

/// Log telemetry initialization globally
pub fn log_telemetry_init(
    module: &str,
    config: &crate::telemetry::TelemetryConfig,
    result: &Result<bool, String>,
) {
    if let Ok(logger) = OTLP_LOGGER.lock() {
        logger.log_telemetry_init(module, config, result);
    }
}

/// Log span operations globally
pub fn log_span_operation(
    module: &str,
    operation: SpanOperation,
    span_name: &str,
    attributes: Option<&serde_json::Value>,
) {
    if let Ok(logger) = OTLP_LOGGER.lock() {
        logger.log_span_operation(module, operation, span_name, attributes);
    }
}

/// Get the current log file path
pub fn get_log_file_path() -> PathBuf {
    if let Ok(logger) = OTLP_LOGGER.lock() {
        logger.log_file_path().clone()
    } else {
        PathBuf::from("/tmp/otlp_exports.jsonl")
    }
}