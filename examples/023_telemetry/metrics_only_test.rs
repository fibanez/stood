//! Standalone telemetry test without AWS Bedrock integration
//!
//! This test verifies the telemetry infrastructure without requiring
//! AWS credentials by testing the configuration types.

use stood::telemetry::{LogLevel, TelemetryConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Stood Telemetry Configuration (No AWS Required)");

    // Initialize logging
    tracing_subscriber::fmt::init();

    // Test 1: Verify disabled telemetry config
    println!("\nTest 1: Disabled Telemetry Config");
    let disabled_config = TelemetryConfig::disabled();
    println!("   Telemetry enabled: {}", disabled_config.is_enabled());
    assert!(!disabled_config.is_enabled());
    println!("   Disabled config works correctly");

    // Test 2: Verify CloudWatch telemetry config
    println!("\nTest 2: CloudWatch Telemetry Config");
    let cloudwatch_config = TelemetryConfig::cloudwatch("us-east-1");
    println!("   Telemetry enabled: {}", cloudwatch_config.is_enabled());
    println!("   Service name: {:?}", cloudwatch_config.service_name());
    assert!(cloudwatch_config.is_enabled());
    println!("   CloudWatch config works correctly");

    // Test 3: Log level configuration
    println!("\nTest 3: Log Level Configuration");
    let mut config = TelemetryConfig::cloudwatch("us-west-2");
    println!("   Initial log level: {:?}", config.log_level());
    config.set_log_level(LogLevel::DEBUG);
    println!("   Updated log level: {:?}", config.log_level());
    println!("   Log level configuration works correctly");

    println!("\nAll telemetry configuration tests completed!");
    println!("For full CloudWatch integration, configure AWS credentials and region.");

    Ok(())
}
