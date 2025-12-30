//! Span exporter traits and implementations
//!
//! This module defines the `SpanExporter` trait for exporting telemetry spans
//! to various backends. The CloudWatch implementation will be added in Milestone 3.

use async_trait::async_trait;
use std::collections::HashMap;
use std::time::Duration;

/// Error during span export
#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    /// Authentication failed
    #[error("Authentication failed: {0}")]
    Auth(String),

    /// Network error during export
    #[error("Network error: {0}")]
    Network(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Rate limited by the backend
    #[error("Rate limited")]
    RateLimited,

    /// Export timeout
    #[error("Export timeout after {0:?}")]
    Timeout(Duration),

    /// Backend returned an error
    #[error("Backend error: {status_code} - {message}")]
    Backend { status_code: u16, message: String },
}

/// Data for a single span to be exported
#[derive(Debug, Clone)]
pub struct SpanData {
    /// Unique trace ID (128-bit hex string)
    pub trace_id: String,
    /// Unique span ID (64-bit hex string)
    pub span_id: String,
    /// Parent span ID (if this is a child span)
    pub parent_span_id: Option<String>,
    /// Operation name (e.g., "chat claude-3-haiku")
    pub name: String,
    /// Span kind (client, server, internal, etc.)
    pub kind: SpanKind,
    /// Start time in nanoseconds since Unix epoch
    pub start_time_unix_nano: u64,
    /// End time in nanoseconds since Unix epoch
    pub end_time_unix_nano: u64,
    /// Span attributes (key-value pairs)
    pub attributes: HashMap<String, AttributeValue>,
    /// Span status
    pub status: SpanStatus,
    /// Events that occurred during the span
    pub events: Vec<SpanEvent>,
}

/// Kind of span
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanKind {
    /// Internal operation
    Internal,
    /// Client making a request
    Client,
    /// Server handling a request
    Server,
    /// Producer sending a message
    Producer,
    /// Consumer receiving a message
    Consumer,
}

/// Status of a span
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpanStatus {
    /// Unset status
    Unset,
    /// Operation completed successfully
    Ok,
    /// Operation failed with an error
    Error { message: String },
}

/// Value of a span attribute
#[derive(Debug, Clone, PartialEq)]
pub enum AttributeValue {
    /// String value
    String(String),
    /// Boolean value
    Bool(bool),
    /// Integer value
    Int(i64),
    /// Float value
    Float(f64),
    /// Array of strings
    StringArray(Vec<String>),
    /// Array of integers
    IntArray(Vec<i64>),
}

impl From<String> for AttributeValue {
    fn from(s: String) -> Self {
        AttributeValue::String(s)
    }
}

impl From<&str> for AttributeValue {
    fn from(s: &str) -> Self {
        AttributeValue::String(s.to_string())
    }
}

impl From<bool> for AttributeValue {
    fn from(b: bool) -> Self {
        AttributeValue::Bool(b)
    }
}

impl From<i64> for AttributeValue {
    fn from(i: i64) -> Self {
        AttributeValue::Int(i)
    }
}

impl From<i32> for AttributeValue {
    fn from(i: i32) -> Self {
        AttributeValue::Int(i as i64)
    }
}

impl From<u32> for AttributeValue {
    fn from(i: u32) -> Self {
        AttributeValue::Int(i as i64)
    }
}

impl From<f64> for AttributeValue {
    fn from(f: f64) -> Self {
        AttributeValue::Float(f)
    }
}

/// Event that occurred during a span
#[derive(Debug, Clone)]
pub struct SpanEvent {
    /// Event name
    pub name: String,
    /// Time of the event in nanoseconds since Unix epoch
    pub time_unix_nano: u64,
    /// Event attributes
    pub attributes: HashMap<String, AttributeValue>,
}

/// Trait for exporting spans to observability backends
///
/// Implementations of this trait handle the actual transport of span data
/// to backends like AWS CloudWatch, Jaeger, or other OTLP-compatible services.
///
/// # Example
///
/// ```ignore
/// use stood::telemetry::exporter::{SpanExporter, SpanData, ExportError};
///
/// struct MyExporter;
///
/// #[async_trait::async_trait]
/// impl SpanExporter for MyExporter {
///     async fn export(&self, spans: Vec<SpanData>) -> Result<(), ExportError> {
///         // Send spans to backend
///         Ok(())
///     }
///
///     async fn shutdown(&self) -> Result<(), ExportError> {
///         // Cleanup resources
///         Ok(())
///     }
/// }
/// ```
#[async_trait]
pub trait SpanExporter: Send + Sync + std::fmt::Debug {
    /// Export a batch of spans to the backend
    ///
    /// This method should be non-blocking and handle failures gracefully.
    /// Implementations should not panic on export failure.
    async fn export(&self, spans: Vec<SpanData>) -> Result<(), ExportError>;

