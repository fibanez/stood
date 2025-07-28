//! OpenTelemetry integration implementation
//!
//! This module provides the actual OpenTelemetry integration when the "telemetry"
//! feature is enabled. It implements OTLP and console exporters following
//! OpenTelemetry best practices.

use std::collections::HashMap;
use std::time::Duration;

use opentelemetry::{
    global,
    trace::{Span, Tracer, TraceContextExt},
    Context, KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    trace::TracerProvider,
    Resource,
};
use tracing::{error, info};

use super::{semantic_conventions, SpanStatus, TelemetryConfig, TraceInfo};
use super::otlp_debug::{log_otlp_export, log_telemetry_init, log_span_operation, OtlpExportType, SpanOperation};
use super::metrics::{OtelMetricsCollector, SharedMetricsCollector, create_metrics_exporter};
use crate::StoodError;

/// OpenTelemetry tracer wrapper for the Stood library
#[derive(Clone)]
pub struct StoodTracer {
    config: TelemetryConfig,
    metrics_collector: Option<SharedMetricsCollector>,
}

impl std::fmt::Debug for StoodTracer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StoodTracer")
            .field("config", &self.config)
            .field("has_metrics_collector", &self.metrics_collector.is_some())
            .finish()
    }
}

impl StoodTracer {
    /// Initialize OpenTelemetry based on the configuration
    pub fn init(config: TelemetryConfig) -> Result<Option<Self>, StoodError> {
        let module = "telemetry::otel::StoodTracer::init";
        
        if !config.enabled {
            info!("Telemetry disabled, skipping OpenTelemetry initialization");
            log_telemetry_init(module, &config, &Ok(false));
            return Ok(None);
        }

        config.validate()?;

        info!("Initializing OpenTelemetry with config: {:?}", config);
        
        // Log the initialization attempt
        log_telemetry_init(module, &config, &Ok(true));

        // Create resource with service information and GenAI attributes
        let mut resource_attrs = vec![
            KeyValue::new("service.name", config.service_name.clone()),
            KeyValue::new("service.version", config.service_version.clone()),
            KeyValue::new(
                semantic_conventions::STOOD_VERSION,
                env!("CARGO_PKG_VERSION"),
            ),
            KeyValue::new(
                semantic_conventions::GEN_AI_SYSTEM,
                semantic_conventions::SYSTEM_ANTHROPIC_BEDROCK,
            ),
        ];

        // Add custom service attributes
        for (key, value) in &config.service_attributes {
            resource_attrs.push(KeyValue::new(format!("service.{}", key), value.clone()));
        }

        let resource = Resource::new(resource_attrs);

        // Create tracer provider with batch processors for production efficiency
        let mut builder = TracerProvider::builder()
            .with_config(opentelemetry_sdk::trace::Config::default().with_resource(resource));

        // Add OTLP exporter if endpoint is configured
        if let Some(ref endpoint) = config.otlp_endpoint {
            info!("Configuring OTLP exporter for endpoint: {}", endpoint);

            // Log OTLP exporter creation attempt
            log_otlp_export(
                module,
                OtlpExportType::Traces,
                endpoint,
                "OTLP exporter initialization",
                0,
                &Ok(()),
            );

            // Smart protocol detection - use HTTP for port 4320, gRPC for others
            let use_http = endpoint.contains(":4320") || endpoint.contains(":4318");
            
            // Create batch processor with optimized settings
            let batch_processor_result = if use_http {
                info!("Using HTTP protocol for OTLP traces (endpoint: {})", endpoint);
                let http_endpoint = if endpoint.ends_with("/v1/traces") {
                    endpoint.clone()
                } else {
                    format!("{}/v1/traces", endpoint)
                };
                opentelemetry_otlp::new_exporter()
                    .http()
                    .with_endpoint(http_endpoint)
                    .build_span_exporter()
            } else {
                info!("Using gRPC protocol for OTLP traces (endpoint: {})", endpoint);
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint(endpoint)
                    .build_span_exporter()
            }
                .map_err(|e| {
                    let error = format!("Failed to build OTLP exporter: {}", e);
                    log_otlp_export(
                        module,
                        OtlpExportType::Traces,
                        endpoint,
                        "OTLP exporter build failed",
                        0,
                        &Err(error.clone()),
                    );
                    StoodError::configuration_error(error)
                });

            match config.export_mode.as_str() {
                "batch" => {
                    // Use batch processor for production (high throughput)
                    let batch_processor = opentelemetry_sdk::trace::BatchSpanProcessor::builder(
                        batch_processor_result?,
                        opentelemetry_sdk::runtime::Tokio,
                    )
                    .build();
                    builder = builder.with_span_processor(batch_processor);
                    info!("✅ OTLP batch processor configured for high throughput");
                }
                "simple" | _ => {
                    // Use standard batch processor for simple mode
                    // The key difference is the name/intent, actual export happens async
                    let simple_batch_processor = opentelemetry_sdk::trace::BatchSpanProcessor::builder(
                        batch_processor_result?,
                        opentelemetry_sdk::runtime::Tokio,
                    )
                    .build();
                    builder = builder.with_span_processor(simple_batch_processor);
                    info!("✅ OTLP processor configured for development (simple mode)");
                }
            }
            
            info!("✅ OTLP exporter configured successfully");
            log_otlp_export(
                module,
                OtlpExportType::Traces,
                endpoint,
                "OTLP exporter configured successfully",
                0,
                &Ok(()),
            );
        }

        // Add console exporter if enabled
        if config.console_export {
            info!("Enabling console exporter for debugging");
            let console_exporter = opentelemetry_stdout::SpanExporter::default();
            let console_processor = opentelemetry_sdk::trace::BatchSpanProcessor::builder(
                console_exporter,
                opentelemetry_sdk::runtime::Tokio,
            )
            .build();

            builder = builder.with_span_processor(console_processor);
        }

        let tracer_provider = builder.build();

        // Set global tracer provider
        global::set_tracer_provider(tracer_provider);

        info!("OpenTelemetry initialized successfully with batch processors");

        // Initialize metrics if available
        let metrics_collector = Self::init_metrics(&config)?;

        Ok(Some(Self { 
            config,
            metrics_collector,
        }))
    }

