# Stood Agent Metrics Implementation Plan

## 📊 **Summary: Complete Agentic Metrics Strategy**

### **🎯 Key Metrics Identified (35 Total)**

**Core Agent Performance (8 metrics)**
- Token usage (input/output/total/per-request/cost)
- Request/invocation counts and latency
- Model invocation timing

**Tool Execution (7 metrics)**  
- Tool calls, execution duration, parallel tracking
- Success rates, timeouts, validation failures

**Agentic Reasoning (8 metrics)**
- Reasoning cycles, conversation turns, context utilization
- Planning duration, model switches, token limits

**System Resources (6 metrics)**
- Memory usage, connection pools, concurrency
- Thread utilization, async operations

**Quality & Reliability (6 metrics)**
- Error tracking by type, health scores, uptime
- Circuit breaker status, performance degradation

## 🎯 **Comprehensive Agentic Workload Metrics**

### **1. Core Agent Performance Metrics**

#### **Token Usage Metrics**
- `agent_tokens_input_total` (Counter) - Total input tokens consumed
- `agent_tokens_output_total` (Counter) - Total output tokens generated  
- `agent_tokens_total` (Counter) - Combined input + output tokens
- `agent_tokens_per_request` (Histogram) - Token distribution per request
- `agent_token_cost_estimate` (Gauge) - Estimated cost based on model pricing

#### **Request & Invocation Metrics**
- `agent_requests_total` (Counter) - Total agent requests by status (success/error)
- `agent_model_invocations_total` (Counter) - Total model API calls
- `agent_cycles_total` (Counter) - Total event loop cycles executed
- `agent_request_duration` (Histogram) - End-to-end request latency
- `agent_model_invocation_duration` (Histogram) - Model API call latency

### **2. Tool Execution Metrics**

#### **Tool Performance**
- `agent_tool_calls_total` (Counter) - Tool invocations by tool_name and status
- `agent_tool_execution_duration` (Histogram) - Tool execution time by tool_name
- `agent_tool_parallel_executions` (Gauge) - Current parallel tool executions
- `agent_tool_queue_depth` (Gauge) - Tools waiting for execution
- `agent_tool_timeout_rate` (Counter) - Tool timeouts by tool_name

#### **Tool Success/Failure Tracking**
- `agent_tool_success_rate` (Gauge) - Success percentage by tool_name (0-1)
- `agent_tool_retry_attempts` (Counter) - Tool retry attempts by tool_name
- `agent_tool_validation_failures` (Counter) - Input validation failures

### **3. Agentic Reasoning Metrics**

#### **Decision Making & Flow Control**
- `agent_reasoning_cycles_per_request` (Histogram) - Cycles needed to complete request
- `agent_conversation_turns` (Counter) - Back-and-forth interactions
- `agent_context_length` (Histogram) - Context window utilization
- `agent_planning_duration` (Histogram) - Time spent in planning phase
- `agent_reflection_cycles` (Counter) - Self-correction iterations

#### **Model Selection & Strategy**
- `agent_model_switches` (Counter) - Model changes mid-conversation
- `agent_temperature_adjustments` (Counter) - Temperature modifications
- `agent_max_tokens_hit` (Counter) - Requests hitting token limits
- `agent_streaming_vs_batch` (Counter) - Streaming vs batch processing choice

### **4. System Resource Metrics**

#### **Memory & Performance**
- `agent_memory_usage_bytes` (Gauge) - Current memory consumption
- `agent_connection_pool_active` (Gauge) - Active connections to model APIs
- `agent_connection_pool_idle` (Gauge) - Idle connections available
- `agent_batch_processing_efficiency` (Gauge) - Batch utilization rate (0-1)

#### **Concurrency & Threading**
- `agent_concurrent_requests` (Gauge) - Currently processing requests
- `agent_thread_pool_utilization` (Gauge) - Thread pool usage (0-1)
- `agent_async_tasks_active` (Gauge) - Active async operations

### **5. Quality & Reliability Metrics**

#### **Error Tracking**
- `agent_errors_total` (Counter) - Errors by error_type and component
- `agent_model_errors` (Counter) - Model API errors by error_code
- `agent_tool_errors` (Counter) - Tool execution errors by tool_name
- `agent_timeout_errors` (Counter) - Timeout failures by component
- `agent_rate_limit_hits` (Counter) - Rate limiting encounters

