//! Background refresh task for push-based updates

use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager};
use tokio::time::interval;

use crate::usage::models::UsageDataDelta;
use crate::usage::pricing::PricingCalculator;
use crate::usage::CacheManager;
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

            // Always check for changes and emit event (for heartbeat indicator)
            let has_file_changes = cache.has_changes(None);

            if has_file_changes {
                // Perform incremental load and get delta
                let pricing = PricingCalculator::default();
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
    });
}
