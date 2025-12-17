//! Data models for OpenTelemetry telemetry data

use serde::{Deserialize, Serialize};

/// OTLP ExportMetricsServiceRequest (JSON format)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportMetricsServiceRequest {
    pub resource_metrics: Option<Vec<ResourceMetrics>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceMetrics {
    pub resource: Option<Resource>,
    pub scope_metrics: Option<Vec<ScopeMetrics>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    pub attributes: Option<Vec<KeyValue>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScopeMetrics {
    pub scope: Option<InstrumentationScope>,
    pub metrics: Option<Vec<Metric>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstrumentationScope {
    pub name: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metric {
    pub name: Option<String>,
    pub description: Option<String>,
    pub unit: Option<String>,
    pub sum: Option<Sum>,
    pub gauge: Option<Gauge>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sum {
    pub data_points: Option<Vec<NumberDataPoint>>,
    pub aggregation_temporality: Option<i32>,
    pub is_monotonic: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Gauge {
    pub data_points: Option<Vec<NumberDataPoint>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NumberDataPoint {
    pub attributes: Option<Vec<KeyValue>>,
    pub start_time_unix_nano: Option<String>,
    pub time_unix_nano: Option<String>,
    pub as_double: Option<f64>,
    pub as_int: Option<String>,  // OTLP uses string for int64
}

/// OTLP ExportLogsServiceRequest (JSON format)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportLogsServiceRequest {
    pub resource_logs: Option<Vec<ResourceLogs>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceLogs {
    pub resource: Option<Resource>,
    pub scope_logs: Option<Vec<ScopeLogs>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScopeLogs {
    pub scope: Option<InstrumentationScope>,
    pub log_records: Option<Vec<LogRecord>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogRecord {
    pub time_unix_nano: Option<String>,
    pub observed_time_unix_nano: Option<String>,
    pub severity_number: Option<i32>,
    pub severity_text: Option<String>,
    pub body: Option<AnyValue>,
    pub attributes: Option<Vec<KeyValue>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyValue {
    pub key: Option<String>,
    pub value: Option<AnyValue>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnyValue {
    pub string_value: Option<String>,
    pub bool_value: Option<bool>,
    pub int_value: Option<String>,  // OTLP uses string for int64
    pub double_value: Option<f64>,
    pub array_value: Option<ArrayValue>,
    pub kvlist_value: Option<KvlistValue>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArrayValue {
    pub values: Option<Vec<AnyValue>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KvlistValue {
    pub values: Option<Vec<KeyValue>>,
}

/// Parsed metric data for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedMetric {
    pub name: String,
    pub timestamp_ns: i64,
    pub value: f64,
    pub attributes: std::collections::HashMap<String, String>,
}

/// Parsed log/event data for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedEvent {
    pub name: String,
    pub timestamp_ns: i64,
    pub attributes: std::collections::HashMap<String, String>,
}

impl KeyValue {
    /// Extract string value from KeyValue
    pub fn get_string_value(&self) -> Option<String> {
        self.value.as_ref().and_then(|v| {
            v.string_value.clone()
                .or_else(|| v.int_value.clone())
                .or_else(|| v.double_value.map(|d| d.to_string()))
                .or_else(|| v.bool_value.map(|b| b.to_string()))
        })
    }
}

impl NumberDataPoint {
    /// Get the numeric value as f64
    pub fn get_value(&self) -> f64 {
        self.as_double.unwrap_or_else(|| {
            self.as_int
                .as_ref()
                .and_then(|s| s.parse::<i64>().ok())
                .map(|i| i as f64)
                .unwrap_or(0.0)
        })
    }

    /// Get timestamp in nanoseconds
    pub fn get_timestamp_ns(&self) -> i64 {
        self.time_unix_nano
            .as_ref()
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0)
    }

    /// Extract attributes as a HashMap
    pub fn get_attributes(&self) -> std::collections::HashMap<String, String> {
        let mut map = std::collections::HashMap::new();
        if let Some(attrs) = &self.attributes {
            for kv in attrs {
                if let (Some(key), Some(value)) = (&kv.key, kv.get_string_value()) {
                    map.insert(key.clone(), value);
                }
            }
        }
        map
    }
}

impl LogRecord {
    /// Get timestamp in nanoseconds
    pub fn get_timestamp_ns(&self) -> i64 {
        self.time_unix_nano
            .as_ref()
            .or(self.observed_time_unix_nano.as_ref())
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0)
    }

    /// Extract event name from attributes
    pub fn get_event_name(&self) -> Option<String> {
        self.attributes.as_ref().and_then(|attrs| {
            attrs.iter()
                .find(|kv| kv.key.as_deref() == Some("event.name"))
                .and_then(|kv| kv.get_string_value())
        })
    }

    /// Extract attributes as a HashMap
    pub fn get_attributes(&self) -> std::collections::HashMap<String, String> {
        let mut map = std::collections::HashMap::new();
        if let Some(attrs) = &self.attributes {
            for kv in attrs {
                if let (Some(key), Some(value)) = (&kv.key, kv.get_string_value()) {
                    map.insert(key.clone(), value);
                }
            }
        }
        map
    }
}