#### **Health & Availability**
- `agent_health_score` (Gauge) - Overall system health (0-1)
- `agent_uptime_seconds` (Counter) - Service uptime
- `agent_circuit_breaker_state` (Gauge) - Circuit breaker status (0=closed, 1=open)
- `agent_degraded_performance_events` (Counter) - Performance degradation incidents

### **6. Business & Usage Metrics**

#### **User Experience**
- `agent_response_quality_score` (Histogram) - Subjective quality ratings
- `agent_user_satisfaction` (Gauge) - User feedback scores
- `agent_task_completion_rate` (Gauge) - Successful task completion (0-1)
- `agent_response_relevance` (Histogram) - Response relevance scores

#### **Cost & Efficiency**
- `agent_cost_per_request` (Histogram) - Dollar cost per request
- `agent_tokens_per_dollar` (Gauge) - Token efficiency metric
- `agent_resource_utilization` (Gauge) - Overall resource efficiency (0-1)

## 🏗️ **Metrics Collection Architecture Design**

### **Architecture Overview**

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Agent Core    │───▶│  Metrics Layer  │───▶│  OTLP Export    │
│                 │    │                 │    │                 │
│ • Event Loop    │    │ • Collectors    │    │ • Batch Export  │
│ • Tool Executor │    │ • Aggregators   │    │ • Auto Retry    │
│ • Model Client  │    │ • Formatters    │    │ • Compression   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                │
                                ▼
                       ┌─────────────────┐
                       │  Telemetry      │
                       │  Services       │
                       │                 │
                       │ • Prometheus    │
                       │ • Grafana       │
                       │ • Alerting      │
                       └─────────────────┘
```

### **1. Core Metrics Collection Layer**

#### **MetricsCollector Trait**
```rust
pub trait MetricsCollector: Send + Sync {
    fn record_counter(&self, name: &str, value: u64, labels: &[KeyValue]);
    fn record_histogram(&self, name: &str, value: f64, labels: &[KeyValue]);
    fn record_gauge(&self, name: &str, value: f64, labels: &[KeyValue]);
    fn increment(&self, name: &str, labels: &[KeyValue]);
}
```

#### **AgentMetrics Central Hub**
```rust
pub struct AgentMetrics {
    collector: Arc<dyn MetricsCollector>,
    token_counter: Counter<u64>,
    request_duration: Histogram<f64>,
    tool_execution_duration: Histogram<f64>,
    active_requests: UpDownCounter<i64>,
}
```

### **2. Integration Points Architecture**

#### **Event Loop Integration**
- **Hook Points**: Before/after each cycle, model invocation, tool execution
- **Data Collection**: Duration, token usage, success/failure, concurrency levels
- **Batch Processing**: Aggregate metrics within cycles for efficiency

#### **Tool Execution Integration** 
- **Instrumentation**: Automatic wrapping of tool executions
- **Parallel Tracking**: Concurrent execution monitoring
- **Resource Utilization**: Memory, CPU, network usage per tool

#### **Callback Integration**
- **Real-time Events**: Token usage, completion events, errors
- **Streaming Metrics**: Incremental updates during long operations
- **Custom Metrics**: User-defined business metrics via callbacks

### **3. OTLP Export Strategy**

#### **Metrics SDK Integration**
```rust
pub struct OtlpMetricsExporter {
    meter: Meter,
    exporter: MetricExporter,
    resource: Resource,
    batch_config: BatchConfig,
}
```

#### **Smart Batching**
- **Time-based batching**: Export every 10-30 seconds
- **Size-based batching**: Export when batch reaches 1000 metrics
- **Priority batching**: High-priority metrics exported immediately
- **Compression**: Gzip compression for bandwidth efficiency

#### **Semantic Conventions**
```rust
pub mod semantic_conventions {
    pub const AGENT_REQUEST_DURATION: &str = "agent.request.duration";
    pub const AGENT_TOKENS_INPUT: &str = "agent.tokens.input.total";
    pub const AGENT_TOOL_EXECUTION_DURATION: &str = "agent.tool.execution.duration";
    // ... etc
}
```

## 📋 **Implementation Plan for OTLP Metrics**

### **Phase 1: Foundation (Week 1-2)** ✅ **COMPLETED**

#### **1.1 Add OpenTelemetry Metrics Dependencies** ✅ **COMPLETED**
```toml
# Cargo.toml additions
[dependencies]
opentelemetry = { version = "0.24", features = ["metrics"], optional = true }
opentelemetry_sdk = { version = "0.24", features = ["metrics", "rt-tokio"], optional = true }
opentelemetry-otlp = { version = "0.17", features = ["metrics", "grpc-tonic"], optional = true }
```

#### **1.2 Create Core Metrics Infrastructure** ✅ **COMPLETED**
**Files created:**
- ✅ `src/telemetry/metrics/mod.rs` - Main metrics module
- ✅ `src/telemetry/metrics/collector.rs` - MetricsCollector trait and implementations
- ✅ `src/telemetry/metrics/exporter.rs` - OTLP metrics exporter
- ✅ `src/telemetry/metrics/semantic_conventions.rs` - Metric naming standards
- ✅ `src/telemetry/metrics/system.rs` - System resource metrics collection

#### **1.3 Extend StoudTracer for Metrics** ✅ **COMPLETED**
**File**: `src/telemetry/otel.rs`
```rust
impl StoodTracer {
    pub fn init_with_metrics(config: TelemetryConfig) -> StoodResult<Self> {
        // Initialize both tracing and metrics exporters
        let tracer = // existing trace setup
        let meter = // new metrics setup
        Ok(Self { tracer, meter: Some(meter), config })
    }
    
