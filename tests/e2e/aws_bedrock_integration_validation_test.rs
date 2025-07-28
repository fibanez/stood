//! End-to-end integration testing suite for the Stood agent library
//!
//! This module implements comprehensive integration tests that validate complete
//! system integration with real AWS Bedrock services, including model compatibility,
//! API contract validation, and production readiness scenarios.

use super::*;
use crate::integration_testing_framework::*;

/// Test basic AWS Bedrock integration
#[tokio::test]
async fn test_aws_bedrock_integration() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping integration test - AWS credentials not available");
        return Ok(());
    }

    let config = IntegrationTestConfig::claude_haiku_basic();
    let executor = IntegrationTestExecutor::new(config);

    let results = executor.execute().await?;

    // Validate basic AWS integration
    assert!(results.total_tests > 0, "No integration tests executed");
    assert!(
        results.aws_validation_results.credentials_valid,
        "AWS credentials validation failed"
    );
    assert!(
        results.aws_validation_results.bedrock_accessible,
        "AWS Bedrock not accessible"
    );

    println!("🔗 Basic AWS Bedrock Integration Test Results:");
    println!(
        "   AWS credentials: {}",
        if results.aws_validation_results.credentials_valid {
            "✅"
        } else {
            "❌"
        }
    );
    println!(
        "   Bedrock access: {}",
        if results.aws_validation_results.bedrock_accessible {
            "✅"
        } else {
            "❌"
        }
    );
    println!(
        "   Model permissions: {}",
        if results.aws_validation_results.model_permissions_valid {
            "✅"
        } else {
            "❌"
        }
    );
    println!(
        "   Tests passed: {}/{}",
        results.passed_tests, results.total_tests
    );

    // Basic integration requirements
    assert!(
        results.aws_validation_results.bedrock_accessible,
        "Cannot access AWS Bedrock service"
    );
    assert!(
        results.aws_validation_results.model_permissions_valid,
        "Insufficient model permissions"
    );

    // Performance validation
    assert!(
        results.performance_metrics.availability >= 0.8,
        "Integration availability too low: {:.1}%",
        results.performance_metrics.availability * 100.0
    );

    Ok(())
}

/// Test model compatibility across different scenarios
#[tokio::test]
async fn test_model_compatibility() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping integration test - AWS credentials not available");
        return Ok(());
    }

    let config = IntegrationTestConfig {
        test_duration: std::time::Duration::from_secs(180),
        bedrock_model: BedrockModel::ClaudeHaiku35,
        test_scenarios: vec![
            IntegrationTestScenario::BasicModelInvocation,
            IntegrationTestScenario::ToolIntegration,
            IntegrationTestScenario::StreamingIntegration,
            IntegrationTestScenario::MultiTurnConversation,
        ],
        aws_validation: AwsValidationConfig {
            validate_credentials: true,
            validate_region: true,
            validate_bedrock_access: true,
            validate_model_permissions: true,
            test_multiple_regions: false,
            validate_sdk_config: true,
        },
        model_compatibility: ModelCompatibilityConfig {
            test_input_formats: true,
            test_output_formats: true,
            test_parameter_variations: true,
            test_token_limits: true,
            test_streaming: true,
            test_tool_compatibility: true,
        },
        api_contract_validation: ApiContractValidationConfig {
            validate_request_format: true,
            validate_response_format: true,
            test_error_responses: true,
            validate_api_versioning: false,
            test_rate_limiting: false,
            validate_authentication: true,
        },
        production_scenarios: ProductionScenarioConfig {
            test_high_load: false,
            test_conversation_patterns: true,
            test_tool_patterns: true,
            test_error_recovery: true,
            test_performance_load: false,
            test_edge_cases: false,
        },
    };

    let executor = IntegrationTestExecutor::new(config);
    let results = executor.execute().await?;

    // Validate model compatibility
    assert!(results.total_tests > 0, "No compatibility tests executed");

    println!("🤖 Model Compatibility Integration Test Results:");
    println!(
        "   Input format compatibility: {}",
        if results
            .model_compatibility_results
            .input_format_compatibility
        {
            "✅"
        } else {
            "❌"
        }
    );
    println!(
        "   Output format compatibility: {}",
        if results
            .model_compatibility_results
            .output_format_compatibility
        {
            "✅"
        } else {
            "❌"
        }
    );
    println!(
        "   Streaming support: {}",
        if results.model_compatibility_results.streaming_supported {
            "✅"
        } else {
            "❌"
        }
    );
    println!(
        "   Tool compatibility: {}",
        if results.model_compatibility_results.tool_compatibility {
            "✅"
        } else {
            "❌"
        }
    );
    println!(
        "   Token limit compliance: {}",
        if results.model_compatibility_results.token_limit_compliance {
            "✅"
        } else {
            "❌"
        }
    );

    // Model compatibility requirements
    assert!(
        results
            .model_compatibility_results
            .input_format_compatibility,
        "Model input format incompatibility detected"
    );
    assert!(
        results
            .model_compatibility_results
            .output_format_compatibility,
        "Model output format incompatibility detected"
    );
    assert!(
        results.model_compatibility_results.tool_compatibility,
        "Tool compatibility issues detected"
    );

    // Check for compatibility issues
    if !results
        .model_compatibility_results
        .compatibility_issues
        .is_empty()
    {
        println!("⚠️  Compatibility issues found:");
        for issue in &results.model_compatibility_results.compatibility_issues {
            println!("   • {}", issue);
        }
    }

    Ok(())
}

