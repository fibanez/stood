//! Smart Telemetry Configuration Test
//!
//! This example demonstrates the CloudWatch Gen AI Observability telemetry system.
//!
//! Run with:
//! ```bash
//! # Test telemetry configuration
//! cargo run --example 023_smart_telemetry_test
//! ```

use stood::telemetry::{AwsCredentialSource, LogLevel, TelemetryConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize basic logging
    tracing_subscriber::fmt::init();

    println!("CloudWatch Telemetry Configuration Test");
    println!("==========================================");

    // Test 1: Disabled config
    println!("\nTest 1: Disabled telemetry");
    let disabled = TelemetryConfig::disabled();
    println!("   Enabled: {}", disabled.is_enabled());
    println!("   Service Name: {:?}", disabled.service_name());

    // Test 2: CloudWatch with environment credentials
    println!("\nTest 2: CloudWatch with environment credentials");
    let cloudwatch_env = TelemetryConfig::cloudwatch("us-east-1");
    println!("   Enabled: {}", cloudwatch_env.is_enabled());
    println!("   Service Name: {:?}", cloudwatch_env.service_name());
    println!("   Log Level: {:?}", cloudwatch_env.log_level());

    // Test 3: CloudWatch with explicit credentials
    println!("\nTest 3: CloudWatch with explicit credentials");
    let cloudwatch_explicit = TelemetryConfig::CloudWatch {
        region: "us-west-2".to_string(),
        credentials: AwsCredentialSource::Profile("my-profile".to_string()),
        service_name: "explicit-service".to_string(),
        service_version: "1.0.0".to_string(),
        agent_id: Some("test-agent".to_string()),
        log_level: LogLevel::DEBUG,
        content_capture: true,
    };
    println!("   Enabled: {}", cloudwatch_explicit.is_enabled());
    println!("   Service Name: {:?}", cloudwatch_explicit.service_name());

    // Test 4: Log level modification
    println!("\nTest 4: Log level modification");
    let mut config = TelemetryConfig::cloudwatch("eu-west-1");
    println!("   Initial log level: {:?}", config.log_level());
    config.set_log_level(LogLevel::TRACE);
    println!("   Updated log level: {:?}", config.log_level());

    // Test 5: Builder pattern
    println!("\nTest 5: Builder pattern configuration");
    let builder_config = TelemetryConfig::cloudwatch("ap-northeast-1")
        .with_service_name("my-agent")
        .with_log_level(LogLevel::DEBUG)
        .with_content_capture(false);
    println!("   Service Name: {:?}", builder_config.service_name());
    println!("   Log Level: {:?}", builder_config.log_level());

    println!("\nCloudWatch telemetry configuration test completed!");
    println!("\nKey Features:");
    println!("   - CloudWatch Gen AI Observability support");
    println!("   - AWS SigV4 authentication for X-Ray OTLP endpoint");
    println!("   - Multiple credential sources (Environment, Profile, IAM Role, Explicit)");
    println!("   - Configurable log levels and content capture");

    Ok(())
}
