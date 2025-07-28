//! OTLP metrics exporter implementation for sending metrics to telemetry backends

use opentelemetry::{
    global,
    metrics::{Meter, MeterProvider},
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    metrics::{
        reader::{DefaultAggregationSelector, DefaultTemporalitySelector},
        PeriodicReader, SdkMeterProvider,
    },
    Resource,
};
use std::time::Duration;
use crate::telemetry::TelemetryConfig;

/// OTLP metrics exporter configuration
#[derive(Debug, Clone)]
pub struct MetricsExporterConfig {
    /// OTLP endpoint for metrics
    pub endpoint: String,
    /// Export interval for batching metrics
    pub export_interval: Duration,
    /// Export timeout for individual requests
    pub export_timeout: Duration,
    /// Maximum batch size for metrics
    pub max_batch_size: usize,
    /// Enable compression for exports
    pub compression: bool,
}

impl Default for MetricsExporterConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:4318/v1/metrics".to_string(),
            export_interval: Duration::from_secs(10),
            export_timeout: Duration::from_secs(30),
            max_batch_size: 1000,
            compression: true,
        }
    }
}

impl From<&TelemetryConfig> for MetricsExporterConfig {
    fn from(config: &TelemetryConfig) -> Self {
        let endpoint = config
            .otlp_endpoint
            .clone()
            .unwrap_or_else(|| "http://localhost:4318".to_string());
        
        // Ensure endpoint has metrics path
        let metrics_endpoint = if endpoint.ends_with("/v1/metrics") {
            endpoint
        } else if endpoint.ends_with('/') {
            format!("{}v1/metrics", endpoint)
        } else {
            format!("{}/v1/metrics", endpoint)
        };

        Self {
            endpoint: metrics_endpoint,
            export_interval: Duration::from_secs(10),
            export_timeout: Duration::from_secs(30),
            max_batch_size: 1000,
            compression: true,
        }
    }
}

/// OpenTelemetry OTLP metrics exporter
pub struct OtlpMetricsExporter {
    meter_provider: SdkMeterProvider,
    meter: Meter,
    _config: MetricsExporterConfig,
}

impl OtlpMetricsExporter {
    /// Initialize a new OTLP metrics exporter with smart protocol detection
    pub fn init(
        config: MetricsExporterConfig,
        resource: Resource,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        tracing::debug!("Initializing OTLP metrics exporter with endpoint: {}", config.endpoint);

        // Smart protocol detection based on endpoint
        let use_http = Self::should_use_http_protocol(&config.endpoint);
        
        let exporter = if use_http {
            tracing::debug!("Using HTTP protocol for OTLP metrics export");
            // Try HTTP first - this should work for port 4320 
            match Self::create_http_exporter(&config) {
                Ok(exp) => exp,
                Err(e) => {
                    tracing::warn!("HTTP exporter failed ({}), trying GRPC fallback", e);
                    Self::create_grpc_exporter(&config)?
                }
            }
        } else {
            tracing::debug!("Using GRPC protocol for OTLP metrics export");
            // Try GRPC first for standard ports
            match Self::create_grpc_exporter(&config) {
                Ok(exp) => exp,
                Err(e) => {
                    tracing::warn!("GRPC exporter failed ({}), trying HTTP fallback", e);
                    Self::create_http_exporter(&config)?
                }
            }
        };

        // Create periodic reader with the exporter
        let reader = PeriodicReader::builder(
            exporter,
            opentelemetry_sdk::runtime::Tokio,
        )
        .with_interval(config.export_interval)
        .with_timeout(config.export_timeout)
        .build();

        // Create meter provider with reader
        let meter_provider = SdkMeterProvider::builder()
            .with_resource(resource)
            .with_reader(reader)
            .build();

        // Get meter from provider
        let meter = meter_provider.meter("stood-agent");

        // Set as global meter provider
        global::set_meter_provider(meter_provider.clone());

        tracing::info!("OTLP metrics exporter initialized successfully");

        Ok(Self {
            meter_provider,
            meter,
            _config: config,
        })
    }

    /// Get the meter for creating metric instruments
    pub fn meter(&self) -> &Meter {
        &self.meter
    }

    /// Get the meter provider
    pub fn meter_provider(&self) -> &SdkMeterProvider {
        &self.meter_provider
    }

    /// Flush all pending metrics exports
    pub async fn flush(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Force export of all pending metrics
        self.meter_provider.force_flush()?;
        Ok(())
    }

    /// Shutdown the metrics exporter and flush remaining data
    pub async fn shutdown(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        tracing::info!("Shutting down OTLP metrics exporter");
        
        // Flush remaining metrics
        self.meter_provider.force_flush()?;
        
        // Shutdown the meter provider
        self.meter_provider.shutdown()?;
        
        Ok(())
    }