    /// Initialize tracing subscriber with OpenTelemetry integration
    ///
    /// This bridges Rust's `tracing` crate with OpenTelemetry, allowing
    /// automatic span creation and correlation from tracing macros.
    ///
    /// Note: This should be called AFTER OpenTelemetry initialization
    pub fn init_tracing_subscriber() -> Result<(), StoodError> {
        use tracing_subscriber::{layer::SubscriberExt};
        #[allow(unused_imports)] // Used in certain telemetry configurations
        use tracing_subscriber::util::SubscriberInitExt;

        // Create OpenTelemetry tracing layer with proper context propagation
        // The layer will automatically use the global tracer provider we set earlier
        // Ensure it uses the same tracer name for consistency
        let telemetry_layer = tracing_opentelemetry::layer()
            .with_location(true)
            .with_tracked_inactivity(true)
            .with_threads(true);

        // Create console/format layer for human-readable output
        let format_layer = tracing_subscriber::fmt::layer()
            .with_target(true)
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true);

        // Create filter layer based on environment
        let filter_layer = tracing_subscriber::EnvFilter::try_from_default_env()
            .or_else(|_| tracing_subscriber::EnvFilter::try_new("stood=debug,info"))
            .map_err(|e| {
                StoodError::configuration_error(format!("Failed to create tracing filter: {}", e))
            })?;

        // Initialize the global subscriber (replacing any existing one)
        let subscriber = tracing_subscriber::registry()
            .with(filter_layer)
            .with(format_layer)
            .with(telemetry_layer);
            
        tracing::subscriber::set_global_default(subscriber)
            .map_err(|e| {
                StoodError::configuration_error(format!(
                    "Failed to set global tracing subscriber: {}",
                    e
                ))
            })?;

