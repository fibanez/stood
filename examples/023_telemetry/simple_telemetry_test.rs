//! Simple Telemetry Test
//! 
//! This is a minimal test to verify telemetry is working end-to-end.

use stood::telemetry::TelemetryConfig;

#[tokio::main] 
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    println!("🧪 Simple Telemetry Test");
    println!("========================");
    
    // Test smart auto-detection
    println!("\n📋 Testing smart telemetry auto-detection...");
    let config = TelemetryConfig::from_env();
    
    println!("Telemetry Config:");
    println!("  Enabled: {}", config.enabled);
    println!("  OTLP Endpoint: {:?}", config.otlp_endpoint);
    println!("  Console Export: {}", config.console_export);
    println!("  Service Name: {}", config.service_name);
    
    // Try to initialize telemetry (always enabled)
    {
        use stood::telemetry::StoodTracer;
        
        println!("\n📊 Initializing telemetry...");
        match StoodTracer::init(config) {
            Ok(Some(tracer)) => {
                println!("✅ Telemetry initialized successfully!");
                
                // Create a test span
                let mut span = tracer.start_agent_span("test_operation");
                span.set_attribute("test.type", "simple_test");
                span.set_attribute("test.value", 42);
                
                // Simulate some work
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                
                span.set_success();
                span.finish();
                
                println!("✅ Test span created and finished");
                
                // Wait a bit for telemetry to be exported
                println!("⏳ Waiting for telemetry export...");
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                
                // Shutdown gracefully
                tracer.shutdown();
                println!("✅ Telemetry shutdown complete");
            }
            Ok(None) => {
                println!("⚠️ Telemetry disabled (no endpoints available)");
            }
            Err(e) => {
                println!("❌ Telemetry initialization failed: {}", e);
            }
        }
    }
    
    println!("\n🎯 Test complete!");
    Ok(())
}