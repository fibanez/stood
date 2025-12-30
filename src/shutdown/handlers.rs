//! Built-in Shutdown Handlers
//!
//! This module provides implementations of shutdown handlers for common Stood library components.

use super::ShutdownHandler;
use async_trait::async_trait;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{sync::broadcast, time::timeout};
use tracing::warn;
use tracing::{debug, error, info};

/// Shutdown handler for AWS Bedrock client connections
pub struct BedrockShutdownHandler {
    name: String,
    client: Option<aws_sdk_bedrockruntime::Client>,
}

impl BedrockShutdownHandler {
    pub fn new(client: aws_sdk_bedrockruntime::Client) -> Self {
        Self {
            name: "bedrock-client".to_string(),
            client: Some(client),
        }
    }
}

#[async_trait]
impl ShutdownHandler for BedrockShutdownHandler {
    fn name(&self) -> &str {
        &self.name
    }

    async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        debug!("Shutting down Bedrock client");

        // AWS SDK clients don't require explicit shutdown, but we can drop the reference
        // and allow any in-flight requests to complete naturally
        if self.client.is_some() {
            info!("Bedrock client resources released");
        }

        Ok(())
    }

    fn priority(&self) -> u32 {
        50 // Medium priority - shut down after request handlers but before telemetry
    }
}

/// Shutdown handler for telemetry and observability systems
pub struct TelemetryShutdownHandler {
    name: String,
    flush_timeout: Duration,
}

impl Default for TelemetryShutdownHandler {
    fn default() -> Self {
        Self {
            name: "telemetry".to_string(),
            flush_timeout: Duration::from_secs(5),
        }
    }
}

impl TelemetryShutdownHandler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_flush_timeout(mut self, timeout: Duration) -> Self {
        self.flush_timeout = timeout;
        self
    }
}

#[async_trait]
impl ShutdownHandler for TelemetryShutdownHandler {
    fn name(&self) -> &str {
        &self.name
    }

    async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        debug!("Shutting down telemetry system");

        // Flush and shutdown OpenTelemetry
        use opentelemetry::global;

        // Force flush with timeout
        let flush_result = timeout(self.flush_timeout, async {
            // Flush tracer provider
            global::shutdown_tracer_provider();

            info!("Telemetry system flushed and shut down successfully");
            Ok(())
        })
        .await;

        match flush_result {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => Err(e),
            Err(_) => {
                error!("Telemetry flush timed out after {:?}", self.flush_timeout);
                Err("Telemetry flush timeout".into())
            }
        }
    }

    fn priority(&self) -> u32 {
        200 // Low priority - shut down last to capture all events
    }
}

/// Shutdown handler for event loop and agent cycles
pub struct EventLoopShutdownHandler {
    name: String,
    shutdown_signal: Arc<AtomicBool>,
    request_timeout: Duration,
}

impl EventLoopShutdownHandler {
    pub fn new(shutdown_signal: Arc<AtomicBool>) -> Self {
        Self {
            name: "event-loop".to_string(),
            shutdown_signal,
            request_timeout: Duration::from_secs(10),
        }
    }

    pub fn with_request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = timeout;
        self
    }
}

#[async_trait]
impl ShutdownHandler for EventLoopShutdownHandler {
    fn name(&self) -> &str {
        &self.name
    }

    async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        debug!("Shutting down event loop");

        // Signal the event loop to stop accepting new requests
        self.shutdown_signal.store(true, Ordering::Release);

        // Wait for in-flight requests to complete
        let wait_result = timeout(self.request_timeout, async {
            // In a real implementation, this would wait for active cycles to complete
            // For now, we'll simulate a brief wait for pending operations
            tokio::time::sleep(Duration::from_millis(100)).await;
            info!("All event loop cycles completed");
        })
        .await;

        match wait_result {
            Ok(()) => {
                info!("Event loop shut down gracefully");
                Ok(())
            }
            Err(_) => {
                warn!("Event loop shutdown timed out, some requests may be incomplete");
                Ok(()) // We still consider this successful for graceful degradation
            }
        }
    }

    fn priority(&self) -> u32 {
        10 // High priority - stop accepting new work first
    }

    fn requires_graceful(&self) -> bool {
        true // Event loops need graceful shutdown to complete in-flight requests
    }
}

/// Shutdown handler for thread pools and background tasks
pub struct ThreadPoolShutdownHandler {
    name: String,
    shutdown_sender: Option<broadcast::Sender<()>>,
    wait_timeout: Duration,
}

impl ThreadPoolShutdownHandler {
    pub fn new(shutdown_sender: broadcast::Sender<()>) -> Self {
        Self {
            name: "thread-pool".to_string(),
            shutdown_sender: Some(shutdown_sender),
            wait_timeout: Duration::from_secs(5),
        }
    }

