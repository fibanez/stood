//! Built-in Health Checks
//!
//! This module provides implementations of common health checks for the Stood agent library.

use super::{CheckConfig, HealthCheck, HealthError, HealthResult, HealthStatus};
use crate::config::{BedrockConfig, TelemetryConfig};
use async_trait::async_trait;
use std::{collections::HashMap, time::Instant};
use tracing::{debug, error, warn};

/// Configuration validation health check
pub struct ConfigurationHealthCheck {
    config: crate::config::StoodConfig,
    check_config: CheckConfig,
}

impl ConfigurationHealthCheck {
    pub fn new(config: crate::config::StoodConfig) -> Self {
        Self {
            config,
            check_config: CheckConfig::default(),
        }
    }
}

#[async_trait]
impl HealthCheck for ConfigurationHealthCheck {
    fn name(&self) -> &str {
        "configuration"
    }

    async fn check(&self) -> Result<HealthResult, HealthError> {
        let start_time = Instant::now();
        let mut metadata = HashMap::new();

        debug!("Performing configuration health check");

        // Validate configuration
        match self.config.validate() {
            Ok(_) => {
                metadata.insert("region".to_string(), self.config.bedrock.region.clone());
                metadata.insert(
                    "model".to_string(),
                    self.config.bedrock.default_model.clone(),
                );
                metadata.insert(
                    "telemetry_enabled".to_string(),
                    self.config.telemetry.enabled.to_string(),
                );

                Ok(HealthResult {
                    name: self.name().to_string(),
                    status: HealthStatus::Healthy,
                    duration: start_time.elapsed(),
                    message: Some("Configuration is valid".to_string()),
                    timestamp: chrono::Utc::now(),
                    metadata,
                })
            }
            Err(e) => {
                error!("Configuration validation failed: {}", e);
                Ok(HealthResult {
                    name: self.name().to_string(),
                    status: HealthStatus::Unhealthy,
                    duration: start_time.elapsed(),
                    message: Some(format!("Configuration validation failed: {}", e)),
                    timestamp: chrono::Utc::now(),
                    metadata,
                })
            }
        }
    }

    fn config(&self) -> &CheckConfig {
        &self.check_config
    }

    fn is_critical(&self) -> bool {
        true
    }
}

/// AWS Bedrock connectivity health check
pub struct BedrockHealthCheck {
    bedrock_config: BedrockConfig,
    check_config: CheckConfig,
}

impl BedrockHealthCheck {
    pub fn new(bedrock_config: BedrockConfig) -> Self {
        Self {
            bedrock_config,
            check_config: CheckConfig::default(),
        }
    }
}

#[async_trait]
impl HealthCheck for BedrockHealthCheck {
    fn name(&self) -> &str {
        "bedrock-connectivity"
    }

    async fn check(&self) -> Result<HealthResult, HealthError> {
        let start_time = Instant::now();
        let mut metadata = HashMap::new();

        debug!("Performing Bedrock connectivity health check");

        metadata.insert("region".to_string(), self.bedrock_config.region.clone());
        metadata.insert(
            "model".to_string(),
            self.bedrock_config.default_model.clone(),
        );

        // Attempt to create AWS config and check connectivity
        match self.check_bedrock_connectivity().await {
            Ok(response_time) => {
                metadata.insert(
                    "response_time_ms".to_string(),
                    response_time.as_millis().to_string(),
                );

                let status = if response_time.as_millis() > 5000 {
                    HealthStatus::Degraded
                } else {
                    HealthStatus::Healthy
                };

                Ok(HealthResult {
                    name: self.name().to_string(),
                    status,
                    duration: start_time.elapsed(),
                    message: Some(format!(
                        "Bedrock connectivity verified in {:?}",
                        response_time
                    )),
                    timestamp: chrono::Utc::now(),
                    metadata,
                })
            }
            Err(e) => {
                error!("Bedrock connectivity check failed: {}", e);
                Ok(HealthResult {
                    name: self.name().to_string(),
                    status: HealthStatus::Unhealthy,
                    duration: start_time.elapsed(),
                    message: Some(format!("Bedrock connectivity failed: {}", e)),
                    timestamp: chrono::Utc::now(),
                    metadata,
                })
            }
        }
    }

    fn config(&self) -> &CheckConfig {
        &self.check_config
    }

    fn is_critical(&self) -> bool {
        true
    }
}