    /// Gracefully shutdown the exporter
    ///
    /// This should flush any pending spans and release resources.
    async fn shutdown(&self) -> Result<(), ExportError>;

    /// Check if the exporter is healthy
    ///
    /// Returns true if the exporter is ready to accept spans.
    fn is_healthy(&self) -> bool {
        true
    }
}

/// No-op exporter that discards all spans
///
/// Used when telemetry is disabled.
#[derive(Debug, Clone, Default)]
pub struct NoOpExporter;

#[async_trait]
impl SpanExporter for NoOpExporter {
    async fn export(&self, _spans: Vec<SpanData>) -> Result<(), ExportError> {
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), ExportError> {
        Ok(())
    }
}

// ============================================================================
// OTLP JSON Serialization
// ============================================================================

/// Serialize spans to OTLP JSON format
///
/// This follows the OpenTelemetry Protocol JSON encoding:
/// https://opentelemetry.io/docs/specs/otlp/#json-protobuf-encoding
pub mod otlp {
    use super::*;
    use serde::Serialize;

    /// OTLP ExportTraceServiceRequest
    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ExportTraceServiceRequest {
        pub resource_spans: Vec<ResourceSpans>,
    }

    /// Resource spans container
    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ResourceSpans {
        pub resource: Resource,
        pub scope_spans: Vec<ScopeSpans>,
    }

    /// Resource attributes
    #[derive(Debug, Serialize)]
    pub struct Resource {
        pub attributes: Vec<KeyValue>,
    }

    /// Scope spans container
    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ScopeSpans {
        pub scope: InstrumentationScope,
        pub spans: Vec<OtlpSpan>,
    }

    /// Instrumentation scope
    #[derive(Debug, Serialize)]
    pub struct InstrumentationScope {
        pub name: String,
        pub version: String,
    }

    /// OTLP Span representation
    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct OtlpSpan {
        pub trace_id: String,
        pub span_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub parent_span_id: Option<String>,
        pub name: String,
        pub kind: u32,
        pub start_time_unix_nano: String,
        pub end_time_unix_nano: String,
        pub attributes: Vec<KeyValue>,
        pub status: Status,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        pub events: Vec<Event>,
    }

    /// OTLP KeyValue
    #[derive(Debug, Serialize)]
    pub struct KeyValue {
        pub key: String,
        pub value: AnyValue,
    }

    /// OTLP AnyValue
    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct AnyValue {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub string_value: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub bool_value: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub int_value: Option<String>, // Encoded as string per OTLP spec
        #[serde(skip_serializing_if = "Option::is_none")]
        pub double_value: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub array_value: Option<ArrayValue>,
    }

    /// OTLP ArrayValue
    #[derive(Debug, Serialize)]
    pub struct ArrayValue {
        pub values: Vec<AnyValue>,
    }

    /// OTLP Status
    #[derive(Debug, Serialize)]
    pub struct Status {
        pub code: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub message: Option<String>,
    }

    /// OTLP Event
    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Event {
        pub name: String,
        pub time_unix_nano: String,
        pub attributes: Vec<KeyValue>,
    }

    impl From<&AttributeValue> for AnyValue {
        fn from(value: &AttributeValue) -> Self {
            match value {
                AttributeValue::String(s) => AnyValue {
                    string_value: Some(s.clone()),
                    bool_value: None,
                    int_value: None,
                    double_value: None,
                    array_value: None,
                },
                AttributeValue::Bool(b) => AnyValue {
                    string_value: None,
                    bool_value: Some(*b),
                    int_value: None,
                    double_value: None,
                    array_value: None,
                },
                AttributeValue::Int(i) => AnyValue {
                    string_value: None,
                    bool_value: None,
                    int_value: Some(i.to_string()),
                    double_value: None,
                    array_value: None,
                },
                AttributeValue::Float(f) => AnyValue {
                    string_value: None,
                    bool_value: None,
                    int_value: None,
                    double_value: Some(*f),
                    array_value: None,
                },
                AttributeValue::StringArray(arr) => AnyValue {
                    string_value: None,
                    bool_value: None,
                    int_value: None,
                    double_value: None,
                    array_value: Some(ArrayValue {
                        values: arr
                            .iter()
                            .map(|s| AnyValue {
                                string_value: Some(s.clone()),
                                bool_value: None,
                                int_value: None,
                                double_value: None,
                                array_value: None,
                            })
                            .collect(),
                    }),
                },
                AttributeValue::IntArray(arr) => AnyValue {
                    string_value: None,
                    bool_value: None,
                    int_value: None,
                    double_value: None,
                    array_value: Some(ArrayValue {
                        values: arr
                            .iter()
                            .map(|i| AnyValue {
                                string_value: None,
                                bool_value: None,
                                int_value: Some(i.to_string()),
                                double_value: None,
                                array_value: None,
                            })
                            .collect(),
                    }),
                },
            }
        }
    }

