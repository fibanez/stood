//! Simple Telemetry Test
//!
//! This is a minimal test to verify telemetry configuration works.
//!
//! Run with: cargo run --example 023_simple_telemetry_test

use stood::telemetry::TelemetryConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("Simple Telemetry Test");
    println!("========================");

    // Test CloudWatch telemetry configuration
    println!("\nTesting CloudWatch telemetry configuration...");

    // Create a CloudWatch config (disabled by default until AWS is configured)
    let config = TelemetryConfig::disabled();

    println!("Telemetry Config:");
    println!("  Enabled: {}", config.is_enabled());
    println!("  Service Name: {:?}", config.service_name());
    println!("  Log Level: {:?}", config.log_level());

    // Test CloudWatch variant (region only)
    println!("\nTesting CloudWatch variant (region only)...");
    let cloudwatch_config = TelemetryConfig::cloudwatch("us-east-1");
    println!("  Enabled: {}", cloudwatch_config.is_enabled());
    println!("  Service Name: {:?}", cloudwatch_config.service_name());

    // Test CloudWatch variant with service name
    println!("\nTesting CloudWatch variant (with service name)...");
    let cloudwatch_config = TelemetryConfig::cloudwatch_with_service("us-east-1", "test-service");
    println!("  Enabled: {}", cloudwatch_config.is_enabled());
    println!("  Service Name: {:?}", cloudwatch_config.service_name());

    println!("\nTest complete!");
    println!("To use CloudWatch telemetry:");
    println!("   1. Configure AWS credentials (environment, profile, or IAM role)");
    println!("   2. Enable Transaction Search in CloudWatch Console");
    println!("   3. Set trace destination to CloudWatch Logs");
    println!(
        "   4. Use TelemetryConfig::cloudwatch(region) or cloudwatch_with_service(region, name)"
    );

    Ok(())
}