    pub fn record_metric(&self, metric: &str, value: f64, labels: &[KeyValue]) {
        // Record metric via OpenTelemetry meter
    }
}
```

### **Phase 2: Core Agent Metrics (Week 3-4)** ✅ **COMPLETED**

#### **2.1 Implement High-Priority Metrics** ✅ **COMPLETED**
**Integration points in `src/agent/event_loop.rs`:**
- ✅ Event loop metrics recording implemented (lines 228-234)
- ✅ Request start/end tracking with status labels
- ✅ Concurrent request gauge tracking
- ✅ Token usage metrics collection
- ✅ Model invocation counters

#### **2.2 Tool Execution Metrics** ✅ **COMPLETED**
**Integration in `src/tools/executor.rs`:**
- ✅ Tool execution timing and success tracking (lines 595-607)
- ✅ `with_metrics_collector()` method implemented (lines 473-476)
- ✅ Integration properly wired in event loop (event_loop.rs:175-181)
- ✅ ToolMetrics recording with duration, success, and error tracking

### **Phase 3: Advanced Metrics (Week 5-6)**

#### **3.1 System Resource Metrics**
**Add to `src/telemetry/metrics/system.rs`:**

```rust
pub struct SystemMetricsCollector {
    meter: Meter,
    memory_gauge: Gauge<f64>,
    connection_pool_gauge: Gauge<f64>,
    thread_utilization_gauge: Gauge<f64>,
}

impl SystemMetricsCollector {
    pub async fn collect_system_metrics(&self) {
        // Memory usage
        if let Ok(memory_info) = sys_info::mem_info() {
            let used_mb = (memory_info.total - memory_info.free) / 1024;
            self.memory_gauge.record(used_mb as f64, &[]);
        }
        
        // Thread pool utilization
        let thread_utilization = self.calculate_thread_utilization();
        self.thread_utilization_gauge.record(thread_utilization, &[]);
    }
}
```

#### **3.2 Error and Quality Metrics**
**Integration in error handling:**

```rust
impl StoodError {
    pub fn record_error_metric(&self, metrics: &AgentMetrics) {
        let error_type = match self {
            StoodError::ModelError(_) => "model_error",
            StoodError::ToolError(_) => "tool_error",
            StoodError::ConfigurationError(_) => "configuration_error",
            StoodError::InvalidInput(_) => "invalid_input",
            _ => "other_error",
        };
        
        metrics.increment("agent_errors_total", &[
            KeyValue::new("error_type", error_type),
            KeyValue::new("component", self.component())
        ]);
    }
}
```

### **Phase 4: Configuration and Optimization (Week 7-8)**

#### **4.1 Metrics Configuration**
**Extend `TelemetryConfig`:**

```rust
#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    // Existing fields...
    
    // New metrics fields
    pub metrics_enabled: bool,
    pub metrics_export_interval: Duration,
    pub metrics_batch_size: usize,
    pub metrics_compression: bool,
    pub custom_metrics: HashMap<String, MetricConfig>,
}

