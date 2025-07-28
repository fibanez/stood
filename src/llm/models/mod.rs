//! Model definitions organized by provider.
//!
//! This module defines models as pure metadata structs with no logic.
//! Each provider module exports its model structs for use with the single API pattern.

use crate::llm::traits::{LlmModel, ModelCapabilities, ProviderType};

/// AWS Bedrock provider models
///
/// Note: All model IDs use the "us." prefix for cross-region inference capability.
/// This allows models to be accessed from any AWS region, not just us-east-1.
#[allow(non_snake_case)]
pub mod Bedrock {
    use super::*;

    /// Claude 3.5 Sonnet v2 via AWS Bedrock
    #[derive(Debug, Clone, Copy)]
    pub struct Claude35Sonnet;

    /// Claude 3.5 Haiku via AWS Bedrock  
    #[derive(Debug, Clone, Copy)]
    pub struct Claude35Haiku;

    /// Claude 3 Haiku via AWS Bedrock
    #[derive(Debug, Clone, Copy)]
    pub struct ClaudeHaiku3;

    /// Claude 3 Opus via AWS Bedrock
    #[derive(Debug, Clone, Copy)]
    pub struct ClaudeOpus3;

    /// Amazon Nova Lite via AWS Bedrock
    #[derive(Debug, Clone, Copy)]
    pub struct NovaLite;

    /// Amazon Nova Pro via AWS Bedrock
    #[derive(Debug, Clone, Copy)]
    pub struct NovaPro;

    /// Amazon Nova Micro via AWS Bedrock
    #[derive(Debug, Clone, Copy)]
    pub struct NovaMicro;

    // Implement LlmModel trait for all Bedrock models