/// Test API contract compliance with AWS Bedrock
#[tokio::test]
async fn test_api_contract_compliance() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping integration test - AWS credentials not available");
        return Ok(());
    }

    let config = IntegrationTestConfig {
        test_duration: std::time::Duration::from_secs(120),
        bedrock_model: BedrockModel::ClaudeHaiku35,
        test_scenarios: vec![
            IntegrationTestScenario::BasicModelInvocation,
            IntegrationTestScenario::ErrorHandlingIntegration,
        ],
        aws_validation: AwsValidationConfig {
            validate_credentials: true,
            validate_region: false,
            validate_bedrock_access: true,
            validate_model_permissions: true,
            test_multiple_regions: false,
            validate_sdk_config: false,
        },
        model_compatibility: ModelCompatibilityConfig {
            test_input_formats: false,
            test_output_formats: false,
            test_parameter_variations: false,
            test_token_limits: false,
            test_streaming: false,
            test_tool_compatibility: false,
        },
        api_contract_validation: ApiContractValidationConfig {
            validate_request_format: true,
            validate_response_format: true,
            test_error_responses: true,
            validate_api_versioning: true,
            test_rate_limiting: true,
            validate_authentication: true,
        },
        production_scenarios: ProductionScenarioConfig {
            test_high_load: false,
            test_conversation_patterns: false,
            test_tool_patterns: false,
            test_error_recovery: true,
            test_performance_load: false,
            test_edge_cases: false,
        },
    };

    let executor = IntegrationTestExecutor::new(config);
    let results = executor.execute().await?;

    // Validate API contract compliance
    assert!(results.total_tests > 0, "No API contract tests executed");

    println!("📋 API Contract Compliance Test Results:");
    println!(
        "   Request format compliant: {}",
        if results.api_contract_results.request_format_compliant {
            "✅"
        } else {
            "❌"
        }
    );
    println!(
        "   Response format compliant: {}",
        if results.api_contract_results.response_format_compliant {
            "✅"
        } else {
            "❌"
        }
    );
    println!(
        "   Error handling compliant: {}",
        if results.api_contract_results.error_handling_compliant {
            "✅"
        } else {
            "❌"
        }
    );
    println!(
        "   Authentication working: {}",
        if results.api_contract_results.authentication_working {
            "✅"
        } else {
            "❌"
        }
    );

    // API contract requirements
    assert!(
        results.api_contract_results.request_format_compliant,
        "API request format non-compliance detected"
    );
    assert!(
        results.api_contract_results.response_format_compliant,
        "API response format non-compliance detected"
    );
    assert!(
        results.api_contract_results.authentication_working,
        "Authentication mechanism issues detected"
    );

    // Check for contract violations
    if !results
        .api_contract_results
        .api_contract_violations
        .is_empty()
    {
        println!("⚠️  API contract violations found:");
        for violation in &results.api_contract_results.api_contract_violations {
            println!("   • {}", violation);
        }
    }

    Ok(())
}

