//! Background refresh task for push-based updates

use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager};
use tokio::time::interval;

use crate::usage::models::{OverallStats, UsageData, UsageDataDelta};
use crate::usage::pricing::PricingCalculator;
use crate::usage::telemetry::{DataSourceType, TelemetryReader, TelemetryStorage, get_active_data_source};
use crate::AppState;

/// Event name for usage data updates
pub const USAGE_DATA_UPDATED_EVENT: &str = "usage-data-updated";

/// Start the background refresh task
pub fn start_background_refresh(app: AppHandle, refresh_interval_secs: u64) {
    let app_handle = app.clone();

    tauri::async_runtime::spawn(async move {
        let mut ticker = interval(Duration::from_secs(refresh_interval_secs));

        // Skip the first tick (immediate)
        ticker.tick().await;

        loop {
            ticker.tick().await;

            let data_source = get_active_data_source();

            // Get the app state
            let state = match app_handle.try_state::<AppState>() {
                Some(s) => s,
                None => {
                    log::warn!("AppState not available, skipping refresh");
                    continue;
                }
            };

            // Try to acquire the lock
            let mut cache = match state.cache.lock() {
                Ok(c) => c,
                Err(e) => {
                    log::warn!("Failed to acquire cache lock: {}", e);
                    continue;
                }
            };

            let pricing = PricingCalculator::default();

            match data_source {
                DataSourceType::Telemetry => {
                    // Hybrid mode: read both telemetry and JSONL, merge them
                    // Telemetry provides: overall stats (tokens, cost, burn_rate, today_stats, daily_usage, model_distribution)
                    // JSONL provides: projects, session_start_time, time_to_reset_minutes

                    // Read telemetry data
                    let telemetry_data = match TelemetryStorage::new(None) {
                        Ok(storage) => {
                            let reader = TelemetryReader::new(storage);
                            reader.get_usage_data(None, None).ok()
                        }
                        Err(e) => {
                            log::warn!("Failed to create telemetry storage: {}", e);
                            None
                        }
                    };

                    // Read JSONL data (always, for project info)
                    let jsonl_data = cache.incremental_load(None, &pricing).ok();

                    // Merge data: JSONL projects + telemetry overall stats
                    let merged_data = merge_telemetry_jsonl(telemetry_data, jsonl_data);

                    if let Some(data) = merged_data {
                        let delta = UsageDataDelta {
                            has_changes: true,
                            full_refresh: false, // Use mergeDelta, don't trigger loading state
                            updated_projects: data.projects,
                            overall_stats: Some(data.overall_stats),
                            daily_usage: Some(data.daily_usage),
                        };

                        log::debug!(
                            "Emitting telemetry+jsonl merged data: {} projects",
                            delta.updated_projects.len()
                        );

                        if let Err(e) = app_handle.emit(USAGE_DATA_UPDATED_EVENT, &delta) {
                            log::error!("Failed to emit event: {}", e);
                        }
                    } else {
                        // No data available, emit heartbeat
                        let delta = UsageDataDelta {
                            has_changes: false,
                            ..Default::default()
                        };
                        if let Err(e) = app_handle.emit(USAGE_DATA_UPDATED_EVENT, &delta) {
                            log::error!("Failed to emit heartbeat event: {}", e);
                        }
                    }
                }
                DataSourceType::Jsonl => {
                    // JSONL mode: check for file changes and refresh cache
                    let has_file_changes = cache.has_changes(None);

                    if has_file_changes {
                        // Perform incremental load and get delta
                        match cache.incremental_load_with_delta(None, &pricing) {
                            Ok((_data, delta)) => {
                                log::info!(
                                    "Emitting usage-data-updated event: {} updated projects, has_changes={}",
                                    delta.updated_projects.len(),
                                    delta.has_changes
                                );

                                if let Err(e) = app_handle.emit(USAGE_DATA_UPDATED_EVENT, &delta) {
                                    log::error!("Failed to emit event: {}", e);
                                }
                            }
                            Err(e) => {
                                log::warn!("Background refresh failed: {}", e);
                            }
                        }
                    } else {
                        // No changes, emit heartbeat event
                        let delta = UsageDataDelta {
                            has_changes: false,
                            ..Default::default()
                        };

                        if let Err(e) = app_handle.emit(USAGE_DATA_UPDATED_EVENT, &delta) {
                            log::error!("Failed to emit heartbeat event: {}", e);
                        }
                    }
                }
            }
        }
    });
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
                data_source: None, // Will be set by command layer
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