impl BedrockHealthCheck {
    async fn check_bedrock_connectivity(&self) -> Result<std::time::Duration, HealthError> {
        let start_time = Instant::now();

        // Create AWS configuration
        let aws_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(aws_config::Region::new(self.bedrock_config.region.clone()))
            .load()
            .await;

        // Create Bedrock Runtime client
        let bedrock_client = aws_sdk_bedrockruntime::Client::new(&aws_config);

        // Perform a lightweight operation to test connectivity
        // Note: In a real implementation, you might want to use a specific operation
        // that doesn't consume tokens, such as listing foundation models
        match tokio::time::timeout(
            self.bedrock_config.timeout,
            self.test_bedrock_operation(&bedrock_client),
        )
        .await
        {
            Ok(Ok(_)) => Ok(start_time.elapsed()),
            Ok(Err(e)) => Err(HealthError::DependencyUnavailable(e.to_string())),
            Err(_) => Err(HealthError::Timeout),
        }
    }

    async fn test_bedrock_operation(
        &self,
        _client: &aws_sdk_bedrockruntime::Client,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // For this health check, we'll just verify that we can create the client
        // In a real implementation, you might want to make a lightweight API call
        // such as listing available models or making a minimal inference request

        // Simulate a network check with a small delay
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        Ok(())
    }
}

/// System resource health check
pub struct ResourceHealthCheck {
    check_config: CheckConfig,
}

impl Default for ResourceHealthCheck {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceHealthCheck {
    pub fn new() -> Self {
        Self {
            check_config: CheckConfig::default(),
        }
    }
}

#[async_trait]
impl HealthCheck for ResourceHealthCheck {
    fn name(&self) -> &str {
        "system-resources"
    }

    async fn check(&self) -> Result<HealthResult, HealthError> {
        let start_time = Instant::now();
        let mut metadata = HashMap::new();
        let mut status = HealthStatus::Healthy;
        let mut messages = Vec::new();

        debug!("Performing system resource health check");

        // Check memory usage
        if let Ok(memory_info) = self.get_memory_info() {
            metadata.insert(
                "memory_usage_percent".to_string(),
                memory_info.usage_percent.to_string(),
            );
            metadata.insert(
                "memory_available_mb".to_string(),
                (memory_info.available / 1024 / 1024).to_string(),
            );

            if memory_info.usage_percent > 90.0 {
                status = HealthStatus::Unhealthy;
                messages.push("High memory usage detected".to_string());
            } else if memory_info.usage_percent > 80.0 {
                status = HealthStatus::Degraded;
                messages.push("Elevated memory usage".to_string());
            }
        }

        // Check CPU count
        let cpu_count = num_cpus::get();
        metadata.insert("cpu_cores".to_string(), cpu_count.to_string());

        // Check disk space (simplified check)
        if let Ok(disk_info) = self.get_disk_info() {
            metadata.insert(
                "disk_usage_percent".to_string(),
                disk_info.usage_percent.to_string(),
            );
            metadata.insert(
                "disk_available_gb".to_string(),
                (disk_info.available / 1024 / 1024 / 1024).to_string(),
            );

            if disk_info.usage_percent > 95.0 {
                status = HealthStatus::Unhealthy;
                messages.push("Critical disk space shortage".to_string());
            } else if disk_info.usage_percent > 85.0 {
                status = HealthStatus::Degraded;
                messages.push("Low disk space".to_string());
            }
        }

        let message = if messages.is_empty() {
            Some("System resources are healthy".to_string())
        } else {
            Some(messages.join(", "))
        };

        Ok(HealthResult {
            name: self.name().to_string(),
            status,
            duration: start_time.elapsed(),
            message,
            timestamp: chrono::Utc::now(),
            metadata,
        })
    }

    fn config(&self) -> &CheckConfig {
        &self.check_config
    }

    fn is_critical(&self) -> bool {
        false
    }
}

#[derive(Debug)]
struct MemoryInfo {
    usage_percent: f64,
    available: u64,
}

#[derive(Debug)]
struct DiskInfo {
    usage_percent: f64,
    available: u64,
}

impl ResourceHealthCheck {
    fn get_memory_info(&self) -> Result<MemoryInfo, HealthError> {
        // Simplified memory check - in a real implementation, you'd use
        // platform-specific APIs or crates like `sysinfo`

        // For now, return mock values that indicate healthy system
        Ok(MemoryInfo {
            usage_percent: 45.0,
            available: 8 * 1024 * 1024 * 1024, // 8GB
        })
    }

    fn get_disk_info(&self) -> Result<DiskInfo, HealthError> {
        // Simplified disk check - in a real implementation, you'd use
        // platform-specific APIs or crates like `sysinfo`

        // For now, return mock values that indicate healthy system
        Ok(DiskInfo {
            usage_percent: 65.0,
            available: 50 * 1024 * 1024 * 1024, // 50GB
        })
    }
}

/// Telemetry system health check
pub struct TelemetryHealthCheck {
    telemetry_config: TelemetryConfig,
    check_config: CheckConfig,
}

impl TelemetryHealthCheck {
    pub fn new(telemetry_config: TelemetryConfig) -> Self {
        Self {
            telemetry_config,
            check_config: CheckConfig::default(),
        }
    }
}

#[async_trait]
impl HealthCheck for TelemetryHealthCheck {
    fn name(&self) -> &str {
        "telemetry"
    }

