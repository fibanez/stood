//! Debug test to run in test environment
//! Run with: cargo test debug_in_test_env -- --nocapture

use std::time::Duration;
use stood::agent::Agent;
use stood::llm::models::LMStudio;

#[tokio::test]
async fn debug_in_test_env() {
    println!("🔍 Testing Agent in Test Environment");
    println!("===================================\n");

    // Configure registry
    stood::llm::registry::ProviderRegistry::configure()
        .await
        .unwrap();
    println!("✅ Provider registry configured");

    // Test with timeout
    let result = tokio::time::timeout(Duration::from_secs(10), async {
        let mut agent = Agent::builder()
            .model(LMStudio::Gemma3_12B)
            .system_prompt("You are a helpful assistant.")
            .temperature(0.0)
            .max_tokens(50)
            .build()
            .await?;

        agent.execute("What is 2+2?").await
    })
    .await;

    match result {
        Ok(Ok(response)) => {
            println!("✅ SUCCESS: {}", response.response);
        }
        Ok(Err(e)) => {
            println!("❌ AGENT ERROR: {}", e);
        }
        Err(_) => {
            println!("⏰ TIMEOUT: Agent execution timed out in test environment");
            println!("   This confirms the issue is specific to test environment");
        }
    }
}