        info!("Tracing subscriber initialized with OpenTelemetry integration");
        Ok(())
    }

    /// Get a tracer instance
    fn tracer(&self) -> opentelemetry::global::BoxedTracer {
        global::tracer("stood-agent")
    }

    /// Initialize metrics collector with OTLP export
    fn init_metrics(config: &TelemetryConfig) -> Result<Option<SharedMetricsCollector>, StoodError> {
        if !config.enabled {
            return Ok(None);
        }

        match create_metrics_exporter(config) {
            Ok(Some(exporter)) => {
                let meter = exporter.meter();
                match OtelMetricsCollector::new(meter.clone()) {
                    Ok(collector) => {
                        info!("Metrics collector initialized successfully");
                        Ok(Some(std::sync::Arc::new(collector)))
                    }
                    Err(e) => {
                        error!("Failed to create metrics collector: {}", e);
                        Ok(None)
                    }
                }
            }
            Ok(None) => {
                info!("Metrics exporter not configured, metrics collection disabled");
                Ok(None)
            }
            Err(e) => {
                error!("Failed to create metrics exporter: {}", e);
                // Don't fail initialization, just disable metrics
                Ok(None)
            }
        }
    }

    /// Get the metrics collector if available
    pub fn metrics_collector(&self) -> Option<&SharedMetricsCollector> {
        self.metrics_collector.as_ref()
    }

    /// Record metrics for a request
    pub fn record_request_metrics(&self, metrics: &crate::telemetry::metrics::RequestMetrics) {
        if let Some(ref collector) = self.metrics_collector {
            collector.record_request_metrics(metrics);
        }
    }

    /// Record metrics for token usage  
    pub fn record_token_metrics(&self, metrics: &crate::telemetry::metrics::TokenMetrics) {
        if let Some(ref collector) = self.metrics_collector {
            collector.record_token_metrics(metrics);
        }
    }

    /// Record metrics for tool execution
    pub fn record_tool_metrics(&self, metrics: &crate::telemetry::metrics::ToolMetrics) {
        if let Some(ref collector) = self.metrics_collector {
            collector.record_tool_metrics(metrics);
        }
    }

    /// Record system resource metrics
    pub fn record_system_metrics(&self, metrics: &crate::telemetry::metrics::SystemMetrics) {
        if let Some(ref collector) = self.metrics_collector {
            collector.record_system_metrics(metrics);
        }
    }

    /// Create a new span for agent operations
    pub fn start_agent_span(&self, operation: &str) -> StoodSpan {
        let tracer = self.tracer();
        let span = tracer
            .span_builder(format!("agent.{}", operation))
            .with_kind(opentelemetry::trace::SpanKind::Internal)
            .with_attributes(vec![
                KeyValue::new(
                    semantic_conventions::GEN_AI_OPERATION_NAME,
                    operation.to_string(),
                ),
                KeyValue::new(
                    semantic_conventions::GEN_AI_SYSTEM,
                    semantic_conventions::SYSTEM_ANTHROPIC_BEDROCK,
                ),
            ])
            .start_with_context(&tracer, &Context::current());

        StoodSpan::new(span, operation.to_string())
    }

    /// Create a new span for model inference with comprehensive GenAI attributes
    pub fn start_model_span(&self, model: &str) -> StoodSpan {
        let tracer = self.tracer();
        let span = tracer
            .span_builder("model.inference")
            .with_kind(opentelemetry::trace::SpanKind::Client)
            .with_attributes(vec![
                KeyValue::new(
                    semantic_conventions::GEN_AI_REQUEST_MODEL,
                    model.to_string(),
                ),
                KeyValue::new(
                    semantic_conventions::GEN_AI_RESPONSE_MODEL,
                    model.to_string(),
                ),
                KeyValue::new(
                    semantic_conventions::GEN_AI_OPERATION_NAME,
                    semantic_conventions::OPERATION_CHAT,
                ),
                KeyValue::new(
                    semantic_conventions::GEN_AI_SYSTEM,
                    semantic_conventions::SYSTEM_ANTHROPIC_BEDROCK,
                ),
                KeyValue::new(
                    semantic_conventions::STOOD_VERSION,
                    env!("CARGO_PKG_VERSION"),
                ),
            ])
            .start_with_context(&tracer, &Context::current());

        StoodSpan::new(span, "model.inference".to_string())
    }

    /// Create a new span for model inference with full request parameters
    pub fn start_model_span_with_params(
        &self,
        model: &str,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
        prompt_length: Option<usize>,
    ) -> StoodSpan {
        let tracer = self.tracer();

        let mut attributes = vec![
            KeyValue::new(
                semantic_conventions::GEN_AI_REQUEST_MODEL,
                model.to_string(),
            ),
            KeyValue::new(
                semantic_conventions::GEN_AI_RESPONSE_MODEL,
                model.to_string(),
            ),
            KeyValue::new(
                semantic_conventions::GEN_AI_OPERATION_NAME,
                semantic_conventions::OPERATION_CHAT,
            ),
            KeyValue::new(
                semantic_conventions::GEN_AI_SYSTEM,
                semantic_conventions::SYSTEM_ANTHROPIC_BEDROCK,
            ),
            KeyValue::new(
                semantic_conventions::STOOD_VERSION,
                env!("CARGO_PKG_VERSION"),
            ),
        ];

        // Add optional parameters
        if let Some(temp) = temperature {
            attributes.push(KeyValue::new(
                semantic_conventions::GEN_AI_REQUEST_TEMPERATURE,
                temp as f64,
            ));
        }
        if let Some(tokens) = max_tokens {
            attributes.push(KeyValue::new(
                semantic_conventions::GEN_AI_REQUEST_MAX_TOKENS,
                tokens as i64,
            ));
        }
        if let Some(length) = prompt_length {
            attributes.push(KeyValue::new("gen_ai.prompt.length", length as i64));
        }

        let span = tracer
            .span_builder("model.inference")
            .with_kind(opentelemetry::trace::SpanKind::Client)
            .with_attributes(attributes)
            .start_with_context(&tracer, &Context::current());

        StoodSpan::new(span, "model.inference".to_string())
    }

    /// Create a new span for model inference with full request parameters and parent context
    pub fn start_model_span_with_params_and_context(
        &self,
        model: &str,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
        prompt_length: Option<usize>,
        parent_context: &Context,
    ) -> StoodSpan {
        let tracer = self.tracer();

        let mut attributes = vec![
            KeyValue::new(
                semantic_conventions::GEN_AI_REQUEST_MODEL,
                model.to_string(),
            ),
            KeyValue::new(
                semantic_conventions::GEN_AI_RESPONSE_MODEL,
                model.to_string(),
            ),
            KeyValue::new(
                semantic_conventions::GEN_AI_OPERATION_NAME,
                semantic_conventions::OPERATION_CHAT,
            ),
            KeyValue::new(
                semantic_conventions::GEN_AI_SYSTEM,
                semantic_conventions::SYSTEM_ANTHROPIC_BEDROCK,
            ),
            KeyValue::new(
                semantic_conventions::STOOD_VERSION,
                env!("CARGO_PKG_VERSION"),
            ),
        ];

        // Add optional parameters
        if let Some(temp) = temperature {
            attributes.push(KeyValue::new(
                semantic_conventions::GEN_AI_REQUEST_TEMPERATURE,
                temp as f64,
            ));
        }
        if let Some(tokens) = max_tokens {
            attributes.push(KeyValue::new(
                semantic_conventions::GEN_AI_REQUEST_MAX_TOKENS,
                tokens as i64,
            ));
        }
        if let Some(length) = prompt_length {
            attributes.push(KeyValue::new("gen_ai.prompt.length", length as i64));
        }

        let span = tracer
            .span_builder("model.inference")
            .with_kind(opentelemetry::trace::SpanKind::Client)
            .with_attributes(attributes)
            .start_with_context(&tracer, parent_context);

        StoodSpan::new(span, "model.inference".to_string())
    }

    /// Create a new span for model inference with dynamic provider/model attributes
    pub fn start_model_span_with_dynamic_attributes(
        &self,
        model: &dyn crate::llm::traits::LlmModel,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
        prompt_length: Option<usize>,
        parent_context: &Context,
    ) -> StoodSpan {
        let tracer = self.tracer();

        let mut attributes = vec![
            KeyValue::new(
                semantic_conventions::GEN_AI_REQUEST_MODEL,
                model.model_id().to_string(),
            ),
            KeyValue::new(
                semantic_conventions::GEN_AI_RESPONSE_MODEL,
                model.model_id().to_string(),
            ),
            KeyValue::new(
                semantic_conventions::GEN_AI_OPERATION_NAME,
                semantic_conventions::OPERATION_CHAT,
            ),
            KeyValue::new(
                semantic_conventions::GEN_AI_SYSTEM,
                semantic_conventions::SYSTEM_ANTHROPIC_BEDROCK,
            ),
            KeyValue::new(
                semantic_conventions::STOOD_VERSION,
                env!("CARGO_PKG_VERSION"),
            ),
        ];

        // Add dynamic provider and model attributes
        let mut dynamic_attrs = ProviderModelAttributes::extract_attributes(model);
        attributes.append(&mut dynamic_attrs);

        // Add optional parameters
        if let Some(temp) = temperature {
            attributes.push(KeyValue::new(
                semantic_conventions::GEN_AI_REQUEST_TEMPERATURE,
                temp as f64,
            ));
        }
        if let Some(tokens) = max_tokens {
            attributes.push(KeyValue::new(
                semantic_conventions::GEN_AI_REQUEST_MAX_TOKENS,
                tokens as i64,
            ));
        }
        if let Some(length) = prompt_length {
            attributes.push(KeyValue::new("gen_ai.prompt.length", length as i64));
        }

        let span = tracer
            .span_builder("model.inference")
            .with_kind(opentelemetry::trace::SpanKind::Client)
            .with_attributes(attributes)
            .start_with_context(&tracer, parent_context);

        StoodSpan::new(span, "model.inference".to_string())
    }

    /// Create a new span for tool execution
    pub fn start_tool_span(&self, tool_name: &str) -> StoodSpan {
        let tracer = self.tracer();
        let span = tracer
            .span_builder(format!("tool.{}", tool_name))
            .with_kind(opentelemetry::trace::SpanKind::Internal)
            .with_attributes(vec![
                KeyValue::new(
                    semantic_conventions::GEN_AI_TOOL_NAME,
                    tool_name.to_string(),
                ),
                KeyValue::new(
                    semantic_conventions::GEN_AI_OPERATION_NAME,
                    semantic_conventions::OPERATION_TOOL_CALL,
                ),
            ])
            .start_with_context(&tracer, &Context::current());

        StoodSpan::new(span, format!("tool.{}", tool_name))
    }

    /// Create a new span for event loop model interactions
    pub fn start_cycle_span(&self, cycle_id: &str) -> StoodSpan {
        let tracer = self.tracer();
        let span = tracer
            .span_builder("agent.model_interaction")
            .with_kind(opentelemetry::trace::SpanKind::Internal)
            .with_attributes(vec![
                KeyValue::new(semantic_conventions::STOOD_CYCLE_ID, cycle_id.to_string()),
                KeyValue::new(
                    semantic_conventions::GEN_AI_OPERATION_NAME,
                    semantic_conventions::OPERATION_AGENT_CYCLE,
                ),
            ])
            .start_with_context(&tracer, &Context::current());

        StoodSpan::new(span, "agent.model_interaction".to_string())
    }

    /// Create a new model interaction span with explicit parent context
    pub fn start_cycle_span_with_context(&self, cycle_id: &str, parent_context: &Context) -> StoodSpan {
        let tracer = self.tracer();
        let span = tracer
            .span_builder("agent.model_interaction")
            .with_kind(opentelemetry::trace::SpanKind::Internal)
            .with_attributes(vec![
                KeyValue::new(semantic_conventions::STOOD_CYCLE_ID, cycle_id.to_string()),
                KeyValue::new(
                    semantic_conventions::GEN_AI_OPERATION_NAME,
                    semantic_conventions::OPERATION_AGENT_CYCLE,
                ),
            ])
            .start_with_context(&tracer, parent_context);

        StoodSpan::new(span, "agent.model_interaction".to_string())
    }

    /// Create a new span for event loop model interactions with dynamic attributes
    pub fn start_cycle_span_with_dynamic_attributes(
        &self,
        cycle_id: &str,
        model: &dyn crate::llm::traits::LlmModel,
        parent_context: Option<&Context>,
    ) -> StoodSpan {
        let tracer = self.tracer();
        
        let mut attributes = vec![
            KeyValue::new(semantic_conventions::STOOD_CYCLE_ID, cycle_id.to_string()),
            KeyValue::new(
                semantic_conventions::GEN_AI_OPERATION_NAME,
                semantic_conventions::OPERATION_AGENT_CYCLE,
            ),
        ];

        // Add dynamic provider and model attributes to cycle span
        let mut dynamic_attrs = ProviderModelAttributes::extract_attributes(model);
        attributes.append(&mut dynamic_attrs);

        let span = if let Some(parent_ctx) = parent_context {
            tracer
                .span_builder("agent.model_interaction")
                .with_kind(opentelemetry::trace::SpanKind::Internal)
                .with_attributes(attributes)
                .start_with_context(&tracer, parent_ctx)
        } else {
            tracer
                .span_builder("agent.model_interaction")
                .with_kind(opentelemetry::trace::SpanKind::Internal)
                .with_attributes(attributes)
                .start_with_context(&tracer, &Context::current())
        };

        StoodSpan::new(span, "agent.model_interaction".to_string())
    }

    /// Create a new span for tool execution with parent context
    pub fn start_tool_span_with_parent_context(&self, tool_name: &str, parent_context: &Context) -> StoodSpan {
        let tracer = self.tracer();
        let span = tracer
            .span_builder("tool.execution")
            .with_kind(opentelemetry::trace::SpanKind::Internal)
            .with_attributes(vec![
                KeyValue::new(semantic_conventions::GEN_AI_TOOL_NAME, tool_name.to_string()),
                KeyValue::new(
                    semantic_conventions::GEN_AI_OPERATION_NAME,
                    semantic_conventions::OPERATION_TOOL_CALL,
                ),
            ])
            .start_with_context(&tracer, parent_context);

        StoodSpan::new(span, format!("tool.{}", tool_name))
    }

    /// Create a new span for agent operations with agent context
    pub fn start_agent_span_with_context(
        &self,
        operation: &str,
        agent_context: &crate::agent::AgentContext,
    ) -> StoodSpan {
        let tracer = self.tracer();
        let mut attributes = vec![
            KeyValue::new(
                semantic_conventions::GEN_AI_OPERATION_NAME,
                operation.to_string(),
            ),
            KeyValue::new(
                semantic_conventions::GEN_AI_SYSTEM,
                semantic_conventions::SYSTEM_ANTHROPIC_BEDROCK,
            ),
            KeyValue::new("agent.id", agent_context.agent_id.clone()),
            KeyValue::new("agent.type", agent_context.agent_type.clone()),
        ];

        if let Some(ref name) = agent_context.agent_name {
            attributes.push(KeyValue::new("agent.name", name.clone()));
        }

        let span = tracer
            .span_builder(format!("agent.{}", operation))
            .with_kind(opentelemetry::trace::SpanKind::Internal)
            .with_attributes(attributes)
            .start_with_context(&tracer, &Context::current());

        StoodSpan::new(span, operation.to_string())
    }

    /// Create a new span for model inference with agent context
    pub fn start_model_span_with_context(
        &self,
        model: &str,
        agent_context: &crate::agent::AgentContext,
    ) -> StoodSpan {
        let tracer = self.tracer();
        let mut attributes = vec![
            KeyValue::new(
                semantic_conventions::GEN_AI_REQUEST_MODEL,
                model.to_string(),
            ),
            KeyValue::new(
                semantic_conventions::GEN_AI_RESPONSE_MODEL,
                model.to_string(),
            ),
            KeyValue::new(
                semantic_conventions::GEN_AI_OPERATION_NAME,
                semantic_conventions::OPERATION_CHAT,
            ),
            KeyValue::new(
                semantic_conventions::GEN_AI_SYSTEM,
                semantic_conventions::SYSTEM_ANTHROPIC_BEDROCK,
            ),
            KeyValue::new(
                semantic_conventions::STOOD_VERSION,
                env!("CARGO_PKG_VERSION"),
            ),
            KeyValue::new("agent.id", agent_context.agent_id.clone()),
            KeyValue::new("agent.type", agent_context.agent_type.clone()),
        ];

        if let Some(ref name) = agent_context.agent_name {
            attributes.push(KeyValue::new("agent.name", name.clone()));
        }

        let span = tracer
            .span_builder("model.inference")
            .with_kind(opentelemetry::trace::SpanKind::Client)
            .with_attributes(attributes)
            .start_with_context(&tracer, &Context::current());

        StoodSpan::new(span, "model.inference".to_string())
    }

    /// Create a new span for tool execution with agent context
    pub fn start_tool_span_with_context(
        &self,
        tool_name: &str,
        agent_context: &crate::agent::AgentContext,
    ) -> StoodSpan {
        let tracer = self.tracer();
        let mut attributes = vec![
            KeyValue::new(
                semantic_conventions::GEN_AI_TOOL_NAME,
                tool_name.to_string(),
            ),
            KeyValue::new(
                semantic_conventions::GEN_AI_OPERATION_NAME,
                semantic_conventions::OPERATION_TOOL_CALL,
            ),
            KeyValue::new("agent.id", agent_context.agent_id.clone()),
            KeyValue::new("agent.type", agent_context.agent_type.clone()),
        ];

        if let Some(ref name) = agent_context.agent_name {
            attributes.push(KeyValue::new("agent.name", name.clone()));
        }

        let span = tracer
            .span_builder(format!("tool.{}", tool_name))
            .with_kind(opentelemetry::trace::SpanKind::Internal)
            .with_attributes(attributes)
            .start_with_context(&tracer, &Context::current());

        StoodSpan::new(span, format!("tool.{}", tool_name))
    }

    /// Get the current trace context for correlation
    pub fn current_context(&self) -> Context {
        Context::current()
    }

    /// Shutdown the tracer provider gracefully
    pub fn shutdown(&self) {
        global::shutdown_tracer_provider();
        info!("OpenTelemetry tracer provider shutdown successfully");
    }
}

