//! OpenTelemetry GenAI Semantic Conventions
//!
//! Implements span naming and attributes per:
//! <https://opentelemetry.io/docs/specs/semconv/gen-ai/>
//!
//! These types ensure correct attribute names and values for CloudWatch
//! Gen AI Observability dashboards.

/// GenAI operation types
///
/// Maps to OTEL semantic convention span names: `{operation} {model/agent/tool}`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenAiOperation {
    /// Chat completion (e.g., "chat claude-3-haiku")
    Chat,
    /// Text completion
    TextCompletion,
    /// Generic content generation
    GenerateContent,
    /// Embedding generation
    Embeddings,
    /// Agent creation
    CreateAgent,
    /// Agent invocation (e.g., "invoke_agent my-agent")
    InvokeAgent,
    /// Tool execution (e.g., "execute_tool get_weather")
    ExecuteTool,
}

impl GenAiOperation {
    /// Get OTEL-compliant operation name
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Chat => "chat",
            Self::TextCompletion => "text_completion",
            Self::GenerateContent => "generate_content",
            Self::Embeddings => "embeddings",
            Self::CreateAgent => "create_agent",
            Self::InvokeAgent => "invoke_agent",
            Self::ExecuteTool => "execute_tool",
        }
    }

    /// Generate span name following OTEL convention: "{operation} {target}"
    pub fn span_name(&self, target: &str) -> String {
        format!("{} {}", self.as_str(), target)
    }
}

impl std::fmt::Display for GenAiOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// GenAI provider identifiers
///
/// Maps to `gen_ai.provider.name` attribute values
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenAiProvider {
    /// AWS Bedrock (Claude, Nova, Titan, etc.)
    AwsBedrock,
    /// Anthropic direct API
    Anthropic,
    /// OpenAI
    OpenAi,
    /// Azure OpenAI Service
    AzureOpenAi,
    /// Google Cloud Vertex AI
    GcpVertexAi,
    /// LM Studio (local)
    LmStudio,
    /// Ollama (local)
    Ollama,
    /// Custom provider
    Custom(String),
}

impl GenAiProvider {
    /// Get OTEL-compliant provider name
    pub fn as_str(&self) -> &str {
        match self {
            Self::AwsBedrock => "aws.bedrock",
            Self::Anthropic => "anthropic",
            Self::OpenAi => "openai",
            Self::AzureOpenAi => "azure.ai.openai",
            Self::GcpVertexAi => "gcp.vertex_ai",
            Self::LmStudio => "lm_studio",
            Self::Ollama => "ollama",
            Self::Custom(s) => s,
        }
    }
}

impl std::fmt::Display for GenAiProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Default for GenAiProvider {
    fn default() -> Self {
        Self::AwsBedrock
    }
}

/// GenAI tool types
///
/// Maps to `gen_ai.tool.type` attribute values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GenAiToolType {
    /// Function tool (most common)
    #[default]
    Function,
    /// Extension tool
    Extension,
    /// Datastore tool (RAG)
    Datastore,
}

impl GenAiToolType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Function => "function",
            Self::Extension => "extension",
            Self::Datastore => "datastore",
        }
    }
}

impl std::fmt::Display for GenAiToolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// OTEL GenAI semantic convention attribute names
///
/// These constants ensure attribute names match exactly what CloudWatch
/// Gen AI Observability expects. Using incorrect names will cause traces
/// to not appear in the dashboards.
pub mod attrs {
    // ========================================================================
    // Core attributes (required for all GenAI spans)
    // ========================================================================

    /// The operation being performed (chat, invoke_agent, execute_tool, etc.)
    pub const OPERATION_NAME: &str = "gen_ai.operation.name";

    /// The AI provider (aws.bedrock, anthropic, openai, etc.)
    pub const PROVIDER_NAME: &str = "gen_ai.provider.name";

    // ========================================================================
    // Request attributes
    // ========================================================================

    /// The model ID being used (e.g., "anthropic.claude-3-haiku-20240307-v1:0")
    pub const REQUEST_MODEL: &str = "gen_ai.request.model";

    /// Maximum tokens requested
    pub const REQUEST_MAX_TOKENS: &str = "gen_ai.request.max_tokens";

    /// Temperature parameter (0.0-1.0)
    pub const REQUEST_TEMPERATURE: &str = "gen_ai.request.temperature";

    /// Top-p (nucleus sampling) parameter
    pub const REQUEST_TOP_P: &str = "gen_ai.request.top_p";

    /// Stop sequences
    pub const REQUEST_STOP_SEQUENCES: &str = "gen_ai.request.stop_sequences";