    /// Smart protocol detection based on endpoint characteristics
    fn should_use_http_protocol(endpoint: &str) -> bool {
        // Port-based detection (common patterns)
        if endpoint.contains(":4318") || endpoint.contains(":4320") || endpoint.contains(":8080") {
            return true; // These ports typically use HTTP
        }
        
        if endpoint.contains(":4317") || endpoint.contains(":4319") {
            return false; // These ports typically use GRPC
        }
        
        // Cloud provider detection
        if endpoint.contains("honeycomb.io") || 
           endpoint.contains("amazonaws.com") ||
           endpoint.contains("newrelic.com") ||
           endpoint.contains("datadoghq.com") {
            return true; // Most cloud providers prefer HTTP
        }
        
        // HTTPS endpoints typically prefer HTTP protocol  
        if endpoint.starts_with("https://") {
            return true;
        }
        
        // Default to HTTP for broader compatibility
        true
    }

    /// Create HTTP-based OTLP exporter
    fn create_http_exporter(
        config: &MetricsExporterConfig,
    ) -> Result<opentelemetry_otlp::MetricsExporter, Box<dyn std::error::Error + Send + Sync>> {
        // For HTTP, we need to ensure the endpoint has the correct path
        let http_endpoint = if config.endpoint.ends_with("/v1/metrics") {
            config.endpoint.clone()
        } else if config.endpoint.ends_with("/") {
            format!("{}v1/metrics", config.endpoint)
        } else {
            format!("{}/v1/metrics", config.endpoint)
        };

        tracing::debug!("Creating HTTP OTLP exporter for: {}", http_endpoint);
        
        // Use HTTP protocol with reqwest client
        let exporter = opentelemetry_otlp::new_exporter()
            .http()
            .with_endpoint(&http_endpoint)
            .with_timeout(config.export_timeout)
            .build_metrics_exporter(
                Box::new(DefaultAggregationSelector::new()),
                Box::new(DefaultTemporalitySelector::new()),
            )?;
            
        Ok(exporter)
    }

    /// Create GRPC-based OTLP exporter
    fn create_grpc_exporter(
        config: &MetricsExporterConfig,
    ) -> Result<opentelemetry_otlp::MetricsExporter, Box<dyn std::error::Error + Send + Sync>> {
        tracing::debug!("Creating GRPC OTLP exporter for: {}", config.endpoint);
        
        let exporter = opentelemetry_otlp::new_exporter()
            .tonic()
            .with_endpoint(&config.endpoint)
            .with_timeout(config.export_timeout)
            .build_metrics_exporter(
                Box::new(DefaultAggregationSelector::new()),
                Box::new(DefaultTemporalitySelector::new()),
            )?;
            
        Ok(exporter)
    }
}

/// Create a metrics exporter from telemetry configuration
pub fn create_metrics_exporter(
    telemetry_config: &TelemetryConfig,
) -> Result<Option<OtlpMetricsExporter>, Box<dyn std::error::Error + Send + Sync>> {
    if !telemetry_config.enabled {
        tracing::debug!("Telemetry disabled, skipping metrics exporter creation");
        return Ok(None);
    }

    let metrics_config = MetricsExporterConfig::from(telemetry_config);
    
    // Create resource with service information
    let resource = Resource::new(vec![
        KeyValue::new("service.name", telemetry_config.service_name.clone()),
        KeyValue::new("service.version", telemetry_config.service_version.clone()),
        KeyValue::new("telemetry.sdk.name", "stood-agent"),
        KeyValue::new("telemetry.sdk.language", "rust"),
    ]);

    // Add custom service attributes
    let mut resource_attributes = vec![];
    for (key, value) in &telemetry_config.service_attributes {
        resource_attributes.push(KeyValue::new(key.clone(), value.clone()));
    }
    
    if !resource_attributes.is_empty() {
        let custom_resource = Resource::new(resource_attributes);
        let combined_resource = resource.merge(&custom_resource);
        let exporter = OtlpMetricsExporter::init(metrics_config, combined_resource)?;
        Ok(Some(exporter))
    } else {
        let exporter = OtlpMetricsExporter::init(metrics_config, resource)?;
        Ok(Some(exporter))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telemetry::TelemetryConfig;

    #[test]
    fn test_metrics_exporter_config_from_telemetry_config() {
        let telemetry_config = TelemetryConfig {
            enabled: true,
            otlp_endpoint: Some("http://localhost:4318".to_string()),
            ..Default::default()
        };

        let metrics_config = MetricsExporterConfig::from(&telemetry_config);
        assert_eq!(metrics_config.endpoint, "http://localhost:4318/v1/metrics");
    }

    #[test]
    fn test_metrics_exporter_config_with_existing_path() {
        let telemetry_config = TelemetryConfig {
            enabled: true,
            otlp_endpoint: Some("http://localhost:4318/v1/metrics".to_string()),
            ..Default::default()
        };

        let metrics_config = MetricsExporterConfig::from(&telemetry_config);
        assert_eq!(metrics_config.endpoint, "http://localhost:4318/v1/metrics");
    }
}