/// Wrapper around OpenTelemetry span with Stood-specific functionality
pub struct StoodSpan {
    span: opentelemetry::global::BoxedSpan,
    operation: String,
    start_time: chrono::DateTime<chrono::Utc>,
}

impl StoodSpan {
    fn new(span: opentelemetry::global::BoxedSpan, operation: String) -> Self {
        let module = "telemetry::otel::StoodSpan";
        
        // Log span creation
        log_span_operation(
            module,
            SpanOperation::Start,
            &operation,
            Some(&serde_json::json!({"start_time": chrono::Utc::now()})),
        );
        
        Self {
            span,
            operation,
            start_time: chrono::Utc::now(),
        }
    }

    /// Add an attribute to the span
    pub fn set_attribute(&mut self, key: &str, value: impl Into<opentelemetry::Value>) {
        self.span
            .set_attribute(KeyValue::new(key.to_string(), value));
    }

    /// Add multiple attributes to the span
    pub fn set_attributes(&mut self, attributes: Vec<KeyValue>) {
        for attr in attributes {
            self.span.set_attribute(attr);
        }
    }

    /// Set the span status to success
    pub fn set_success(&mut self) {
        self.span.set_status(opentelemetry::trace::Status::Ok);
    }

    /// Set the span status to error
    pub fn set_error(&mut self, error: &str) {
        self.span
            .set_status(opentelemetry::trace::Status::error(error.to_string()));
        self.span
            .set_attribute(KeyValue::new("error.message", error.to_string()));
        self.span.set_attribute(KeyValue::new("error", true));
    }