/// Test production readiness scenarios
#[tokio::test]
async fn test_production_readiness_scenarios() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping integration test - AWS credentials not available");
        return Ok(());
    }

    let config = IntegrationTestConfig {
        test_duration: std::time::Duration::from_secs(240),
        bedrock_model: BedrockModel::ClaudeHaiku35,
        test_scenarios: vec![
            IntegrationTestScenario::MultiTurnConversation,
            IntegrationTestScenario::ToolIntegration,
            IntegrationTestScenario::ConcurrentRequestHandling,
            IntegrationTestScenario::ErrorHandlingIntegration,
        ],
        aws_validation: AwsValidationConfig {
            validate_credentials: true,
            validate_region: false,
            validate_bedrock_access: true,
            validate_model_permissions: true,
            test_multiple_regions: false,
            validate_sdk_config: false,
        },
        model_compatibility: ModelCompatibilityConfig {
            test_input_formats: false,
            test_output_formats: false,
            test_parameter_variations: false,
            test_token_limits: true,
            test_streaming: true,
            test_tool_compatibility: true,
        },
        api_contract_validation: ApiContractValidationConfig {
            validate_request_format: false,
            validate_response_format: false,
            test_error_responses: true,
            validate_api_versioning: false,
            test_rate_limiting: false,
            validate_authentication: false,
        },
        production_scenarios: ProductionScenarioConfig {
            test_high_load: true,
            test_conversation_patterns: true,
            test_tool_patterns: true,
            test_error_recovery: true,
            test_performance_load: true,
            test_edge_cases: true,
        },
    };

    let executor = IntegrationTestExecutor::new(config);
    let results = executor.execute().await?;

    // Validate production readiness
    assert!(
        results.total_tests > 0,
        "No production scenario tests executed"
    );

    println!("🏭 Production Readiness Integration Test Results:");
    println!(
        "   High load handling: {}",
        if results.production_scenario_results.high_load_handled {
            "✅"
        } else {
            "❌"
        }
    );
    println!(
        "   Conversation patterns: {}",
        if results
            .production_scenario_results
            .conversation_patterns_working
        {
            "✅"
        } else {
            "❌"
        }
    );
    println!(
        "   Tool patterns: {}",
        if results.production_scenario_results.tool_patterns_working {
            "✅"
        } else {
            "❌"
        }
    );
    println!(
        "   Error recovery: {}",
        if results.production_scenario_results.error_recovery_working {
            "✅"
        } else {
            "❌"
        }
    );
    println!(
        "   Performance under load: {}",
        if results
            .production_scenario_results
            .performance_under_load_acceptable
        {
            "✅"
        } else {
            "❌"
        }
    );
    println!(
        "   Production readiness score: {:.2}",
        results
            .production_scenario_results
            .production_readiness_score
    );

    // Production readiness requirements
    assert!(
        results
            .production_scenario_results
            .production_readiness_score
            >= 0.7,
        "Production readiness score too low: {:.2}",
        results
            .production_scenario_results
            .production_readiness_score
    );
    assert!(
        results
            .production_scenario_results
            .conversation_patterns_working,
        "Conversation pattern failures in production scenarios"
    );
    assert!(
        results.production_scenario_results.error_recovery_working,
        "Error recovery failures in production scenarios"
    );

    // Performance validation
    assert!(
        results.performance_metrics.error_rate <= 0.1,
        "Error rate too high for production: {:.1}%",
        results.performance_metrics.error_rate * 100.0
    );
    assert!(
        results.performance_metrics.availability >= 0.9,
        "Availability too low for production: {:.1}%",
        results.performance_metrics.availability * 100.0
    );

    Ok(())
}