    impl LlmModel for Claude35Sonnet {
        fn model_id(&self) -> &'static str {
            // Note: us. prefix required for cross-region inference in AWS Bedrock
            "us.anthropic.claude-3-5-sonnet-20241022-v2:0"
        }
        fn provider(&self) -> ProviderType {
            ProviderType::Bedrock
        }
        fn context_window(&self) -> usize {
            200_000
        }
        fn max_output_tokens(&self) -> usize {
            8_192
        }
        fn capabilities(&self) -> ModelCapabilities {
            ModelCapabilities {
                max_tokens: Some(8_192),
                supports_tools: true,
                supports_streaming: true,
                supports_thinking: true,
                supports_vision: true,
                context_window: Some(200_000),
            }
        }
        fn display_name(&self) -> &'static str {
            "Claude 3.5 Sonnet"
        }
        fn default_temperature(&self) -> f32 {
            0.7
        }
        fn default_max_tokens(&self) -> u32 {
            8_192
        }
    }

    impl LlmModel for Claude35Haiku {
        fn model_id(&self) -> &'static str {
            // Note: us. prefix required for cross-region inference in AWS Bedrock
            "us.anthropic.claude-3-5-haiku-20241022-v1:0"
        }
        fn provider(&self) -> ProviderType {
            ProviderType::Bedrock
        }
        fn context_window(&self) -> usize {
            200_000
        }
        fn max_output_tokens(&self) -> usize {
            8_192
        }
        fn capabilities(&self) -> ModelCapabilities {
            ModelCapabilities {
                max_tokens: Some(8_192),
                supports_tools: true,
                supports_streaming: true,
                supports_thinking: false,
                supports_vision: true,
                context_window: Some(200_000),
            }
        }
        fn display_name(&self) -> &'static str {
            "Claude 3.5 Haiku"
        }
        fn default_temperature(&self) -> f32 {
            0.8
        }
        fn default_max_tokens(&self) -> u32 {
            8_192
        }
    }

    impl LlmModel for ClaudeHaiku3 {
        fn model_id(&self) -> &'static str {
            // Note: us. prefix required for cross-region inference in AWS Bedrock
            "us.anthropic.claude-3-haiku-20240307-v1:0"
        }
        fn provider(&self) -> ProviderType {
            ProviderType::Bedrock
        }
        fn context_window(&self) -> usize {
            200_000
        }
        fn max_output_tokens(&self) -> usize {
            4_096
        }
        fn capabilities(&self) -> ModelCapabilities {
            ModelCapabilities {
                max_tokens: Some(4_096),
                supports_tools: true,
                supports_streaming: true,
                supports_thinking: false,
                supports_vision: true,
                context_window: Some(200_000),
            }
        }
        fn display_name(&self) -> &'static str {
            "Claude 3 Haiku"
        }
        fn default_temperature(&self) -> f32 {
            0.8
        }
        fn default_max_tokens(&self) -> u32 {
            4_096
        }
    }

    impl LlmModel for ClaudeOpus3 {
        fn model_id(&self) -> &'static str {
            // Note: us. prefix required for cross-region inference in AWS Bedrock
            "us.anthropic.claude-3-opus-20240229-v1:0"
        }
        fn provider(&self) -> ProviderType {
            ProviderType::Bedrock
        }
        fn context_window(&self) -> usize {
            200_000
        }
        fn max_output_tokens(&self) -> usize {
            4_096
        }
        fn capabilities(&self) -> ModelCapabilities {
            ModelCapabilities {
                max_tokens: Some(4_096),
                supports_tools: true,
                supports_streaming: true,
                supports_thinking: false,
                supports_vision: true,
                context_window: Some(200_000),
            }
        }
        fn display_name(&self) -> &'static str {
            "Claude 3 Opus"
        }
        fn default_temperature(&self) -> f32 {
            0.6
        }
        fn default_max_tokens(&self) -> u32 {
            4_096
        }
    }

    impl LlmModel for NovaLite {
        fn model_id(&self) -> &'static str {
            // Note: us. prefix required for cross-region inference in AWS Bedrock
            "us.amazon.nova-lite-v1:0"
        }
        fn provider(&self) -> ProviderType {
            ProviderType::Bedrock
        }
        fn context_window(&self) -> usize {
            300_000
        }
        fn max_output_tokens(&self) -> usize {
            5_000
        }
        fn capabilities(&self) -> ModelCapabilities {
            ModelCapabilities {
                max_tokens: Some(5_000),
                supports_tools: true,
                supports_streaming: true,
                supports_thinking: false,
                supports_vision: false,
                context_window: Some(300_000),
            }
        }
        fn display_name(&self) -> &'static str {
            "Amazon Nova Lite"
        }
        fn default_temperature(&self) -> f32 {
            0.7
        }
        fn default_max_tokens(&self) -> u32 {
            5_000
        }
    }

    impl LlmModel for NovaPro {
        fn model_id(&self) -> &'static str {
            // Note: us. prefix required for cross-region inference in AWS Bedrock
            "us.amazon.nova-pro-v1:0"
        }
        fn provider(&self) -> ProviderType {
            ProviderType::Bedrock
        }
        fn context_window(&self) -> usize {
            300_000
        }
        fn max_output_tokens(&self) -> usize {
            5_000
        }
        fn capabilities(&self) -> ModelCapabilities {
            ModelCapabilities {
                max_tokens: Some(5_000),
                supports_tools: true,
                supports_streaming: true,
                supports_thinking: false,
                supports_vision: true,
                context_window: Some(300_000),
            }
        }
        fn display_name(&self) -> &'static str {
            "Amazon Nova Pro"
        }
        fn default_temperature(&self) -> f32 {
            0.7
        }
        fn default_max_tokens(&self) -> u32 {
            5_000
        }
    }

    impl LlmModel for NovaMicro {
        fn model_id(&self) -> &'static str {
            // Note: us. prefix required for cross-region inference in AWS Bedrock
            "us.amazon.nova-micro-v1:0"
        }
        fn provider(&self) -> ProviderType {
            ProviderType::Bedrock
        }
        fn context_window(&self) -> usize {
            128_000
        }
        fn max_output_tokens(&self) -> usize {
            2_048
        }
        fn capabilities(&self) -> ModelCapabilities {
            ModelCapabilities {
                max_tokens: Some(2_048),
                supports_tools: true,
                supports_streaming: true,
                supports_thinking: false,
                supports_vision: false,
                context_window: Some(128_000),
            }
        }
        fn display_name(&self) -> &'static str {
            "Amazon Nova Micro"
        }
        fn default_temperature(&self) -> f32 {
            0.7
        }
        fn default_max_tokens(&self) -> u32 {
            2_048
        }
    }
}

/// LM Studio provider models
#[allow(non_snake_case)]
pub mod LMStudio {
    use super::*;

    /// Gemma 3 12B model via LM Studio
    #[derive(Debug, Clone, Copy)]
    pub struct Gemma3_12B;

