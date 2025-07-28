//! Health Check System
//!
//! This module provides enterprise-grade health checking capabilities for the Stood agent library,
//! including dependency verification, resource monitoring, and HTTP endpoint support for
//! Kubernetes and container orchestration platforms.

use crate::config::StoodConfig;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
use thiserror::Error;
use tracing::{debug, error, info, warn};

/// Health check errors
#[derive(Debug, Error)]
pub enum HealthError {
    #[error("Health check failed: {0}")]
    CheckFailed(String),
    #[error("Health check timeout")]
    Timeout,
    #[error("Configuration error: {0}")]
    Configuration(String),
    #[error("Dependency unavailable: {0}")]
    DependencyUnavailable(String),
    #[error("Resource constraint: {0}")]
    ResourceConstraint(String),
}

/// Health check result status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    /// Service is healthy and ready
    Healthy,
    /// Service is running but not ready for traffic
    Degraded,
    /// Service is unhealthy
    Unhealthy,
}

/// Individual health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResult {
    /// Name of the health check
    pub name: String,
    /// Current status
    pub status: HealthStatus,
    /// Execution duration
    pub duration: Duration,
    /// Optional message with details
    pub message: Option<String>,
    /// Timestamp of the check
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Overall health check summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSummary {
    /// Overall status (worst of all checks)
    pub status: HealthStatus,
    /// Individual check results
    pub checks: HashMap<String, HealthResult>,
    /// Total execution duration
    pub total_duration: Duration,
    /// Summary timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthConfig {
    /// Whether health checks are enabled
    pub enabled: bool,
    /// Timeout for individual health checks
    pub timeout: Duration,
    /// Interval between health checks
    pub check_interval: Duration,
    /// Maximum number of consecutive failures before marking unhealthy
    pub max_failures: u32,
    /// HTTP server configuration for exposing health endpoints
    pub http: Option<HttpConfig>,
    /// Configuration for individual checks
    pub checks: HashMap<String, CheckConfig>,
}

/// HTTP server configuration for health endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpConfig {
    /// Port to bind the HTTP server
    pub port: u16,
    /// Bind address
    pub host: String,
    /// Enable liveness probe endpoint
    pub enable_liveness: bool,
    /// Enable readiness probe endpoint
    pub enable_readiness: bool,
    /// Enable metrics endpoint
    pub enable_metrics: bool,
}

/// Configuration for individual health checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckConfig {
    /// Whether this check is enabled
    pub enabled: bool,
    /// Timeout for this specific check
    pub timeout: Option<Duration>,
    /// Weight for determining overall health
    pub weight: f32,
    /// Check-specific configuration
    pub config: HashMap<String, String>,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            timeout: Duration::from_secs(5),
            check_interval: Duration::from_secs(30),
            max_failures: 3,
            http: Some(HttpConfig::default()),
            checks: HashMap::new(),
        }
    }
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            host: "0.0.0.0".to_string(),
            enable_liveness: true,
            enable_readiness: true,
            enable_metrics: true,
        }
    }
}

impl Default for CheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            timeout: None,
            weight: 1.0,
            config: HashMap::new(),
        }
    }
}

/// Trait for implementing health checks
#[async_trait]
pub trait HealthCheck: Send + Sync {
    /// Name of the health check
    fn name(&self) -> &str;

    /// Execute the health check
    async fn check(&self) -> Result<HealthResult, HealthError>;

    /// Get the configuration for this check
    fn config(&self) -> &CheckConfig;

    /// Whether this check is critical for overall health
    fn is_critical(&self) -> bool {
        true
    }
}

/// Main health checker that manages and executes health checks
pub struct HealthChecker {
    config: HealthConfig,
    checks: HashMap<String, Box<dyn HealthCheck>>,
    failure_counts: HashMap<String, u32>,
    last_results: HashMap<String, HealthResult>,
}

impl HealthChecker {
    /// Create a new health checker with the given configuration
    pub fn new(config: HealthConfig) -> Self {
        Self {
            config,
            checks: HashMap::new(),
            failure_counts: HashMap::new(),
            last_results: HashMap::new(),
        }
    }

    /// Create a health checker from the main Stood configuration
    pub fn from_config(stood_config: &StoodConfig) -> Self {
        let health_config = HealthConfig {
            enabled: stood_config.features.health_checks,
            timeout: Duration::from_secs(5),
            check_interval: Duration::from_secs(30),
            max_failures: 3,
            http: if stood_config.features.health_checks {
                Some(HttpConfig::default())
            } else {
                None
            },
            checks: HashMap::new(),
        };

        let mut checker = Self::new(health_config);

        // Add default health checks based on configuration
        checker.add_default_checks(stood_config);

        checker
    }

    /// Add a health check to the checker
    pub fn add_check(&mut self, check: Box<dyn HealthCheck>) {
        let name = check.name().to_string();
        debug!("Adding health check: {}", name);
        self.checks.insert(name, check);
    }

    /// Remove a health check by name
    pub fn remove_check(&mut self, name: &str) -> Option<Box<dyn HealthCheck>> {
        debug!("Removing health check: {}", name);
        self.failure_counts.remove(name);
        self.last_results.remove(name);
        self.checks.remove(name)
    }