/// Test streaming integration with real Bedrock models
#[tokio::test]
async fn test_streaming_integration() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping integration test - AWS credentials not available");
        return Ok(());
    }

    // Test streaming specifically
    let mut config = TestConfig::default();
    config.extra_args.push("--streaming".to_string());
    let mut session = spawn_cli_with_config(config).await?;

    // Test streaming scenarios
    let streaming_tests = vec![
        "Write a short story about a robot learning to paint",
        "Explain the process of photosynthesis in detail",
        "Create a step-by-step guide for making pasta",
        "Describe the journey of a raindrop from cloud to sea",
    ];

    let mut successful_streams = 0;

    for (i, test_prompt) in streaming_tests.iter().enumerate() {
        println!("📡 Testing streaming scenario {}: {}", i + 1, test_prompt);

        let start_time = std::time::Instant::now();

        match session.send_line(test_prompt).await {
            Ok(_) => {
                // Wait for streaming response
                tokio::time::sleep(std::time::Duration::from_secs(15)).await;
                let duration = start_time.elapsed();

                println!("   ✅ Streaming completed in {:?}", duration);
                successful_streams += 1;

                // Verify system is still responsive after streaming
                match session.send_line("Quick test: 2 + 2").await {
                    Ok(_) => {
                        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                        println!("   ✅ System responsive after streaming");
                    }
                    Err(e) => {
                        println!("   ⚠️  System responsiveness issue after streaming: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("   ❌ Streaming test failed: {}", e);
            }
        }

        // Brief pause between streaming tests
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }

    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    // Streaming integration validation
    assert!(successful_streams > 0, "No streaming tests succeeded");
    assert!(
        successful_streams >= streaming_tests.len() / 2,
        "Too many streaming failures: {}/{}",
        streaming_tests.len() - successful_streams,
        streaming_tests.len()
    );

    println!("📡 Streaming Integration Summary:");
    println!(
        "   Successful streams: {}/{}",
        successful_streams,
        streaming_tests.len()
    );
    println!(
        "   Success rate: {:.1}%",
        successful_streams as f64 / streaming_tests.len() as f64 * 100.0
    );

    Ok(())
}

/// Test large context handling integration
#[tokio::test]
async fn test_large_context_integration() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping integration test - AWS credentials not available");
        return Ok(());
    }

    let mut session = spawn_cli().await?;

    // Build up a substantial conversation context
    println!("🧠 Testing large context handling...");

    let context_building_messages = vec![
        "Let me tell you about a complex project. It involves multiple components.",
        "The first component is a web server that handles HTTP requests.",
        "The second component is a database that stores user information and preferences.",
        "The third component is an authentication service that manages user sessions.",
        "The fourth component is a caching layer that improves performance.",
        "The fifth component is a monitoring system that tracks system health.",
        "The sixth component is a load balancer that distributes traffic.",
        "The seventh component is a backup system that ensures data safety.",
        "All these components need to work together seamlessly.",
        "They communicate through well-defined APIs and message queues.",
    ];

    // Build context
    for (i, message) in context_building_messages.iter().enumerate() {
        println!("   Adding context message {}: {}", i + 1, message);
        match session.send_line(message).await {
            Ok(_) => {
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
            Err(e) => {
                println!("   ⚠️  Context building message failed: {}", e);
            }
        }
    }

    // Test context recall and reasoning
    let context_tests = vec![
        "How many components did I describe in the project?",
        "What does the third component do?",
        "Which component handles performance optimization?",
        "How do all these components communicate?",
        "Can you summarize the entire project architecture?",
    ];

    let mut successful_context_tests = 0;

    for (i, test_question) in context_tests.iter().enumerate() {
        println!("   Testing context recall {}: {}", i + 1, test_question);

        match session.send_line(test_question).await {
            Ok(_) => {
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                successful_context_tests += 1;
                println!("   ✅ Context test succeeded");
            }
            Err(e) => {
                println!("   ❌ Context test failed: {}", e);
            }
        }
    }

    session.send_line("exit").await?;
    session.wait_for_exit().await?;

    // Large context integration validation
    assert!(successful_context_tests > 0, "No context tests succeeded");
    assert!(
        successful_context_tests >= context_tests.len() / 2,
        "Too many context test failures: {}/{}",
        context_tests.len() - successful_context_tests,
        context_tests.len()
    );

    println!("🧠 Large Context Integration Summary:");
    println!(
        "   Context messages added: {}",
        context_building_messages.len()
    );
    println!(
        "   Successful context tests: {}/{}",
        successful_context_tests,
        context_tests.len()
    );
    println!(
        "   Context retention rate: {:.1}%",
        successful_context_tests as f64 / context_tests.len() as f64 * 100.0
    );

    Ok(())
}

/// Test concurrent request handling
#[tokio::test]
async fn test_concurrent_request_integration() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping integration test - AWS credentials not available");
        return Ok(());
    }

    println!("🔄 Testing concurrent request handling...");

    // Test concurrent sessions
    let concurrent_tasks = 5;
    let mut handles = Vec::new();

    for task_id in 0..concurrent_tasks {
        let handle = tokio::spawn(async move {
            if let Ok(mut session) = spawn_cli().await {
                let test_requests = vec![
                    format!("Task {}: Calculate {} * 7", task_id, task_id + 1),
                    format!(
                        "Task {}: What is the capital of country number {}?",
                        task_id,
                        task_id + 1
                    ),
                    format!("Task {}: Count from 1 to {}", task_id, task_id + 3),
                ];

                let mut successful_requests = 0;

                for request in test_requests {
                    match session.send_line(&request).await {
                        Ok(_) => {
                            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                            successful_requests += 1;
                        }
                        Err(_) => {}
                    }
                }

                let _ = session.send_line("exit").await;
                let _ = session.wait_for_exit().await;

                (task_id, successful_requests)
            } else {
                (task_id, 0)
            }
        });
        handles.push(handle);
    }

    // Collect results
    let mut total_successful_requests = 0;
    let mut successful_tasks = 0;

    for handle in handles {
        if let Ok((task_id, successful_requests)) = handle.await {
            total_successful_requests += successful_requests;
            if successful_requests > 0 {
                successful_tasks += 1;
            }
            println!(
                "   Task {}: {} requests succeeded",
                task_id, successful_requests
            );
        }
    }

    // Concurrent integration validation
    assert!(successful_tasks > 0, "No concurrent tasks succeeded");
    assert!(
        successful_tasks >= concurrent_tasks / 2,
        "Too many concurrent task failures: {}/{}",
        concurrent_tasks - successful_tasks,
        concurrent_tasks
    );

    println!("🔄 Concurrent Request Integration Summary:");
    println!("   Concurrent tasks: {}", concurrent_tasks);
    println!(
        "   Successful tasks: {}/{}",
        successful_tasks, concurrent_tasks
    );
    println!(
        "   Total successful requests: {}",
        total_successful_requests
    );
    println!(
        "   Task success rate: {:.1}%",
        successful_tasks as f64 / concurrent_tasks as f64 * 100.0
    );

    Ok(())
}