    /// Llama 3 70B model via LM Studio  
    #[derive(Debug, Clone, Copy)]
    pub struct Llama3_70B;

    /// Gemma 3 27B model via LM Studio
    #[derive(Debug, Clone, Copy)]
    pub struct Gemma3_27B;

    /// Mistral 7B model via LM Studio
    #[derive(Debug, Clone, Copy)]
    pub struct Mistral7B;

    /// Tessa Rust 7B model via LM Studio - specialized for Rust code analysis
    #[derive(Debug, Clone, Copy)]
    pub struct TessaRust7B;

    impl LlmModel for Gemma3_12B {
        fn model_id(&self) -> &'static str {
            "google/gemma-3-12b"
        }
        fn provider(&self) -> ProviderType {
            ProviderType::LmStudio
        }
        fn context_window(&self) -> usize {
            8_192
        }
        fn max_output_tokens(&self) -> usize {
            2_048
        }
        fn capabilities(&self) -> ModelCapabilities {
            ModelCapabilities {
                max_tokens: Some(2_048),
                supports_tools: true, // Depends on LM Studio setup
                supports_streaming: true,
                supports_thinking: false,
                supports_vision: false,
                context_window: Some(8_192),
            }
        }
        fn display_name(&self) -> &'static str {
            "Gemma 3 12B (Local)"
        }
        fn default_temperature(&self) -> f32 {
            0.7
        }
        fn default_max_tokens(&self) -> u32 {
            2_048
        }
    }

    impl LlmModel for Gemma3_27B {
        fn model_id(&self) -> &'static str {
            "google/gemma-3-27b"
        }
        fn provider(&self) -> ProviderType {
            ProviderType::LmStudio
        }
        fn context_window(&self) -> usize {
            8_192
        }
        fn max_output_tokens(&self) -> usize {
            4_096
        }
        fn capabilities(&self) -> ModelCapabilities {
            ModelCapabilities {
                max_tokens: Some(4_096),
                supports_tools: true, // Larger model should support tools better
                supports_streaming: true,
                supports_thinking: false,
                supports_vision: false,
                context_window: Some(8_192),
            }
        }
        fn display_name(&self) -> &'static str {
            "Gemma 3 27B (Local)"
        }
        fn default_temperature(&self) -> f32 {
            0.7
        }
        fn default_max_tokens(&self) -> u32 {
            4_096
        }
    }

    impl LlmModel for Llama3_70B {
        fn model_id(&self) -> &'static str {
            "llama-3-70b"
        }
        fn provider(&self) -> ProviderType {
            ProviderType::LmStudio
        }
        fn context_window(&self) -> usize {
            32_768
        }
        fn max_output_tokens(&self) -> usize {
            4_096
        }
        fn capabilities(&self) -> ModelCapabilities {
            ModelCapabilities {
                max_tokens: Some(4_096),
                supports_tools: false, // Depends on LM Studio setup
                supports_streaming: true,
                supports_thinking: false,
                supports_vision: false,
                context_window: Some(32_768),
            }
        }
        fn display_name(&self) -> &'static str {
            "Llama 3 70B (Local)"
        }
        fn default_temperature(&self) -> f32 {
            0.7
        }
        fn default_max_tokens(&self) -> u32 {
            4_096
        }
    }

    impl LlmModel for Mistral7B {
        fn model_id(&self) -> &'static str {
            "mistralai/mistral-7b-instruct-v0.3"
        }
        fn provider(&self) -> ProviderType {
            ProviderType::LmStudio
        }
        fn context_window(&self) -> usize {
            32_768
        }
        fn max_output_tokens(&self) -> usize {
            2_048
        }
        fn capabilities(&self) -> ModelCapabilities {
            ModelCapabilities {
                max_tokens: Some(2_048),
                supports_tools: false,
                supports_streaming: true,
                supports_thinking: false,
                supports_vision: false,
                context_window: Some(32_768),
            }
        }
        fn display_name(&self) -> &'static str {
            "Mistral 7B (Local)"
        }
        fn default_temperature(&self) -> f32 {
            0.7
        }
        fn default_max_tokens(&self) -> u32 {
            2_048
        }
    }

    impl LlmModel for TessaRust7B {
        fn model_id(&self) -> &'static str {
            "tessa-rust-t1-7b"
        }
        fn provider(&self) -> ProviderType {
            ProviderType::LmStudio
        }
        fn context_window(&self) -> usize {
            8_192
        }
        fn max_output_tokens(&self) -> usize {
            2_048
        }
        fn capabilities(&self) -> ModelCapabilities {
            ModelCapabilities {
                max_tokens: Some(2_048),
                supports_tools: true,
                supports_streaming: true,
                supports_thinking: false,
                supports_vision: false,
                context_window: Some(8_192),
            }
        }
        fn display_name(&self) -> &'static str {
            "Tessa Rust 7B (Local)"
        }
        fn default_temperature(&self) -> f32 {
            0.3  // Lower temperature for more precise code analysis
        }
        fn default_max_tokens(&self) -> u32 {
            2_048
        }
    }
}