impl TelemetryConfig {
    pub fn with_metrics(mut self, enabled: bool) -> Self {
        self.metrics_enabled = enabled;
        self
    }
    
    pub fn metrics_export_interval(mut self, interval: Duration) -> Self {
        self.metrics_export_interval = interval;
        self
    }
}
```

#### **4.2 Performance Optimization**
- **Metric Sampling**: Configurable sampling rates for high-frequency metrics
- **Buffering Strategy**: Smart buffering to reduce OTLP export overhead
- **Label Optimization**: Efficient label management to prevent cardinality explosion
- **Memory Management**: Bounded metric storage with automatic cleanup

### **Phase 5: Integration and Testing (Week 9-10)**

#### **5.1 Update Telemetry Demo**
**Enhance `examples/docs/004_telemetry/telemetry_demo.rs`:**

```rust
async fn run_metrics_demonstration(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    info!("🔢 Starting Metrics Collection Demo");
    
    // Enable all metric types
    let mut agent_with_metrics = self.agent.clone();
    agent_with_metrics.enable_comprehensive_metrics(true);
    
    // Run operations while collecting metrics
    for i in 0..10 {
        let operation = format!("Metric test operation {}", i + 1);
        match agent_with_metrics.execute(&operation).await {
            Ok(result) => {
                info!("✅ Operation {} completed: {} tokens", i + 1, 
                      result.tokens.map(|t| t.total_tokens()).unwrap_or(0));
            }
            Err(e) => {
                warn!("❌ Operation {} failed: {}", i + 1, e);
            }
        }
        
        // Small delay to see metrics accumulation
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    
    // Display metrics summary
    self.show_metrics_summary().await;
    Ok(())
}
```

#### **5.2 Grafana Dashboard Creation**
**Update `grafana/dashboards/stood-telemetry.json`:**
- Add panels for token usage over time
- Tool execution performance charts  
- Error rate tracking
- System resource utilization
- Request latency percentiles
- Concurrent request monitoring

#### **5.3 Testing Strategy**
- **Unit Tests**: Metric collection accuracy
- **Integration Tests**: OTLP export functionality  
- **Load Tests**: Metrics under high throughput
- **Resource Tests**: Memory usage with metrics enabled
- **End-to-End Tests**: Full telemetry pipeline validation

### **Phase 6: Documentation and Deployment (Week 11-12)**

#### **6.1 Documentation Updates**
- **CLAUDE.md**: Add metrics configuration examples
- **README**: Metrics collection overview
- **Grafana Setup Guide**: Dashboard configuration
- **Performance Impact**: Metrics overhead analysis

#### **6.2 Production Readiness**
- **Default Configuration**: Sensible production defaults
- **Feature Flags**: Granular metric enable/disable
- **Performance Benchmarks**: Overhead measurements
- **Monitoring Setup**: Alerting rules for Prometheus

## 🔍 **Milestone: Full JSON LLM Invocation Logging to Telemetry**

### **Current State Analysis**

**✅ Local Debug Logs (Complete JSON)**
- **Location**: `logs/stood.log.*` files  
- **Content**: Full Bedrock API request/response JSON at DEBUG level
- **Implementation**: `src/bedrock/mod.rs` lines 647-650, 802-805, 829-832

**❌ Telemetry Systems (Metadata Only)**
- **OTLP/Jaeger**: Only span metadata (timing, tokens, model params)
- **Missing**: Actual prompt and completion content
- **Infrastructure exists** in `src/telemetry/otel.rs` but not wired up

### **Implementation Tasks**

#### **Task 1: Wire Up Existing Prompt/Completion Recording (Week 2)** ✅ **COMPLETED**
**File**: `src/bedrock/mod.rs`

**Integration Points:**
```rust
// In chat_with_tools_instrumented() method around lines 640-650
#[cfg(feature = "telemetry")]
if let Some(ref tracer) = self.tracer {
    let mut span = tracer.start_model_span("bedrock.chat_with_tools");
    span.record_prompt(&pretty_json); // Wire up existing method
    
    // ... existing request logic ...
    
    span.record_completion(&pretty_response); // Wire up existing method
    span.finish();
}
```

#### **Task 2: Add JSON Telemetry Configuration (Week 3)**
**Extend `TelemetryConfig`:**

```rust
#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    // Existing fields...
    
    // New JSON logging fields
    pub log_full_requests: bool,
    pub log_full_responses: bool,
    pub request_sanitization: SanitizationConfig,
    pub max_payload_size: usize,
    pub enable_request_compression: bool,
}