    // ========================================================================
    // Response attributes
    // ========================================================================

    /// Response ID from the provider
    pub const RESPONSE_ID: &str = "gen_ai.response.id";

    /// Model that generated the response (may differ from request)
    pub const RESPONSE_MODEL: &str = "gen_ai.response.model";

    /// Reasons why generation finished (end_turn, tool_use, max_tokens, etc.)
    pub const RESPONSE_FINISH_REASONS: &str = "gen_ai.response.finish_reasons";

    // ========================================================================
    // Usage attributes (tokens)
    // ========================================================================

    /// Number of input tokens
    pub const USAGE_INPUT_TOKENS: &str = "gen_ai.usage.input_tokens";

    /// Number of output tokens
    pub const USAGE_OUTPUT_TOKENS: &str = "gen_ai.usage.output_tokens";

    // ========================================================================
    // Agent attributes
    // ========================================================================

    /// Unique agent identifier
    pub const AGENT_ID: &str = "gen_ai.agent.id";

    /// Human-readable agent name
    pub const AGENT_NAME: &str = "gen_ai.agent.name";

    /// GenAI system/framework identifier (e.g., "stood", "strands", "langchain")
    /// Note: This is deprecated in OTEL spec but may be required by some dashboards
    pub const SYSTEM: &str = "gen_ai.system";

    /// Agent description
    pub const AGENT_DESCRIPTION: &str = "gen_ai.agent.description";

    /// Conversation/session identifier
    pub const CONVERSATION_ID: &str = "gen_ai.conversation.id";

    // ========================================================================
    // Tool attributes
    // ========================================================================

    /// Tool name
    pub const TOOL_NAME: &str = "gen_ai.tool.name";

    /// Tool type (function, extension, datastore)
    pub const TOOL_TYPE: &str = "gen_ai.tool.type";

    /// Tool description
    pub const TOOL_DESCRIPTION: &str = "gen_ai.tool.description";

    /// Unique ID for this tool call
    pub const TOOL_CALL_ID: &str = "gen_ai.tool.call.id";

    /// JSON-encoded arguments passed to the tool
    pub const TOOL_CALL_ARGUMENTS: &str = "gen_ai.tool.call.arguments";

    /// JSON-encoded result from the tool
    pub const TOOL_CALL_RESULT: &str = "gen_ai.tool.call.result";

    /// JSON-encoded tool definitions available
    pub const TOOL_DEFINITIONS: &str = "gen_ai.tool.definitions";

    // ========================================================================
    // Content attributes (opt-in, PII risk)
    // ========================================================================

    /// Input messages (JSON array) - contains PII
    pub const INPUT_MESSAGES: &str = "gen_ai.input.messages";

    /// Output messages (JSON array) - contains PII
    pub const OUTPUT_MESSAGES: &str = "gen_ai.output.messages";

    /// System prompt - may contain sensitive instructions
    pub const SYSTEM_INSTRUCTIONS: &str = "gen_ai.system_instructions";

    // ========================================================================
    // Stood-specific attributes
    // ========================================================================

    /// Stood library version
    pub const STOOD_VERSION: &str = "stood.version";

    /// Execution cycle ID within an agent run
    pub const STOOD_CYCLE_ID: &str = "stood.cycle.id";

    /// Tool execution ID
    pub const STOOD_TOOL_EXECUTION_ID: &str = "stood.tool.execution_id";

    // ========================================================================
    // AWS CloudWatch GenAI Dashboard attributes
    // ========================================================================
    // These are required for traces to appear in the CloudWatch GenAI Dashboard.
    // The dashboard queries filter on these specific attributes.

    /// Session ID for CloudWatch GenAI Dashboard session tracking
    /// Query uses: attributes.session.id as sessionId
    pub const SESSION_ID: &str = "session.id";

    /// AWS X-Ray origin for span categorization
    /// Query filters on: attributes.aws.xray.origin
    /// Valid values for GenAI agents:
    /// - "AWS::BedrockAgentCore::Runtime" - Agent runtime spans
    /// - "AWS::BedrockAgentCore::RuntimeEndpoint" - Endpoint spans
    /// - "AWS::BedrockAgentCore::Memory" - Memory operation spans
    /// - "AWS::BedrockAgentCore::Browser" - Browser tool spans
    /// - "AWS::BedrockAgentCore::CodeInterpreter" - Code interpreter spans
    pub const AWS_XRAY_ORIGIN: &str = "aws.xray.origin";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_names_match_otel_spec() {
        assert_eq!(GenAiOperation::Chat.as_str(), "chat");
        assert_eq!(GenAiOperation::TextCompletion.as_str(), "text_completion");
        assert_eq!(GenAiOperation::GenerateContent.as_str(), "generate_content");
        assert_eq!(GenAiOperation::Embeddings.as_str(), "embeddings");
        assert_eq!(GenAiOperation::CreateAgent.as_str(), "create_agent");
        assert_eq!(GenAiOperation::InvokeAgent.as_str(), "invoke_agent");
        assert_eq!(GenAiOperation::ExecuteTool.as_str(), "execute_tool");
    }

