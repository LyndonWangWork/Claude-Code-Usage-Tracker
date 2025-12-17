//! Read telemetry data and convert to usage models

use std::collections::HashMap;

use chrono::{DateTime, TimeZone, Utc, Local, NaiveDate};

use crate::usage::models::{
    BurnRate, DailyUsage, ModelStats, OverallStats, TodayStats, UsageData,
};
use crate::usage::pricing::PricingCalculator;

use super::storage::TelemetryStorage;

/// Reader for telemetry data from SQLite storage
pub struct TelemetryReader {
    storage: TelemetryStorage,
    #[allow(dead_code)] // Reserved for future use
    pricing: PricingCalculator,
}

impl TelemetryReader {
    /// Create a new telemetry reader
    pub fn new(storage: TelemetryStorage) -> Self {
        Self {
            storage,
            pricing: PricingCalculator::new(),
        }
    }

    /// Get usage data from telemetry storage
    pub fn get_usage_data(
        &self,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
    ) -> Result<UsageData, Box<dyn std::error::Error + Send + Sync>> {
        // Query all claude_code metrics and events
        let metrics = self.storage.query_metrics_by_prefix("claude_code.", start_time, end_time)?;
        let events = self.storage.query_events_by_prefix("claude_code.", start_time, end_time)?;

        // Aggregate data
        let mut total_input_tokens: u64 = 0;
        let mut total_output_tokens: u64 = 0;
        let mut cache_creation_tokens: u64 = 0;
        let mut cache_read_tokens: u64 = 0;
        let mut total_cost: f64 = 0.0;
        let mut session_count: u32 = 0;
        let mut message_count: u32 = 0;

        // Model distribution tracking
        let mut model_stats: HashMap<String, ModelStats> = HashMap::new();

        // Daily usage tracking
        let mut daily_usage: HashMap<NaiveDate, DailyUsage> = HashMap::new();

        // Process metrics
        for metric in &metrics {
            match metric.name.as_str() {
                "claude_code.token.usage" => {
                    let token_type = metric.attributes.get("type").map(|s| s.as_str()).unwrap_or("");
                    let value = metric.value as u64;
                    let model = metric.attributes.get("model").cloned().unwrap_or_else(|| "unknown".to_string());

                    match token_type {
                        "input" => total_input_tokens += value,
                        "output" => total_output_tokens += value,
                        "cacheRead" => cache_read_tokens += value,
                        "cacheCreation" => cache_creation_tokens += value,
                        _ => {}
                    }

                    // Update model stats
                    let entry = model_stats.entry(model.clone()).or_insert_with(|| ModelStats {
                        model: model.clone(),
                        ..Default::default()
                    });
                    match token_type {
                        "input" => entry.input_tokens += value,
                        "output" => entry.output_tokens += value,
                        "cacheRead" => entry.cache_read_tokens += value,
                        "cacheCreation" => entry.cache_creation_tokens += value,
                        _ => {}
                    }
                    entry.total_tokens = entry.input_tokens + entry.output_tokens;

                    // Update daily usage
                    let date = timestamp_to_local_date(metric.timestamp_ns);
                    let daily = daily_usage.entry(date).or_insert_with(|| DailyUsage {
                        date: date.to_string(),
                        ..Default::default()
                    });
                    match token_type {
                        "input" => daily.input_tokens += value,
                        "output" => daily.output_tokens += value,
                        "cacheRead" => daily.cache_read_tokens += value,
                        "cacheCreation" => daily.cache_creation_tokens += value,
                        _ => {}
                    }
                }
                "claude_code.cost.usage" => {
                    total_cost += metric.value;
                    let model = metric.attributes.get("model").cloned().unwrap_or_else(|| "unknown".to_string());

                    // Update model cost
                    let entry = model_stats.entry(model.clone()).or_insert_with(|| ModelStats {
                        model: model.clone(),
                        ..Default::default()
                    });
                    entry.cost_usd += metric.value;

                    // Update daily cost
                    let date = timestamp_to_local_date(metric.timestamp_ns);
                    let daily = daily_usage.entry(date).or_insert_with(|| DailyUsage {
                        date: date.to_string(),
                        ..Default::default()
                    });
                    daily.cost_usd += metric.value;
                }
                "claude_code.session.count" => {
                    session_count += metric.value as u32;
                }
                _ => {}
            }
        }

        // Process events for message count
        for event in &events {
            if event.name == "claude_code.api_request" {
                message_count += 1;

                // Update daily message count
                let date = timestamp_to_local_date(event.timestamp_ns);
                let daily = daily_usage.entry(date).or_insert_with(|| DailyUsage {
                    date: date.to_string(),
                    ..Default::default()
                });
                daily.message_count += 1;
            }
        }

        // Calculate model percentages
        let total_tokens = total_input_tokens + total_output_tokens;
        for stats in model_stats.values_mut() {
            if total_tokens > 0 {
                stats.percentage = (stats.total_tokens as f64 / total_tokens as f64) * 100.0;
            }
            stats.message_count = message_count; // Approximate
        }

        // Sort model stats by usage
        let mut model_distribution: Vec<_> = model_stats.into_values().collect();
        model_distribution.sort_by(|a, b| b.total_tokens.cmp(&a.total_tokens));

        // Sort daily usage by date
        let mut daily_usage_vec: Vec<_> = daily_usage.into_values().collect();
        daily_usage_vec.sort_by(|a, b| a.date.cmp(&b.date));

        // Calculate today's stats
        let today = Local::now().date_naive();
        let today_stats = daily_usage_vec
            .iter()
            .find(|d| d.date == today.to_string())
            .map(|d| TodayStats {
                cost_usd: d.cost_usd,
                input_tokens: d.input_tokens,
                output_tokens: d.output_tokens,
                total_tokens: d.input_tokens + d.output_tokens,
                message_count: d.message_count,
            })
            .unwrap_or_default();

        // Calculate burn rate from metrics
        let burn_rate = calculate_burn_rate_from_metrics(&metrics);

        // Build overall stats
        let overall_stats = OverallStats {
            total_input_tokens,
            total_output_tokens,
            cache_creation_tokens,
            cache_read_tokens,
            total_cost_usd: total_cost,
            total_messages: message_count,
            total_sessions: session_count,
            project_count: 0, // Telemetry doesn't track projects
            model_distribution,
            session_start_time: None,
            time_to_reset_minutes: 0,
            burn_rate,
            today_stats,
        };

        Ok(UsageData {
            projects: vec![], // Telemetry doesn't track per-project data
            daily_usage: daily_usage_vec,
            overall_stats,
            data_source: None, // Will be set by command layer
        })
    }
}

