//! HTTP Server for Health Check Endpoints
//!
//! This module provides HTTP endpoints for health checks, compatible with Kubernetes
//! and container orchestration platforms.

#[cfg(feature = "http")]
use {
    super::{HealthChecker, HealthStatus},
    axum::{
        extract::State,
        http::StatusCode,
        response::Json,
        routing::get,
        Router,
    },
    serde_json::{json, Value},
    std::sync::Arc,
    tokio::sync::Mutex,
    tower::ServiceBuilder,
    tower_http::cors::CorsLayer,
    tracing::{info, warn},
};

#[cfg(feature = "http")]
/// HTTP server for health check endpoints
pub struct HealthHttpServer {
    health_checker: Arc<Mutex<HealthChecker>>,
    config: super::HttpConfig,
}

#[cfg(feature = "http")]
impl HealthHttpServer {
    /// Create a new HTTP server with the given health checker and configuration
    pub fn new(health_checker: HealthChecker, config: super::HttpConfig) -> Self {
        Self {
            health_checker: Arc::new(Mutex::new(health_checker)),
            config,
        }
    }

    /// Start the HTTP server
    pub async fn start(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let app = self.create_router();
        let bind_addr = format!("{}:{}", self.config.host, self.config.port);

        info!("Starting health check HTTP server on {}", bind_addr);

        let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
        
        axum::serve(listener, app.into_make_service()).await?;

        Ok(())
    }

    /// Create the router with all health check endpoints
    fn create_router(&self) -> Router<Arc<Mutex<HealthChecker>>> {
        let state = self.health_checker.clone();
        
        let mut router = Router::new()
            .route("/health", get(health_handler))
            .with_state(state);

        if self.config.enable_liveness {
            router = router.route("/health/live", get(liveness_handler));
        }

        if self.config.enable_readiness {
            router = router.route("/health/ready", get(readiness_handler));
        }

        if self.config.enable_metrics {
            router = router.route("/metrics", get(metrics_handler));
        }

        // Add CORS and other middleware
        router.layer(
            ServiceBuilder::new()
                .layer(CorsLayer::permissive())
        )
    }

    /// Start the server in the background and return a handle
    pub async fn start_background(self) -> tokio::task::JoinHandle<Result<(), Box<dyn std::error::Error + Send + Sync>>> {
        tokio::spawn(async move {
            self.start().await
        })
    }
}

#[cfg(feature = "http")]
/// Handler for the main health endpoint
async fn health_handler(
    State(health_checker): State<Arc<Mutex<HealthChecker>>>,
) -> Result<Json<Value>, StatusCode> {
    let mut checker = match health_checker.try_lock() {
        Ok(checker) => checker,
        Err(_) => {
            warn!("Health checker is busy, returning cached results");
            let checker = health_checker.lock().await;
            let summary = checker.last_health_summary();
            drop(checker);
            return Ok(Json(json!({
                "status": format!("{:?}", summary.status).to_lowercase(),
                "timestamp": summary.timestamp,
                "checks": summary.checks,
                "note": "Using cached results - health checker was busy"
            })));
        }
    };

    let summary = checker.check_health().await;
    let status_code = match summary.status {
        HealthStatus::Healthy => StatusCode::OK,
        HealthStatus::Degraded => StatusCode::OK, // Still serving traffic but with warnings
        HealthStatus::Unhealthy => StatusCode::SERVICE_UNAVAILABLE,
    };

    let response = json!({
        "status": format!("{:?}", summary.status).to_lowercase(),
        "timestamp": summary.timestamp,
        "total_duration_ms": summary.total_duration.as_millis(),
        "checks": summary.checks
    });

    if status_code == StatusCode::OK {
        Ok(Json(response))
    } else {
        Err(status_code)
    }
}