    /// Add an event to the span
    pub fn add_event(&mut self, name: &str, attributes: Vec<KeyValue>) {
        self.span.add_event(name.to_string(), attributes);
    }

    /// Record token usage in the span
    pub fn record_token_usage(&mut self, input_tokens: u32, output_tokens: u32) {
        self.set_attributes(vec![
            KeyValue::new(
                semantic_conventions::GEN_AI_USAGE_INPUT_TOKENS,
                input_tokens as i64,
            ),
            KeyValue::new(
                semantic_conventions::GEN_AI_USAGE_OUTPUT_TOKENS,
                output_tokens as i64,
            ),
            KeyValue::new(
                semantic_conventions::GEN_AI_USAGE_TOTAL_TOKENS,
                (input_tokens + output_tokens) as i64,
            ),
        ]);
    }

    /// Record model parameters in the span
    pub fn record_model_params(&mut self, temperature: Option<f32>, max_tokens: Option<u32>) {
        if let Some(temp) = temperature {
            self.set_attribute(
                semantic_conventions::GEN_AI_REQUEST_TEMPERATURE,
                temp as f64,
            );
        }
        if let Some(tokens) = max_tokens {
            self.set_attribute(
                semantic_conventions::GEN_AI_REQUEST_MAX_TOKENS,
                tokens as i64,
            );
        }
    }

