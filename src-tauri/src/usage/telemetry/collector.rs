//! OTLP HTTP collector for receiving telemetry data

use std::net::SocketAddr;

use axum::{
    Router,
    routing::post,
    extract::State,
    http::{StatusCode, HeaderMap},
    body::Bytes,
    response::IntoResponse,
};
use log::{info, warn, debug, error};
use tokio::sync::oneshot;
use tower_http::cors::{CorsLayer, Any};

use super::models::{
    ExportMetricsServiceRequest, ExportLogsServiceRequest,
    ParsedMetric, ParsedEvent,
};
use super::storage::TelemetryStorage;

/// Default collector port (OTLP HTTP standard)
pub const DEFAULT_COLLECTOR_PORT: u16 = 4318;

/// Telemetry collector state
#[derive(Clone)]
struct CollectorState {
    storage: TelemetryStorage,
}

/// OTLP HTTP collector
pub struct TelemetryCollector {
    port: u16,
    shutdown_tx: Option<oneshot::Sender<()>>,
    storage: TelemetryStorage,
}

impl TelemetryCollector {
    /// Create a new collector
    pub fn new(port: Option<u16>, data_dir: Option<&str>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let port = port.unwrap_or_else(|| {
            std::env::var("CCM_COLLECTOR_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(DEFAULT_COLLECTOR_PORT)
        });

        let storage = TelemetryStorage::new(data_dir)?;

        Ok(Self {
            port,
            shutdown_tx: None,
            storage,
        })
    }

    /// Get the port the collector is running on
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Get a clone of the storage for reading data
    pub fn storage(&self) -> TelemetryStorage {
        self.storage.clone()
    }

    /// Start the collector server
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let state = CollectorState {
            storage: self.storage.clone(),
        };

        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        let app = Router::new()
            .route("/v1/metrics", post(handle_metrics))
            .route("/v1/logs", post(handle_logs))
            .route("/health", axum::routing::get(health_check))
            .layer(cors)
            .with_state(state);

        let addr = SocketAddr::from(([127, 0, 0, 1], self.port));

        info!("Starting telemetry collector on {}", addr);

        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        self.shutdown_tx = Some(shutdown_tx);

        let listener = tokio::net::TcpListener::bind(addr).await?;

        tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = shutdown_rx.await;
                    info!("Telemetry collector shutting down");
                })
                .await
                .ok();
        });

        Ok(())
    }

    /// Stop the collector server
    pub fn stop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }

    /// Check if the collector is running
    pub fn is_running(&self) -> bool {
        self.shutdown_tx.is_some()
    }
}

/// Health check endpoint
async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

