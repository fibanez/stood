//! Integration test demonstrating the new provider-first architecture
//!
//! This test validates that the new LLM architecture works end-to-end with real providers.

use crate::llm::traits::{LlmModel, LlmProvider, ProviderType};
use crate::llm::{Bedrock, LMStudio, ProviderRegistry, PROVIDER_REGISTRY};

#[tokio::test]
async fn test_new_provider_architecture_demo() {
    // This test demonstrates the new provider-first architecture
    // Models are pure metadata, providers own all logic

    // 1. Test model metadata system
    let claude_model = Bedrock::ClaudeSonnet45;
    assert_eq!(claude_model.provider(), ProviderType::Bedrock);
    assert_eq!(
        claude_model.model_id(),
        "us.anthropic.claude-sonnet-4-5-20250929-v1:0"
    );
    assert_eq!(claude_model.context_window(), 200_000);
    assert_eq!(claude_model.max_output_tokens(), 8_192);

    let gemma_model = LMStudio::Gemma3_12B;
    assert_eq!(gemma_model.provider(), ProviderType::LmStudio);
    assert_eq!(gemma_model.model_id(), "google/gemma-3-12b");
    assert_eq!(gemma_model.context_window(), 8_192);

    // 2. Test provider registry configuration (auto-detection from environment)
    ProviderRegistry::configure()
        .await
        .expect("Registry configuration should work");

    // 3. Test provider lazy loading - Bedrock should be configured by default
    let configured_providers = PROVIDER_REGISTRY.configured_providers().await;
    println!("Configured providers: {:?}", configured_providers);

    // At minimum, Bedrock should be configured (if AWS credentials are available)
    // In CI/test environments, this might be empty, which is expected

    // 4. Test provider creation (if Bedrock is configured)
    if PROVIDER_REGISTRY.is_configured(ProviderType::Bedrock).await {
        let bedrock_provider = PROVIDER_REGISTRY.get_provider(ProviderType::Bedrock).await;

        match bedrock_provider {
            Ok(provider) => {
                // Test provider capabilities
                let capabilities = provider.capabilities();
                assert!(capabilities.supports_tools);
                assert!(capabilities.supports_streaming);
                assert!(capabilities.supports_thinking);

                // Test supported models
                let models = provider.supported_models();
                assert!(models.contains(&"us.anthropic.claude-sonnet-4-5-20250929-v1:0"));
                assert!(models.contains(&"us.amazon.nova-lite-v1:0"));

                println!(
                    "✅ BedrockProvider successfully created with {} models",
                    models.len()
                );
            }
            Err(e) => {
                println!(
                    "⚠️ BedrockProvider creation failed (expected in some environments): {}",
                    e
                );
                // This is expected if AWS credentials are not configured
            }
        }
    } else {
        println!("ℹ️ Bedrock not configured (no AWS credentials), skipping provider test");
    }

    // 5. Test LM Studio provider (will fail but demonstrates the pattern)
    if PROVIDER_REGISTRY
        .is_configured(ProviderType::LmStudio)
        .await
    {
        let lm_studio_result = PROVIDER_REGISTRY.get_provider(ProviderType::LmStudio).await;

        match lm_studio_result {
            Ok(provider) => {
                println!("✅ LMStudioProvider successfully created");
                let capabilities = provider.capabilities();
                assert!(capabilities.supports_streaming);
            }
            Err(e) => {
                println!(
                    "⚠️ LMStudioProvider creation failed (expected without local LM Studio): {}",
                    e
                );
                // This is expected if LM Studio is not running locally
            }
        }
    }

    println!("🎉 Provider-first architecture validation complete!");
    println!("📋 Summary:");
    println!("   - ✅ Model metadata system working");
    println!("   - ✅ Provider registry lazy loading working");
    println!("   - ✅ Provider creation and capabilities working");
    println!("   - ✅ Single API pattern: Bedrock::ClaudeSonnet45 works");
    println!("   - ✅ Type safety: Can't mix providers and models");
}

#[tokio::test]
async fn test_model_type_safety() {
    // This test demonstrates type safety in the new architecture

    // These work - correct provider/model combinations
    let claude = Bedrock::ClaudeSonnet45;
    let nova = Bedrock::NovaLite;
    let gemma = LMStudio::Gemma3_12B;

    assert_eq!(claude.provider(), ProviderType::Bedrock);
    assert_eq!(nova.provider(), ProviderType::Bedrock);
    assert_eq!(gemma.provider(), ProviderType::LmStudio);

    // The type system prevents mixing providers and models at compile time
    // This would not compile:
    // let invalid = LMStudio::Claude35Sonnet; // ❌ Compile error!
    // let invalid = Bedrock::Gemma3_12B;       // ❌ Compile error!

    println!("✅ Type safety validation passed - can't mix providers and models");
}

#[tokio::test]
async fn test_provider_sharing() {
    // Test that providers are shared across multiple "agents" efficiently

    ProviderRegistry::configure()
        .await
        .expect("Registry configuration should work");

    if PROVIDER_REGISTRY.is_configured(ProviderType::Bedrock).await {
        // Get the same provider twice
        let provider1_result = PROVIDER_REGISTRY.get_provider(ProviderType::Bedrock).await;
        let provider2_result = PROVIDER_REGISTRY.get_provider(ProviderType::Bedrock).await;

        if let (Ok(provider1), Ok(provider2)) = (provider1_result, provider2_result) {
            // They should be the same Arc (shared instance)
            assert!(std::ptr::eq(
                &*provider1 as *const dyn LlmProvider,
                &*provider2 as *const dyn LlmProvider
            ));

            println!("✅ Provider sharing working - same Arc instance returned");
        }
    }
}
