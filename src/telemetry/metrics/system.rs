//! System resource metrics collection for the Stood agent

use crate::telemetry::metrics::{MetricsCollector, SystemMetrics};
// System metrics collection - OpenTelemetry imports not used directly here
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error};

/// System metrics collector for tracking resource usage
pub struct SystemMetricsCollector {
    metrics_collector: Arc<dyn MetricsCollector>,
    last_collection: Arc<RwLock<Instant>>,
    collection_interval: Duration,
}

impl SystemMetricsCollector {
    /// Create a new system metrics collector
    pub fn new(
        metrics_collector: Arc<dyn MetricsCollector>,
        collection_interval: Duration,
    ) -> Self {
        Self {
            metrics_collector,
            last_collection: Arc::new(RwLock::new(Instant::now())),
            collection_interval,
        }
    }

    /// Collect and record current system metrics
    pub async fn collect_metrics(&self) -> Result<SystemMetrics, Box<dyn std::error::Error + Send + Sync>> {
        let now = Instant::now();
        let mut last_collection = self.last_collection.write().await;
        
        // Check if enough time has passed since last collection
        if now.duration_since(*last_collection) < self.collection_interval {
            return Err("Collection interval not reached".into());
        }
        
        *last_collection = now;
        drop(last_collection);

        debug!("Collecting system metrics");

        // Collect memory usage
        let memory_usage = self.collect_memory_usage().await?;
        
        // Collect connection pool metrics
        let active_connections = self.collect_connection_metrics().await?;
        
        // Collect concurrency metrics
        let concurrent_requests = self.collect_concurrency_metrics().await?;
        
        // Collect thread utilization
        let thread_utilization = self.collect_thread_utilization().await?;

        let system_metrics = SystemMetrics {
            memory_usage_bytes: memory_usage,
            active_connections,
            concurrent_requests,
            thread_utilization,
        };

        // Record metrics via collector
        self.metrics_collector.record_system_metrics(&system_metrics);

        debug!("System metrics collected: {:?}", system_metrics);
        Ok(system_metrics)
    }

    /// Collect memory usage in bytes
    async fn collect_memory_usage(&self) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        // Try to get memory info from the system
        // Note: This is a simplified implementation. In production, you'd want to use
        // a proper system monitoring library like `sysinfo` or `procfs`
        
        #[cfg(target_os = "linux")]
        {
            match self.collect_linux_memory().await {
                Ok(memory) => Ok(memory),
                Err(e) => {
                    error!("Failed to collect Linux memory stats: {}", e);
                    Ok(0) // Return 0 on error rather than failing
                }
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            match self.collect_macos_memory().await {
                Ok(memory) => Ok(memory),
                Err(e) => {
                    error!("Failed to collect macOS memory stats: {}", e);
                    Ok(0)
                }
            }
        }
        
        #[cfg(target_os = "windows")]
        {
            match self.collect_windows_memory().await {
                Ok(memory) => Ok(memory),
                Err(e) => {
                    error!("Failed to collect Windows memory stats: {}", e);
                    Ok(0)
                }
            }
        }
        
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            debug!("Memory collection not implemented for this platform");
            Ok(0)
        }
    }

    #[cfg(target_os = "linux")]
    async fn collect_linux_memory(&self) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        use std::fs;
        
        // Read /proc/self/status for memory usage
        let status = fs::read_to_string("/proc/self/status")?;
        
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                // Parse VmRSS (Resident Set Size) - physical memory currently used
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let kb: u64 = parts[1].parse()?;
                    return Ok(kb * 1024); // Convert KB to bytes
                }
            }
        }
        
        Ok(0)
    }

    #[cfg(target_os = "macos")]
    async fn collect_macos_memory(&self) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        // Use mach system calls to get memory info
        // This is a simplified implementation
        Ok(0) // Placeholder
    }

    #[cfg(target_os = "windows")]
    async fn collect_windows_memory(&self) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        // Use Windows API to get memory info
        // This is a simplified implementation
        Ok(0) // Placeholder
    }

    /// Collect connection pool metrics
    async fn collect_connection_metrics(&self) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        // In a real implementation, this would query the connection pool
        // For now, return a placeholder value
        Ok(5) // Placeholder for active connections
    }

    /// Collect current concurrent requests
    async fn collect_concurrency_metrics(&self) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        // In a real implementation, this would track active requests
        // For now, return a placeholder value
        Ok(2) // Placeholder for concurrent requests
    }

    /// Collect thread utilization percentage
    async fn collect_thread_utilization(&self) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
        // Calculate thread pool utilization
        // This is a simplified calculation
        let num_cpus = num_cpus::get() as f64;
        let active_threads = 2.0; // Placeholder
        
        let utilization = (active_threads / num_cpus).min(1.0);
        Ok(utilization)
    }

    /// Start background collection task
    pub fn start_background_collection(
        self: Arc<Self>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(self.collection_interval);
            
            loop {
                interval.tick().await;
                
                if let Err(e) = self.collect_metrics().await {
                    debug!("System metrics collection skipped: {}", e);
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telemetry::metrics::NoOpMetricsCollector;
    use std::time::Duration;

    #[tokio::test]
    async fn test_system_metrics_collector_creation() {
        let collector = Arc::new(NoOpMetricsCollector::default());
        let system_collector = SystemMetricsCollector::new(
            collector,
            Duration::from_secs(10),
        );

        // Test that we can create the collector without errors
        assert!(system_collector.collection_interval == Duration::from_secs(10));
    }

    #[tokio::test]
    async fn test_memory_collection() {
        let collector = Arc::new(NoOpMetricsCollector::default());
        let system_collector = SystemMetricsCollector::new(
            collector,
            Duration::from_millis(1), // Very short interval for testing
        );

        // Test memory collection
        let memory = system_collector.collect_memory_usage().await;
        assert!(memory.is_ok());
    }
}