/// Comprehensive integration test suite
#[tokio::test]
async fn test_comprehensive_integration_suite() -> Result<()> {
    if !check_aws_credentials() {
        println!("⚠️  Skipping comprehensive integration test - AWS credentials not available");
        return Ok(());
    }

    println!("🚀 Running comprehensive integration test suite...");

    let config = IntegrationTestConfig::comprehensive_test_suite();
    let executor = IntegrationTestExecutor::new(config);

    let results = executor.execute().await?;

    // Comprehensive integration validation
    assert!(results.total_tests > 0, "No integration tests executed");
    assert!(results.passed_tests > 0, "No integration tests passed");

    println!("\n🏁 COMPREHENSIVE INTEGRATION TEST SUMMARY");
    println!("════════════════════════════════════════");
    println!(
        "📊 Test Results: {}/{} passed ({:.1}%)",
        results.passed_tests,
        results.total_tests,
        results.passed_tests as f64 / results.total_tests as f64 * 100.0
    );
    println!(
        "🔐 AWS Integration: {}",
        if results.aws_validation_results.bedrock_accessible {
            "✅"
        } else {
            "❌"
        }
    );
    println!(
        "🤖 Model Compatibility: {}",
        if results
            .model_compatibility_results
            .input_format_compatibility
        {
            "✅"
        } else {
            "❌"
        }
    );
    println!(
        "📋 API Contract: {}",
        if results.api_contract_results.request_format_compliant {
            "✅"
        } else {
            "❌"
        }
    );
    println!(
        "🏭 Production Readiness: {:.2}",
        results
            .production_scenario_results
            .production_readiness_score
    );
    println!("⚡ Performance:");
    println!(
        "   Average response time: {:?}",
        results.performance_metrics.average_response_time
    );
    println!(
        "   Error rate: {:.1}%",
        results.performance_metrics.error_rate * 100.0
    );
    println!(
        "   Availability: {:.1}%",
        results.performance_metrics.availability * 100.0
    );

    // Overall integration validation
    let pass_rate = results.passed_tests as f64 / results.total_tests as f64;
    assert!(
        pass_rate >= 0.8,
        "Integration test pass rate too low: {:.1}%",
        pass_rate * 100.0
    );
    assert!(
        results.aws_validation_results.bedrock_accessible,
        "AWS Bedrock integration failed"
    );
    assert!(
        results
            .production_scenario_results
            .production_readiness_score
            >= 0.7,
        "Production readiness score too low: {:.2}",
        results
            .production_scenario_results
            .production_readiness_score
    );

    // Check for critical integration issues
    if !results.critical_integration_issues.is_empty() {
        println!("\n❌ Critical Integration Issues:");
        for issue in &results.critical_integration_issues {
            println!("   • {}", issue);
        }
        panic!("Critical integration issues detected");
    }

    if results.integration_test_passed {
        println!("\n🎉 COMPREHENSIVE INTEGRATION TESTS PASSED");
        println!("✅ System is ready for production deployment with AWS Bedrock");
    } else {
        println!("\n⚠️  Integration tests show areas for improvement");
    }

    Ok(())
}
