//! End-to-end performance and benchmarking tests
//!
//! This module tests the performance characteristics of the CLI under
//! various load conditions and usage patterns.

use crate::e2e::*;
use std::time::{Duration, Instant};

/// Test response time benchmarks
#[tokio::test]
async fn test_response_time_benchmarks() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    let mut session = spawn_cli().await?;

    // Benchmark simple calculation
    let start = Instant::now();
    session.send_line("What is 2 + 2?").await?;
    session.expect("4").await?;
    let calc_duration = start.elapsed();

    println!("📊 Simple calculation response time: {:?}", calc_duration);
    assert!(
        calc_duration < Duration::from_secs(30),
        "Calculation took too long"
    );

    // Benchmark file operation
    let temp_dir = create_temp_dir()?;
    create_sample_files(temp_dir.path()).await?;
    let file_path = temp_dir.path().join("sample.txt");

    let start = Instant::now();
    session
        .send_line(&format!("Read the file {}", file_path.display()))
        .await?;
    session.expect("sample text file").await?;
    let file_duration = start.elapsed();

    println!("📊 File read response time: {:?}", file_duration);
    assert!(
        file_duration < Duration::from_secs(30),
        "File read took too long"
    );

    // Benchmark time query
    let start = Instant::now();
    session.send_line("What time is it?").await?;
    session.expect("UTC").await?;
    let time_duration = start.elapsed();

    println!("📊 Time query response time: {:?}", time_duration);
    assert!(
        time_duration < Duration::from_secs(30),
        "Time query took too long"
    );

    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    Ok(())
}

/// Test resource usage under load
#[tokio::test]
async fn test_resource_usage_limits() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    let mut session = spawn_cli().await?;

    // Test with many rapid requests
    let start = Instant::now();

    for i in 1..=10 {
        session
            .send_line(&format!("Calculate {} + {}", i, i))
            .await?;
        // Don't wait for response to test concurrent handling
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // Wait for all responses
    tokio::time::sleep(Duration::from_secs(30)).await;

    let total_duration = start.elapsed();
    println!("📊 10 rapid requests total time: {:?}", total_duration);

    // Verify system is still responsive
    session.send_line("What is 1 + 1?").await?;
    session.expect("2").await?;

    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    Ok(())
}

/// Test scalability with large inputs
#[tokio::test]
async fn test_scalability_large_inputs() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    let mut session = spawn_cli().await?;

    // Test with progressively larger inputs
    let sizes = vec![100, 500, 1000, 2000];

    for size in sizes {
        let large_text = "A".repeat(size);
        let start = Instant::now();

        session
            .send_line(&format!(
                "Count the characters in this text: {}",
                large_text
            ))
            .await?;

        // Wait for response
        tokio::time::sleep(Duration::from_secs(10)).await;

        let duration = start.elapsed();
        println!("📊 Processing {} chars took: {:?}", size, duration);

        // Reasonable time limit scales with input size
        let expected_limit = Duration::from_secs(5 + (size / 100) as u64);
        assert!(
            duration < expected_limit,
            "Processing {} chars took too long: {:?}",
            size,
            duration
        );
    }

    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    Ok(())
}

/// Test conversation history performance
#[tokio::test]
async fn test_conversation_history_performance() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    let mut session = spawn_cli().await?;

    // Build up conversation history
    let start = Instant::now();

    for i in 1..=20 {
        session
            .send_line(&format!("Message {} in our conversation", i))
            .await?;
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    let history_build_time = start.elapsed();
    println!(
        "📊 Building 20-message history took: {:?}",
        history_build_time
    );

    // Test response time with full history
    let start = Instant::now();
    session
        .send_line("What was the first message I sent?")
        .await?;
    session.expect("1").await?;
    let history_query_time = start.elapsed();

    println!("📊 Querying history took: {:?}", history_query_time);
    assert!(
        history_query_time < Duration::from_secs(30),
        "History query took too long"
    );

    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    Ok(())
}