    impl From<&SpanKind> for u32 {
        fn from(kind: &SpanKind) -> Self {
            match kind {
                SpanKind::Internal => 1,
                SpanKind::Server => 2,
                SpanKind::Client => 3,
                SpanKind::Producer => 4,
                SpanKind::Consumer => 5,
            }
        }
    }

    impl From<&SpanStatus> for Status {
        fn from(status: &SpanStatus) -> Self {
            match status {
                SpanStatus::Unset => Status {
                    code: 0,
                    message: None,
                },
                SpanStatus::Ok => Status {
                    code: 1,
                    message: None,
                },
                SpanStatus::Error { message } => Status {
                    code: 2,
                    message: Some(message.clone()),
                },
            }
        }
    }

    /// Serialize spans to OTLP JSON format
    ///
    /// # Arguments
    ///
    /// * `spans` - The spans to serialize
    /// * `service_name` - The service name (per OTel spec: "Logical name of the service")
    /// * `service_version` - The service version
    /// * `agent_id` - The agent ID for log group naming (used to construct log group path)
    pub fn serialize_spans(
        spans: &[SpanData],
        service_name: &str,
        service_version: &str,
        agent_id: &str,
    ) -> Result<Vec<u8>, ExportError> {
        let otlp_spans: Vec<OtlpSpan> = spans
            .iter()
            .map(|s| OtlpSpan {
                trace_id: s.trace_id.clone(),
                span_id: s.span_id.clone(),
                parent_span_id: s.parent_span_id.clone(),
                name: s.name.clone(),
                kind: (&s.kind).into(),
                start_time_unix_nano: s.start_time_unix_nano.to_string(),
                end_time_unix_nano: s.end_time_unix_nano.to_string(),
                attributes: s
                    .attributes
                    .iter()
                    .map(|(k, v)| KeyValue {
                        key: k.clone(),
                        value: v.into(),
                    })
                    .collect(),
                status: (&s.status).into(),
                events: s
                    .events
                    .iter()
                    .map(|e| Event {
                        name: e.name.clone(),
                        time_unix_nano: e.time_unix_nano.to_string(),
                        attributes: e
                            .attributes
                            .iter()
                            .map(|(k, v)| KeyValue {
                                key: k.clone(),
                                value: v.into(),
                            })
                            .collect(),
                    })
                    .collect(),
            })
            .collect();

        let request = ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: Resource {
                    attributes: vec![
                        KeyValue {
                            key: "service.name".to_string(),
                            value: AnyValue {
                                string_value: Some(service_name.to_string()),
                                bool_value: None,
                                int_value: None,
                                double_value: None,
                                array_value: None,
                            },
                        },
                        KeyValue {
                            key: "service.version".to_string(),
                            value: AnyValue {
                                string_value: Some(service_version.to_string()),
                                bool_value: None,
                                int_value: None,
                                double_value: None,
                                array_value: None,
                            },
                        },
                        // CRITICAL: Required for Gen AI Observability Dashboard to recognize this as an agent
                        // The log group MUST physically exist in CloudWatch for spans to appear in the dashboard
                        // See: https://docs.aws.amazon.com/bedrock-agentcore/latest/devguide/observability-configure.html
                        KeyValue {
                            key: "aws.log.group.names".to_string(),
                            value: AnyValue {
                                string_value: Some(format!(
                                    "/aws/bedrock-agentcore/runtimes/{}",
                                    agent_id
                                )),
                                bool_value: None,
                                int_value: None,
                                double_value: None,
                                array_value: None,
                            },
                        },
                        // CRITICAL: Required for CloudWatch GenAI Dashboard query filter
                        // The dashboard filters on: resource.attributes.aws.service.type = "gen_ai_agent"
                        KeyValue {
                            key: "aws.service.type".to_string(),
                            value: AnyValue {
                                string_value: Some("gen_ai_agent".to_string()),
                                bool_value: None,
                                int_value: None,
                                double_value: None,
                                array_value: None,
                            },
                        },
                    ],
                },
                scope_spans: vec![ScopeSpans {
                    scope: InstrumentationScope {
                        // Use LangChain scope for AWS AgentCore Evaluations compatibility
                        // This scope is required for the evaluate API to parse log events correctly
                        name: LANGCHAIN_SCOPE.to_string(),
                        version: env!("CARGO_PKG_VERSION").to_string(),
                    },
                    spans: otlp_spans,
                }],
            }],
        };

        serde_json::to_vec(&request).map_err(|e| ExportError::Serialization(e.to_string()))
    }
}

