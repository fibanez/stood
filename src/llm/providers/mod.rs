//! Provider implementations.
//!
//! Each provider implements the LlmProvider trait and owns all logic for its models.
//!
//! ## Implementation Status
//! 
//! **✅ Fully Implemented:**
//! - `bedrock` - AWS Bedrock integration (Claude, Nova models)
//! - `lm_studio` - Local development and testing
//!
//! **🚧 Planned (Not Yet Implemented):**
//! - `anthropic` - Direct Anthropic API access  
//! - `openai` - OpenAI GPT models
//! - `ollama` - Local LLM hosting
//! - `openrouter` - Multi-provider proxy
//! - `candle` - Rust-native inference
//!
//! See README.md "TODO - Work in Progress" section for implementation roadmap.

// Implemented providers
pub mod bedrock;
pub mod lm_studio;

// Retry utilities for provider resilience
pub mod retry;

// Placeholder providers (not yet implemented - see README.md)
// These modules contain skeleton implementations that return appropriate errors
pub mod anthropic;
pub mod openai;
pub mod ollama;
pub mod openrouter;
pub mod candle;

// Re-export all providers
pub use bedrock::BedrockProvider;
pub use lm_studio::LMStudioProvider;
pub use anthropic::AnthropicProvider;
pub use openai::OpenAIProvider;
pub use ollama::OllamaProvider;
pub use openrouter::OpenRouterProvider;
pub use candle::CandleProvider;