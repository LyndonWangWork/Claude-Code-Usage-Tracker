//! Tauri commands for the usage monitor

use chrono::{DateTime, Utc};
use tauri::{command, State};

use crate::usage::models::{AppConfig, DailyUsage, DataSourceInfo, OverallStats, ProjectStats, UsageData};
use crate::usage::pricing::PricingCalculator;
use crate::usage::stats::{get_usage_data, FilterOptions};
use crate::usage::telemetry::{DataSourceType, get_active_data_source, TelemetryStorage, TelemetryReader};
use crate::usage::telemetry::datasource::get_collector_port;
use crate::AppState;

/// Get complete usage statistics
#[command]
pub fn get_usage_stats(data_path: Option<String>) -> Result<UsageData, String> {
    let data_source = get_active_data_source();

    let mut usage_data = match data_source {
        DataSourceType::Telemetry => {
            // Hybrid mode: read both telemetry and JSONL, merge them
            let telemetry_data = {
                let storage = TelemetryStorage::new(None).map_err(|e| e.to_string())?;
                let reader = TelemetryReader::new(storage);
                reader.get_usage_data(None, None).ok()
            };

            let jsonl_data = {
                let filter = FilterOptions::new();
                get_usage_data(data_path.as_deref(), &filter).ok()
            };

            merge_telemetry_jsonl(telemetry_data, jsonl_data)
                .ok_or_else(|| "No data available from either source".to_string())?
        }
        DataSourceType::Jsonl => {
            // Read from JSONL files only
            let filter = FilterOptions::new();
            get_usage_data(data_path.as_deref(), &filter).map_err(|e| e.to_string())?
        }
    };

    // Add data source info
    usage_data.data_source = Some(create_data_source_info(data_source));

    Ok(usage_data)
}

/// Create data source info for response
fn create_data_source_info(data_source: DataSourceType) -> DataSourceInfo {
    match data_source {
        DataSourceType::Telemetry => DataSourceInfo {
            source_type: "telemetry".to_string(),
            display_name: data_source.display_name().to_string(),
            icon: data_source.icon().to_string(),
            collector_port: Some(get_collector_port()),
        },
        DataSourceType::Jsonl => DataSourceInfo {
            source_type: "jsonl".to_string(),
            display_name: data_source.display_name().to_string(),
            icon: data_source.icon().to_string(),
            collector_port: None,
        },
    }
}

/// Get list of projects with their statistics
#[command]
pub fn get_projects(data_path: Option<String>) -> Result<Vec<ProjectStats>, String> {
    let filter = FilterOptions::new();
    let data = get_usage_data(data_path.as_deref(), &filter).map_err(|e| e.to_string())?;
    Ok(data.projects)
}

/// Get details for a specific project
#[command]
pub fn get_project_details(
    data_path: Option<String>,
    project_path: String,
) -> Result<Option<ProjectStats>, String> {
    let filter = FilterOptions::new().with_project(Some(project_path));
    let data = get_usage_data(data_path.as_deref(), &filter).map_err(|e| e.to_string())?;
    Ok(data.projects.into_iter().next())
}

