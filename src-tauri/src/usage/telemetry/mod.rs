//! OpenTelemetry telemetry data source module
//!
//! This module provides a local OTLP HTTP collector that receives telemetry data
//! from Claude Code and stores it in a local SQLite database.

pub mod collector;
pub mod models;
pub mod storage;
pub mod reader;
pub mod datasource;

pub use collector::TelemetryCollector;
pub use datasource::{DataSourceType, get_active_data_source};
pub use storage::TelemetryStorage;
pub use reader::TelemetryReader;