    /// Execute all health checks and return a summary
    pub async fn check_health(&mut self) -> HealthSummary {
        let start_time = Instant::now();
        let mut results = HashMap::new();
        let mut overall_status = HealthStatus::Healthy;

        if !self.config.enabled {
            debug!("Health checks are disabled");
            return HealthSummary {
                status: HealthStatus::Healthy,
                checks: HashMap::new(),
                total_duration: Duration::from_millis(0),
                timestamp: chrono::Utc::now(),
            };
        }

        info!("Executing {} health checks", self.checks.len());

        // Execute all health checks concurrently
        let mut check_futures = Vec::new();
        for (name, check) in &self.checks {
            let check_name = name.clone();
            let timeout = check.config().timeout.unwrap_or(self.config.timeout);
            let check_future = async move {
                let check_start = Instant::now();
                match tokio::time::timeout(timeout, check.check()).await {
                    Ok(Ok(result)) => {
                        debug!(
                            "Health check '{}' completed: {:?}",
                            check_name, result.status
                        );
                        result
                    }
                    Ok(Err(e)) => {
                        warn!("Health check '{}' failed: {}", check_name, e);
                        HealthResult {
                            name: check_name.clone(),
                            status: HealthStatus::Unhealthy,
                            duration: check_start.elapsed(),
                            message: Some(e.to_string()),
                            timestamp: chrono::Utc::now(),
                            metadata: HashMap::new(),
                        }
                    }
                    Err(_) => {
                        error!("Health check '{}' timed out", check_name);
                        HealthResult {
                            name: check_name.clone(),
                            status: HealthStatus::Unhealthy,
                            duration: timeout,
                            message: Some("Health check timed out".to_string()),
                            timestamp: chrono::Utc::now(),
                            metadata: HashMap::new(),
                        }
                    }
                }
            };
            check_futures.push((name.clone(), check_future));
        }

        // Wait for all checks to complete
        for (name, future) in check_futures {
            let result = future.await;

            // Update failure count
            if result.status == HealthStatus::Unhealthy {
                let count = self.failure_counts.entry(name.clone()).or_insert(0);
                *count += 1;

                if *count >= self.config.max_failures {
                    overall_status = HealthStatus::Unhealthy;
                }
            } else {
                self.failure_counts.insert(name.clone(), 0);
            }

            // Update overall status
            match (&overall_status, &result.status) {
                (HealthStatus::Healthy, HealthStatus::Degraded) => {
                    overall_status = HealthStatus::Degraded;
                }
                (_, HealthStatus::Unhealthy) => {
                    overall_status = HealthStatus::Unhealthy;
                }
                _ => {}
            }

            self.last_results.insert(name.clone(), result.clone());
            results.insert(name, result);
        }

        let total_duration = start_time.elapsed();

        info!(
            "Health check completed in {:?} with status {:?}",
            total_duration, overall_status
        );

        HealthSummary {
            status: overall_status,
            checks: results,
            total_duration,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Get the last health check results without re-executing
    pub fn last_health_summary(&self) -> HealthSummary {
        let overall_status = if self.last_results.is_empty() {
            HealthStatus::Healthy
        } else {
            self.last_results.values().map(|r| &r.status).fold(
                HealthStatus::Healthy,
                |acc, status| match (&acc, status) {
                    (HealthStatus::Healthy, HealthStatus::Degraded) => HealthStatus::Degraded,
                    (_, HealthStatus::Unhealthy) => HealthStatus::Unhealthy,
                    _ => acc,
                },
            )
        };

        HealthSummary {
            status: overall_status,
            checks: self.last_results.clone(),
            total_duration: Duration::from_millis(0),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Check if the service is ready (all critical checks passing)
    pub async fn is_ready(&mut self) -> bool {
        let summary = self.check_health().await;
        summary.status != HealthStatus::Unhealthy
    }

    /// Check if the service is alive (basic liveness check)
    pub async fn is_alive(&self) -> bool {
        // Simple liveness check - service is running if we can execute this
        true
    }

    /// Add default health checks based on configuration
    fn add_default_checks(&mut self, config: &StoodConfig) {
        // Add configuration health check
        self.add_check(Box::new(ConfigurationHealthCheck::new(config.clone())));

        // Add Bedrock connectivity check if enabled
        if config.features.auto_retry {
            self.add_check(Box::new(BedrockHealthCheck::new(config.bedrock.clone())));
        }

        // Add resource health check
        self.add_check(Box::new(ResourceHealthCheck::new()));

        // Add telemetry health check if enabled
        if config.telemetry.enabled {
            self.add_check(Box::new(TelemetryHealthCheck::new(
                config.telemetry.clone(),
            )));
        }
    }
}

// Import built-in health checks
pub mod checks;
pub use checks::*;

// Optional HTTP server support
// TODO: Re-enable HTTP support with proper axum integration
// #[cfg(feature = "http")]
// pub mod http;
// #[cfg(feature = "http")]
// pub use http::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::StoodConfig;

    #[tokio::test]
    async fn test_health_checker_creation() {
        let config = StoodConfig::default();
        let mut checker = HealthChecker::from_config(&config);

        // Should have default checks added
        assert!(!checker.checks.is_empty());

        let summary = checker.check_health().await;
        assert!(!summary.checks.is_empty());
    }

    #[tokio::test]
    async fn test_health_checker_disabled() {
        let mut config = StoodConfig::default();
        config.features.health_checks = false;

        let mut checker = HealthChecker::from_config(&config);
        checker.config.enabled = false;

        let summary = checker.check_health().await;
        assert_eq!(summary.status, HealthStatus::Healthy);
        assert!(summary.checks.is_empty());
    }

    #[tokio::test]
    async fn test_liveness_check() {
        let config = HealthConfig::default();
        let checker = HealthChecker::new(config);

        assert!(checker.is_alive().await);
    }
}