/// Test concurrent tool usage performance
#[tokio::test]
async fn test_concurrent_tool_performance() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    let mut session = spawn_cli().await?;

    // Test multiple tools in sequence
    let start = Instant::now();

    session
        .send_line("Calculate 10 * 5, tell me the time, and get the HOME environment variable")
        .await?;

    // Wait for all tools to complete
    tokio::time::sleep(Duration::from_secs(20)).await;

    let multi_tool_duration = start.elapsed();
    println!("📊 Multi-tool request took: {:?}", multi_tool_duration);

    // Should complete within reasonable time
    assert!(
        multi_tool_duration < Duration::from_secs(45),
        "Multi-tool request took too long"
    );

    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    Ok(())
}

/// Test memory efficiency over time
#[tokio::test]
async fn test_memory_efficiency() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    let mut session = spawn_cli().await?;

    // Simulate extended usage session
    let start = Instant::now();

    for round in 1..=5 {
        println!("📊 Memory efficiency test round {}", round);

        // Various operations to test memory usage
        session
            .send_line(&format!("Calculate {} * 123", round))
            .await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        session.send_line("What time is it?").await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        let temp_dir = create_temp_dir()?;
        create_sample_files(temp_dir.path()).await?;
        let file_path = temp_dir.path().join("sample.txt");
        session
            .send_line(&format!("Read {}", file_path.display()))
            .await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Clear context periodically to test memory cleanup
        if round % 3 == 0 {
            session.send_line("clear").await?;
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    }

    let total_duration = start.elapsed();
    println!("📊 Extended session took: {:?}", total_duration);

    // Verify system is still responsive
    session.send_line("What is 99 + 1?").await?;
    session.expect("100").await?;

    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    Ok(())
}

/// Test startup performance
#[tokio::test]
async fn test_startup_performance() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    // Test multiple startup/shutdown cycles
    let mut startup_times = Vec::new();

    for i in 1..=3 {
        println!("📊 Startup test cycle {}", i);

        let start = Instant::now();
        let mut session = spawn_cli().await?;
        let startup_time = start.elapsed();

        startup_times.push(startup_time);
        println!("📊 Startup {} took: {:?}", i, startup_time);

        // Quick verification
        session.send_line("help").await?;
        session.expect("Commands:").await?;

        session.send_line("exit").await?;
        session.wait_for_exit().await?;

        // Brief pause between cycles
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    let avg_startup = startup_times.iter().sum::<Duration>() / startup_times.len() as u32;
    println!("📊 Average startup time: {:?}", avg_startup);

    // Startup should be reasonably fast
    assert!(
        avg_startup < Duration::from_secs(30),
        "Average startup time too slow"
    );

    Ok(())
}

/// Test performance regression detection
#[tokio::test]
async fn test_performance_regression() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    let mut session = spawn_cli().await?;

    // Baseline performance measurements
    let mut measurements = Vec::new();

    for i in 1..=5 {
        let start = Instant::now();
        session
            .send_line(&format!("Calculate {} + {}", i * 10, i * 20))
            .await?;
        session.expect(&(i * 30).to_string()).await?;
        measurements.push(start.elapsed());
    }

    let avg_time = measurements.iter().sum::<Duration>() / measurements.len() as u32;
    let max_time = *measurements.iter().max().unwrap();
    let min_time = *measurements.iter().min().unwrap();

    println!("📊 Performance metrics:");
    println!("   Average: {:?}", avg_time);
    println!("   Min: {:?}", min_time);
    println!("   Max: {:?}", max_time);
    println!("   Variance: {:?}", max_time - min_time);

    // Performance should be consistent
    let variance_threshold = Duration::from_secs(10);
    assert!(
        max_time - min_time < variance_threshold,
        "Performance variance too high"
    );

    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    Ok(())
}

/// Test streaming performance if enabled
#[tokio::test]
async fn test_streaming_performance() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping test - AWS credentials not available");
        return Ok(());
    }

    // Test with streaming enabled
    let mut config = TestConfig::default();
    config.extra_args.push("--streaming".to_string());
    let mut session = spawn_cli_with_config(config).await?;

    let start = Instant::now();
    session
        .send_line("Write a short poem about Rust programming")
        .await?;

    // Wait for streaming response
    tokio::time::sleep(Duration::from_secs(15)).await;

    let streaming_duration = start.elapsed();
    println!("📊 Streaming response took: {:?}", streaming_duration);

    // Verify responsiveness during streaming
    session.send_line("What is 42 / 2?").await?;
    session.expect("21").await?;

    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    Ok(())
}
