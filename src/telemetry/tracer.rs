//! Tracer implementation for CloudWatch Gen AI Observability
//!
//! This module provides the `StoodTracer` and `StoodSpan` types for creating
//! and managing telemetry spans that comply with OTEL GenAI semantic conventions.

use super::exporter::{
    AttributeValue, CloudWatchExporter, SpanData, SpanEvent, SpanExporter, SpanKind, SpanStatus,
};
use super::genai::{attrs, GenAiOperation, GenAiProvider};
use super::log_event::LogEvent;
use super::session::Session;
use super::TelemetryConfig;
use crate::StoodError;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

// Re-export for backwards compatibility
pub use opentelemetry::{Context, KeyValue};

/// Baggage key for session ID - used by CloudWatch Gen AI Observability
/// to group spans into sessions in the dashboard
pub const SESSION_BAGGAGE_KEY: &str = "session.id";

/// Generate a random 128-bit trace ID as hex string
fn generate_trace_id() -> String {
    format!("{:032x}", fastrand::u128(..))
}

/// Generate a random 64-bit span ID as hex string
fn generate_span_id() -> String {
    format!("{:016x}", fastrand::u64(..))
}

/// Get current time in nanoseconds since Unix epoch
fn now_nanos() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}

/// Tracer for creating GenAI-compliant spans
///
/// The tracer manages span creation, context propagation, and export to
/// CloudWatch Gen AI Observability.
#[derive(Debug)]
pub struct StoodTracer {
    config: TelemetryConfig,
    exporter: Arc<dyn SpanExporter>,
    /// CloudWatch exporter (typed) for log event export
    /// This is the same exporter cast to CloudWatchExporter for log operations
    cloudwatch_exporter: Option<Arc<CloudWatchExporter>>,
    /// Current trace ID (shared across spans in a session)
    trace_id: Arc<Mutex<Option<String>>>,
    /// Pending spans waiting to be exported
    pending_spans: Arc<Mutex<Vec<SpanData>>>,
    /// Pending log events waiting to be exported (for AgentCore Evaluations)
    pending_log_events: Arc<Mutex<Vec<LogEvent>>>,
    /// Span counter for ordering
    span_counter: AtomicU64,
    /// Current session for conversation tracking
    /// Used by CloudWatch Gen AI Observability to group spans
    session: Arc<Mutex<Option<Session>>>,
}

impl Clone for StoodTracer {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            exporter: Arc::clone(&self.exporter),
            cloudwatch_exporter: self.cloudwatch_exporter.clone(),
            trace_id: Arc::clone(&self.trace_id),
            pending_spans: Arc::clone(&self.pending_spans),
            pending_log_events: Arc::clone(&self.pending_log_events),
            span_counter: AtomicU64::new(self.span_counter.load(Ordering::Relaxed)),
            session: Arc::clone(&self.session),
        }
    }
}

impl StoodTracer {
    /// Create a new tracer with the given configuration and exporter
    pub fn new(config: TelemetryConfig, exporter: Arc<dyn SpanExporter>) -> Self {
        Self {
            config,
            exporter,
            cloudwatch_exporter: None,
            trace_id: Arc::new(Mutex::new(None)),
            pending_spans: Arc::new(Mutex::new(Vec::new())),
            pending_log_events: Arc::new(Mutex::new(Vec::new())),
            span_counter: AtomicU64::new(0),
            session: Arc::new(Mutex::new(None)),
        }
    }

    /// Create a new tracer with a CloudWatch exporter that supports log events
    ///
    /// This version stores a typed reference to the CloudWatch exporter to enable
    /// log event export for AgentCore Evaluations.
    pub fn with_cloudwatch_exporter(
        config: TelemetryConfig,
        exporter: Arc<CloudWatchExporter>,
    ) -> Self {
        Self {
            config,
            exporter: exporter.clone() as Arc<dyn SpanExporter>,
            cloudwatch_exporter: Some(exporter),
            trace_id: Arc::new(Mutex::new(None)),
            pending_spans: Arc::new(Mutex::new(Vec::new())),
            pending_log_events: Arc::new(Mutex::new(Vec::new())),
            span_counter: AtomicU64::new(0),
            session: Arc::new(Mutex::new(None)),
        }
    }

    // ========================================================================
    // Session management for CloudWatch Gen AI Observability
    // ========================================================================

    /// Start a new session for conversation tracking
    ///
    /// This sets the session ID that will be included in all spans as both:
    /// - `session.id` in OTEL baggage (for CloudWatch session grouping)
    /// - `gen_ai.conversation.id` attribute (for conversation tracking)
    pub fn start_session(&self) -> Session {
        let session = Session::new();
        self.set_session(session.clone());
        session
    }

    /// Start a session with a specific conversation ID
    pub fn start_session_with_id(&self, conversation_id: impl Into<String>) -> Session {
        let session = Session::with_conversation_id(conversation_id);
        self.set_session(session.clone());
        session
    }

    /// Start a session with specific session and conversation IDs
    ///
    /// Use this when your application needs to control the session ID
    /// for traceability (e.g., to revisit traces later or correlate with
    /// an external session system).
    ///
    /// # Example
    /// ```ignore
    /// // Use your app's session ID
    /// let session = tracer.start_session_with_ids(
    ///     "user-session-abc123",
    ///     "conversation-xyz789"
    /// );
    /// ```
    pub fn start_session_with_ids(
        &self,
        session_id: impl Into<String>,
        conversation_id: impl Into<String>,
    ) -> Session {
        let session = Session::with_ids(session_id, conversation_id);
        self.set_session(session.clone());
        session
    }

