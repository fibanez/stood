//! Test harness for unified trace context management in tests
//!
//! This module provides utilities for integration tests to enable telemetry
//! with unified trace context, ensuring tests create properly connected traces
//! rather than fragmented spans.

use opentelemetry::Context;
use std::collections::HashMap;
use uuid::Uuid;

use super::{otel::StoodTracer, TelemetryConfig};
use crate::StoodError;

/// Test harness for managing unified trace context across test scenarios
pub struct TelemetryTestHarness {
    tracer: Option<StoodTracer>,
    test_trace_id: String,
    root_context: Context,
}

impl TelemetryTestHarness {
    /// Create a new test harness with unified trace context
    pub fn new(test_name: &str) -> Result<Self, StoodError> {
        let test_trace_id = format!("test-{}-{}", test_name, Uuid::new_v4().to_string()[..8].to_string());
        
        // Create test-specific telemetry configuration
        let config = TelemetryConfig {
            enabled: true,
            console_export: true,
            service_name: format!("stood-test-{}", test_name),
            service_version: env!("CARGO_PKG_VERSION").to_string(),
            enable_batch_processor: false, // Use simple processing for tests
            export_mode: "simple".to_string(),
            service_attributes: {
                let mut attrs = HashMap::new();
                attrs.insert("test.name".to_string(), test_name.to_string());
                attrs.insert("test.trace_id".to_string(), test_trace_id.clone());
                attrs.insert("test.environment".to_string(), "integration".to_string());
                attrs
            },
            enable_debug_tracing: true,
            ..TelemetryConfig::default()
        };

        // Initialize tracer with test configuration
        let tracer = StoodTracer::init(config)?;
        
        if let Some(tracer_instance) = tracer {
            // Create root test span to establish unified trace context
            let mut root_span = tracer_instance.start_agent_span(&format!("integration_test.{}", test_name));
            root_span.set_attribute("test.framework", "tokio");
            root_span.set_attribute("test.type", "integration");
            root_span.set_attribute("test.trace_id", test_trace_id.clone());
            
            // Get the context from the root span for propagation
            // We need to work with the current context since we can't clone BoxedSpan easily
            let root_context = Context::current();
            
            Ok(Self {
                tracer: Some(tracer_instance),
                test_trace_id,
                root_context,
            })
        } else {
            // If telemetry is disabled, create a harness with no-op behavior
            Ok(Self {
                tracer: None,
                test_trace_id,
                root_context: Context::current(),
            })
        }
    }

    /// Get the tracer for instrumentation
    pub fn tracer(&self) -> Option<&StoodTracer> {
        self.tracer.as_ref()
    }

    /// Get the unified trace context for test operations
    pub fn trace_context(&self) -> &Context {
        &self.root_context
    }

    /// Get the test trace ID for correlation
    pub fn trace_id(&self) -> &str {
        &self.test_trace_id
    }

    /// Create a scoped context for a test section
    pub fn scoped_context(&self, section_name: &str) -> Context {
        if let Some(ref tracer) = self.tracer {
            let mut section_span = tracer.start_agent_span(&format!("test_section.{}", section_name));
            section_span.set_attribute("test.section", section_name.to_string());
            section_span.set_attribute("test.parent_trace_id", self.test_trace_id.clone());
            
            // Return current context (the section span is already active)
            Context::current()
        } else {
            self.root_context.clone()
        }
    }

    /// Create telemetry configuration for Agent builders that inherits test context
    pub fn agent_telemetry_config(&self) -> TelemetryConfig {
        if self.tracer.is_some() {
            TelemetryConfig {
                enabled: true,
                console_export: true,
                service_name: format!("test-agent-{}", self.test_trace_id),
                service_version: env!("CARGO_PKG_VERSION").to_string(),
                enable_batch_processor: false,
                export_mode: "simple".to_string(),
                service_attributes: {
                    let mut attrs = HashMap::new();
                    attrs.insert("test.agent".to_string(), "true".to_string());
                    attrs.insert("test.trace_id".to_string(), self.test_trace_id.clone());
                    attrs
                },
                enable_debug_tracing: true,
                ..TelemetryConfig::default()
            }
        } else {
            TelemetryConfig {
                enabled: false,
                ..TelemetryConfig::default()
            }
        }
    }

    /// Helper to create test assertion span
    pub fn assert_span(&self, assertion_name: &str) -> Option<crate::telemetry::otel::StoodSpan> {
        self.tracer.as_ref().map(|tracer| {
            let mut span = tracer.start_agent_span(&format!("test_assertion.{}", assertion_name));
            span.set_attribute("test.assertion", assertion_name.to_string());
            span.set_attribute("test.trace_id", self.test_trace_id.clone());
            span
        })
    }

    /// Complete the test and shutdown telemetry
    pub fn shutdown(mut self) {
        if let Some(tracer) = self.tracer.take() {
            // Create test completion span
            let mut completion_span = tracer.start_agent_span("test_completion");
            completion_span.set_attribute("test.result", "completed");
            completion_span.set_attribute("test.trace_id", self.test_trace_id.clone());
            completion_span.set_success();
            completion_span.finish();
            
            // Shutdown the tracer
            tracer.shutdown();
        }
    }
}