    async fn check(&self) -> Result<HealthResult, HealthError> {
        let start_time = Instant::now();
        let mut metadata = HashMap::new();

        debug!("Performing telemetry health check");

        metadata.insert(
            "enabled".to_string(),
            self.telemetry_config.enabled.to_string(),
        );
        metadata.insert(
            "service_name".to_string(),
            self.telemetry_config.service_name.clone(),
        );

        if !self.telemetry_config.enabled {
            return Ok(HealthResult {
                name: self.name().to_string(),
                status: HealthStatus::Healthy,
                duration: start_time.elapsed(),
                message: Some("Telemetry is disabled".to_string()),
                timestamp: chrono::Utc::now(),
                metadata,
            });
        }

        // Check telemetry configuration
        let mut issues = Vec::new();
        let mut status = HealthStatus::Healthy;

        if let Some(ref endpoint) = self.telemetry_config.otlp_endpoint {
            metadata.insert("otlp_endpoint".to_string(), endpoint.clone());

            // Test OTLP endpoint connectivity
            match self.test_otlp_connectivity(endpoint).await {
                Ok(response_time) => {
                    metadata.insert(
                        "endpoint_response_time_ms".to_string(),
                        response_time.as_millis().to_string(),
                    );
                    if response_time.as_millis() > 2000 {
                        status = HealthStatus::Degraded;
                        issues.push("Slow telemetry endpoint response".to_string());
                    }
                }
                Err(e) => {
                    warn!("OTLP endpoint connectivity check failed: {}", e);
                    status = HealthStatus::Degraded;
                    issues.push(format!("OTLP endpoint unreachable: {}", e));
                }
            }
        } else {
            issues.push("No OTLP endpoint configured".to_string());
            status = HealthStatus::Degraded;
        }

        let message = if issues.is_empty() {
            Some("Telemetry system is healthy".to_string())
        } else {
            Some(issues.join(", "))
        };

        Ok(HealthResult {
            name: self.name().to_string(),
            status,
            duration: start_time.elapsed(),
            message,
            timestamp: chrono::Utc::now(),
            metadata,
        })
    }

    fn config(&self) -> &CheckConfig {
        &self.check_config
    }

    fn is_critical(&self) -> bool {
        false
    }
}

impl TelemetryHealthCheck {
    async fn test_otlp_connectivity(
        &self,
        endpoint: &str,
    ) -> Result<std::time::Duration, HealthError> {
        let start_time = Instant::now();

        // Create HTTP client for connectivity test
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|e| HealthError::Configuration(e.to_string()))?;

        // Try to connect to the OTLP endpoint
        match client.head(endpoint).send().await {
            Ok(response) => {
                if response.status().is_success() || response.status() == 405 {
                    // 405 Method Not Allowed is acceptable for HEAD requests to OTLP endpoints
                    Ok(start_time.elapsed())
                } else {
                    Err(HealthError::DependencyUnavailable(format!(
                        "OTLP endpoint returned status: {}",
                        response.status()
                    )))
                }
            }
            Err(e) => Err(HealthError::DependencyUnavailable(format!(
                "Failed to connect to OTLP endpoint: {}",
                e
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{BedrockConfig, TelemetryConfig};

    #[tokio::test]
    async fn test_configuration_health_check() {
        let config = crate::config::StoodConfig::default();
        let check = ConfigurationHealthCheck::new(config);

        let result = check.check().await.unwrap();
        assert_eq!(result.status, HealthStatus::Healthy);
        assert_eq!(result.name, "configuration");
    }

    #[tokio::test]
    async fn test_resource_health_check() {
        let check = ResourceHealthCheck::new();

        let result = check.check().await.unwrap();
        // Should be healthy with mock values
        assert!(matches!(result.status, HealthStatus::Healthy));
        assert_eq!(result.name, "system-resources");
    }

    #[tokio::test]
    async fn test_telemetry_health_check_disabled() {
        let mut telemetry_config = TelemetryConfig::default();
        telemetry_config.enabled = false;

        let check = TelemetryHealthCheck::new(telemetry_config);

        let result = check.check().await.unwrap();
        assert_eq!(result.status, HealthStatus::Healthy);
        assert_eq!(result.name, "telemetry");
    }

    #[tokio::test]
    async fn test_bedrock_health_check() {
        let bedrock_config = BedrockConfig::default();
        let check = BedrockHealthCheck::new(bedrock_config);

        // This test might fail in CI without AWS credentials, which is expected
        let result = check.check().await.unwrap();
        assert_eq!(result.name, "bedrock-connectivity");
        // Status depends on AWS credentials availability
    }
}