// ============================================================================
// CloudWatch Exporter
// ============================================================================

use super::aws_auth::{
    cloudwatch_logs_endpoint, sign_request, xray_otlp_endpoint, AwsCredentialsProvider,
};
use super::log_event::{LogEvent, LANGCHAIN_SCOPE};
use super::AwsCredentialSource;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Exporter that sends spans to AWS CloudWatch via X-Ray OTLP endpoint
/// and log events to CloudWatch Logs for AgentCore Evaluations.
///
/// Uses SigV4 authentication and OTLP JSON format for spans,
/// and CloudWatch Logs PutLogEvents API for log events.
#[derive(Debug)]
pub struct CloudWatchExporter {
    /// AWS credentials provider
    credentials_provider: AwsCredentialsProvider,
    /// X-Ray endpoint URL for spans
    endpoint: String,
    /// CloudWatch Logs endpoint URL for log events
    logs_endpoint: String,
    /// Service name for resource attributes
    service_name: String,
    /// Service version for resource attributes
    service_version: String,
    /// Agent ID for log group naming
    /// Used to construct: /aws/bedrock-agentcore/runtimes/{agent_id}
    agent_id: String,
    /// HTTP client
    client: reqwest::Client,
    /// Whether the exporter is healthy
    healthy: Arc<AtomicBool>,
    /// Export timeout
    timeout: Duration,
    /// Cached sequence token for CloudWatch Logs (optional, for optimization)
    sequence_token: Arc<tokio::sync::Mutex<Option<String>>>,
}

impl CloudWatchExporter {
    /// Create a new CloudWatch exporter
    pub fn new(
        region: impl Into<String>,
        credentials: AwsCredentialSource,
        service_name: impl Into<String>,
        service_version: impl Into<String>,
    ) -> Self {
        let region = region.into();
        let endpoint = xray_otlp_endpoint(&region);
        let logs_endpoint = cloudwatch_logs_endpoint(&region);
        let service_name = service_name.into();

        Self {
            credentials_provider: AwsCredentialsProvider::new(credentials, region),
            endpoint,
            logs_endpoint,
            agent_id: service_name.clone(), // Default to service_name
            service_name,
            service_version: service_version.into(),
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("Failed to create HTTP client"),
            healthy: Arc::new(AtomicBool::new(true)),
            timeout: Duration::from_secs(10),
            sequence_token: Arc::new(tokio::sync::Mutex::new(None)),
        }
    }

