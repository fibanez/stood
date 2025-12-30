//! CloudWatch Log Group Management for GenAI Observability
//!
//! This module provides functionality to create and manage CloudWatch Log Groups
//! required for the GenAI Observability Dashboard.
//!
//! # Why This Is Required
//!
//! Per AWS documentation, for agents running outside of the AgentCore runtime,
//! the log group `/aws/bedrock-agentcore/runtimes/{agent_id}` MUST physically
//! exist in CloudWatch. Setting `aws.log.group.names` as a resource attribute
//! alone is NOT sufficient for spans to appear in the GenAI Dashboard.
//!
//! # Log Group Format
//!
//! The log group name follows the AgentCore runtime pattern:
//! ```text
//! /aws/bedrock-agentcore/runtimes/{agent_id}
//! ```
//!
//! With a log stream named:
//! ```text
//! runtime-logs
//! ```
//!
//! # Example
//!
//! ```ignore
//! use stood::telemetry::log_group::{AgentLogGroup, LogGroupManager};
//!
//! // Create log group config
//! let log_group = AgentLogGroup::new("my-agent-001");
//!
//! // Create manager and ensure log group exists
//! let manager = LogGroupManager::new("us-east-1").await?;
//! manager.ensure_exists(&log_group).await?;
//! ```

use aws_sdk_cloudwatchlogs::Client as CloudWatchLogsClient;
use std::sync::atomic::{AtomicBool, Ordering};

/// Errors that can occur during log group management
#[derive(Debug, thiserror::Error)]
pub enum LogGroupError {
    /// Failed to create log group
    #[error("Failed to create log group '{0}': {1}")]
    CreateLogGroupFailed(String, String),

    /// Failed to create log stream
    #[error("Failed to create log stream '{0}' in log group '{1}': {2}")]
    CreateLogStreamFailed(String, String, String),

    /// Failed to check if log group exists
    #[error("Failed to check log group existence: {0}")]
    DescribeFailed(String),

    /// AWS SDK configuration error
    #[error("AWS SDK configuration error: {0}")]
    SdkConfig(String),
}

/// Configuration for an agent's CloudWatch log group
///
/// Represents the log group and stream configuration required for
/// an agent to appear in the CloudWatch GenAI Observability Dashboard.
#[derive(Debug, Clone)]
pub struct AgentLogGroup {
    /// The full log group name
    /// Format: `/aws/bedrock-agentcore/runtimes/{agent_id}`
    pub log_group_name: String,

    /// The log stream name within the log group
    /// Typically "runtime-logs"
    pub log_stream_name: String,

    /// The agent ID used to construct the log group name
    pub agent_id: String,
}

impl AgentLogGroup {
    /// Create a new agent log group configuration
    ///
    /// The log group name follows the AgentCore runtime pattern:
    /// `/aws/bedrock-agentcore/runtimes/{agent_id}`
    ///
    /// # Arguments
    ///
    /// * `agent_id` - Unique identifier for the agent. This should be a
    ///   stable identifier that you can use to find traces later.
    ///   Examples: "qanda-agent-001", "customer-support-prod"
    pub fn new(agent_id: impl Into<String>) -> Self {
        let agent_id = agent_id.into();
        Self {
            log_group_name: format!("/aws/bedrock-agentcore/runtimes/{}", agent_id),
            log_stream_name: "runtime-logs".to_string(),
            agent_id,
        }
    }

    /// Create with a custom log stream name
    pub fn with_log_stream(mut self, stream_name: impl Into<String>) -> Self {
        self.log_stream_name = stream_name.into();
        self
    }

    /// Get the log group name
    pub fn log_group_name(&self) -> &str {
        &self.log_group_name
    }

    /// Get the log stream name
    pub fn log_stream_name(&self) -> &str {
        &self.log_stream_name
    }

    /// Get the agent ID
    pub fn agent_id(&self) -> &str {
        &self.agent_id
    }
}

