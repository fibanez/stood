//! Graceful Shutdown System
//!
//! This module provides enterprise-grade graceful shutdown capabilities for the Stood agent library,
//! including signal handling, resource cleanup, and coordinated shutdown procedures.

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{
    signal,
    sync::{broadcast, Notify},
    time::timeout,
};
use tracing::{debug, error, info, warn};

// Import built-in shutdown handlers
pub mod handlers;
pub use handlers::*;

/// Shutdown signals and reasons
#[derive(Debug, Clone, PartialEq)]
pub enum ShutdownReason {
    /// Graceful shutdown requested (SIGTERM)
    Graceful,
    /// Immediate shutdown requested (SIGINT)
    Immediate,
    /// Application-initiated shutdown
    ApplicationRequest,
    /// Shutdown due to critical error
    CriticalError(String),
}

/// Shutdown configuration
#[derive(Debug, Clone)]
pub struct ShutdownConfig {
    /// Maximum time to wait for graceful shutdown
    pub graceful_timeout: Duration,
    /// Time to wait for in-flight requests to complete
    pub request_timeout: Duration,
    /// Time to wait for resource cleanup
    pub cleanup_timeout: Duration,
    /// Whether to force shutdown after timeout
    pub force_after_timeout: bool,
    /// Enable telemetry flush during shutdown
    pub flush_telemetry: bool,
}

impl Default for ShutdownConfig {
    fn default() -> Self {
        Self {
            graceful_timeout: Duration::from_secs(30),
            request_timeout: Duration::from_secs(15),
            cleanup_timeout: Duration::from_secs(10),
            force_after_timeout: true,
            flush_telemetry: true,
        }
    }
}

/// Shutdown manager coordinates graceful shutdown across the application
pub struct ShutdownManager {
    config: ShutdownConfig,
    shutdown_sender: broadcast::Sender<ShutdownReason>,
    is_shutting_down: Arc<AtomicBool>,
    shutdown_complete: Arc<Notify>,
    cleanup_handlers: Vec<Box<dyn ShutdownHandler>>,
}

/// Trait for components that need cleanup during shutdown
#[async_trait::async_trait]
pub trait ShutdownHandler: Send + Sync {
    /// Name of the component being shut down
    fn name(&self) -> &str;

    /// Perform graceful shutdown of the component
    async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Get priority for shutdown order (lower values shut down first)
    fn priority(&self) -> u32 {
        100
    }

    /// Whether this handler requires graceful shutdown (vs immediate)
    fn requires_graceful(&self) -> bool {
        true
    }
}

impl ShutdownManager {
    /// Create a new shutdown manager
    pub fn new(config: ShutdownConfig) -> Self {
        let (shutdown_sender, _) = broadcast::channel(16);

        Self {
            config,
            shutdown_sender,
            is_shutting_down: Arc::new(AtomicBool::new(false)),
            shutdown_complete: Arc::new(Notify::new()),
            cleanup_handlers: Vec::new(),
        }
    }

    /// Create a default shutdown manager
    pub fn default() -> Self {
        Self::new(ShutdownConfig::default())
    }

    /// Get a shutdown receiver for listening to shutdown signals
    pub fn subscribe(&self) -> broadcast::Receiver<ShutdownReason> {
        self.shutdown_sender.subscribe()
    }

    /// Check if shutdown is in progress
    pub fn is_shutting_down(&self) -> bool {
        self.is_shutting_down.load(Ordering::Acquire)
    }

    /// Register a shutdown handler
    pub fn register_handler(&mut self, handler: Box<dyn ShutdownHandler>) {
        debug!("Registering shutdown handler: {}", handler.name());
        self.cleanup_handlers.push(handler);
    }