#[derive(Debug, Clone)]
pub struct SanitizationConfig {
    pub redact_pii: bool,
    pub redact_patterns: Vec<String>,
    pub max_token_preview: Option<usize>,
}
```

#### **Task 3: Implement Smart Payload Management (Week 4)**
**Add to `src/telemetry/otel.rs`:**

```rust
impl StoodTracer {
    pub fn record_request_payload(&self, payload: &str, sanitize: bool) {
        if !self.config.log_full_requests { return; }
        
        let processed_payload = if sanitize {
            self.sanitize_payload(payload)
        } else {
            payload.to_string()
        };
        
        // Check size limits
        if processed_payload.len() > self.config.max_payload_size {
            let truncated = self.truncate_with_summary(&processed_payload);
            self.record_span_event("request.payload.truncated", &[
                KeyValue::new("original_size", processed_payload.len() as i64),
                KeyValue::new("truncated_size", truncated.len() as i64),
            ]);
            self.current_span().set_attribute("request.payload", truncated);
        } else {
            self.current_span().set_attribute("request.payload", processed_payload);
        }
    }
    
    fn sanitize_payload(&self, payload: &str) -> String {
        // Implement PII redaction, pattern masking, etc.
    }
}
```

#### **Task 4: Add Structured JSON Events (Week 5)**
**Enhanced event recording:**

```rust
impl StoodTracer {
    pub fn record_structured_request(&self, request: &BedrockRequest) {
        let structured_data = json!({
            "model": request.model_id,
            "messages": request.messages.len(),
            "tools": request.tools.as_ref().map(|t| t.len()).unwrap_or(0),
            "max_tokens": request.inference_config.max_tokens,
            "temperature": request.inference_config.temperature,
            "system_prompt_length": request.system.as_ref().map(|s| s.len()).unwrap_or(0),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        
        self.current_span().add_event("bedrock.request.structured", &[
            KeyValue::new("request.data", structured_data.to_string())
        ]);
    }
    
    pub fn record_structured_response(&self, response: &BedrockResponse) {
        let structured_data = json!({
            "message_content_length": response.message.content.len(),
            "stop_reason": format!("{:?}", response.stop_reason),
            "duration_ms": response.duration.as_millis(),
            "attempts": response.attempts_made,
            "tokens": response.token_usage.as_ref().map(|t| json!({
                "input": t.input_tokens,
                "output": t.output_tokens,
                "total": t.total_tokens()
            })),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        
        self.current_span().add_event("bedrock.response.structured", &[
            KeyValue::new("response.data", structured_data.to_string())
        ]);
    }
}
```

#### **Task 5: Privacy and Security Controls (Week 6)**
**Add privacy-first configurations:**

```rust
impl TelemetryConfig {
    pub fn with_request_logging(mut self, enabled: bool) -> Self {
        self.log_full_requests = enabled;
        self
    }
    
    pub fn with_pii_redaction(mut self, enabled: bool) -> Self {
        self.request_sanitization.redact_pii = enabled;
        self
    }
    
    pub fn development_mode(mut self) -> Self {
        // Full logging for development
        self.log_full_requests = true;
        self.log_full_responses = true;
        self.request_sanitization.redact_pii = false;
        self
    }
    
    pub fn production_mode(mut self) -> Self {
        // Privacy-conscious for production
        self.log_full_requests = false;
        self.log_full_responses = false;
        self.request_sanitization.redact_pii = true;
        self
    }
}
```

#### **Task 6: Enhanced Demo and Validation (Week 7)**
**Update telemetry demo to showcase JSON logging:**

```rust
async fn run_json_logging_demo(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    info!("📝 Starting JSON Logging Demo");
    
    // Enable full JSON logging
    let config = TelemetryConfig::from_env()
        .development_mode()
        .with_request_logging(true);
    
    let tracer = StoodTracer::init(config)?;
    
    // Run a sample conversation
    let response = self.agent.execute("Explain quantum computing in simple terms").await?;
    
    info!("✅ JSON logging demo completed");
    info!("🔍 Check Jaeger for full request/response data");
    
    Ok(())
}
```

### **JSON Logging Benefits**

**Development & Debugging**
- Full conversation replay capability
- Request/response correlation with traces
- Model behavior analysis
- Prompt engineering optimization

**Production Monitoring**
- Conversation quality assessment
- Error context preservation
- Model performance analysis
- Compliance and audit trails

**Security & Privacy**
- Configurable PII redaction
- Payload size limits
- Opt-in/opt-out controls
- Development vs production modes

## 🔧 **Technical Implementation Details**

### **Current Telemetry Infrastructure Analysis**

#### **Core Types and Structures**
```rust
// From src/telemetry/mod.rs
pub struct TelemetryConfig {
    pub enabled: bool,
    pub otlp_endpoint: Option<String>,
    pub console_export: bool,
    pub service_name: String,
    pub service_version: String,
    pub enable_batch_processor: bool,
    pub service_attributes: HashMap<String, String>,
    pub enable_debug_tracing: bool,
}

pub struct EventLoopMetrics {
    pub cycles: Vec<CycleMetrics>,
    pub total_tokens: TokenUsage,
    pub total_duration: Duration,
    pub tool_executions: Vec<ToolExecutionMetric>,
    pub traces: Vec<TraceInfo>,
    pub accumulated_usage: AccumulatedMetrics,
}

pub struct StoodTracer {
    config: TelemetryConfig,
}
```

#### **Integration Points**
- **Smart Endpoint Detection**: `TelemetryConfig::from_env_with_detection()` auto-detects OTLP on ports 4318, 4320
- **Debug Logging**: OTLP exports logged to `~/.local/share/stood-telemetry/logs/otlp_exports.jsonl`
- **GenAI Conventions**: Existing semantic conventions in telemetry module
- **Existing Hooks**: Methods `record_prompt()` and `record_completion()` exist but unused

### **Bedrock Client Architecture**

#### **Core Implementation**
```rust
// From src/bedrock/mod.rs
pub struct BedrockClient {
    client: BedrockRuntimeClient,
    config: BedrockClientConfig,
    retry_executor: RetryExecutor,
    #[cfg(feature = "telemetry")]
    tracer: Option<StoodTracer>,
}

pub struct BedrockResponse {
    pub message: Message,
    pub duration: Duration,
    pub stop_reason: StopReason,
    pub raw_response: String,
    pub attempts_made: u32,
    pub total_duration: Duration,
    pub token_usage: Option<TokenUsage>,
    pub request_id: Option<String>,
}
```

#### **Metrics Integration Points**
- **`chat_with_tools_instrumented()`**: Primary telemetry integration method
- **Token Tracking**: Complete usage data in `parse_response()` 
- **Retry Metrics**: Attempt counting and timing
- **Error Context**: `BedrockErrorContext` for detailed error metrics

### **Event Loop and Agent Flow**

#### **Core Execution Architecture**
```rust
// From src/agent/event_loop.rs
pub struct EventLoop {
    agent: Agent,
    tool_registry: ToolRegistry,
    tool_executor: ToolExecutor,
    config: EventLoopConfig,
    metrics: EventLoopMetrics,
    stream_events: Vec<StreamEvent>,
    callback_handler: Option<Arc<dyn CallbackHandler>>,
    #[cfg(feature = "telemetry")]
    tracer: Option<StoodTracer>,
    performance_logger: PerformanceLogger,
    performance_tracer: PerformanceTracer,
}

pub struct EventLoopResult {
    pub response: String,
    pub cycles_executed: u32,
    pub total_duration: Duration,
    pub metrics: EventLoopMetrics,
    pub success: bool,
    pub error: Option<String>,
    pub was_streamed: bool,
    pub stream_events: Vec<StreamEvent>,
}
```

#### **5-Phase Execution Cycle**
1. **Reasoning Phase**: Model invocation with context
2. **Tool Selection**: Tool calling decision and validation
3. **Execution Phase**: Parallel tool execution with timing
4. **Reflection Phase**: Results integration and validation  
5. **Response Phase**: Final response generation

### **Tool System Architecture**

#### **Core Tool Framework**
```rust
// From src/tools/mod.rs
#[async_trait]
pub trait Tool: Send + Sync + std::fmt::Debug {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters_schema(&self) -> Value;
    async fn execute(&self, parameters: Option<Value>) -> Result<ToolResult, ToolError>;
    fn is_available(&self) -> bool { true }
    fn source(&self) -> ToolSource { ToolSource::Custom }
}

pub struct ToolRegistry {
    tools: Arc<RwLock<HashMap<String, Arc<dyn Tool>>>>,
}

pub struct ToolExecutor {
    config: ExecutorConfig,
}
```

#### **Metrics Collection Opportunities**
- **Tool Registration**: Dynamic tool availability tracking
- **Execution Timing**: Per-tool performance profiling
- **Parallel Execution**: Concurrency utilization metrics
- **Success/Failure**: Tool reliability tracking

### **Callback System for Real-Time Metrics**

#### **Event-Driven Architecture**
```rust
// From src/agent/callbacks/mod.rs
pub trait CallbackHandler: Send + Sync {
    fn on_event(&self, event: CallbackEvent) -> Result<(), CallbackError>;
}

pub enum CallbackHandlerConfig {
    None,
    Printing(PrintingConfig),
    Custom(Arc<dyn CallbackHandler>),
    Composite(Vec<CallbackHandlerConfig>),
    Performance(tracing::Level),
    Batching { inner: Box<CallbackHandlerConfig>, batch_config: BatchConfig },
}

pub enum CallbackEvent {
    AgentStart { agent_id: String },
    TokenUsage { input_tokens: u32, output_tokens: u32 },
    ToolExecution(ToolEvent),
    // ... more events
}
```

#### **Metrics Integration Strategy**
- **Real-Time Events**: Stream metrics as operations complete
- **Batching Support**: Efficient high-frequency metric collection
- **Composite Handlers**: Multiple metrics destinations
- **Performance Optimization**: Minimal overhead callback processing

### **Error Handling and Metrics**

#### **Structured Error Types**
```rust
// From src/error.rs
pub enum StoodError {
    InvalidInput(String),
    ConfigurationError(String),
    ModelError(String),
    ToolError(String),
    ConversationError(String),
}

impl StoodError {
    pub fn is_retryable(&self) -> bool;
    pub fn is_auth_error(&self) -> bool;
    pub fn is_user_error(&self) -> bool;
    pub fn retry_delay_ms(&self) -> Option<u64>;
}
```

#### **Error Metrics Integration**
- **Error Classification**: Automatic categorization for metrics
- **Retry Tracking**: Attempt counting and backoff timing
- **Error Context**: Rich metadata for troubleshooting
- **Component Attribution**: Error source identification

### **Performance Monitoring Infrastructure**

#### **Current Components**
```rust
pub struct PerformanceLogger {
    pub cycle_times: Vec<Duration>,
    pub tool_times: HashMap<String, Vec<Duration>>,
    pub model_invoke_times: Vec<Duration>,
    pub total_cycles: u32,
    pub total_input_tokens: u32,
    pub total_output_tokens: u32,
}

pub struct PerformanceTracer {
    // Performance tracking implementation with RAII guards
}
```

### **Dependencies and Feature Configuration**

#### **Current Telemetry Stack**
```toml
# From Cargo.toml
[dependencies]
opentelemetry = { version = "0.24", optional = true }
opentelemetry-otlp = { version = "0.17", optional = true }
opentelemetry_sdk = { version = "0.24", features = ["rt-tokio"], optional = true }
opentelemetry-semantic-conventions = { version = "0.16", optional = true }
opentelemetry-stdout = { version = "0.5", optional = true }
tracing-opentelemetry = { version = "0.25", optional = true }

[features]
telemetry = [
    "opentelemetry",
    "opentelemetry-otlp", 
    "opentelemetry_sdk",
    "opentelemetry-semantic-conventions",
    "opentelemetry-stdout",
    "tracing-opentelemetry"
]
```

#### **Required Additions for Metrics**
```toml
# Enhanced telemetry dependencies
opentelemetry = { version = "0.24", features = ["metrics"], optional = true }
opentelemetry_sdk = { version = "0.24", features = ["metrics", "rt-tokio"], optional = true }
opentelemetry-otlp = { version = "0.17", features = ["metrics", "grpc-tonic"], optional = true }
```

### **Configuration Patterns**

#### **Smart Configuration System**
```rust
impl TelemetryConfig {
    // Follow existing patterns
    pub fn from_env() -> Self; // Current implementation
    pub fn from_env_with_detection() -> Self; // Current implementation
    
    // New metric configurations
    pub fn with_metrics(mut self, enabled: bool) -> Self;
    pub fn metrics_export_interval(mut self, interval: Duration) -> Self;
    pub fn development_mode(mut self) -> Self;
    pub fn production_mode(mut self) -> Self;
}
```

### **Module Organization Strategy**

#### **Recommended Structure**
```
src/telemetry/
├── mod.rs              # Core types (existing)
├── otel.rs             # StoodTracer (existing)
├── otlp_debug.rs       # Debug logging (existing)
├── logging.rs          # Logging config (existing)
└── metrics/            # New metrics module
    ├── mod.rs          # Core metrics types
    ├── collector.rs    # MetricsCollector trait
    ├── exporter.rs     # OTLP metrics exporter
    ├── semantic_conventions.rs  # Metrics naming
    └── system.rs       # System resource metrics
```

### **Integration Pattern Consistency**

#### **Follow Existing Patterns**
```rust
// Initialization (follow StoudTracer::init)
impl MetricsCollector {
    pub fn init(config: TelemetryConfig) -> StoodResult<Option<Self>>;
}

// Agent Integration (follow tracer pattern)
pub struct Agent {
    #[cfg(feature = "telemetry")]
    tracer: Option<StoodTracer>,
    #[cfg(feature = "telemetry")]
    metrics_collector: Option<MetricsCollector>,
}

// Configuration (follow TelemetryConfig methods)
impl TelemetryConfig {
    pub fn with_metrics(mut self, enabled: bool) -> Self;
    pub fn metrics_batch_size(mut self, size: usize) -> Self;
}
```

This comprehensive technical analysis provides the detailed foundation needed to implement the complete agentic metrics strategy while maintaining consistency with the existing Stood architecture and patterns.

## 🏗️ **Architecture Highlights**

✅ **Leverages Existing Infrastructure**: Built on current EventLoopMetrics, PerformanceMetrics, and StoodTracer
✅ **OTLP-Native**: Seamless integration with existing trace export pipeline  
✅ **Zero-Disruption**: Metrics layer sits above existing code with minimal changes
✅ **Production-Ready**: Batching, compression, sampling, memory management

## 📋 **Implementation Roadmap (12 weeks)**

**Phase 1-2 (Weeks 1-4)**: Foundation + Core Metrics
- OpenTelemetry metrics SDK integration
- Token usage, request latency, tool execution metrics

**Phase 3-4 (Weeks 5-8)**: Advanced Features  
- System resource monitoring, error tracking
- Configuration system, performance optimization

**Phase 5-6 (Weeks 9-12)**: Production Deployment
- Enhanced demo, Grafana dashboards, documentation
- Load testing, monitoring setup, deployment guides

## 🚀 **Expected Benefits**

**Operational Visibility**
- Real-time agent performance monitoring
- Cost tracking and optimization opportunities  
- Proactive error detection and alerting

**Development Insights**
- Tool performance profiling
- Model efficiency analysis
- Agentic reasoning pattern identification

**Business Intelligence**
- User experience metrics
- Resource utilization optimization
- Cost per interaction tracking

This comprehensive metrics strategy transforms Stood from a functional agent library into a fully observable, production-ready agentic platform with enterprise-grade monitoring capabilities.