/// Convert nanosecond timestamp to local date
fn timestamp_to_local_date(timestamp_ns: i64) -> NaiveDate {
    let secs = timestamp_ns / 1_000_000_000;
    let nsecs = (timestamp_ns % 1_000_000_000) as u32;
    Utc.timestamp_opt(secs, nsecs)
        .single()
        .map(|dt| dt.with_timezone(&Local).date_naive())
        .unwrap_or_else(|| Local::now().date_naive())
}

use super::models::ParsedMetric;

/// Calculate burn rate from metrics within the last hour
/// Returns (tokens_per_minute, cost_per_hour)
fn calculate_burn_rate_from_metrics(metrics: &[ParsedMetric]) -> Option<BurnRate> {
    let now = Utc::now();
    let one_hour_ago = now - chrono::Duration::hours(1);
    let one_hour_ago_ns = one_hour_ago.timestamp_nanos_opt().unwrap_or(0);

    // Filter metrics within the last hour
    let mut tokens_last_hour: u64 = 0;
    let mut cost_last_hour: f64 = 0.0;
    let mut has_recent_activity = false;

    for metric in metrics {
        if metric.timestamp_ns < one_hour_ago_ns {
            continue;
        }

        has_recent_activity = true;

        match metric.name.as_str() {
            "claude_code.token.usage" => {
                let token_type = metric.attributes.get("type").map(|s| s.as_str()).unwrap_or("");
                // Only count input + output tokens for burn rate (like JSONL mode)
                if token_type == "input" || token_type == "output" {
                    tokens_last_hour += metric.value as u64;
                }
            }
            "claude_code.cost.usage" => {
                cost_last_hour += metric.value;
            }
            _ => {}
        }
    }

    if !has_recent_activity || tokens_last_hour == 0 {
        return None;
    }

    // Calculate the actual time span within the last hour
    // Find the earliest and latest metric timestamps within the hour window
    let mut earliest_ns: Option<i64> = None;
    let mut latest_ns: Option<i64> = None;

    for metric in metrics {
        if metric.timestamp_ns >= one_hour_ago_ns {
            match earliest_ns {
                None => earliest_ns = Some(metric.timestamp_ns),
                Some(e) if metric.timestamp_ns < e => earliest_ns = Some(metric.timestamp_ns),
                _ => {}
            }
            match latest_ns {
                None => latest_ns = Some(metric.timestamp_ns),
                Some(l) if metric.timestamp_ns > l => latest_ns = Some(metric.timestamp_ns),
                _ => {}
            }
        }
    }

    // Calculate burn rate
    // If we have data spanning less than 1 minute, use the actual span to extrapolate
    let minutes_span = match (earliest_ns, latest_ns) {
        (Some(e), Some(l)) => {
            let span_minutes = (l - e) as f64 / 1_000_000_000.0 / 60.0;
            // Use at least 1 minute, at most 60 minutes
            span_minutes.max(1.0).min(60.0)
        }
        _ => 60.0, // Default to full hour if we can't determine span
    };

    let tokens_per_minute = (tokens_last_hour as f64 / minutes_span * 100.0).round() / 100.0;
    let cost_per_hour = (cost_last_hour / minutes_span * 60.0 * 10000.0).round() / 10000.0;

    Some(BurnRate {
        tokens_per_minute,
        cost_per_hour,
    })
}