    /// Set custom timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self.client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .expect("Failed to create HTTP client");
        self
    }

    /// Set agent ID for log group naming
    ///
    /// The agent ID is used to construct the log group name:
    /// `/aws/bedrock-agentcore/runtimes/{agent_id}`
    pub fn with_agent_id(mut self, agent_id: impl Into<String>) -> Self {
        self.agent_id = agent_id.into();
        self
    }

    /// Export spans with retry logic
    async fn export_with_retry(
        &self,
        spans: Vec<SpanData>,
        max_retries: u32,
    ) -> Result<(), ExportError> {
        let mut last_error = None;

        for attempt in 0..=max_retries {
            if attempt > 0 {
                // Exponential backoff: 100ms, 200ms, 400ms, ...
                let delay = Duration::from_millis(100 * (1 << (attempt - 1)));
                tokio::time::sleep(delay).await;
            }

            match self.try_export(&spans).await {
                Ok(()) => {
                    self.healthy.store(true, Ordering::Relaxed);
                    return Ok(());
                }
                Err(e) => {
                    tracing::warn!("CloudWatch export attempt {} failed: {}", attempt + 1, e);
                    last_error = Some(e);

                    // Don't retry on auth or serialization errors
                    if matches!(
                        last_error,
                        Some(ExportError::Auth(_)) | Some(ExportError::Serialization(_))
                    ) {
                        break;
                    }
                }
            }
        }

        self.healthy.store(false, Ordering::Relaxed);
        Err(last_error.unwrap_or_else(|| ExportError::Network("Unknown error".to_string())))
    }

    /// Try to export spans once
    async fn try_export(&self, spans: &[SpanData]) -> Result<(), ExportError> {
        // Serialize spans to OTLP JSON
        let body = otlp::serialize_spans(
            spans,
            &self.service_name,
            &self.service_version,
            &self.agent_id,
        )?;

        // Get credentials
        let credentials = self
            .credentials_provider
            .resolve()
            .await
            .map_err(|e| ExportError::Auth(e.to_string()))?;

        // Prepare headers
        let headers = vec![
            ("Content-Type".to_string(), "application/json".to_string()),
            (
                "Host".to_string(),
                self.endpoint
                    .trim_start_matches("https://")
                    .split('/')
                    .next()
                    .unwrap_or("")
                    .to_string(),
            ),
        ];

        // Sign the request
        let signed_headers = sign_request(
            "POST",
            &self.endpoint,
            &headers,
            &body,
            &credentials,
            self.credentials_provider.region(),
            "xray",
        )
        .map_err(|e| ExportError::Auth(e.to_string()))?;

        // Build and send request
        let mut request = self.client.post(&self.endpoint).body(body);

        for (name, value) in signed_headers {
            request = request.header(&name, &value);
        }

        let response = request.send().await.map_err(|e| {
            if e.is_timeout() {
                ExportError::Timeout(self.timeout)
            } else {
                ExportError::Network(e.to_string())
            }
        })?;

        let status = response.status();
        let response_text = response.text().await.unwrap_or_else(|_| "Could not read response".to_string());
        tracing::debug!("X-Ray OTLP response: status={}, body={}", status, response_text);

        if status.is_success() {
            Ok(())
        } else if status.as_u16() == 429 {
            Err(ExportError::RateLimited)
        } else {
            Err(ExportError::Backend {
                status_code: status.as_u16(),
                message: response_text,
            })
        }
    }

    // ========================================================================
    // CloudWatch Logs Export (for AgentCore Evaluations)
    // ========================================================================

    /// Get the log group name for this exporter
    pub fn log_group_name(&self) -> String {
        format!("/aws/bedrock-agentcore/runtimes/{}", self.agent_id)
    }

    /// Get the log stream name for runtime logs
    pub fn log_stream_name(&self) -> &str {
        "runtime-logs"
    }

    /// Export log events to CloudWatch Logs for AgentCore Evaluations
    ///
    /// Log events contain the prompt/response content that evaluations like
    /// Correctness and Conciseness require.
    pub async fn export_logs(&self, events: Vec<LogEvent>) -> Result<(), ExportError> {
        if events.is_empty() {
            return Ok(());
        }

        // Ensure log group and stream exist
        self.ensure_log_stream_exists().await?;

        // Export with retry
        self.export_logs_with_retry(events, 2).await
    }

    /// Export logs with retry logic
    async fn export_logs_with_retry(
        &self,
        events: Vec<LogEvent>,
        max_retries: u32,
    ) -> Result<(), ExportError> {
        let mut last_error = None;

        for attempt in 0..=max_retries {
            if attempt > 0 {
                let delay = Duration::from_millis(100 * (1 << (attempt - 1)));
                tokio::time::sleep(delay).await;
            }

            match self.try_export_logs(&events).await {
                Ok(()) => return Ok(()),
                Err(e) => {
                    tracing::warn!(
                        "CloudWatch Logs export attempt {} failed: {}",
                        attempt + 1,
                        e
                    );
                    last_error = Some(e);

                    if matches!(
                        last_error,
                        Some(ExportError::Auth(_)) | Some(ExportError::Serialization(_))
                    ) {
                        break;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| ExportError::Network("Unknown error".to_string())))
    }

    /// Try to export logs once using PutLogEvents API
    async fn try_export_logs(&self, events: &[LogEvent]) -> Result<(), ExportError> {
        let log_group = self.log_group_name();
        let log_stream = self.log_stream_name();

        // Build PutLogEvents request body
        let log_events: Vec<serde_json::Value> = events
            .iter()
            .map(|e| {
                serde_json::json!({
                    "timestamp": e.time_unix_nano / 1_000_000, // Convert nanos to millis
                    "message": serde_json::to_string(e).unwrap_or_default()
                })
            })
            .collect();

        let mut request_body = serde_json::json!({
            "logGroupName": log_group,
            "logStreamName": log_stream,
            "logEvents": log_events
        });

        // Include sequence token if available (for optimization)
        {
            let token = self.sequence_token.lock().await;
            if let Some(ref t) = *token {
                request_body["sequenceToken"] = serde_json::json!(t);
            }
        }

        let body = serde_json::to_vec(&request_body)
            .map_err(|e| ExportError::Serialization(e.to_string()))?;

        // Get credentials
        let credentials = self
            .credentials_provider
            .resolve()
            .await
            .map_err(|e| ExportError::Auth(e.to_string()))?;

        // Prepare headers for CloudWatch Logs API
        let host = self
            .logs_endpoint
            .trim_start_matches("https://")
            .trim_end_matches('/')
            .to_string();

        let headers = vec![
            (
                "Content-Type".to_string(),
                "application/x-amz-json-1.1".to_string(),
            ),
            ("Host".to_string(), host),
            (
                "X-Amz-Target".to_string(),
                "Logs_20140328.PutLogEvents".to_string(),
            ),
        ];

        // Sign the request for CloudWatch Logs service
        let signed_headers = sign_request(
            "POST",
            &self.logs_endpoint,
            &headers,
            &body,
            &credentials,
            self.credentials_provider.region(),
            "logs", // CloudWatch Logs service
        )
        .map_err(|e| ExportError::Auth(e.to_string()))?;

        // Build and send request
        let mut request = self.client.post(&self.logs_endpoint).body(body);

        for (name, value) in signed_headers {
            request = request.header(&name, &value);
        }

        let response = request.send().await.map_err(|e| {
            if e.is_timeout() {
                ExportError::Timeout(self.timeout)
            } else {
                ExportError::Network(e.to_string())
            }
        })?;

        let status = response.status();
        if status.is_success() {
            // Extract and cache the next sequence token
            if let Ok(resp_body) = response.json::<serde_json::Value>().await {
                if let Some(token) = resp_body.get("nextSequenceToken").and_then(|t| t.as_str()) {
                    let mut cached_token = self.sequence_token.lock().await;
                    *cached_token = Some(token.to_string());
                }
            }
            Ok(())
        } else if status.as_u16() == 429 {
            Err(ExportError::RateLimited)
        } else {
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(ExportError::Backend {
                status_code: status.as_u16(),
                message,
            })
        }
    }

    /// Ensure log group and stream exist, creating them if necessary
    async fn ensure_log_stream_exists(&self) -> Result<(), ExportError> {
        let log_group = self.log_group_name();
        let log_stream = self.log_stream_name();

        let credentials = self
            .credentials_provider
            .resolve()
            .await
            .map_err(|e| ExportError::Auth(e.to_string()))?;

        let host = self
            .logs_endpoint
            .trim_start_matches("https://")
            .trim_end_matches('/')
            .to_string();

        // Try to create log group (ignore if exists)
        let _ = self
            .cloudwatch_logs_api_call(
                "Logs_20140328.CreateLogGroup",
                &serde_json::json!({ "logGroupName": log_group }),
                &credentials,
                &host,
            )
            .await;

        // Try to create log stream (ignore if exists)
        let _ = self
            .cloudwatch_logs_api_call(
                "Logs_20140328.CreateLogStream",
                &serde_json::json!({
                    "logGroupName": log_group,
                    "logStreamName": log_stream
                }),
                &credentials,
                &host,
            )
            .await;

        Ok(())
    }

    /// Make a CloudWatch Logs API call
    async fn cloudwatch_logs_api_call(
        &self,
        target: &str,
        body: &serde_json::Value,
        credentials: &aws_credential_types::Credentials,
        host: &str,
    ) -> Result<serde_json::Value, ExportError> {
        let body_bytes =
            serde_json::to_vec(body).map_err(|e| ExportError::Serialization(e.to_string()))?;

        let headers = vec![
            (
                "Content-Type".to_string(),
                "application/x-amz-json-1.1".to_string(),
            ),
            ("Host".to_string(), host.to_string()),
            ("X-Amz-Target".to_string(), target.to_string()),
        ];

        let signed_headers = sign_request(
            "POST",
            &self.logs_endpoint,
            &headers,
            &body_bytes,
            credentials,
            self.credentials_provider.region(),
            "logs",
        )
        .map_err(|e| ExportError::Auth(e.to_string()))?;

        let mut request = self.client.post(&self.logs_endpoint).body(body_bytes);

        for (name, value) in signed_headers {
            request = request.header(&name, &value);
        }

        let response = request.send().await.map_err(|e| {
            if e.is_timeout() {
                ExportError::Timeout(self.timeout)
            } else {
                ExportError::Network(e.to_string())
            }
        })?;

        let status = response.status();
        if status.is_success() {
            response
                .json::<serde_json::Value>()
                .await
                .map_err(|e| ExportError::Serialization(e.to_string()))
        } else {
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(ExportError::Backend {
                status_code: status.as_u16(),
                message,
            })
        }
    }
}

#[async_trait]
impl SpanExporter for CloudWatchExporter {
    async fn export(&self, spans: Vec<SpanData>) -> Result<(), ExportError> {
        if spans.is_empty() {
            return Ok(());
        }

        // Use 2 retries by default
        self.export_with_retry(spans, 2).await
    }

    async fn shutdown(&self) -> Result<(), ExportError> {
        // No cleanup needed for HTTP client
        Ok(())
    }

    fn is_healthy(&self) -> bool {
        self.healthy.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_otlp_serialize_spans() {
        let spans = vec![SpanData {
            trace_id: "0123456789abcdef0123456789abcdef".to_string(),
            span_id: "fedcba9876543210".to_string(),
            parent_span_id: Some("abcdef0123456789".to_string()),
            name: "chat claude-3-haiku".to_string(),
            kind: SpanKind::Client,
            start_time_unix_nano: 1000000000,
            end_time_unix_nano: 2000000000,
            attributes: {
                let mut attrs = HashMap::new();
                attrs.insert(
                    "gen_ai.system".to_string(),
                    AttributeValue::String("aws.bedrock".to_string()),
                );
                attrs.insert(
                    "gen_ai.operation.name".to_string(),
                    AttributeValue::String("chat".to_string()),
                );
                attrs.insert(
                    "gen_ai.usage.input_tokens".to_string(),
                    AttributeValue::Int(100),
                );
                attrs.insert(
                    "gen_ai.usage.output_tokens".to_string(),
                    AttributeValue::Int(50),
                );
                attrs
            },
            status: SpanStatus::Ok,
            events: vec![],
        }];

        // agent_id defaults to service_name for backwards compatibility
        let result = otlp::serialize_spans(&spans, "test-service", "1.0.0", "test-service");
        assert!(result.is_ok());

        let json_bytes = result.unwrap();
        let json_str = String::from_utf8(json_bytes).unwrap();

        // Verify key structure elements
        assert!(json_str.contains("resourceSpans"));
        assert!(json_str.contains("scopeSpans"));
        assert!(json_str.contains("service.name"));
        assert!(json_str.contains("test-service"));
        assert!(json_str.contains("0123456789abcdef0123456789abcdef")); // trace_id
        assert!(json_str.contains("fedcba9876543210")); // span_id
        assert!(json_str.contains("chat claude-3-haiku")); // name
        assert!(json_str.contains("gen_ai.system"));
        assert!(json_str.contains("aws.bedrock"));

        // CRITICAL: Verify aws.log.group.names resource attribute for Gen AI Dashboard
        assert!(
            json_str.contains("aws.log.group.names"),
            "aws.log.group.names resource attribute is required for Gen AI Dashboard"
        );
        assert!(
            json_str.contains("/aws/bedrock-agentcore/runtimes/test-service"),
            "aws.log.group.names must follow format /aws/bedrock-agentcore/runtimes/<agent_id>"
        );
    }

    #[test]
    fn test_resource_has_required_attributes_for_genai_dashboard() {
        // Create a minimal span to test resource serialization
        let spans = vec![SpanData {
            trace_id: "00000000000000000000000000000001".to_string(),
            span_id: "0000000000000001".to_string(),
            parent_span_id: None,
            name: "test".to_string(),
            kind: SpanKind::Internal,
            start_time_unix_nano: 1000000000,
            end_time_unix_nano: 2000000000,
            attributes: HashMap::new(),
            status: SpanStatus::Ok,
            events: vec![],
        }];

        let service_name = "my-stood-agent";
        let agent_id = "my-agent-001"; // agent_id can differ from service_name
        let result = otlp::serialize_spans(&spans, service_name, "1.0.0", agent_id);
        assert!(result.is_ok());

        let json_bytes = result.unwrap();
        let json_str = String::from_utf8(json_bytes).unwrap();

        // Verify service.name is set
        assert!(
            json_str.contains("service.name"),
            "service.name resource attribute must be set"
        );
        assert!(
            json_str.contains(service_name),
            "service.name must contain the service name"
        );

        // Verify aws.log.group.names is set (CRITICAL for Gen AI Dashboard)
        assert!(
            json_str.contains("aws.log.group.names"),
            "aws.log.group.names resource attribute must be set for Gen AI Dashboard"
        );

        // Verify format follows AgentCore runtime pattern using agent_id (not service_name)
        let expected_log_group = format!("/aws/bedrock-agentcore/runtimes/{}", agent_id);
        assert!(
            json_str.contains(&expected_log_group),
            "aws.log.group.names must follow format /aws/bedrock-agentcore/runtimes/<agent_id>"
        );
    }

    #[test]
    fn test_otlp_serialize_empty_spans() {
        let spans: Vec<SpanData> = vec![];
        let result = otlp::serialize_spans(&spans, "test-service", "1.0.0", "test-agent");
        assert!(result.is_ok());
    }

    #[test]
    fn test_otlp_span_kind_conversion() {
        assert_eq!(u32::from(&SpanKind::Internal), 1);
        assert_eq!(u32::from(&SpanKind::Server), 2);
        assert_eq!(u32::from(&SpanKind::Client), 3);
        assert_eq!(u32::from(&SpanKind::Producer), 4);
        assert_eq!(u32::from(&SpanKind::Consumer), 5);
    }

    #[test]
    fn test_otlp_status_conversion() {
        let unset: otlp::Status = (&SpanStatus::Unset).into();
        assert_eq!(unset.code, 0);
        assert!(unset.message.is_none());

        let ok: otlp::Status = (&SpanStatus::Ok).into();
        assert_eq!(ok.code, 1);
        assert!(ok.message.is_none());

        let error: otlp::Status = (&SpanStatus::Error {
            message: "test error".to_string(),
        })
            .into();
        assert_eq!(error.code, 2);
        assert_eq!(error.message, Some("test error".to_string()));
    }

    #[test]
    fn test_otlp_attribute_value_conversion() {
        // String
        let string_val: otlp::AnyValue = (&AttributeValue::String("test".to_string())).into();
        assert_eq!(string_val.string_value, Some("test".to_string()));

        // Bool
        let bool_val: otlp::AnyValue = (&AttributeValue::Bool(true)).into();
        assert_eq!(bool_val.bool_value, Some(true));

        // Int (encoded as string per OTLP spec)
        let int_val: otlp::AnyValue = (&AttributeValue::Int(42)).into();
        assert_eq!(int_val.int_value, Some("42".to_string()));

        // Float
        let float_val: otlp::AnyValue = (&AttributeValue::Float(3.14)).into();
        assert_eq!(float_val.double_value, Some(3.14));

        // String array
        let str_array: otlp::AnyValue =
            (&AttributeValue::StringArray(vec!["a".to_string(), "b".to_string()])).into();
        assert!(str_array.array_value.is_some());
        let arr = str_array.array_value.unwrap();
        assert_eq!(arr.values.len(), 2);

        // Int array
        let int_array: otlp::AnyValue = (&AttributeValue::IntArray(vec![1, 2, 3])).into();
        assert!(int_array.array_value.is_some());
        let arr = int_array.array_value.unwrap();
        assert_eq!(arr.values.len(), 3);
    }

    #[test]
    fn test_cloudwatch_exporter_creation() {
        let exporter = CloudWatchExporter::new(
            "us-east-1",
            AwsCredentialSource::Environment,
            "test-service",
            "1.0.0",
        );

        assert!(exporter.is_healthy());
    }

    #[test]
    fn test_attribute_value_from() {
        let s: AttributeValue = "hello".into();
        assert!(matches!(s, AttributeValue::String(_)));

        let b: AttributeValue = true.into();
        assert!(matches!(b, AttributeValue::Bool(true)));

        let i: AttributeValue = 42i64.into();
        assert!(matches!(i, AttributeValue::Int(42)));

        let f: AttributeValue = 3.14f64.into();
        assert!(matches!(f, AttributeValue::Float(_)));
    }

    #[test]
    fn test_span_status() {
        let ok = SpanStatus::Ok;
        assert_eq!(ok, SpanStatus::Ok);

        let err = SpanStatus::Error {
            message: "failed".to_string(),
        };
        assert!(matches!(err, SpanStatus::Error { .. }));
    }

    #[tokio::test]
    async fn test_noop_exporter() {
        let exporter = NoOpExporter;

        let spans = vec![SpanData {
            trace_id: "abc123".to_string(),
            span_id: "def456".to_string(),
            parent_span_id: None,
            name: "test".to_string(),
            kind: SpanKind::Internal,
            start_time_unix_nano: 0,
            end_time_unix_nano: 1000,
            attributes: HashMap::new(),
            status: SpanStatus::Ok,
            events: vec![],
        }];

        assert!(exporter.export(spans).await.is_ok());
        assert!(exporter.shutdown().await.is_ok());
        assert!(exporter.is_healthy());
    }
}