#[cfg(feature = "http")]
/// Handler for the liveness probe endpoint
async fn liveness_handler(
    State(health_checker): State<Arc<Mutex<HealthChecker>>>,
) -> Result<Json<Value>, StatusCode> {
    let checker = health_checker.lock().await;
    let is_alive = checker.is_alive().await;
    
    if is_alive {
        Ok(Json(json!({
            "status": "alive",
            "timestamp": chrono::Utc::now()
        })))
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

#[cfg(feature = "http")]
/// Handler for the readiness probe endpoint
async fn readiness_handler(
    State(health_checker): State<Arc<Mutex<HealthChecker>>>,
) -> Result<Json<Value>, StatusCode> {
    let mut checker = health_checker.lock().await;
    let is_ready = checker.is_ready().await;
    
    if is_ready {
        Ok(Json(json!({
            "status": "ready",
            "timestamp": chrono::Utc::now()
        })))
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

#[cfg(feature = "http")]
/// Handler for Prometheus-compatible metrics endpoint
async fn metrics_handler(
    State(health_checker): State<Arc<Mutex<HealthChecker>>>,
) -> Result<String, StatusCode> {
    let checker = health_checker.lock().await;
    let summary = checker.last_health_summary();
    
    let mut metrics = String::new();
    
    // Overall health status metric
    let overall_status_value = match summary.status {
        HealthStatus::Healthy => 1,
        HealthStatus::Degraded => 0,
        HealthStatus::Unhealthy => -1,
    };
    
    metrics.push_str(&format!(
        "# HELP stood_health_status Overall health status (-1=unhealthy, 0=degraded, 1=healthy)\n"
    ));
    metrics.push_str(&format!(
        "# TYPE stood_health_status gauge\n"
    ));
    metrics.push_str(&format!(
        "stood_health_status {}\n"
    , overall_status_value));
    
    // Individual check metrics
    metrics.push_str(&format!(
        "# HELP stood_health_check_status Individual health check status (0=unhealthy, 1=healthy)\n"
    ));
    metrics.push_str(&format!(
        "# TYPE stood_health_check_status gauge\n"
    ));
    
    for (name, result) in &summary.checks {
        let status_value = match result.status {
            HealthStatus::Healthy => 1,
            HealthStatus::Degraded => 1, // Still consider as healthy for binary metrics
            HealthStatus::Unhealthy => 0,
        };
        
        metrics.push_str(&format!(
            "stood_health_check_status{{check=\"{}\"}} {}\n",
            name, status_value
        ));
    }
    
    // Check duration metrics
    metrics.push_str(&format!(
        "# HELP stood_health_check_duration_seconds Duration of health checks in seconds\n"
    ));
    metrics.push_str(&format!(
        "# TYPE stood_health_check_duration_seconds gauge\n"
    ));
    
    for (name, result) in &summary.checks {
        metrics.push_str(&format!(
            "stood_health_check_duration_seconds{{check=\"{}\"}} {:.6}\n",
            name, result.duration.as_secs_f64()
        ));
    }
    
    Ok(metrics)
}

#[cfg(not(feature = "http"))]
/// Stub implementation when HTTP feature is disabled
pub struct HealthHttpServer;

#[cfg(not(feature = "http"))]
impl HealthHttpServer {
    pub fn new(_health_checker: super::HealthChecker, _config: super::HttpConfig) -> Self {
        Self
    }

    pub async fn start(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Err("HTTP feature is not enabled. Enable with --features http".into())
    }

    pub async fn start_background(self) -> tokio::task::JoinHandle<Result<(), Box<dyn std::error::Error + Send + Sync>>> {
        tokio::spawn(async move {
            Err("HTTP feature is not enabled. Enable with --features http".into())
        })
    }
}

#[cfg(all(test, feature = "http"))]
mod tests {
    use super::*;
    use crate::config::StoodConfig;

    #[tokio::test]
    async fn test_health_http_server_creation() {
        let config = StoodConfig::default();
        let health_checker = HealthChecker::from_config(&config);
        let http_config = super::super::HttpConfig::default();
        
        let _server = HealthHttpServer::new(health_checker, http_config);
        
        // Just verify we can create the server
        // Actually starting it would require binding to a port
        assert!(true);
    }
}

#[cfg(all(test, not(feature = "http")))]
mod tests {
    use super::*;
    use crate::config::StoodConfig;

    #[tokio::test]
    async fn test_health_http_server_stub() {
        let config = StoodConfig::default();
        let health_checker = HealthChecker::from_config(&config);
        let http_config = super::super::HttpConfig::default();
        
        let server = HealthHttpServer::new(health_checker, http_config);
        
        // Should return error when HTTP feature is disabled
        let result = server.start().await;
        assert!(result.is_err());
    }
}