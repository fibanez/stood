//! Smart Telemetry Auto-Detection Test
//!
//! This example demonstrates the new adaptive telemetry system that:
//! 1. Auto-detects available OTLP endpoints
//! 2. Falls back gracefully: OTLP -> Console -> Disabled
//! 3. Works without explicit configuration
//!
//! Run with:
//! ```bash
//! # With telemetry stack running (should auto-detect port 4320)
//! cargo run --example smart_telemetry_test
//!
//! # Without stack (should fall back to console in debug, disabled in release)
//! cargo run --example smart_telemetry_test
//!
//! # Telemetry is now always enabled - no feature flags needed!
//! ```

use tracing::info;
use stood::telemetry::TelemetryConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize basic logging to see what's happening
    tracing_subscriber::fmt::init();

    println!("🧪 Smart Telemetry Auto-Detection Test");
    println!("======================================");

    // Test 1: Auto-detection from environment
    println!("\n📋 Test 1: Auto-detection from environment");
    let config = TelemetryConfig::from_env();
    
    println!("   Enabled: {}", config.enabled);
    println!("   OTLP Endpoint: {:?}", config.otlp_endpoint);
    println!("   Console Export: {}", config.console_export);
    println!("   Service Name: {}", config.service_name);

    // Test 2: Manual endpoint detection
    println!("\n📋 Test 2: Manual endpoint detection test");
    let detected_endpoint = test_endpoint_detection().await;
    println!("   Detected endpoint: {:?}", detected_endpoint);

    // Test 3: Validation with smart fallbacks
    println!("\n📋 Test 3: Configuration validation");
    match config.validate() {
        Ok(_) => println!("   ✅ Configuration is valid"),
        Err(e) => println!("   ❌ Configuration error: {}", e),
    }

    // Test 4: Telemetry initialization (always enabled)
    println!("\n📋 Test 4: Telemetry initialization");
    {
        println!("   ✅ Telemetry is always enabled");
        
        // Try to initialize actual telemetry
        use stood::telemetry::StoodTracer;
        
        match StoodTracer::init(config.clone()) {
            Ok(Some(tracer)) => {
                println!("   ✅ Telemetry initialized successfully");
                tracer.shutdown();
            }
            Ok(None) => {
                println!("   ⚠️  Telemetry disabled (no available endpoints)");
            }
            Err(e) => {
                println!("   ❌ Telemetry initialization failed: {}", e);
            }
        }
    }

    // Test 5: Agent initialization with smart telemetry
    println!("\n📋 Test 5: Agent with smart telemetry");
    test_agent_telemetry().await?;

    println!("\n🎯 Smart telemetry test completed!");
    println!("\n💡 Key Features Demonstrated:");
    println!("   • Auto-detection of OTLP endpoints on common ports");
    println!("   • Graceful degradation to console export in development");
    println!("   • Zero-configuration operation");
    println!("   • Intelligent validation with fallbacks");
    
    Ok(())
}

/// Test the endpoint detection logic
async fn test_endpoint_detection() -> Option<String> {
    let common_endpoints = [
        "http://localhost:4318", // OTLP HTTP
        "http://localhost:4320", // Common alternative 
        "http://otel-collector:4318", // Docker compose
        "http://127.0.0.1:4318",
    ];

    for endpoint in &common_endpoints {
        if check_endpoint_availability(endpoint) {
            info!("🎯 Detected available endpoint: {}", endpoint);
            return Some(endpoint.to_string());
        } else {
            info!("❌ Endpoint not available: {}", endpoint);
        }
    }

    None
}

/// Quick TCP connection test to see if a port is open
fn check_endpoint_availability(endpoint: &str) -> bool {
    use std::net::{SocketAddr, TcpStream};
    use std::time::Duration;
    
    if let Ok(url) = url::Url::parse(endpoint) {
        if let Some(host) = url.host_str() {
            let port = url.port().unwrap_or(4318);
            
            if let Ok(addr) = format!("{}:{}", host, port).parse::<SocketAddr>() {
                return TcpStream::connect_timeout(&addr, Duration::from_millis(100)).is_ok();
            }
        }
    }
    false
}

/// Test Agent initialization with smart telemetry
async fn test_agent_telemetry() -> Result<(), Box<dyn std::error::Error>> {
    use stood::agent::Agent;
    
    // Test with automatic telemetry detection
    info!("Initializing agent with smart telemetry detection...");
    
    let mut agent = Agent::builder()
        .system_prompt("You are a test assistant")
        .build().await?;
    
    println!("   ✅ Agent initialized successfully");
    
    // Try a simple execution
    let response = agent.execute("Say hello").await?;
    println!("   🤖 Agent response: {}", response.response);
    
    // Check if we have execution details
    println!("   📊 Execution completed with {} cycles", response.execution.cycles);
    if let Some(tokens) = &response.execution.tokens {
        println!("   📊 Total tokens used: {}", tokens.input_tokens + tokens.output_tokens);
    } else {
        println!("   📊 No token usage data available");
    }
    
    Ok(())
}