    /// Record response information
    pub fn record_response(&mut self, response_id: &str, finish_reason: &str) {
        self.set_attributes(vec![
            KeyValue::new(
                semantic_conventions::GEN_AI_RESPONSE_ID,
                response_id.to_string(),
            ),
            KeyValue::new(
                semantic_conventions::GEN_AI_RESPONSE_FINISH_REASONS,
                finish_reason.to_string(),
            ),
        ]);
    }

    /// Record prompt information for GenAI operations
    pub fn record_prompt(&mut self, prompt: &str, message_count: Option<usize>) {
        let mut attributes = vec![
            KeyValue::new("gen_ai.prompt.content", prompt.to_string()),
            KeyValue::new("gen_ai.prompt.length", prompt.len() as i64),
        ];

        if let Some(count) = message_count {
            attributes.push(KeyValue::new("gen_ai.prompt.message_count", count as i64));
        }

        self.set_attributes(attributes);

        // Add event for prompt submission
        self.add_event(
            "prompt.submitted",
            vec![KeyValue::new("prompt.length", prompt.len() as i64)],
        );
    }

    /// Record completion information for GenAI operations
    pub fn record_completion(&mut self, completion: &str, completion_id: Option<&str>) {
        let mut attributes = vec![
            KeyValue::new("gen_ai.completion.content", completion.to_string()),
            KeyValue::new("gen_ai.completion.length", completion.len() as i64),
        ];

        if let Some(id) = completion_id {
            attributes.push(KeyValue::new("gen_ai.completion.id", id.to_string()));
        }

        self.set_attributes(attributes);

        // Add event for completion received
        self.add_event(
            "completion.received",
            vec![KeyValue::new("completion.length", completion.len() as i64)],
        );
    }

    /// Record tool execution details with parameters and results
    pub fn record_tool_execution(
        &mut self,
        tool_name: &str,
        input: &str,
        output: &str,
        success: bool,
    ) {
        self.set_attributes(vec![
            KeyValue::new(
                semantic_conventions::GEN_AI_TOOL_NAME,
                tool_name.to_string(),
            ),
            KeyValue::new("tool.input.content", input.to_string()),
            KeyValue::new("tool.input.length", input.len() as i64),
            KeyValue::new("tool.output.content", output.to_string()),
            KeyValue::new("tool.output.length", output.len() as i64),
            KeyValue::new("tool.execution.success", success),
        ]);

        // Add event for tool execution
        let event_name = if success {
            "tool.execution.completed"
        } else {
            "tool.execution.failed"
        };
        self.add_event(
            event_name,
            vec![
                KeyValue::new("tool.name", tool_name.to_string()),
                KeyValue::new("tool.success", success),
            ],
        );
    }

    /// Record agent model interaction information
    pub fn record_agent_model_interaction(&mut self, interaction_id: &str, message_count: usize, tool_count: usize) {
        self.set_attributes(vec![
            KeyValue::new(semantic_conventions::STOOD_CYCLE_ID, interaction_id.to_string()),
            KeyValue::new("agent.model_interaction.message_count", message_count as i64),
            KeyValue::new("agent.model_interaction.tool_count", tool_count as i64),
        ]);

        // Add event for model interaction completion
        self.add_event(
            "agent.model_interaction.completed",
            vec![
                KeyValue::new("model_interaction.id", interaction_id.to_string()),
                KeyValue::new("model_interaction.messages", message_count as i64),
                KeyValue::new("model_interaction.tools", tool_count as i64),
            ],
        );
    }

    /// Record agent context information in the span
    pub fn record_agent_context(&mut self, agent_context: &crate::agent::AgentContext) {
        let mut attributes = vec![
            KeyValue::new("agent.id", agent_context.agent_id.clone()),
            KeyValue::new("agent.type", agent_context.agent_type.clone()),
        ];

        if let Some(ref name) = agent_context.agent_name {
            attributes.push(KeyValue::new("agent.name", name.clone()));
        }

        self.set_attributes(attributes);
    }

    /// Record parent agent context for hierarchy tracking
    pub fn record_parent_agent_context(&mut self, parent_context: &crate::agent::AgentContext) {
        let mut attributes = vec![
            KeyValue::new("agent.parent.id", parent_context.agent_id.clone()),
            KeyValue::new("agent.parent.type", parent_context.agent_type.clone()),
        ];

        if let Some(ref name) = parent_context.agent_name {
            attributes.push(KeyValue::new("agent.parent.name", name.clone()));
        }

        self.set_attributes(attributes);
    }

    /// Get trace information for metrics correlation
    pub fn trace_info(&self) -> TraceInfo {
        let span_context = self.span.span_context();

        TraceInfo {
            trace_id: span_context.trace_id().to_string(),
            span_id: span_context.span_id().to_string(),
            operation: self.operation.clone(),
            start_time: self.start_time,
            duration: chrono::Utc::now()
                .signed_duration_since(self.start_time)
                .to_std()
                .unwrap_or(Duration::ZERO),
            status: SpanStatus::Ok, // This would need to be tracked separately
            attributes: HashMap::new(), // Would need to extract from span
        }
    }

    /// Finish the span
    pub fn finish(mut self) {
        self.span.end();
    }

    /// Finish the span with an error
    pub fn finish_with_error(mut self, error: &str) {
        self.set_error(error);
        self.span.end();
    }