    /// Start listening for shutdown signals
    pub async fn listen_for_signals(&self) {
        let shutdown_sender = self.shutdown_sender.clone();
        let is_shutting_down = self.is_shutting_down.clone();

        tokio::spawn(async move {
            #[cfg(unix)]
            {
                let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate())
                    .expect("Failed to register SIGTERM handler");
                let mut sigint = signal::unix::signal(signal::unix::SignalKind::interrupt())
                    .expect("Failed to register SIGINT handler");

                tokio::select! {
                    _ = sigterm.recv() => {
                        info!("Received SIGTERM, initiating graceful shutdown");
                        is_shutting_down.store(true, Ordering::Release);
                        let _ = shutdown_sender.send(ShutdownReason::Graceful);
                    }
                    _ = sigint.recv() => {
                        info!("Received SIGINT, initiating immediate shutdown");
                        is_shutting_down.store(true, Ordering::Release);
                        let _ = shutdown_sender.send(ShutdownReason::Immediate);
                    }
                }
            }

            #[cfg(not(unix))]
            {
                // For non-Unix systems (like Windows), only handle Ctrl+C
                match signal::ctrl_c().await {
                    Ok(()) => {
                        info!("Received Ctrl+C, initiating graceful shutdown");
                        is_shutting_down.store(true, Ordering::Release);
                        let _ = shutdown_sender.send(ShutdownReason::Graceful);
                    }
                    Err(err) => {
                        error!("Failed to listen for Ctrl+C: {}", err);
                    }
                }
            }
        });
    }

    /// Initiate shutdown programmatically
    pub fn shutdown(&self, reason: ShutdownReason) {
        if self
            .is_shutting_down
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            info!("Initiating shutdown: {:?}", reason);
            let _ = self.shutdown_sender.send(reason);
        } else {
            debug!("Shutdown already in progress");
        }
    }

    /// Wait for shutdown to complete
    pub async fn wait_for_shutdown(&self) {
        self.shutdown_complete.notified().await;
    }

    /// Execute the shutdown procedure
    pub async fn execute_shutdown(
        &mut self,
        reason: ShutdownReason,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Executing shutdown procedure: {:?}", reason);

        let shutdown_timeout = match reason {
            ShutdownReason::Immediate => Duration::from_secs(5),
            _ => self.config.graceful_timeout,
        };

        let shutdown_result =
            timeout(shutdown_timeout, self.perform_shutdown(reason.clone())).await;

        match shutdown_result {
            Ok(Ok(())) => {
                info!("Shutdown completed successfully");
                self.shutdown_complete.notify_waiters();
                Ok(())
            }
            Ok(Err(e)) => {
                error!("Shutdown completed with errors: {}", e);
                self.shutdown_complete.notify_waiters();
                Err(e)
            }
            Err(_) => {
                error!("Shutdown timed out after {:?}", shutdown_timeout);
                if self.config.force_after_timeout {
                    warn!("Forcing shutdown due to timeout");
                    self.force_shutdown().await;
                }
                self.shutdown_complete.notify_waiters();
                Err("Shutdown timeout".into())
            }
        }
    }

    /// Perform the actual shutdown steps
    async fn perform_shutdown(
        &mut self,
        reason: ShutdownReason,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Step 1: Sort handlers by priority
        self.cleanup_handlers.sort_by_key(|h| h.priority());

        // Step 2: Filter handlers based on shutdown type
        let handlers: Vec<_> = match reason {
            ShutdownReason::Immediate => self
                .cleanup_handlers
                .iter()
                .filter(|h| !h.requires_graceful())
                .collect(),
            _ => self.cleanup_handlers.iter().collect(),
        };

        info!("Shutting down {} components", handlers.len());

        // Step 3: Shutdown components in priority order
        let mut errors = Vec::new();
        for handler in handlers {
            debug!("Shutting down component: {}", handler.name());

            let handler_timeout = match reason {
                ShutdownReason::Immediate => Duration::from_secs(2),
                _ => self.config.cleanup_timeout,
            };

            match timeout(handler_timeout, handler.shutdown()).await {
                Ok(Ok(())) => {
                    debug!("Successfully shut down: {}", handler.name());
                }
                Ok(Err(e)) => {
                    error!("Error shutting down {}: {}", handler.name(), e);
                    errors.push(format!("{}: {}", handler.name(), e));
                }
                Err(_) => {
                    error!("Timeout shutting down: {}", handler.name());
                    errors.push(format!("{}: timeout", handler.name()));
                }
            }
        }

        // Step 4: Final cleanup
        if self.config.flush_telemetry {
            self.flush_telemetry().await;
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(format!("Shutdown errors: {}", errors.join(", ")).into())
        }
    }

    /// Force immediate shutdown
    async fn force_shutdown(&self) {
        warn!("Performing forced shutdown");

        // Minimal cleanup for forced shutdown
        if self.config.flush_telemetry {
            if let Err(e) = timeout(Duration::from_secs(2), self.flush_telemetry()).await {
                error!("Failed to flush telemetry during forced shutdown: {:?}", e);
            }
        }
    }

    /// Flush telemetry data
    async fn flush_telemetry(&self) {
        debug!("Flushing telemetry data");

        
        {
            use opentelemetry::global;
            global::shutdown_tracer_provider();
            debug!("Telemetry flushed successfully");
        }
    }
}

/// Shutdown guard that ensures cleanup on drop
pub struct ShutdownGuard {
    shutdown_manager: Arc<tokio::sync::Mutex<ShutdownManager>>,
    _handle: tokio::task::JoinHandle<()>,
}

