//! Claude Code Usage Monitor - Tauri Application

mod commands;
pub mod usage;

use std::sync::Mutex;

use commands::{
    check_data_directory, get_config, get_daily_usage, get_overall_stats, get_project_details,
    get_projects, get_usage_stats, get_usage_stats_incremental, set_config, get_data_source_status,
};
use usage::{start_background_refresh, CacheManager, TelemetryCollector, get_active_data_source, DataSourceType};

/// Application state containing the cache manager and telemetry collector
pub struct AppState {
    pub cache: Mutex<CacheManager>,
    pub telemetry_collector: Mutex<Option<TelemetryCollector>>,
}

/// Default refresh interval in seconds
const BACKGROUND_REFRESH_INTERVAL_SECS: u64 = 5;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            cache: Mutex::new(CacheManager::new()),
            telemetry_collector: Mutex::new(None),
        })
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Check if telemetry is enabled and start collector
            let data_source = get_active_data_source();
            log::info!("Active data source: {:?}", data_source);

            if data_source == DataSourceType::Telemetry {
                log::info!("Telemetry enabled, starting local collector...");
                match TelemetryCollector::new(None, None) {
                    Ok(mut collector) => {
                        let port = collector.port();
                        log::info!("Created telemetry collector on port {}", port);
                        // Start collector in async context
                        tauri::async_runtime::spawn(async move {
                            log::info!("Starting telemetry collector async task...");
                            if let Err(e) = collector.start().await {
                                log::error!("Failed to start telemetry collector: {}", e);
                            } else {
                                log::info!("Telemetry collector is now running");
                                // 保持 collector 存活，防止 shutdown_tx 被丢弃导致服务器关闭
                                // 这个 future 永远不会完成，所以 collector 会一直存活
                                std::future::pending::<()>().await;
                            }
                        });
                        log::info!("Telemetry collector spawn completed");
                    }
                    Err(e) => {
                        log::error!("Failed to create telemetry collector: {}", e);
                    }
                }
            } else {
                log::info!("Telemetry not enabled, using JSONL data source");
            }

            // Start background refresh task
            start_background_refresh(app.handle().clone(), BACKGROUND_REFRESH_INTERVAL_SECS);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_usage_stats,
            get_usage_stats_incremental,
            get_projects,
            get_project_details,
            get_daily_usage,
            get_overall_stats,
            get_config,
            set_config,
            check_data_directory,
            get_data_source_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
