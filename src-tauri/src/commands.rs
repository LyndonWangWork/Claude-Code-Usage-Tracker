//! Tauri commands for the usage monitor

use chrono::{DateTime, Utc};
use tauri::{command, State};

use crate::usage::models::{AppConfig, DailyUsage, OverallStats, ProjectStats, UsageData};
use crate::usage::pricing::PricingCalculator;
use crate::usage::stats::{get_usage_data, FilterOptions};
use crate::AppState;

/// Get complete usage statistics
#[command]
pub fn get_usage_stats(data_path: Option<String>) -> Result<UsageData, String> {
    let filter = FilterOptions::new();
    get_usage_data(data_path.as_deref(), &filter).map_err(|e| e.to_string())
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
    let pricing = PricingCalculator::new();
    let mut cache = state.cache.lock().map_err(|e| e.to_string())?;

    if force_full.unwrap_or(false) {
        // Force full refresh - clear cache and reload all data
        cache.full_load(data_path.as_deref(), &pricing)
            .map_err(|e| e.to_string())
    } else {
        // Incremental refresh - only read changed files
        cache.incremental_load(data_path.as_deref(), &pricing)
            .map_err(|e| e.to_string())
    }
}
