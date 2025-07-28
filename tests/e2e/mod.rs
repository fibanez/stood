//! End-to-end tests for the Stood agentic CLI
//!
//! This module contains integration tests that spawn the actual CLI application
//! and test it end-to-end using expectrl for interactive testing.

#[macro_use]
pub mod lib;
pub mod basic_tests;
pub mod debug_test;
pub mod error_handling_tests;
pub mod interactive_tests;
pub mod performance_tests;
pub mod tool_integration_tests;
pub mod visible_demo_test;
// pub mod load_tests; // Module file doesn't exist
pub mod integration_tests;
pub mod performance_regression_tests;
pub mod reliability_tests;
pub mod stress_tests;

// Testing framework modules
pub mod load_testing_framework;
pub mod integration_testing_framework;
pub mod performance_regression_framework;
pub mod reliability_testing_framework;
pub mod stress_testing_framework;

// Re-export commonly used items
pub use lib::{
    check_aws_credentials, create_sample_files, create_temp_dir, spawn_cli, spawn_cli_visible,
    spawn_cli_with_config, CliSession, Result, TestConfig,
};