    /// Get the span context for creating child spans
    pub fn context(&self) -> Context {
        let span_context = self.span.span_context();
        Context::current().with_remote_span_context(span_context.clone())
    }

    /// Get a reference to the underlying span
    pub fn span(&self) -> &opentelemetry::global::BoxedSpan {
        &self.span
    }
}

/// Helper for extracting dynamic provider and model attributes
pub struct ProviderModelAttributes;

impl ProviderModelAttributes {
    /// Extract dynamic provider and model attributes from an LlmModel
    pub fn extract_attributes(model: &dyn crate::llm::traits::LlmModel) -> Vec<KeyValue> {
        let mut attributes = Vec::new();
        
        // Provider information
        let provider_type = model.provider();
        attributes.push(KeyValue::new(
            semantic_conventions::GEN_AI_PROVIDER_TYPE,
            format!("{:?}", provider_type).to_lowercase(),
        ));
        attributes.push(KeyValue::new(
            semantic_conventions::STOOD_PROVIDER_TYPE,
            format!("{:?}", provider_type),
        ));
        
        // Provider name mapping
        let provider_name = match provider_type {
            crate::llm::traits::ProviderType::Bedrock => "aws_bedrock",
            crate::llm::traits::ProviderType::LmStudio => "lm_studio", 
            crate::llm::traits::ProviderType::Anthropic => "anthropic",
            crate::llm::traits::ProviderType::OpenAI => "openai",
            crate::llm::traits::ProviderType::Ollama => "ollama",
            crate::llm::traits::ProviderType::OpenRouter => "openrouter",
            crate::llm::traits::ProviderType::Candle => "candle",
        };
        attributes.push(KeyValue::new(
            semantic_conventions::GEN_AI_PROVIDER_NAME,
            provider_name,
        ));
        
        // Model information
        attributes.push(KeyValue::new(
            semantic_conventions::GEN_AI_MODEL_DISPLAY_NAME,
            model.display_name(),
        ));
        
        // Model capabilities
        let capabilities = model.capabilities();
        attributes.push(KeyValue::new(
            semantic_conventions::STOOD_MODEL_SUPPORTS_TOOLS,
            capabilities.supports_tools,
        ));
        attributes.push(KeyValue::new(
            semantic_conventions::STOOD_MODEL_SUPPORTS_STREAMING,
            capabilities.supports_streaming,
        ));
        attributes.push(KeyValue::new(
            "gen_ai.model.supports_thinking",
            capabilities.supports_thinking,
        ));
        
        // Model context window and output limits
        attributes.push(KeyValue::new(
            "gen_ai.model.context_window",
            model.context_window() as i64,
        ));
        attributes.push(KeyValue::new(
            "gen_ai.model.max_output_tokens",
            model.max_output_tokens() as i64,
        ));
        
        attributes
    }
}

/// Enhanced Semantic Conventions (Phase 8) - Advanced Event Tracking
pub struct EnhancedSemanticConventions;

impl EnhancedSemanticConventions {
    /// Create an enhanced model span with comprehensive context
    pub fn create_enhanced_model_span(
        tracer: &StoodTracer,
        model: &dyn crate::llm::traits::LlmModel,
        request_context: &EnhancedRequestContext,
        parent_context: &Context,
    ) -> StoodSpan {
        let otel_tracer = tracer.tracer();
        
        let mut attributes = vec![
            KeyValue::new(
                semantic_conventions::GEN_AI_REQUEST_MODEL,
                model.model_id().to_string(),
            ),
            KeyValue::new(
                semantic_conventions::GEN_AI_RESPONSE_MODEL,
                model.model_id().to_string(),
            ),
            KeyValue::new(
                semantic_conventions::GEN_AI_OPERATION_NAME,
                semantic_conventions::OPERATION_CHAT,
            ),
            KeyValue::new(
                semantic_conventions::GEN_AI_SYSTEM,
                semantic_conventions::SYSTEM_ANTHROPIC_BEDROCK,
            ),
            KeyValue::new(
                semantic_conventions::STOOD_VERSION,
                env!("CARGO_PKG_VERSION"),
            ),
        ];

        // Add dynamic provider and model attributes
        let mut dynamic_attrs = ProviderModelAttributes::extract_attributes(model);
        attributes.append(&mut dynamic_attrs);

        // Add request context
        if let Some(temp) = request_context.temperature {
            attributes.push(KeyValue::new(
                semantic_conventions::GEN_AI_REQUEST_TEMPERATURE,
                temp as f64,
            ));
        }
        if let Some(tokens) = request_context.max_tokens {
            attributes.push(KeyValue::new(
                semantic_conventions::GEN_AI_REQUEST_MAX_TOKENS,
                tokens as i64,
            ));
        }
        if let Some(length) = request_context.prompt_length {
            attributes.push(KeyValue::new("gen_ai.prompt.length", length as i64));
        }
        
        // Enhanced request tracking
        attributes.push(KeyValue::new(
            "gen_ai.request.id",
            request_context.request_id.clone(),
        ));
        attributes.push(KeyValue::new(
            semantic_conventions::GEN_AI_SESSION,
            request_context.session_id.clone(),
        ));
        
        // Message count and conversation depth
        attributes.push(KeyValue::new(
            "gen_ai.conversation.depth",
            request_context.conversation_depth as i64,
        ));
        attributes.push(KeyValue::new(
            "gen_ai.message.count",
            request_context.message_count as i64,
        ));

        let span = otel_tracer
            .span_builder("model.inference")
            .with_kind(opentelemetry::trace::SpanKind::Client)
            .with_attributes(attributes)
            .start_with_context(&otel_tracer, parent_context);

        StoodSpan::new(span, "model.inference".to_string())
    }

