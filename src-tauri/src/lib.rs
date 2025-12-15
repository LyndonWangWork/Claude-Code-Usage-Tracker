//! Claude Code Usage Monitor - Tauri Application

mod commands;
pub mod usage;

use std::sync::Mutex;

use commands::{
    check_data_directory, get_config, get_daily_usage, get_overall_stats, get_project_details,
    get_projects, get_usage_stats, get_usage_stats_incremental, set_config,
};
use usage::{start_background_refresh, CacheManager};

/// Application state containing the cache manager
pub struct AppState {
    pub cache: Mutex<CacheManager>,
}

/// Default refresh interval in seconds
const BACKGROUND_REFRESH_INTERVAL_SECS: u64 = 5;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            cache: Mutex::new(CacheManager::new()),
        })
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