    #[test]
    fn test_span_name_format() {
        assert_eq!(
            GenAiOperation::Chat.span_name("claude-3-haiku"),
            "chat claude-3-haiku"
        );
        assert_eq!(
            GenAiOperation::InvokeAgent.span_name("my-agent"),
            "invoke_agent my-agent"
        );
        assert_eq!(
            GenAiOperation::ExecuteTool.span_name("get_weather"),
            "execute_tool get_weather"
        );
    }

    #[test]
    fn test_provider_names_match_otel_spec() {
        assert_eq!(GenAiProvider::AwsBedrock.as_str(), "aws.bedrock");
        assert_eq!(GenAiProvider::Anthropic.as_str(), "anthropic");
        assert_eq!(GenAiProvider::OpenAi.as_str(), "openai");
        assert_eq!(GenAiProvider::AzureOpenAi.as_str(), "azure.ai.openai");
        assert_eq!(GenAiProvider::GcpVertexAi.as_str(), "gcp.vertex_ai");
        assert_eq!(GenAiProvider::LmStudio.as_str(), "lm_studio");
        assert_eq!(GenAiProvider::Ollama.as_str(), "ollama");
        assert_eq!(
            GenAiProvider::Custom("custom".to_string()).as_str(),
            "custom"
        );
    }

    #[test]
    fn test_tool_type_names() {
        assert_eq!(GenAiToolType::Function.as_str(), "function");
        assert_eq!(GenAiToolType::Extension.as_str(), "extension");
        assert_eq!(GenAiToolType::Datastore.as_str(), "datastore");
    }

    #[test]
    fn test_attribute_names_match_otel_spec() {
        // Core attributes - must match exactly for CloudWatch parsing
        assert_eq!(attrs::OPERATION_NAME, "gen_ai.operation.name");
        assert_eq!(attrs::PROVIDER_NAME, "gen_ai.provider.name");

        // Request attributes
        assert_eq!(attrs::REQUEST_MODEL, "gen_ai.request.model");
        assert_eq!(attrs::REQUEST_MAX_TOKENS, "gen_ai.request.max_tokens");
        assert_eq!(attrs::REQUEST_TEMPERATURE, "gen_ai.request.temperature");

        // Response attributes
        assert_eq!(attrs::RESPONSE_ID, "gen_ai.response.id");
        assert_eq!(attrs::RESPONSE_MODEL, "gen_ai.response.model");
        assert_eq!(
            attrs::RESPONSE_FINISH_REASONS,
            "gen_ai.response.finish_reasons"
        );

        // Usage attributes
        assert_eq!(attrs::USAGE_INPUT_TOKENS, "gen_ai.usage.input_tokens");
        assert_eq!(attrs::USAGE_OUTPUT_TOKENS, "gen_ai.usage.output_tokens");

        // Agent attributes
        assert_eq!(attrs::AGENT_ID, "gen_ai.agent.id");
        assert_eq!(attrs::AGENT_NAME, "gen_ai.agent.name");
        assert_eq!(attrs::CONVERSATION_ID, "gen_ai.conversation.id");

        // Tool attributes
        assert_eq!(attrs::TOOL_NAME, "gen_ai.tool.name");
        assert_eq!(attrs::TOOL_TYPE, "gen_ai.tool.type");
        assert_eq!(attrs::TOOL_CALL_ID, "gen_ai.tool.call.id");
    }

    #[test]
    fn test_default_provider_is_bedrock() {
        assert_eq!(GenAiProvider::default(), GenAiProvider::AwsBedrock);
    }

    #[test]
    fn test_default_tool_type_is_function() {
        assert_eq!(GenAiToolType::default(), GenAiToolType::Function);
    }

    #[test]
    fn test_display_implementations() {
        assert_eq!(format!("{}", GenAiOperation::Chat), "chat");
        assert_eq!(format!("{}", GenAiProvider::AwsBedrock), "aws.bedrock");
        assert_eq!(format!("{}", GenAiToolType::Function), "function");
    }
}