/// Get daily usage data
#[command]
pub fn get_daily_usage(
    data_path: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Vec<DailyUsage>, String> {
    let start = start_date
        .as_ref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    let end = end_date
        .as_ref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    let filter = FilterOptions::new().with_date_range(start, end);
    let data = get_usage_data(data_path.as_deref(), &filter).map_err(|e| e.to_string())?;
    Ok(data.daily_usage)
}

/// Get overall statistics
#[command]
pub fn get_overall_stats(data_path: Option<String>) -> Result<OverallStats, String> {
    let filter = FilterOptions::new();
    let data = get_usage_data(data_path.as_deref(), &filter).map_err(|e| e.to_string())?;
    Ok(data.overall_stats)
}

/// Get application configuration
#[command]
pub fn get_config() -> AppConfig {
    // For now, return default config
    // In a real app, this would load from a config file
    AppConfig::default()
}

/// Set application configuration
#[command]
pub fn set_config(config: AppConfig) -> Result<(), String> {
    // For now, just validate
    // In a real app, this would save to a config file
    log::info!("Config updated: {:?}", config);
    Ok(())
}

/// Check if the Claude data directory exists and is accessible
#[command]
pub fn check_data_directory(data_path: Option<String>) -> Result<bool, String> {
    use crate::usage::config::get_projects_dir;

    let projects_dir = get_projects_dir(data_path.as_deref());
    Ok(projects_dir.exists() && projects_dir.is_dir())
}

/// Get usage statistics with incremental refresh (only reads changed files)
#[command]
pub fn get_usage_stats_incremental(
    state: State<AppState>,
    data_path: Option<String>,
    force_full: Option<bool>,
) -> Result<UsageData, String> {
    let data_source = get_active_data_source();

    let mut usage_data = match data_source {
        DataSourceType::Telemetry => {
            // Hybrid mode: read both telemetry and JSONL, merge them
            let telemetry_data = {
                let storage = TelemetryStorage::new(None).map_err(|e| e.to_string())?;
                let reader = TelemetryReader::new(storage);
                reader.get_usage_data(None, None).ok()
            };

            let jsonl_data = {
                let pricing = PricingCalculator::new();
                let mut cache = state.cache.lock().map_err(|e| e.to_string())?;

                if force_full.unwrap_or(false) {
                    cache.full_load(data_path.as_deref(), &pricing).ok()
                } else {
                    cache.incremental_load(data_path.as_deref(), &pricing).ok()
                }
            };

            merge_telemetry_jsonl(telemetry_data, jsonl_data)
                .ok_or_else(|| "No data available from either source".to_string())?
        }
        DataSourceType::Jsonl => {
            // Use cache for JSONL
            let pricing = PricingCalculator::new();
            let mut cache = state.cache.lock().map_err(|e| e.to_string())?;

            if force_full.unwrap_or(false) {
                cache.full_load(data_path.as_deref(), &pricing)
                    .map_err(|e| e.to_string())?
            } else {
                cache.incremental_load(data_path.as_deref(), &pricing)
                    .map_err(|e| e.to_string())?
            }
        }
    };

    // Add data source info
    usage_data.data_source = Some(create_data_source_info(data_source));

    Ok(usage_data)
}

/// Get current data source status
#[command]
pub fn get_data_source_status() -> DataSourceInfo {
    let data_source = get_active_data_source();
    create_data_source_info(data_source)
}

/// Merge telemetry data with JSONL data
/// - Telemetry: burn_rate, today_stats, daily_usage, model_distribution
/// - JSONL: projects, tokens, cost, messages (for consistency with project data)
fn merge_telemetry_jsonl(
    telemetry_data: Option<UsageData>,
    jsonl_data: Option<UsageData>,
) -> Option<UsageData> {
    match (telemetry_data, jsonl_data) {
        (Some(telemetry), Some(jsonl)) => {
            // Merge: use JSONL for project-related totals (to keep percentages consistent)
            // Use telemetry for real-time metrics (burn_rate, today_stats)
            let merged_overall = OverallStats {
                // From JSONL (consistent with project data for percentage calculations)
                total_input_tokens: jsonl.overall_stats.total_input_tokens,
                total_output_tokens: jsonl.overall_stats.total_output_tokens,
                cache_creation_tokens: jsonl.overall_stats.cache_creation_tokens,
                cache_read_tokens: jsonl.overall_stats.cache_read_tokens,
                total_cost_usd: jsonl.overall_stats.total_cost_usd,
                total_messages: jsonl.overall_stats.total_messages,
                total_sessions: jsonl.overall_stats.total_sessions,
                project_count: jsonl.overall_stats.project_count,
                session_start_time: jsonl.overall_stats.session_start_time,
                time_to_reset_minutes: jsonl.overall_stats.time_to_reset_minutes,

                // From telemetry (real-time metrics)
                model_distribution: telemetry.overall_stats.model_distribution,
                burn_rate: telemetry.overall_stats.burn_rate,
                today_stats: telemetry.overall_stats.today_stats,
            };

            Some(UsageData {
                projects: jsonl.projects,
                daily_usage: telemetry.daily_usage,
                overall_stats: merged_overall,
                data_source: None, // Will be set by caller
            })
        }
        (Some(telemetry), None) => {
            // Only telemetry available
            Some(telemetry)
        }
        (None, Some(jsonl)) => {
            // Only JSONL available
            Some(jsonl)
        }
        (None, None) => None,
    }
}
