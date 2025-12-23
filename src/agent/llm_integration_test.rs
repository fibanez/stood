//! Integration tests for Agent with new LLM provider system
//!
//! These tests verify that the Agent can be constructed using the new
//! provider-first architecture while maintaining backward compatibility.

#[cfg(test)]
mod tests {
    use crate::agent::Agent;
    use crate::llm::models::Bedrock;
    use crate::llm::traits::LlmModel;

    #[tokio::test]
    async fn test_agent_builder_new_api() {
        // Test that we can create an agent using the new LLM API
        let model = Bedrock::ClaudeSonnet45;

        // Verify model metadata is correct
        assert_eq!(model.model_id(), "us.anthropic.claude-sonnet-4-5-20250929-v1:0");
        assert_eq!(model.provider(), crate::llm::traits::ProviderType::Bedrock);
        assert_eq!(model.context_window(), 200_000);
        assert_eq!(model.max_output_tokens(), 8_192);

        // Test agent builder accepts the new API
        let _builder = Agent::builder()
            .model(Bedrock::ClaudeSonnet45)
            .temperature(0.7)
            .max_tokens(1000);

        // Builder should work without errors
        assert!(true);
    }

    #[tokio::test]
    async fn test_agent_builder_lm_studio() {
        // Test that we can use LM Studio models
        let _builder = Agent::builder()
            .model(crate::llm::models::LMStudio::Gemma3_12B)
            .temperature(0.5)
            .max_tokens(2000);

        // Builder should work without errors
        assert!(true);
    }

    #[test]
    fn test_model_metadata() {
        // Test model metadata is correct for new Claude 4.5 models
        let sonnet = Bedrock::ClaudeSonnet45;
        let haiku = Bedrock::ClaudeHaiku45;
        let opus = Bedrock::ClaudeOpus45;
        let nova_lite = Bedrock::NovaLite;

        // Verify model IDs are correct
        assert_eq!(sonnet.model_id(), "us.anthropic.claude-sonnet-4-5-20250929-v1:0");
        assert_eq!(haiku.model_id(), "us.anthropic.claude-haiku-4-5-20251001-v1:0");
        assert_eq!(opus.model_id(), "us.anthropic.claude-opus-4-5-20251101-v1:0");
        assert_eq!(nova_lite.model_id(), "us.amazon.nova-lite-v1:0");
    }

    #[test]
    fn test_all_models_available() {
        // Verify all new Claude 4.5 models are defined
        let _bedrock_sonnet = Bedrock::ClaudeSonnet45;
        let _bedrock_haiku = Bedrock::ClaudeHaiku45;
        let _bedrock_opus = Bedrock::ClaudeOpus45;
        let _bedrock_nova_lite = Bedrock::NovaLite;
        let _bedrock_nova_pro = Bedrock::NovaPro;
        let _bedrock_nova_micro = Bedrock::NovaMicro;

        // LM Studio models
        let _lm_studio_gemma = crate::llm::models::LMStudio::Gemma3_12B;
        let _lm_studio_llama = crate::llm::models::LMStudio::Llama3_70B;
        let _lm_studio_mistral = crate::llm::models::LMStudio::Mistral7B;

        // All models should be available
        assert!(true);
    }

    #[test]
    #[allow(deprecated)]
    fn test_legacy_models_still_available() {
        // Verify legacy models still compile (for backward compatibility)
        let _legacy_sonnet = Bedrock::Claude35Sonnet;
        let _legacy_haiku = Bedrock::Claude35Haiku;
        let _legacy_haiku3 = Bedrock::ClaudeHaiku3;
        let _legacy_opus3 = Bedrock::ClaudeOpus3;

        // Legacy models should still be usable but will emit deprecation warnings
        assert!(true);
    }
}