    /// Set the current session
    pub fn set_session(&self, session: Session) {
        let mut s = self.session.lock().unwrap();
        *s = Some(session);
    }

    /// Get the current session, if any
    pub fn current_session(&self) -> Option<Session> {
        self.session.lock().unwrap().clone()
    }

    /// Get the current session ID, if any
    pub fn current_session_id(&self) -> Option<String> {
        self.session
            .lock()
            .unwrap()
            .as_ref()
            .map(|s| s.id().to_string())
    }

    /// Get the current conversation ID, if any
    pub fn current_conversation_id(&self) -> Option<String> {
        self.session
            .lock()
            .unwrap()
            .as_ref()
            .map(|s| s.conversation_id().to_string())
    }

    /// Clear the current session
    pub fn clear_session(&self) {
        let mut s = self.session.lock().unwrap();
        *s = None;
    }

    /// Initialize the tracer from configuration
    ///
    /// Returns `Ok(None)` if telemetry is disabled.
    /// Returns `Ok(Some(tracer))` if telemetry is enabled.
    ///
    /// **Important**: This is a synchronous initializer. For full GenAI Dashboard
    /// support (which requires creating CloudWatch Log Groups), use `init_async()` instead.
    pub fn init(config: TelemetryConfig) -> Result<Option<Self>, StoodError> {
        match &config {
            TelemetryConfig::Disabled { .. } => {
                tracing::debug!("Telemetry disabled, skipping tracer initialization");
                Ok(None)
            }
            TelemetryConfig::CloudWatch {
                region,
                credentials,
                service_name,
                service_version,
                agent_id,
                ..
            } => {
                // Derive agent_id from explicit config or fall back to service_name
                let effective_agent_id = agent_id
                    .as_ref()
                    .cloned()
                    .unwrap_or_else(|| service_name.clone());

                // Create CloudWatch exporter with configured credentials
                let exporter = Arc::new(
                    CloudWatchExporter::new(
                        region.clone(),
                        credentials.clone(),
                        service_name.clone(),
                        service_version.clone(),
                    )
                    .with_agent_id(&effective_agent_id),
                );

                tracing::info!(
                    "Telemetry tracer initialized with CloudWatch exporter (region: {}, service: {}, agent_id: {})",
                    region,
                    service_name,
                    effective_agent_id
                );

                // Note: Log group creation happens asynchronously in init_async()
                // For sync init, we log a warning if the user should be using init_async
                tracing::debug!(
                    "For full GenAI Dashboard support, ensure log group exists: /aws/bedrock-agentcore/runtimes/{}",
                    effective_agent_id
                );

                Ok(Some(Self::with_cloudwatch_exporter(config, exporter)))
            }
        }
    }