    /// Enhance response span with comprehensive response details
    pub fn enhance_response_span(
        span: &mut StoodSpan,
        response_context: &EnhancedResponseContext,
    ) {
        let mut attributes = vec![
            KeyValue::new(
                semantic_conventions::GEN_AI_RESPONSE_ID,
                response_context.response_id.clone(),
            ),
            KeyValue::new(
                semantic_conventions::GEN_AI_RESPONSE_FINISH_REASONS,
                response_context.finish_reason.clone(),
            ),
        ];

        // Token usage
        if let Some(ref usage) = response_context.token_usage {
            attributes.push(KeyValue::new(
                semantic_conventions::GEN_AI_USAGE_INPUT_TOKENS,
                usage.input_tokens as i64,
            ));
            attributes.push(KeyValue::new(
                semantic_conventions::GEN_AI_USAGE_OUTPUT_TOKENS,
                usage.output_tokens as i64,
            ));
            attributes.push(KeyValue::new(
                semantic_conventions::GEN_AI_USAGE_TOTAL_TOKENS,
                usage.total_tokens as i64,
            ));
        }

        // Performance metrics
        if let Some(first_token_ms) = response_context.first_token_latency_ms {
            attributes.push(KeyValue::new(
                semantic_conventions::GEN_AI_LATENCY_FIRST_TOKEN,
                first_token_ms as i64,
            ));
        }
        if let Some(throughput) = response_context.throughput_tokens_per_second {
            attributes.push(KeyValue::new(
                "gen_ai.performance.throughput",
                throughput,
            ));
        }

        // Quality and content metrics
        attributes.push(KeyValue::new(
            "gen_ai.response.quality_score",
            response_context.quality_score,
        ));
        
        if let Some(ref content_preview) = response_context.content_preview {
            attributes.push(KeyValue::new(
                "gen_ai.content.preview",
                content_preview.clone(),
            ));
        }

        span.set_attributes(attributes);

        // Enhanced events
        span.add_event(
            "response.completed",
            vec![
                KeyValue::new("response.id", response_context.response_id.clone()),
                KeyValue::new("response.finish_reason", response_context.finish_reason.clone()),
                KeyValue::new("response.quality_score", response_context.quality_score),
            ],
        );
    }

    /// Enhance error span with comprehensive error details
    pub fn enhance_error_span(
        span: &mut StoodSpan,
        error_context: &EnhancedErrorContext,
    ) {
        let mut attributes = vec![
            KeyValue::new(
                semantic_conventions::GEN_AI_ERROR_TYPE,
                error_context.error_type.clone(),
            ),
            KeyValue::new(
                semantic_conventions::GEN_AI_ERROR_CODE,
                error_context.error_code.clone(),
            ),
            KeyValue::new(
                semantic_conventions::GEN_AI_ERROR_MESSAGE,
                error_context.error_message.clone(),
            ),
        ];

        // Recovery information
        if error_context.is_recoverable {
            attributes.push(KeyValue::new(
                semantic_conventions::GEN_AI_ERROR_RECOVERABLE,
                true,
            ));
        }
        if error_context.retry_count > 0 {
            attributes.push(KeyValue::new(
                semantic_conventions::GEN_AI_ERROR_RETRY_COUNT,
                error_context.retry_count as i64,
            ));
        }

        // Context information
        if let Some(ref context) = error_context.context {
            attributes.push(KeyValue::new(
                "gen_ai.error.context",
                context.clone(),
            ));
        }

        span.set_attributes(attributes);
        span.set_error(&error_context.error_message);

        // Enhanced error event
        span.add_event(
            "error.occurred",
            vec![
                KeyValue::new("error.type", error_context.error_type.clone()),
                KeyValue::new("error.code", error_context.error_code.clone()),
                KeyValue::new("error.recoverable", error_context.is_recoverable),
                KeyValue::new("error.retry_count", error_context.retry_count as i64),
            ],
        );
    }

    /// Record enhanced event with structured data
    pub fn record_enhanced_event(
        span: &mut StoodSpan,
        event_name: &str,
        event_data: &HashMap<String, serde_json::Value>,
    ) {
        let mut attributes = vec![
            KeyValue::new(
                semantic_conventions::GEN_AI_EVENT_NAME,
                event_name.to_string(),
            ),
        ];

        // Convert event data to attributes
        for (key, value) in event_data {
            let attribute_key = format!("event.{}", key);
            match value {
                serde_json::Value::String(s) => {
                    attributes.push(KeyValue::new(attribute_key, s.clone()));
                }
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        attributes.push(KeyValue::new(attribute_key, i));
                    } else if let Some(f) = n.as_f64() {
                        attributes.push(KeyValue::new(attribute_key, f));
                    }
                }
                serde_json::Value::Bool(b) => {
                    attributes.push(KeyValue::new(attribute_key, *b));
                }
                _ => {
                    attributes.push(KeyValue::new(attribute_key, value.to_string()));
                }
            }
        }

        span.add_event(event_name, attributes);
    }
}

/// Enhanced request context for comprehensive telemetry
pub struct EnhancedRequestContext {
    pub request_id: String,
    pub session_id: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub prompt_length: Option<usize>,
    pub conversation_depth: usize,
    pub message_count: usize,
}

/// Enhanced response context for comprehensive telemetry
pub struct EnhancedResponseContext {
    pub response_id: String,
    pub finish_reason: String,
    pub token_usage: Option<crate::llm::traits::Usage>,
    pub first_token_latency_ms: Option<u64>,
    pub throughput_tokens_per_second: Option<f64>,
    pub quality_score: f64,
    pub content_preview: Option<String>,
}

/// Enhanced error context for comprehensive telemetry
pub struct EnhancedErrorContext {
    pub error_type: String,
    pub error_code: String,
    pub error_message: String,
    pub is_recoverable: bool,
    pub retry_count: u32,
    pub context: Option<String>,
}