/// Anthropic Direct API provider models  
#[allow(non_snake_case)]
pub mod Anthropic {
    use super::*;

    /// Claude 3.5 Sonnet via Anthropic Direct API
    #[derive(Debug, Clone, Copy)]
    pub struct Claude35Sonnet;

    /// Claude 3.5 Haiku via Anthropic Direct API
    #[derive(Debug, Clone, Copy)]
    pub struct Claude35Haiku;

    /// Claude 3 Opus via Anthropic Direct API
    #[derive(Debug, Clone, Copy)]
    pub struct Claude3Opus;

    impl LlmModel for Claude35Sonnet {
        fn model_id(&self) -> &'static str {
            "claude-3-5-sonnet-20241022"
        }
        fn provider(&self) -> ProviderType {
            ProviderType::Anthropic
        }
        fn context_window(&self) -> usize {
            200_000
        }
        fn max_output_tokens(&self) -> usize {
            8_192
        }
        fn capabilities(&self) -> ModelCapabilities {
            ModelCapabilities {
                max_tokens: Some(8_192),
                supports_tools: true,
                supports_streaming: false, // TODO: Implement streaming
                supports_thinking: false,  // TODO: Add thinking mode
                supports_vision: false,    // TODO: Add vision support
                context_window: Some(200_000),
            }
        }
        fn display_name(&self) -> &'static str {
            "Claude 3.5 Sonnet (Direct)"
        }
        fn default_temperature(&self) -> f32 {
            0.7
        }
        fn default_max_tokens(&self) -> u32 {
            8_192
        }
    }

    impl LlmModel for Claude35Haiku {
        fn model_id(&self) -> &'static str {
            "claude-3-5-haiku-20241022"
        }
        fn provider(&self) -> ProviderType {
            ProviderType::Anthropic
        }
        fn context_window(&self) -> usize {
            200_000
        }
        fn max_output_tokens(&self) -> usize {
            8_192
        }
        fn capabilities(&self) -> ModelCapabilities {
            ModelCapabilities {
                max_tokens: Some(8_192),
                supports_tools: true,
                supports_streaming: false, // TODO: Implement streaming
                supports_thinking: false,  // TODO: Add thinking mode
                supports_vision: false,    // TODO: Add vision support
                context_window: Some(200_000),
            }
        }
        fn display_name(&self) -> &'static str {
            "Claude 3.5 Haiku (Direct)"
        }
        fn default_temperature(&self) -> f32 {
            0.8
        }
        fn default_max_tokens(&self) -> u32 {
            8_192
        }
    }

    impl LlmModel for Claude3Opus {
        fn model_id(&self) -> &'static str {
            "claude-3-opus-20240229"
        }
        fn provider(&self) -> ProviderType {
            ProviderType::Anthropic
        }
        fn context_window(&self) -> usize {
            200_000
        }
        fn max_output_tokens(&self) -> usize {
            4_096
        }
        fn capabilities(&self) -> ModelCapabilities {
            ModelCapabilities {
                max_tokens: Some(4_096),
                supports_tools: true,
                supports_streaming: false, // TODO: Implement streaming
                supports_thinking: false,  // TODO: Add thinking mode
                supports_vision: false,    // TODO: Add vision support
                context_window: Some(200_000),
            }
        }
        fn display_name(&self) -> &'static str {
            "Claude 3 Opus (Direct)"
        }
        fn default_temperature(&self) -> f32 {
            0.6
        }
        fn default_max_tokens(&self) -> u32 {
            4_096
        }
    }
}

// Provider modules are available as: use stood::llm::models::Bedrock::Claude35Sonnet;
// or via the top-level re-export: use stood::llm::{Bedrock, LMStudio, Anthropic};