    /// Initialize the tracer asynchronously with log group creation
    ///
    /// This is the recommended initialization method for CloudWatch GenAI Observability.
    /// It ensures the required log group exists before starting telemetry export.
    ///
    /// # Log Group Requirement
    ///
    /// For spans to appear in the CloudWatch GenAI Observability Dashboard,
    /// the log group `/aws/bedrock-agentcore/runtimes/{agent_id}` MUST exist.
    /// This method automatically creates it if it doesn't exist.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = TelemetryConfig::cloudwatch("us-east-1")
    ///     .with_service_name("my-service")
    ///     .with_agent_id("my-agent-001");
    ///
    /// let tracer = StoodTracer::init_async(config).await?;
    /// ```
    pub async fn init_async(config: TelemetryConfig) -> Result<Option<Self>, StoodError> {
        crate::perf_checkpoint!("stood.tracer.init_async.start");
        let _init_guard = crate::perf_guard!("stood.tracer.init_async");

        match &config {
            TelemetryConfig::Disabled { .. } => {
                tracing::debug!("Telemetry disabled, skipping tracer initialization");
                crate::perf_checkpoint!("stood.tracer.init_async.disabled");
                Ok(None)
            }
            TelemetryConfig::CloudWatch {
                region,
                credentials,
                service_name,
                service_version,
                agent_id,
                skip_log_group_check,
                ..
            } => {
                crate::perf_checkpoint!("stood.tracer.init_async.cloudwatch", &format!("region={}", region));

                // Derive agent_id from explicit config or fall back to service_name
                let effective_agent_id = agent_id
                    .as_ref()
                    .cloned()
                    .unwrap_or_else(|| service_name.clone());

                // CRITICAL: Create log group BEFORE initializing exporter
                // This is required for spans to appear in the GenAI Dashboard
                // Skip if skip_log_group_check is set (log groups pre-created at startup)
                if *skip_log_group_check {
                    crate::perf_checkpoint!("stood.tracer.log_group.skipped", &effective_agent_id);
                    tracing::info!(
                        "SKIP_LOG_GROUP_CHECK: Skipping log group check for agent_id={} (pre-created at startup)",
                        effective_agent_id
                    );
                } else {
                    tracing::info!(
                        "LOG_GROUP_CHECK: Performing log group check for agent_id={}",
                        effective_agent_id
                    );
                    crate::perf_checkpoint!("stood.tracer.log_group.start");
                    let log_group_result = crate::perf_timed!("stood.tracer.log_group_manager_new", {
                        super::log_group::LogGroupManager::new(region.clone()).await
                    });
                    match log_group_result {
                        Ok(manager) => {
                            let log_group_config =
                                super::log_group::AgentLogGroup::new(&effective_agent_id);
                            let ensure_result = crate::perf_timed!("stood.tracer.log_group_ensure_exists", {
                                manager.ensure_exists(&log_group_config).await
                            });
                            match ensure_result {
                                Ok(created) => {
                                    if created {
                                        crate::perf_checkpoint!("stood.tracer.log_group.created", &log_group_config.log_group_name());
                                        tracing::info!(
                                            "Created CloudWatch log group: {}",
                                            log_group_config.log_group_name()
                                        );
                                    } else {
                                        crate::perf_checkpoint!("stood.tracer.log_group.exists", &log_group_config.log_group_name());
                                        tracing::debug!(
                                            "CloudWatch log group already exists: {}",
                                            log_group_config.log_group_name()
                                        );
                                    }
                                }
                                Err(e) => {
                                    crate::perf_checkpoint!("stood.tracer.log_group.error", &e.to_string());
                                    // Log warning but continue - telemetry will still work,
                                    // just won't appear in GenAI Dashboard
                                    tracing::warn!(
                                        "Failed to create log group '{}': {}. Spans may not appear in GenAI Dashboard.",
                                        log_group_config.log_group_name(),
                                        e
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            crate::perf_checkpoint!("stood.tracer.log_group_manager.error", &e.to_string());
                            tracing::warn!(
                                "Failed to initialize log group manager: {}. Spans may not appear in GenAI Dashboard.",
                                e
                            );
                        }
                    }
                    crate::perf_checkpoint!("stood.tracer.log_group.end");
                }

                // Create CloudWatch exporter with configured credentials
                let exporter = crate::perf_timed!("stood.tracer.cloudwatch_exporter_new", {
                    Arc::new(
                        CloudWatchExporter::new(
                            region.clone(),
                            credentials.clone(),
                            service_name.clone(),
                            service_version.clone(),
                        )
                        .with_agent_id(&effective_agent_id),
                    )
                });

                tracing::info!(
                    "Telemetry tracer initialized with CloudWatch exporter (region: {}, service: {}, agent_id: {})",
                    region,
                    service_name,
                    effective_agent_id
                );

                crate::perf_checkpoint!("stood.tracer.init_async.end");
                Ok(Some(Self::with_cloudwatch_exporter(config, exporter)))
            }
        }
    }

    /// Initialize tracing subscriber for log correlation
    pub fn init_tracing_subscriber() -> Result<(), StoodError> {
        // This will be implemented when we add log correlation
        Ok(())
    }

    /// Get or create the current trace ID
    fn get_or_create_trace_id(&self) -> String {
        let mut trace_id = self.trace_id.lock().unwrap();
        if trace_id.is_none() {
            *trace_id = Some(generate_trace_id());
        }
        trace_id.clone().unwrap()
    }

    /// Start a new trace (resets trace ID)
    pub fn start_trace(&self) {
        let mut trace_id = self.trace_id.lock().unwrap();
        *trace_id = Some(generate_trace_id());
    }

    /// Get the current trace ID, if any
    pub fn current_trace_id(&self) -> Option<String> {
        self.trace_id.lock().unwrap().clone()
    }

    // ========================================================================
    // GenAI-compliant span creation methods
    // ========================================================================

    /// Start a chat completion span
    ///
    /// Span name: "chat {model}"
    pub fn start_chat_span(&self, model: &str) -> StoodSpan {
        let mut span = self.create_span(GenAiOperation::Chat.span_name(model), SpanKind::Client);
        span.set_attribute(attrs::OPERATION_NAME, GenAiOperation::Chat.as_str());
        span.set_attribute(attrs::PROVIDER_NAME, GenAiProvider::AwsBedrock.as_str());
        span.set_attribute(attrs::REQUEST_MODEL, model);
        // AWS CloudWatch GenAI Dashboard attributes
        span.set_attribute(attrs::AWS_XRAY_ORIGIN, "AWS::BedrockAgentCore::Runtime");
        self.add_session_attributes(&mut span);
        span
    }

    /// Start an agent invocation span
    ///
    /// Span name: "invoke_agent {agent_name}"
    pub fn start_invoke_agent_span(&self, agent_name: &str, agent_id: Option<&str>) -> StoodSpan {
        let mut span = self.create_span(
            GenAiOperation::InvokeAgent.span_name(agent_name),
            SpanKind::Internal,
        );
        span.set_attribute(attrs::OPERATION_NAME, GenAiOperation::InvokeAgent.as_str());
        span.set_attribute(attrs::PROVIDER_NAME, GenAiProvider::AwsBedrock.as_str());
        span.set_attribute(attrs::AGENT_NAME, agent_name);
        if let Some(id) = agent_id {
            span.set_attribute(attrs::AGENT_ID, id);
        }
        // AWS CloudWatch GenAI Dashboard attributes
        span.set_attribute(attrs::AWS_XRAY_ORIGIN, "AWS::BedrockAgentCore::Runtime");
        // LangChain/Traceloop attribute required for AgentCore Evaluations
        span.set_attribute("traceloop.span.kind", "workflow");
        self.add_session_attributes(&mut span);
        span
    }

    /// Start a tool execution span
    ///
    /// Span name: "execute_tool {tool_name}"
    pub fn start_execute_tool_span(
        &self,
        tool_name: &str,
        tool_call_id: Option<&str>,
    ) -> StoodSpan {
        let mut span = self.create_span(
            GenAiOperation::ExecuteTool.span_name(tool_name),
            SpanKind::Internal,
        );
        span.set_attribute(attrs::OPERATION_NAME, GenAiOperation::ExecuteTool.as_str());
        span.set_attribute(attrs::PROVIDER_NAME, GenAiProvider::AwsBedrock.as_str());
        span.set_attribute(attrs::TOOL_NAME, tool_name);
        span.set_attribute(
            attrs::TOOL_TYPE,
            super::genai::GenAiToolType::Function.as_str(),
        );
        if let Some(id) = tool_call_id {
            span.set_attribute(attrs::TOOL_CALL_ID, id);
        }
        // AWS CloudWatch GenAI Dashboard attributes
        span.set_attribute(attrs::AWS_XRAY_ORIGIN, "AWS::BedrockAgentCore::Runtime");
        self.add_session_attributes(&mut span);
        span
    }

    /// Add session attributes to a span (session.id for CloudWatch GenAI Dashboard)
    fn add_session_attributes(&self, span: &mut StoodSpan) {
        if let Ok(session) = self.session.lock() {
            if let Some(ref s) = *session {
                // session.id is required by CloudWatch GenAI Dashboard query
                span.set_attribute(attrs::SESSION_ID, s.id());
                // Also set gen_ai.conversation.id for OTEL compliance
                span.set_attribute(attrs::CONVERSATION_ID, s.conversation_id());
            }
        }
    }

    // ========================================================================
    // Backwards-compatible span creation methods
    // ========================================================================

    /// Create a generic span (backwards compatibility)
    pub fn start_span(&self, name: &str) -> StoodSpan {
        self.create_span(name.to_string(), SpanKind::Internal)
    }

    /// Create an agent span (backwards compatibility)
    pub fn start_agent_span(&self, name: &str) -> StoodSpan {
        self.start_invoke_agent_span(name, None)
    }

    /// Create a cycle span with dynamic attributes (backwards compatibility)
    pub fn start_cycle_span_with_dynamic_attributes(
        &self,
        name: &str,
        cycle_id: &str,
        attributes: Vec<(&str, String)>,
        parent_context: Option<Context>,
    ) -> StoodSpan {
        let mut span =
            self.create_span_with_parent(name.to_string(), SpanKind::Internal, parent_context);
        span.set_attribute(attrs::STOOD_CYCLE_ID, cycle_id);
        for (key, value) in attributes {
            span.set_attribute(key, value);
        }
        span
    }

    /// Create a model span with dynamic attributes (backwards compatibility)
    pub fn start_model_span_with_dynamic_attributes(
        &self,
        name: &str,
        model: &str,
        attributes: Vec<(&str, String)>,
        parent_context: Option<Context>,
    ) -> StoodSpan {
        let mut span =
            self.create_span_with_parent(name.to_string(), SpanKind::Client, parent_context);
        span.set_attribute(attrs::REQUEST_MODEL, model);
        span.set_attribute(attrs::OPERATION_NAME, GenAiOperation::Chat.as_str());
        span.set_attribute(attrs::PROVIDER_NAME, GenAiProvider::AwsBedrock.as_str());
        for (key, value) in attributes {
            span.set_attribute(key, value);
        }
        span
    }

    /// Create a tool span (backwards compatibility)
    pub fn start_tool_span(&self, name: &str) -> StoodSpan {
        self.start_execute_tool_span(name, None)
    }

    /// Create a tool span with parent context (backwards compatibility)
    pub fn start_tool_span_with_parent_context(
        &self,
        name: &str,
        parent_context: &Context,
    ) -> StoodSpan {
        let mut span = self.create_span_with_parent(
            GenAiOperation::ExecuteTool.span_name(name),
            SpanKind::Internal,
            Some(parent_context.clone()),
        );
        span.set_attribute(attrs::OPERATION_NAME, GenAiOperation::ExecuteTool.as_str());
        span.set_attribute(attrs::TOOL_NAME, name);
        span
    }

    /// Create a tool span with name and parent context (backwards compatibility)
    pub fn start_tool_span_with_name_and_parent(
        &self,
        tool_name: &str,
        parent_context: Option<Context>,
    ) -> StoodSpan {
        let mut span = self.create_span_with_parent(
            GenAiOperation::ExecuteTool.span_name(tool_name),
            SpanKind::Internal,
            parent_context,
        );
        span.set_attribute(attrs::OPERATION_NAME, GenAiOperation::ExecuteTool.as_str());
        span.set_attribute(attrs::TOOL_NAME, tool_name);
        span
    }

    /// Create a model span (backwards compatibility)
    pub fn start_model_span(&self, name: &str) -> StoodSpan {
        self.start_chat_span(name)
    }

    // ========================================================================
    // Internal helpers
    // ========================================================================

    fn create_span(&self, name: String, kind: SpanKind) -> StoodSpan {
        self.create_span_with_parent(name, kind, None)
    }

    fn create_span_with_parent(
        &self,
        name: String,
        kind: SpanKind,
        _parent_context: Option<Context>,
    ) -> StoodSpan {
        let trace_id = self.get_or_create_trace_id();
        let span_id = generate_span_id();
        let start_time = now_nanos();
        let order = self.span_counter.fetch_add(1, Ordering::Relaxed);

        // Initialize span attributes with session info if available
        // This is critical for CloudWatch Gen AI Observability to recognize spans
        let mut attributes: HashMap<String, AttributeValue> = HashMap::new();

        // Note: gen_ai.provider.name is set per-span type (chat, invoke_agent, execute_tool)
        // per OpenTelemetry AWS Bedrock semantic conventions

        // Add session attributes for CloudWatch Gen AI dashboard
        if let Some(session) = self.current_session() {
            // session.id - Required for CloudWatch to group spans into sessions
            attributes.insert(
                SESSION_BAGGAGE_KEY.to_string(),
                AttributeValue::String(session.id().to_string()),
            );

            // gen_ai.conversation.id - Required for conversation tracking
            attributes.insert(
                attrs::CONVERSATION_ID.to_string(),
                AttributeValue::String(session.conversation_id().to_string()),
            );

            // gen_ai.agent.id - Recommended for agent identification
            if let Some(agent_id) = session.agent_id() {
                attributes.insert(
                    attrs::AGENT_ID.to_string(),
                    AttributeValue::String(agent_id.to_string()),
                );
            }

            // gen_ai.agent.name - Recommended for agent identification
            if let Some(agent_name) = session.agent_name() {
                attributes.insert(
                    attrs::AGENT_NAME.to_string(),
                    AttributeValue::String(agent_name.to_string()),
                );
            }
        }

        StoodSpan {
            data: SpanData {
                trace_id,
                span_id,
                parent_span_id: None, // TODO: extract from parent context
                name,
                kind,
                start_time_unix_nano: start_time,
                end_time_unix_nano: 0,
                attributes,
                status: SpanStatus::Unset,
                events: Vec::new(),
            },
            pending_spans: Arc::clone(&self.pending_spans),
            exporter: Arc::clone(&self.exporter),
            order,
            finished: false,
            context: Context::current(),
        }
    }

    // ========================================================================
    // Log Event Management (for AgentCore Evaluations)
    // ========================================================================

    /// Queue a log event for export
    ///
    /// Log events contain the prompt/response content that AgentCore Evaluations
    /// like Correctness and Conciseness require. They are exported to CloudWatch
    /// Logs alongside spans.
    pub fn queue_log_event(&self, event: LogEvent) {
        if let Ok(mut pending) = self.pending_log_events.lock() {
            pending.push(event);
        }
    }

    /// Create and queue a log event for an agent invocation
    ///
    /// This is a convenience method that creates a log event with the given
    /// prompt and response content and queues it for export.
    pub fn queue_agent_invocation_log(
        &self,
        trace_id: &str,
        span_id: &str,
        system_prompt: Option<&str>,
        user_prompt: &str,
        assistant_response: &str,
    ) {
        let session_id = self
            .current_session_id()
            .unwrap_or_else(|| "unknown".to_string());

        let event = LogEvent::for_agent_invocation(
            trace_id,
            span_id,
            session_id,
            system_prompt,
            user_prompt,
            assistant_response,
        );

        self.queue_log_event(event);
    }

    /// Create and queue a log event for an agent invocation with tool results
    ///
    /// This variant includes tool execution results in the conversation history,
    /// which is required for AgentCore Faithfulness evaluation to verify that
    /// the assistant's response is grounded in the tool outputs.
    pub fn queue_agent_invocation_with_tools_log(
        &self,
        trace_id: &str,
        span_id: &str,
        system_prompt: Option<&str>,
        user_prompt: &str,
        tool_results: &[(String, String, String)],
        assistant_response: &str,
    ) {
        let session_id = self
            .current_session_id()
            .unwrap_or_else(|| "unknown".to_string());

        let event = LogEvent::for_agent_invocation_with_tools(
            trace_id,
            span_id,
            session_id,
            system_prompt,
            user_prompt,
            tool_results,
            assistant_response,
        );

        self.queue_log_event(event);
    }

    /// Create and queue a log event for a tool execution
    ///
    /// This is required for AgentCore Evaluations - each tool span with
    /// scope "strands.telemetry.tracer" must have a corresponding log event.
    pub fn queue_tool_execution_log(
        &self,
        trace_id: &str,
        span_id: &str,
        tool_name: &str,
        tool_input: &str,
        tool_output: &str,
    ) {
        let session_id = self
            .current_session_id()
            .unwrap_or_else(|| "unknown".to_string());

        let event = LogEvent::for_tool_execution(
            trace_id,
            span_id,
            session_id,
            tool_name,
            tool_input,
            tool_output,
        );

        self.queue_log_event(event);
    }

    /// Create and queue a log event for a chat/model completion
    ///
    /// This is required for AgentCore Evaluations - each chat span with
    /// scope "strands.telemetry.tracer" must have a corresponding log event.
    /// The event captures the user input and model response for evaluation.
    pub fn queue_chat_completion_log(
        &self,
        trace_id: &str,
        span_id: &str,
        model: &str,
        user_input: &str,
        assistant_output: &str,
    ) {
        let session_id = self
            .current_session_id()
            .unwrap_or_else(|| "unknown".to_string());

        let event = LogEvent::for_chat_completion(
            trace_id,
            span_id,
            session_id,
            model,
            user_input,
            assistant_output,
        );

        self.queue_log_event(event);
    }

    /// Get the number of pending log events
    pub fn pending_log_events_count(&self) -> usize {
        self.pending_log_events.lock().map(|p| p.len()).unwrap_or(0)
    }

    // ========================================================================
    // Export and Shutdown
    // ========================================================================

    /// Shutdown the tracer and flush pending spans and log events
    pub fn shutdown(&self) {
        // Flush any pending spans
        let spans = {
            let mut pending = self.pending_spans.lock().unwrap();
            std::mem::take(&mut *pending)
        };

        // Flush any pending log events
        let log_events = {
            let mut pending = self.pending_log_events.lock().unwrap();
            std::mem::take(&mut *pending)
        };

        if !spans.is_empty() {
            let exporter = Arc::clone(&self.exporter);
            // Fire-and-forget export in background
            tokio::spawn(async move {
                if let Err(e) = exporter.export(spans).await {
                    tracing::warn!("Failed to export spans during shutdown: {}", e);
                }
            });
        }

        if !log_events.is_empty() {
            if let Some(ref cw_exporter) = self.cloudwatch_exporter {
                let exporter = Arc::clone(cw_exporter);
                tokio::spawn(async move {
                    if let Err(e) = exporter.export_logs(log_events).await {
                        tracing::warn!("Failed to export log events during shutdown: {}", e);
                    }
                });
            }
        }

        tracing::debug!("StoodTracer shutdown complete");
    }

    /// Export pending spans and log events
    pub async fn flush(&self) -> Result<(), super::exporter::ExportError> {
        // Export spans
        let spans = {
            let mut pending = self.pending_spans.lock().unwrap();
            std::mem::take(&mut *pending)
        };

        tracing::debug!("Flushing {} pending spans", spans.len());

        if !spans.is_empty() {
            tracing::info!("Exporting {} spans to CloudWatch/X-Ray", spans.len());
            self.exporter.export(spans).await?;
            tracing::debug!("Spans exported successfully");
        }

        // Export log events
        let log_events = {
            let mut pending = self.pending_log_events.lock().unwrap();
            std::mem::take(&mut *pending)
        };

        tracing::debug!("Flushing {} pending log events", log_events.len());

        if !log_events.is_empty() {
            if let Some(ref cw_exporter) = self.cloudwatch_exporter {
                tracing::info!("Exporting {} log events to CloudWatch Logs", log_events.len());
                cw_exporter.export_logs(log_events).await?;
                tracing::debug!("Log events exported successfully");
            }
        }

        Ok(())
    }

    /// Check if the tracer is healthy
    pub fn is_healthy(&self) -> bool {
        self.exporter.is_healthy()
    }

    /// Get the telemetry configuration
    pub fn config(&self) -> &TelemetryConfig {
        &self.config
    }

    /// Check if log event export is available
    ///
    /// Returns true if a CloudWatch exporter is configured that can export
    /// log events for AgentCore Evaluations.
    pub fn can_export_log_events(&self) -> bool {
        self.cloudwatch_exporter.is_some()
    }
}

/// A span representing a unit of work
///
/// Spans are created by the tracer and automatically exported when finished.
#[derive(Debug)]
pub struct StoodSpan {
    data: SpanData,
    pending_spans: Arc<Mutex<Vec<SpanData>>>,
    #[allow(dead_code)] // Will be used in Milestone 4 for span export
    exporter: Arc<dyn SpanExporter>,
    #[allow(dead_code)] // Will be used in Milestone 4 for span ordering
    order: u64,
    finished: bool,
    context: Context,
}

impl StoodSpan {
    /// Create a new no-op span (for disabled telemetry)
    pub fn new() -> Self {
        Self {
            data: SpanData {
                trace_id: String::new(),
                span_id: String::new(),
                parent_span_id: None,
                name: String::new(),
                kind: SpanKind::Internal,
                start_time_unix_nano: 0,
                end_time_unix_nano: 0,
                attributes: HashMap::new(),
                status: SpanStatus::Unset,
                events: Vec::new(),
            },
            pending_spans: Arc::new(Mutex::new(Vec::new())),
            exporter: Arc::new(super::exporter::NoOpExporter),
            order: 0,
            finished: true, // Mark as finished so it's never exported
            context: Context::current(),
        }
    }

    /// Get the OpenTelemetry context for this span (for context propagation)
    pub fn context(&self) -> Context {
        self.context.clone()
    }

    /// Get the trace ID
    pub fn trace_id(&self) -> &str {
        &self.data.trace_id
    }

    /// Get the span ID
    pub fn span_id(&self) -> &str {
        &self.data.span_id
    }

    /// Set an attribute on the span
    pub fn set_attribute(&mut self, key: &str, value: impl Into<AttributeValue>) {
        if !self.finished {
            self.data.attributes.insert(key.to_string(), value.into());
        }
    }

    /// Set a string attribute
    pub fn set_string_attribute(&mut self, key: &str, value: impl ToString) {
        self.set_attribute(key, AttributeValue::String(value.to_string()));
    }

    /// Set an integer attribute
    pub fn set_int_attribute(&mut self, key: &str, value: i64) {
        self.set_attribute(key, AttributeValue::Int(value));
    }

    /// Set a float attribute
    pub fn set_float_attribute(&mut self, key: &str, value: f64) {
        self.set_attribute(key, AttributeValue::Float(value));
    }

    /// Set a boolean attribute
    pub fn set_bool_attribute(&mut self, key: &str, value: bool) {
        self.set_attribute(key, AttributeValue::Bool(value));
    }

    /// Record an error on the span
    pub fn record_error(&mut self, error: &str) {
        if !self.finished {
            self.data.status = SpanStatus::Error {
                message: error.to_string(),
            };
            self.add_event(
                "exception",
                vec![KeyValue::new("exception.message", error.to_string())],
            );
        }
    }

    /// Set error status on the span
    pub fn set_error(&mut self, error: &str) {
        self.record_error(error);
    }

    /// Mark span as successful
    pub fn set_success(&mut self) {
        if !self.finished {
            self.data.status = SpanStatus::Ok;
        }
    }

    /// Add an event to the span
    pub fn add_event(&mut self, name: &str, attributes: Vec<KeyValue>) {
        if !self.finished {
            let event_attrs: HashMap<String, AttributeValue> = attributes
                .into_iter()
                .map(|kv| {
                    let key = kv.key.to_string();
                    let value = match &kv.value {
                        opentelemetry::Value::String(s) => AttributeValue::String(s.to_string()),
                        opentelemetry::Value::Bool(b) => AttributeValue::Bool(*b),
                        opentelemetry::Value::I64(i) => AttributeValue::Int(*i),
                        opentelemetry::Value::F64(f) => AttributeValue::Float(*f),
                        _ => AttributeValue::String(format!("{:?}", kv.value)),
                    };
                    (key, value)
                })
                .collect();

            self.data.events.push(SpanEvent {
                name: name.to_string(),
                time_unix_nano: now_nanos(),
                attributes: event_attrs,
            });
        }
    }

    /// Record token usage
    pub fn record_tokens(&mut self, input_tokens: u32, output_tokens: u32) {
        self.set_int_attribute(attrs::USAGE_INPUT_TOKENS, input_tokens as i64);
        self.set_int_attribute(attrs::USAGE_OUTPUT_TOKENS, output_tokens as i64);
    }

    /// Record response metadata
    pub fn record_response(&mut self, response_id: &str, finish_reasons: &[&str]) {
        self.set_string_attribute(attrs::RESPONSE_ID, response_id);
        if !finish_reasons.is_empty() {
            self.set_attribute(
                attrs::RESPONSE_FINISH_REASONS,
                AttributeValue::StringArray(finish_reasons.iter().map(|s| s.to_string()).collect()),
            );
        }
    }

    /// Finish the span and queue for export
    pub fn finish(mut self) {
        if !self.finished {
            self.data.end_time_unix_nano = now_nanos();
            self.finished = true;

            // Queue for export
            if let Ok(mut pending) = self.pending_spans.lock() {
                pending.push(self.data.clone());
            }
        }
    }

    /// Finish the span with a specific duration
    ///
    /// Use this when you know the actual execution duration (e.g., from tool execution metrics)
    /// and want the span timeline to accurately reflect when the operation completed.
    ///
    /// The end time is calculated as: start_time + duration
    pub fn finish_with_duration(mut self, duration: std::time::Duration) {
        if !self.finished {
            // Calculate end time from start time + actual duration
            self.data.end_time_unix_nano =
                self.data.start_time_unix_nano + duration.as_nanos() as u64;
            self.finished = true;

            // Queue for export
            if let Ok(mut pending) = self.pending_spans.lock() {
                pending.push(self.data.clone());
            }
        }
    }

    /// End the span (alias for finish)
    pub fn end(self) {
        self.finish();
    }
}

impl Default for StoodSpan {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for StoodSpan {
    fn drop(&mut self) {
        // Auto-finish on drop if not already finished
        if !self.finished && !self.data.trace_id.is_empty() {
            self.data.end_time_unix_nano = now_nanos();
            self.finished = true;
            if let Ok(mut pending) = self.pending_spans.lock() {
                pending.push(self.data.clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracer_disabled_returns_none() {
        let config = TelemetryConfig::default();
        let result = StoodTracer::init(config).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_chat_span_has_correct_attributes() {
        let config = TelemetryConfig::cloudwatch("us-east-1");
        let exporter = Arc::new(super::super::exporter::NoOpExporter);
        let tracer = StoodTracer::new(config, exporter);

        let span = tracer.start_chat_span("claude-3-haiku");
        assert!(span.data.name.contains("chat"));
        assert!(span.data.name.contains("claude-3-haiku"));
        assert_eq!(
            span.data.attributes.get(attrs::OPERATION_NAME),
            Some(&AttributeValue::String("chat".to_string()))
        );
        assert_eq!(
            span.data.attributes.get(attrs::PROVIDER_NAME),
            Some(&AttributeValue::String("aws.bedrock".to_string()))
        );
    }

    #[test]
    fn test_tool_span_has_correct_attributes() {
        let config = TelemetryConfig::cloudwatch("us-east-1");
        let exporter = Arc::new(super::super::exporter::NoOpExporter);
        let tracer = StoodTracer::new(config, exporter);

        let span = tracer.start_execute_tool_span("get_weather", Some("tool-123"));
        assert!(span.data.name.contains("execute_tool"));
        assert!(span.data.name.contains("get_weather"));
        assert_eq!(
            span.data.attributes.get(attrs::TOOL_NAME),
            Some(&AttributeValue::String("get_weather".to_string()))
        );
        assert_eq!(
            span.data.attributes.get(attrs::TOOL_CALL_ID),
            Some(&AttributeValue::String("tool-123".to_string()))
        );
    }

    #[test]
    fn test_agent_span_has_correct_attributes() {
        let config = TelemetryConfig::cloudwatch("us-east-1");
        let exporter = Arc::new(super::super::exporter::NoOpExporter);
        let tracer = StoodTracer::new(config, exporter);

        let span = tracer.start_invoke_agent_span("my-agent", Some("agent-456"));
        assert!(span.data.name.contains("invoke_agent"));
        assert!(span.data.name.contains("my-agent"));
        assert_eq!(
            span.data.attributes.get(attrs::AGENT_NAME),
            Some(&AttributeValue::String("my-agent".to_string()))
        );
        assert_eq!(
            span.data.attributes.get(attrs::AGENT_ID),
            Some(&AttributeValue::String("agent-456".to_string()))
        );
    }

    #[test]
    fn test_span_token_recording() {
        let config = TelemetryConfig::cloudwatch("us-east-1");
        let exporter = Arc::new(super::super::exporter::NoOpExporter);
        let tracer = StoodTracer::new(config, exporter);
        let mut span = tracer.start_chat_span("test-model");

        span.record_tokens(100, 50);

        assert_eq!(
            span.data.attributes.get(attrs::USAGE_INPUT_TOKENS),
            Some(&AttributeValue::Int(100))
        );
        assert_eq!(
            span.data.attributes.get(attrs::USAGE_OUTPUT_TOKENS),
            Some(&AttributeValue::Int(50))
        );
    }

    #[test]
    fn test_span_error_recording() {
        let config = TelemetryConfig::cloudwatch("us-east-1");
        let exporter = Arc::new(super::super::exporter::NoOpExporter);
        let tracer = StoodTracer::new(config, exporter);
        let mut span = tracer.start_chat_span("test-model");

        span.record_error("Something went wrong");

        assert!(matches!(span.data.status, SpanStatus::Error { .. }));
        assert_eq!(span.data.events.len(), 1);
        assert_eq!(span.data.events[0].name, "exception");
    }

    #[test]
    fn test_trace_id_persistence() {
        let config = TelemetryConfig::cloudwatch("us-east-1");
        let exporter = Arc::new(super::super::exporter::NoOpExporter);
        let tracer = StoodTracer::new(config, exporter);

        let span1 = tracer.start_chat_span("model1");
        let span2 = tracer.start_chat_span("model2");

        // Both spans should share the same trace ID
        assert_eq!(span1.trace_id(), span2.trace_id());
        assert!(!span1.trace_id().is_empty());
    }

    #[test]
    fn test_span_ids_are_unique() {
        let config = TelemetryConfig::cloudwatch("us-east-1");
        let exporter = Arc::new(super::super::exporter::NoOpExporter);
        let tracer = StoodTracer::new(config, exporter);

        let span1 = tracer.start_chat_span("model1");
        let span2 = tracer.start_chat_span("model2");

        assert_ne!(span1.span_id(), span2.span_id());
    }

    #[test]
    fn test_new_trace_resets_trace_id() {
        let config = TelemetryConfig::cloudwatch("us-east-1");
        let exporter = Arc::new(super::super::exporter::NoOpExporter);
        let tracer = StoodTracer::new(config, exporter);

        let span1 = tracer.start_chat_span("model1");
        let trace_id_1 = span1.trace_id().to_string();

        tracer.start_trace();

        let span2 = tracer.start_chat_span("model2");
        let trace_id_2 = span2.trace_id().to_string();

        assert_ne!(trace_id_1, trace_id_2);
    }

    #[test]
    fn test_session_creates_with_id() {
        let config = TelemetryConfig::cloudwatch("us-east-1");
        let exporter = Arc::new(super::super::exporter::NoOpExporter);
        let tracer = StoodTracer::new(config, exporter);

        let session = tracer.start_session();
        assert!(!session.id().is_empty());
        assert!(!session.conversation_id().is_empty());
        assert!(tracer.current_session().is_some());
    }

    #[test]
    fn test_span_includes_session_attributes() {
        let config = TelemetryConfig::cloudwatch("us-east-1");
        let exporter = Arc::new(super::super::exporter::NoOpExporter);
        let tracer = StoodTracer::new(config, exporter);

        // Start a session with agent info
        let mut session = tracer.start_session();
        session.set_agent_name("test-agent");
        session.set_agent_id("agent-123");
        tracer.set_session(session.clone());

        // Create a span - should inherit session attributes
        let span = tracer.start_chat_span("claude-haiku");

        // Verify session.id is set
        assert_eq!(
            span.data.attributes.get(SESSION_BAGGAGE_KEY),
            Some(&AttributeValue::String(session.id().to_string()))
        );

        // Verify gen_ai.conversation.id is set
        assert_eq!(
            span.data.attributes.get(attrs::CONVERSATION_ID),
            Some(&AttributeValue::String(
                session.conversation_id().to_string()
            ))
        );

        // Verify gen_ai.agent.id is set
        assert_eq!(
            span.data.attributes.get(attrs::AGENT_ID),
            Some(&AttributeValue::String("agent-123".to_string()))
        );

        // Verify gen_ai.agent.name is set
        assert_eq!(
            span.data.attributes.get(attrs::AGENT_NAME),
            Some(&AttributeValue::String("test-agent".to_string()))
        );
    }

    #[test]
    fn test_span_without_session_has_no_session_attributes() {
        let config = TelemetryConfig::cloudwatch("us-east-1");
        let exporter = Arc::new(super::super::exporter::NoOpExporter);
        let tracer = StoodTracer::new(config, exporter);

        // Don't start a session
        let span = tracer.start_chat_span("claude-haiku");

        // Verify session.id is NOT set
        assert!(span.data.attributes.get(SESSION_BAGGAGE_KEY).is_none());

        // Verify gen_ai.conversation.id is NOT set
        assert!(span.data.attributes.get(attrs::CONVERSATION_ID).is_none());
    }

    #[test]
    fn test_clear_session_removes_session_from_spans() {
        let config = TelemetryConfig::cloudwatch("us-east-1");
        let exporter = Arc::new(super::super::exporter::NoOpExporter);
        let tracer = StoodTracer::new(config, exporter);

        // Start and then clear session
        tracer.start_session();
        tracer.clear_session();

        // Create a span - should NOT have session attributes
        let span = tracer.start_chat_span("claude-haiku");
        assert!(span.data.attributes.get(SESSION_BAGGAGE_KEY).is_none());
    }
}