/// Handle incoming metrics data
async fn handle_metrics(
    State(state): State<CollectorState>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    debug!("Received metrics request, {} bytes", body.len());

    // Determine content type and decode accordingly
    let content_type = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let json_body = if content_type.contains("protobuf") {
        // For protobuf, we'd need to decode it - for now, return error
        warn!("Protobuf format not yet supported, please use http/json");
        return (StatusCode::UNSUPPORTED_MEDIA_TYPE, "Use http/json format");
    } else {
        // Check if body is gzip compressed
        let encoding = headers
            .get("content-encoding")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        if encoding.contains("gzip") {
            match decompress_gzip(&body) {
                Ok(decompressed) => decompressed,
                Err(e) => {
                    warn!("Failed to decompress gzip: {}", e);
                    return (StatusCode::BAD_REQUEST, "Failed to decompress");
                }
            }
        } else {
            body.to_vec()
        }
    };

    // Parse JSON
    let request: ExportMetricsServiceRequest = match serde_json::from_slice(&json_body) {
        Ok(req) => req,
        Err(e) => {
            warn!("Failed to parse metrics JSON: {}", e);
            debug!("Body: {}", String::from_utf8_lossy(&json_body));
            return (StatusCode::BAD_REQUEST, "Invalid JSON");
        }
    };

    // Extract and store metrics
    let metrics = extract_metrics(&request);
    if !metrics.is_empty() {
        match state.storage.store_metrics(&metrics) {
            Ok(count) => {
                debug!("Stored {} metrics", count);
            }
            Err(e) => {
                error!("Failed to store metrics: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, "Storage error");
            }
        }
    }

    (StatusCode::OK, "")
}

/// Handle incoming logs/events data
async fn handle_logs(
    State(state): State<CollectorState>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    debug!("Received logs request, {} bytes", body.len());

    let content_type = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let json_body = if content_type.contains("protobuf") {
        warn!("Protobuf format not yet supported, please use http/json");
        return (StatusCode::UNSUPPORTED_MEDIA_TYPE, "Use http/json format");
    } else {
        let encoding = headers
            .get("content-encoding")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        if encoding.contains("gzip") {
            match decompress_gzip(&body) {
                Ok(decompressed) => decompressed,
                Err(e) => {
                    warn!("Failed to decompress gzip: {}", e);
                    return (StatusCode::BAD_REQUEST, "Failed to decompress");
                }
            }
        } else {
            body.to_vec()
        }
    };

    // Parse JSON
    let request: ExportLogsServiceRequest = match serde_json::from_slice(&json_body) {
        Ok(req) => req,
        Err(e) => {
            warn!("Failed to parse logs JSON: {}", e);
            debug!("Body: {}", String::from_utf8_lossy(&json_body));
            return (StatusCode::BAD_REQUEST, "Invalid JSON");
        }
    };

    // Extract and store events
    let events = extract_events(&request);
    if !events.is_empty() {
        match state.storage.store_events(&events) {
            Ok(count) => {
                debug!("Stored {} events", count);
            }
            Err(e) => {
                error!("Failed to store events: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, "Storage error");
            }
        }
    }

    (StatusCode::OK, "")
}

/// Decompress gzip data
fn decompress_gzip(data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    use flate2::read::GzDecoder;
    use std::io::Read;

    let mut decoder = GzDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    Ok(decompressed)
}

/// Extract metrics from OTLP request
fn extract_metrics(request: &ExportMetricsServiceRequest) -> Vec<ParsedMetric> {
    let mut metrics = Vec::new();

    if let Some(resource_metrics) = &request.resource_metrics {
        for rm in resource_metrics {
            // Extract resource attributes for context
            let mut resource_attrs = std::collections::HashMap::new();
            if let Some(resource) = &rm.resource {
                if let Some(attrs) = &resource.attributes {
                    for kv in attrs {
                        if let (Some(key), Some(value)) = (&kv.key, kv.get_string_value()) {
                            resource_attrs.insert(key.clone(), value);
                        }
                    }
                }
            }

            if let Some(scope_metrics) = &rm.scope_metrics {
                for sm in scope_metrics {
                    if let Some(metric_list) = &sm.metrics {
                        for metric in metric_list {
                            let name = metric.name.clone().unwrap_or_default();

                            // Only process claude_code metrics
                            if !name.starts_with("claude_code.") {
                                continue;
                            }

                            // Extract data points from sum or gauge
                            let data_points = metric.sum
                                .as_ref()
                                .and_then(|s| s.data_points.as_ref())
                                .or_else(|| metric.gauge.as_ref().and_then(|g| g.data_points.as_ref()));

                            if let Some(points) = data_points {
                                for point in points {
                                    let mut attrs = resource_attrs.clone();
                                    attrs.extend(point.get_attributes());

                                    metrics.push(ParsedMetric {
                                        name: name.clone(),
                                        timestamp_ns: point.get_timestamp_ns(),
                                        value: point.get_value(),
                                        attributes: attrs,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    metrics
}

/// Extract events from OTLP logs request
fn extract_events(request: &ExportLogsServiceRequest) -> Vec<ParsedEvent> {
    let mut events = Vec::new();

    if let Some(resource_logs) = &request.resource_logs {
        for rl in resource_logs {
            // Extract resource attributes
            let mut resource_attrs = std::collections::HashMap::new();
            if let Some(resource) = &rl.resource {
                if let Some(attrs) = &resource.attributes {
                    for kv in attrs {
                        if let (Some(key), Some(value)) = (&kv.key, kv.get_string_value()) {
                            resource_attrs.insert(key.clone(), value);
                        }
                    }
                }
            }

            if let Some(scope_logs) = &rl.scope_logs {
                for sl in scope_logs {
                    if let Some(log_records) = &sl.log_records {
                        for record in log_records {
                            if let Some(event_name) = record.get_event_name() {
                                // Only process claude_code events
                                if !event_name.starts_with("claude_code.") {
                                    continue;
                                }

                                let mut attrs = resource_attrs.clone();
                                attrs.extend(record.get_attributes());

                                events.push(ParsedEvent {
                                    name: event_name,
                                    timestamp_ns: record.get_timestamp_ns(),
                                    attributes: attrs,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    events
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_metrics() {
        let json = r#"{
            "resourceMetrics": [{
                "resource": {
                    "attributes": [
                        {"key": "service.name", "value": {"stringValue": "claude-code"}}
                    ]
                },
                "scopeMetrics": [{
                    "metrics": [{
                        "name": "claude_code.token.usage",
                        "sum": {
                            "dataPoints": [{
                                "timeUnixNano": "1700000000000000000",
                                "asInt": "1000",
                                "attributes": [
                                    {"key": "type", "value": {"stringValue": "input"}}
                                ]
                            }]
                        }
                    }]
                }]
            }]
        }"#;

        let request: ExportMetricsServiceRequest = serde_json::from_str(json).unwrap();
        let metrics = extract_metrics(&request);

        assert_eq!(metrics.len(), 1);
        assert_eq!(metrics[0].name, "claude_code.token.usage");
        assert_eq!(metrics[0].value, 1000.0);
        assert_eq!(metrics[0].attributes.get("type"), Some(&"input".to_string()));
    }
}
