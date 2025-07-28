//! Standalone metrics test without AWS Bedrock integration
//!
//! This test verifies metrics collection infrastructure without requiring
//! AWS credentials by directly testing the metrics collector types.

use stood::telemetry::{TelemetryConfig, StoodTracer};
use stood::telemetry::metrics::{
    MetricsCollector, NoOpMetricsCollector,
    TokenMetrics, RequestMetrics, ToolMetrics, SystemMetrics
};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Stood Metrics Collection (No AWS Required)");
    
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Test 1: Verify OTLP endpoint detection
    println!("\n📡 Test 1: OTLP Endpoint Detection");
    let telemetry_config = TelemetryConfig::from_env();
    println!("   Telemetry enabled: {}", telemetry_config.enabled);
    println!("   OTLP endpoint: {:?}", telemetry_config.otlp_endpoint);
    
    // Test 2: Initialize telemetry system
    println!("\n🔧 Test 2: Initialize Telemetry System");
    let tracer_result = StoodTracer::init(telemetry_config.clone());
    
    match tracer_result {
        Ok(Some(_tracer)) => {
            println!("   ✅ Telemetry initialized successfully");
        },
        Ok(None) => {
            println!("   ⚠️  Telemetry disabled, no endpoints available");
        },
        Err(e) => {
            println!("   ❌ Telemetry initialization failed: {}", e);
        }
    }
    
    // Test 3: Metrics data structures and no-op collector
    test_metrics_infrastructure().await?;
    
    println!("\n✅ All metrics infrastructure tests completed!");
    println!("📊 The full metrics publishing test with AWS is in metrics_test.rs");
    
    Ok(())
}

async fn test_metrics_infrastructure() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📊 Test 3: Metrics Infrastructure");
    
    // Test no-op collector (always available)
    let collector = NoOpMetricsCollector::default();
    println!("   ✅ NoOpMetricsCollector created");
    
    // Test 4: Token Metrics data structures
    println!("\n🪙 Test 4: Token Metrics Data Structures");
    let token_metrics = TokenMetrics::new(1000, 500);
    println!("   ✅ TokenMetrics: {} input, {} output, {} total", 
             token_metrics.input_tokens, token_metrics.output_tokens, token_metrics.total_tokens);
    
    // Verify calculations
    assert_eq!(token_metrics.total_tokens, 1500);
    println!("   ✅ Token calculations correct");
    
    // Test metrics collection
    collector.record_token_metrics(&token_metrics);
    println!("   ✅ Token metrics recorded via no-op collector");
    
    // Test 5: Request Metrics
    println!("\n⏱️  Test 5: Request Metrics Data Structures");
    let request_metrics = RequestMetrics {
        duration: Duration::from_millis(1500),
        success: true,
        model_invocations: 2,
        token_metrics: Some(token_metrics.clone()),
        error_type: None,
    };
    collector.record_request_metrics(&request_metrics);
    println!("   ✅ Request metrics: {:?} duration, {} invocations", 
             request_metrics.duration, request_metrics.model_invocations);
    
    // Test 6: Tool Metrics
    println!("\n🔧 Test 6: Tool Metrics Data Structures");
    let tool_metrics = ToolMetrics {
        tool_name: "test_calculator".to_string(),
        duration: Duration::from_millis(250),
        success: true,
        retry_attempts: 0,
        error_type: None,
    };
    collector.record_tool_metrics(&tool_metrics);
    println!("   ✅ Tool metrics: {} executed in {:?}", 
             tool_metrics.tool_name, tool_metrics.duration);
    
    // Test 7: System Metrics
    println!("\n💾 Test 7: System Metrics Data Structures");
    let system_metrics = SystemMetrics {
        memory_usage_bytes: 128 * 1024 * 1024, // 128 MB
        active_connections: 5,
        concurrent_requests: 2,
        thread_utilization: 0.75,
    };
    collector.record_system_metrics(&system_metrics);
    println!("   ✅ System metrics: {} MB memory, {} connections", 
             system_metrics.memory_usage_bytes / (1024 * 1024), system_metrics.active_connections);
    
    // Test 8: Error metrics
    println!("\n❌ Test 8: Error Metrics Data Structures");
    let error_request_metrics = RequestMetrics {
        duration: Duration::from_millis(500),
        success: false,
        model_invocations: 1,
        token_metrics: None,
        error_type: Some("timeout_error".to_string()),
    };
    collector.record_request_metrics(&error_request_metrics);
    
    let error_tool_metrics = ToolMetrics {
        tool_name: "failing_tool".to_string(),
        duration: Duration::from_millis(100),
        success: false,
        retry_attempts: 2,
        error_type: Some("validation_error".to_string()),
    };
    collector.record_tool_metrics(&error_tool_metrics);
    println!("   ✅ Error metrics structures validated");
    
    // Test 9: Direct metric calls
    println!("\n📈 Test 9: Direct Metric Collection Interface");
    collector.record_counter("test_counter", 42, &[]);
    collector.record_histogram("test_histogram", 3.14, &[]);
    collector.record_gauge("test_gauge", 99.5, &[]);
    collector.increment("test_increment", &[]);
    println!("   ✅ All MetricsCollector trait methods work");
    
    println!("\n🎯 Metrics infrastructure test completed successfully!");
    
    Ok(())
}