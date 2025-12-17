//! Data source type detection and management

use serde::{Deserialize, Serialize};

/// Data source types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DataSourceType {
    /// Local JSONL files (default)
    Jsonl,
    /// OpenTelemetry telemetry data
    Telemetry,
}

impl DataSourceType {
    /// Get display name for UI
    pub fn display_name(&self) -> &'static str {
        match self {
            DataSourceType::Jsonl => "Local Files",
            DataSourceType::Telemetry => "Telemetry",
        }
    }

    /// Get icon for UI
    pub fn icon(&self) -> &'static str {
        match self {
            DataSourceType::Jsonl => "📁",
            DataSourceType::Telemetry => "📡",
        }
    }
}

impl Default for DataSourceType {
    fn default() -> Self {
        DataSourceType::Jsonl
    }
}

impl std::fmt::Display for DataSourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataSourceType::Jsonl => write!(f, "jsonl"),
            DataSourceType::Telemetry => write!(f, "telemetry"),
        }
    }
}

/// Detect the active data source based on environment variables
pub fn get_active_data_source() -> DataSourceType {
    if is_telemetry_enabled() {
        DataSourceType::Telemetry
    } else {
        DataSourceType::Jsonl
    }
}

/// Check if telemetry is enabled via environment variable
pub fn is_telemetry_enabled() -> bool {
    let env_value = std::env::var("CLAUDE_CODE_ENABLE_TELEMETRY");
    // log::info!("CLAUDE_CODE_ENABLE_TELEMETRY = {:?}", env_value);
    env_value
        .map(|v| v == "1" || v.to_lowercase() == "true")
        .unwrap_or(false)
}

/// Get the collector port from environment or default
pub fn get_collector_port() -> u16 {
    std::env::var("CCM_COLLECTOR_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(super::collector::DEFAULT_COLLECTOR_PORT)
}

/// Data source status information
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSourceStatus {
    /// Current data source type
    pub source_type: DataSourceType,
    /// Display name for UI
    pub display_name: String,
    /// Icon for UI
    pub icon: String,
    /// Whether telemetry collector is running (only for telemetry source)
    pub collector_running: bool,
    /// Collector port (only for telemetry source)
    pub collector_port: Option<u16>,
}

impl DataSourceStatus {
    /// Create status for JSONL data source
    pub fn jsonl() -> Self {
        Self {
            source_type: DataSourceType::Jsonl,
            display_name: DataSourceType::Jsonl.display_name().to_string(),
            icon: DataSourceType::Jsonl.icon().to_string(),
            collector_running: false,
            collector_port: None,
        }
    }

    /// Create status for telemetry data source
    pub fn telemetry(running: bool, port: u16) -> Self {
        Self {
            source_type: DataSourceType::Telemetry,
            display_name: DataSourceType::Telemetry.display_name().to_string(),
            icon: DataSourceType::Telemetry.icon().to_string(),
            collector_running: running,
            collector_port: Some(port),
        }
    }
}