impl Drop for TelemetryTestHarness {
    fn drop(&mut self) {
        // Ensure cleanup if shutdown wasn't called explicitly
        if let Some(tracer) = self.tracer.take() {
            tracer.shutdown();
        }
    }
}

/// Builder for creating test harnesses with specific configurations
pub struct TelemetryTestHarnessBuilder {
    test_name: String,
    enable_telemetry: bool,
    console_export: bool,
    custom_attributes: HashMap<String, String>,
}

impl TelemetryTestHarnessBuilder {
    /// Create a new test harness builder
    pub fn new(test_name: &str) -> Self {
        Self {
            test_name: test_name.to_string(),
            enable_telemetry: true,
            console_export: true,
            custom_attributes: HashMap::new(),
        }
    }

    /// Enable or disable telemetry for this test
    pub fn with_telemetry(mut self, enabled: bool) -> Self {
        self.enable_telemetry = enabled;
        self
    }

    /// Enable or disable console export
    pub fn with_console_export(mut self, enabled: bool) -> Self {
        self.console_export = enabled;
        self
    }

    /// Add custom test attributes
    pub fn with_attribute(mut self, key: &str, value: &str) -> Self {
        self.custom_attributes.insert(key.to_string(), value.to_string());
        self
    }

    /// Build the test harness
    pub fn build(self) -> Result<TelemetryTestHarness, StoodError> {
        if !self.enable_telemetry {
            return Ok(TelemetryTestHarness {
                tracer: None,
                test_trace_id: format!("disabled-{}", Uuid::new_v4().to_string()[..8].to_string()),
                root_context: Context::current(),
            });
        }

        let test_trace_id = format!("test-{}-{}", self.test_name, Uuid::new_v4().to_string()[..8].to_string());
        
        let mut service_attributes = HashMap::new();
        service_attributes.insert("test.name".to_string(), self.test_name.clone());
        service_attributes.insert("test.trace_id".to_string(), test_trace_id.clone());
        service_attributes.insert("test.environment".to_string(), "integration".to_string());
        service_attributes.extend(self.custom_attributes);

        let config = TelemetryConfig {
            enabled: true,
            console_export: self.console_export,
            service_name: format!("stood-test-{}", self.test_name),
            service_version: env!("CARGO_PKG_VERSION").to_string(),
            enable_batch_processor: false,
            export_mode: "simple".to_string(),
            service_attributes,
            enable_debug_tracing: true,
            ..TelemetryConfig::default()
        };

        let tracer = StoodTracer::init(config)?;
        
        if let Some(tracer_instance) = tracer {
            let mut root_span = tracer_instance.start_agent_span(&format!("integration_test.{}", self.test_name));
            root_span.set_attribute("test.framework", "tokio");
            root_span.set_attribute("test.type", "integration");
            root_span.set_attribute("test.trace_id", test_trace_id.clone());
            
            let root_context = Context::current();
            
            Ok(TelemetryTestHarness {
                tracer: Some(tracer_instance),
                test_trace_id,
                root_context,
            })
        } else {
            Ok(TelemetryTestHarness {
                tracer: None,
                test_trace_id,
                root_context: Context::current(),
            })
        }
    }
}

/// Helper macro for creating telemetry test harness
#[macro_export]
macro_rules! telemetry_test_harness {
    ($test_name:expr) => {
        $crate::telemetry::test_harness::TelemetryTestHarnessBuilder::new($test_name)
            .build()
            .expect("Failed to create telemetry test harness")
    };
    ($test_name:expr, $enabled:expr) => {
        $crate::telemetry::test_harness::TelemetryTestHarnessBuilder::new($test_name)
            .with_telemetry($enabled)
            .build()
            .expect("Failed to create telemetry test harness")
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_harness_creation() {
        let harness = TelemetryTestHarness::new("test_creation").unwrap();
        assert!(!harness.trace_id().is_empty());
        assert!(harness.trace_id().starts_with("test-"));
        harness.shutdown();
    }

    #[tokio::test]
    async fn test_harness_builder() {
        let harness = TelemetryTestHarnessBuilder::new("test_builder")
            .with_telemetry(true)
            .with_console_export(true)
            .with_attribute("custom.test", "value")
            .build()
            .unwrap();
        
        assert!(harness.tracer().is_some());
        assert!(!harness.trace_id().is_empty());
        harness.shutdown();
    }

    #[tokio::test]
    async fn test_harness_disabled_telemetry() {
        let harness = TelemetryTestHarnessBuilder::new("test_disabled")
            .with_telemetry(false)
            .build()
            .unwrap();
        
        assert!(harness.tracer().is_none());
        assert!(harness.trace_id().contains("disabled"));
        harness.shutdown();
    }

    #[tokio::test]
    async fn test_scoped_context() {
        let harness = TelemetryTestHarness::new("test_scoped").unwrap();
        let _scoped_ctx = harness.scoped_context("test_section");
        // Context should be different from root but in same trace
        harness.shutdown();
    }

    #[tokio::test]
    async fn test_agent_telemetry_config() {
        let harness = TelemetryTestHarness::new("test_agent_config").unwrap();
        let config = harness.agent_telemetry_config();
        
        assert!(config.enabled);
        assert!(config.console_export);
        assert!(config.service_name.contains("test-agent"));
        harness.shutdown();
    }
}