/// Manager for CloudWatch Log Groups required for GenAI Observability
///
/// This manager handles the creation and verification of log groups
/// that are required for spans to appear in the CloudWatch GenAI
/// Observability Dashboard.
#[derive(Debug)]
pub struct LogGroupManager {
    client: CloudWatchLogsClient,
    region: String,
    initialized: AtomicBool,
}

impl LogGroupManager {
    /// Create a new LogGroupManager using the default AWS credentials chain
    ///
    /// This will attempt to load credentials from:
    /// 1. Environment variables (AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY)
    /// 2. AWS credential file (~/.aws/credentials)
    /// 3. IAM role (for EC2/ECS/Lambda)
    pub async fn new(region: impl Into<String>) -> Result<Self, LogGroupError> {
        let region = region.into();

        let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(aws_types::region::Region::new(region.clone()))
            .load()
            .await;

        let client = CloudWatchLogsClient::new(&config);

        Ok(Self {
            client,
            region,
            initialized: AtomicBool::new(false),
        })
    }

    /// Create a LogGroupManager with a custom AWS SDK config
    pub fn from_sdk_config(config: &aws_types::SdkConfig, region: impl Into<String>) -> Self {
        let client = CloudWatchLogsClient::new(config);

        Self {
            client,
            region: region.into(),
            initialized: AtomicBool::new(false),
        }
    }

    /// Get the AWS region this manager is configured for
    pub fn region(&self) -> &str {
        &self.region
    }

    /// Check if the manager has been initialized (log group created/verified)
    pub fn is_initialized(&self) -> bool {
        self.initialized.load(Ordering::Relaxed)
    }

    /// Ensure the log group and log stream exist
    ///
    /// This method is idempotent - it can be called multiple times safely.
    /// If the log group and stream already exist, this is a no-op.
    ///
    /// # Returns
    ///
    /// Returns `Ok(true)` if the log group was created, `Ok(false)` if it already existed.
    pub async fn ensure_exists(&self, config: &AgentLogGroup) -> Result<bool, LogGroupError> {
        let created = self.ensure_log_group_exists(&config.log_group_name).await?;
        self.ensure_log_stream_exists(&config.log_group_name, &config.log_stream_name)
            .await?;

        self.initialized.store(true, Ordering::Relaxed);
        Ok(created)
    }

    /// Create a log group if it doesn't exist
    ///
    /// # Returns
    ///
    /// Returns `Ok(true)` if the log group was created, `Ok(false)` if it already existed.
    async fn ensure_log_group_exists(&self, log_group_name: &str) -> Result<bool, LogGroupError> {
        // Check if log group already exists
        let exists = self.log_group_exists(log_group_name).await?;
        if exists {
            tracing::debug!("Log group '{}' already exists", log_group_name);
            return Ok(false);
        }

        // Create the log group
        tracing::info!("Creating log group '{}'", log_group_name);

        self.client
            .create_log_group()
            .log_group_name(log_group_name)
            .send()
            .await
            .map_err(|e| {
                // Check if it's a ResourceAlreadyExistsException (race condition)
                let err_str = e.to_string();
                if err_str.contains("ResourceAlreadyExistsException") {
                    tracing::debug!(
                        "Log group '{}' was created by another process",
                        log_group_name
                    );
                    // Return early with success since it exists now
                    return LogGroupError::CreateLogGroupFailed(
                        log_group_name.to_string(),
                        "Already exists (created by another process)".to_string(),
                    );
                }
                LogGroupError::CreateLogGroupFailed(log_group_name.to_string(), err_str)
            })?;

        tracing::info!("Created log group '{}'", log_group_name);
        Ok(true)
    }