impl ShutdownGuard {
    /// Create a new shutdown guard that listens for signals
    pub async fn new(shutdown_manager: ShutdownManager) -> Self {
        shutdown_manager.listen_for_signals().await;

        let shutdown_manager = Arc::new(tokio::sync::Mutex::new(shutdown_manager));
        let shutdown_manager_clone = shutdown_manager.clone();

        let handle = tokio::spawn(async move {
            let mut receiver = {
                let manager = shutdown_manager_clone.lock().await;
                manager.subscribe()
            };

            #[allow(clippy::never_loop)]
            while let Ok(reason) = receiver.recv().await {
                let mut manager = shutdown_manager_clone.lock().await;
                if let Err(e) = manager.execute_shutdown(reason).await {
                    error!("Shutdown execution failed: {}", e);
                }
                break;
            }
        });

        Self {
            shutdown_manager,
            _handle: handle,
        }
    }

    /// Get a reference to the shutdown manager
    pub async fn manager(&self) -> tokio::sync::MutexGuard<'_, ShutdownManager> {
        self.shutdown_manager.lock().await
    }

    /// Initiate shutdown
    pub async fn shutdown(&self, reason: ShutdownReason) {
        let manager = self.shutdown_manager.lock().await;
        manager.shutdown(reason);
    }

    /// Wait for shutdown to complete
    pub async fn wait_for_shutdown(&self) {
        let manager = self.shutdown_manager.lock().await;
        manager.wait_for_shutdown().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use tokio::time::{sleep, Duration};

    struct TestHandler {
        name: String,
        shutdown_counter: Arc<AtomicU32>,
        shutdown_delay: Duration,
        should_fail: bool,
    }

    impl TestHandler {
        fn new(name: &str, counter: Arc<AtomicU32>) -> Self {
            Self {
                name: name.to_string(),
                shutdown_counter: counter,
                shutdown_delay: Duration::from_millis(10),
                should_fail: false,
            }
        }

        fn with_delay(mut self, delay: Duration) -> Self {
            self.shutdown_delay = delay;
            self
        }

        fn with_failure(mut self) -> Self {
            self.should_fail = true;
            self
        }
    }

    #[async_trait::async_trait]
    impl ShutdownHandler for TestHandler {
        fn name(&self) -> &str {
            &self.name
        }

        async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            sleep(self.shutdown_delay).await;

            if self.should_fail {
                return Err("Test failure".into());
            }

            self.shutdown_counter.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_shutdown_manager_creation() {
        let config = ShutdownConfig::default();
        let manager = ShutdownManager::new(config);

        assert!(!manager.is_shutting_down());
    }

    #[tokio::test]
    async fn test_graceful_shutdown() {
        let counter = Arc::new(AtomicU32::new(0));
        let mut manager = ShutdownManager::default();

        manager.register_handler(Box::new(TestHandler::new("test1", counter.clone())));
        manager.register_handler(Box::new(TestHandler::new("test2", counter.clone())));

        let result = manager.execute_shutdown(ShutdownReason::Graceful).await;

        assert!(result.is_ok());
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_shutdown_with_errors() {
        let counter = Arc::new(AtomicU32::new(0));
        let mut manager = ShutdownManager::default();

        manager.register_handler(Box::new(TestHandler::new("test1", counter.clone())));
        manager.register_handler(Box::new(
            TestHandler::new("test2", counter.clone()).with_failure(),
        ));

        let result = manager.execute_shutdown(ShutdownReason::Graceful).await;

        assert!(result.is_err());
        assert_eq!(counter.load(Ordering::SeqCst), 1); // Only successful handler incremented
    }

    #[tokio::test]
    async fn test_shutdown_timeout() {
        let counter = Arc::new(AtomicU32::new(0));
        let mut config = ShutdownConfig::default();
        config.graceful_timeout = Duration::from_millis(50);
        config.cleanup_timeout = Duration::from_millis(10);

        let mut manager = ShutdownManager::new(config);

        manager.register_handler(Box::new(
            TestHandler::new("slow", counter.clone()).with_delay(Duration::from_millis(100)),
        ));

        let result = manager.execute_shutdown(ShutdownReason::Graceful).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_immediate_shutdown() {
        let counter = Arc::new(AtomicU32::new(0));
        let mut manager = ShutdownManager::default();

        // Create a handler that doesn't require graceful shutdown
        struct ImmediateHandler {
            name: String,
            counter: Arc<AtomicU32>,
        }

        impl ImmediateHandler {
            fn new(name: &str, counter: Arc<AtomicU32>) -> Self {
                Self {
                    name: name.to_string(),
                    counter,
                }
            }
        }

        #[async_trait::async_trait]
        impl ShutdownHandler for ImmediateHandler {
            fn name(&self) -> &str {
                &self.name
            }

            async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
                self.counter.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }

            fn requires_graceful(&self) -> bool {
                false // This handler can be shut down immediately
            }
        }

        manager.register_handler(Box::new(ImmediateHandler::new(
            "immediate",
            counter.clone(),
        )));

        let result = manager.execute_shutdown(ShutdownReason::Immediate).await;

        assert!(result.is_ok());
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }
}