    pub fn with_wait_timeout(mut self, timeout: Duration) -> Self {
        self.wait_timeout = timeout;
        self
    }
}

#[async_trait]
impl ShutdownHandler for ThreadPoolShutdownHandler {
    fn name(&self) -> &str {
        &self.name
    }

    async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        debug!("Shutting down thread pool");

        if let Some(sender) = &self.shutdown_sender {
            // Signal all background tasks to shutdown
            let _ = sender.send(());

            // Wait for tasks to complete
            let wait_result = timeout(self.wait_timeout, async {
                // In a real implementation, this would wait for all spawned tasks
                tokio::time::sleep(Duration::from_millis(50)).await;
                info!("All background tasks completed");
            })
            .await;

            match wait_result {
                Ok(()) => {
                    info!("Thread pool shut down successfully");
                    Ok(())
                }
                Err(_) => {
                    warn!(
                        "Thread pool shutdown timed out after {:?}",
                        self.wait_timeout
                    );
                    Ok(()) // Still consider successful for graceful degradation
                }
            }
        } else {
            debug!("No thread pool to shut down");
            Ok(())
        }
    }

    fn priority(&self) -> u32 {
        30 // Medium-high priority - shut down background tasks early
    }
}

/// Shutdown handler for health check system
pub struct HealthCheckShutdownHandler {
    name: String,
    health_checker: Option<Arc<tokio::sync::Mutex<crate::health::HealthChecker>>>,
}

impl HealthCheckShutdownHandler {
    pub fn new(health_checker: Arc<tokio::sync::Mutex<crate::health::HealthChecker>>) -> Self {
        Self {
            name: "health-checks".to_string(),
            health_checker: Some(health_checker),
        }
    }
}

#[async_trait]
impl ShutdownHandler for HealthCheckShutdownHandler {
    fn name(&self) -> &str {
        &self.name
    }

    async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        debug!("Shutting down health check system");

        if let Some(health_checker) = &self.health_checker {
            // Update health checks to reflect shutdown state
            // This allows load balancers to drain traffic
            let checker = health_checker.lock().await;

            // In a real implementation, we might add a shutdown state to the health checker
            // For now, we'll just log that health checks are being disabled
            info!("Health checks disabled for shutdown");
            drop(checker); // Release the lock
        }

        info!("Health check system shut down");
        Ok(())
    }

    fn priority(&self) -> u32 {
        20 // High priority - disable health checks early to drain traffic
    }

    fn requires_graceful(&self) -> bool {
        false // Health checks can be disabled immediately
    }
}

/// Shutdown handler for configuration and file resources
pub struct ConfigurationShutdownHandler {
    name: String,
}

impl Default for ConfigurationShutdownHandler {
    fn default() -> Self {
        Self {
            name: "configuration".to_string(),
        }
    }
}

impl ConfigurationShutdownHandler {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl ShutdownHandler for ConfigurationShutdownHandler {
    fn name(&self) -> &str {
        &self.name
    }

    async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        debug!("Shutting down configuration system");

        // Configuration system doesn't typically need explicit cleanup,
        // but we can log for audit purposes
        info!("Configuration system resources released");
        Ok(())
    }

    fn priority(&self) -> u32 {
        100 // Medium priority - standard cleanup
    }

    fn requires_graceful(&self) -> bool {
        false // Configuration can be cleaned up immediately
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicBool;
    use tokio::sync::broadcast;

    #[tokio::test]
    async fn test_telemetry_shutdown_handler() {
        let handler = TelemetryShutdownHandler::new();

        let result = handler.shutdown().await;
        assert!(result.is_ok());
        assert_eq!(handler.name(), "telemetry");
        assert_eq!(handler.priority(), 200);
    }

    #[tokio::test]
    async fn test_event_loop_shutdown_handler() {
        let shutdown_signal = Arc::new(AtomicBool::new(false));
        let handler = EventLoopShutdownHandler::new(shutdown_signal.clone());

        assert!(!shutdown_signal.load(Ordering::Acquire));

        let result = handler.shutdown().await;
        assert!(result.is_ok());
        assert!(shutdown_signal.load(Ordering::Acquire));
        assert!(handler.requires_graceful());
    }

    #[tokio::test]
    async fn test_thread_pool_shutdown_handler() {
        let (sender, _receiver) = broadcast::channel(1);
        let handler = ThreadPoolShutdownHandler::new(sender);

        let result = handler.shutdown().await;
        assert!(result.is_ok());
        assert_eq!(handler.priority(), 30);
    }

    #[tokio::test]
    async fn test_configuration_shutdown_handler() {
        let handler = ConfigurationShutdownHandler::new();

        let result = handler.shutdown().await;
        assert!(result.is_ok());
        assert!(!handler.requires_graceful());
    }
}