    /// Check if a log group exists
    async fn log_group_exists(&self, log_group_name: &str) -> Result<bool, LogGroupError> {
        let result = self
            .client
            .describe_log_groups()
            .log_group_name_prefix(log_group_name)
            .limit(1)
            .send()
            .await
            .map_err(|e| LogGroupError::DescribeFailed(e.to_string()))?;

        // Check if any of the returned log groups exactly match
        for lg in result.log_groups() {
            if lg.log_group_name() == Some(log_group_name) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Create a log stream if it doesn't exist
    async fn ensure_log_stream_exists(
        &self,
        log_group_name: &str,
        log_stream_name: &str,
    ) -> Result<bool, LogGroupError> {
        // Check if log stream already exists
        let exists = self
            .log_stream_exists(log_group_name, log_stream_name)
            .await?;
        if exists {
            tracing::debug!(
                "Log stream '{}' in '{}' already exists",
                log_stream_name,
                log_group_name
            );
            return Ok(false);
        }

        // Create the log stream
        tracing::info!(
            "Creating log stream '{}' in '{}'",
            log_stream_name,
            log_group_name
        );

        self.client
            .create_log_stream()
            .log_group_name(log_group_name)
            .log_stream_name(log_stream_name)
            .send()
            .await
            .map_err(|e| {
                // Check if it's a ResourceAlreadyExistsException (race condition)
                let err_str = e.to_string();
                if err_str.contains("ResourceAlreadyExistsException") {
                    tracing::debug!(
                        "Log stream '{}' was created by another process",
                        log_stream_name
                    );
                    // This is actually fine, we can ignore this error
                    return LogGroupError::CreateLogStreamFailed(
                        log_stream_name.to_string(),
                        log_group_name.to_string(),
                        "Already exists (created by another process)".to_string(),
                    );
                }
                LogGroupError::CreateLogStreamFailed(
                    log_stream_name.to_string(),
                    log_group_name.to_string(),
                    err_str,
                )
            })?;

        tracing::info!(
            "Created log stream '{}' in '{}'",
            log_stream_name,
            log_group_name
        );
        Ok(true)
    }

    /// Check if a log stream exists
    async fn log_stream_exists(
        &self,
        log_group_name: &str,
        log_stream_name: &str,
    ) -> Result<bool, LogGroupError> {
        let result = self
            .client
            .describe_log_streams()
            .log_group_name(log_group_name)
            .log_stream_name_prefix(log_stream_name)
            .limit(1)
            .send()
            .await
            .map_err(|e| LogGroupError::DescribeFailed(e.to_string()))?;

        // Check if any of the returned log streams exactly match
        for ls in result.log_streams() {
            if ls.log_stream_name() == Some(log_stream_name) {
                return Ok(true);
            }
        }

        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_log_group_new() {
        let config = AgentLogGroup::new("my-agent");

        assert_eq!(
            config.log_group_name(),
            "/aws/bedrock-agentcore/runtimes/my-agent"
        );
        assert_eq!(config.log_stream_name(), "runtime-logs");
        assert_eq!(config.agent_id(), "my-agent");
    }

    #[test]
    fn test_agent_log_group_with_custom_stream() {
        let config = AgentLogGroup::new("test-agent").with_log_stream("custom-stream");

        assert_eq!(
            config.log_group_name(),
            "/aws/bedrock-agentcore/runtimes/test-agent"
        );
        assert_eq!(config.log_stream_name(), "custom-stream");
    }

    #[test]
    fn test_agent_log_group_format() {
        // Test various agent IDs
        let configs = vec![
            ("simple", "/aws/bedrock-agentcore/runtimes/simple"),
            ("with-dashes", "/aws/bedrock-agentcore/runtimes/with-dashes"),
            (
                "with_underscores",
                "/aws/bedrock-agentcore/runtimes/with_underscores",
            ),
            ("agent123", "/aws/bedrock-agentcore/runtimes/agent123"),
        ];

        for (agent_id, expected_log_group) in configs {
            let config = AgentLogGroup::new(agent_id);
            assert_eq!(config.log_group_name(), expected_log_group);
        }
    }
}
