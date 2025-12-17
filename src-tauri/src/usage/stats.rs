//! Statistics calculation for usage data

use std::collections::HashMap;

use chrono::{DateTime, Datelike, Timelike, Utc};

use crate::usage::models::{BurnRate, DailyUsage, ModelStats, OverallStats, ProjectStats, UsageData, UsageEntry};
use crate::usage::pricing::PricingCalculator;
use crate::usage::reader::{load_all_entries, ProjectData, ReaderError};

/// Session duration in minutes (5 hours)
const SESSION_DURATION_MINUTES: i64 = 300;

/// Filter options for usage data
#[derive(Debug, Default)]
pub struct FilterOptions {
    /// Filter by start date (inclusive)
    pub start_date: Option<DateTime<Utc>>,
    /// Filter by end date (inclusive)
    pub end_date: Option<DateTime<Utc>>,
    /// Filter by project path (decoded)
    pub project_path: Option<String>,
}

impl FilterOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_date_range(mut self, start: Option<DateTime<Utc>>, end: Option<DateTime<Utc>>) -> Self {
        self.start_date = start;
        self.end_date = end;
        self
    }

    pub fn with_project(mut self, project: Option<String>) -> Self {
        self.project_path = project;
        self
    }

    /// Check if an entry passes the filter
    pub fn matches(&self, entry: &UsageEntry, project_path: Option<&str>) -> bool {
        // Check date range
        if let Some(start) = &self.start_date {
            if entry.timestamp < *start {
                return false;
            }
        }
        if let Some(end) = &self.end_date {
            if entry.timestamp > *end {
                return false;
            }
        }

        // Check project
        if let Some(filter_project) = &self.project_path {
            if let Some(entry_project) = project_path {
                if entry_project != filter_project {
                    return false;
                }
            }
        }

        true
    }
}

/// Normalize model name for consistent grouping
fn normalize_model_name(model: &str) -> String {
    let model_lower = model.to_lowercase();

    // Keep new claude-4 model names as-is
    if model_lower.contains("claude-opus-4-")
        || model_lower.contains("claude-sonnet-4-")
        || model_lower.contains("claude-haiku-4-")
        || model_lower.contains("opus-4-")
        || model_lower.contains("sonnet-4-")
        || model_lower.contains("haiku-4-")
    {
        return model_lower;
    }

    // Normalize older model names
    if model_lower.contains("opus") {
        if model_lower.contains("4-") {
            return model_lower;
        }
        return "claude-3-opus".to_string();
    }
    if model_lower.contains("sonnet") {
        if model_lower.contains("4-") {
            return model_lower;
        }
        if model_lower.contains("3.5") || model_lower.contains("3-5") {
            return "claude-3-5-sonnet".to_string();
        }
        return "claude-3-sonnet".to_string();
    }
    if model_lower.contains("haiku") {
        if model_lower.contains("3.5") || model_lower.contains("3-5") {
            return "claude-3-5-haiku".to_string();
        }
        return "claude-3-haiku".to_string();
    }

    model.to_string()
}

/// Calculate model distribution from entries
fn calculate_model_distribution(entries: &[UsageEntry]) -> Vec<ModelStats> {
    let mut model_map: HashMap<String, ModelStats> = HashMap::new();
    let mut total_tokens: u64 = 0;

    for entry in entries {
        let model_key = normalize_model_name(&entry.model);
        let entry_total = entry.input_tokens + entry.output_tokens;
        total_tokens += entry_total;

        let stats = model_map.entry(model_key.clone()).or_insert_with(|| ModelStats {
            model: model_key,
            ..Default::default()
        });

        stats.input_tokens += entry.input_tokens;
        stats.output_tokens += entry.output_tokens;
        stats.cache_creation_tokens += entry.cache_creation_tokens;
        stats.cache_read_tokens += entry.cache_read_tokens;
        stats.cost_usd += entry.cost_usd;
        stats.message_count += 1;
        stats.total_tokens += entry_total;
    }

    // Calculate percentages and round costs
    let mut model_list: Vec<_> = model_map
        .into_values()
        .map(|mut m| {
            m.percentage = if total_tokens > 0 {
                (m.total_tokens as f64 / total_tokens as f64) * 100.0
            } else {
                0.0
            };
            m.cost_usd = (m.cost_usd * 1_000_000.0).round() / 1_000_000.0;
            m.percentage = (m.percentage * 100.0).round() / 100.0;
            m
        })
        .collect();

    // Sort by total tokens descending
    model_list.sort_by(|a, b| b.total_tokens.cmp(&a.total_tokens));
    model_list
}

/// Session block for proportional burn rate calculation (matches Python's block structure)
#[derive(Debug)]
struct SessionBlock {
    start_time: DateTime<Utc>,
    actual_end_time: DateTime<Utc>,
    total_tokens: u64,  // input + output only (like Python's totalTokens)
    total_cost: f64,
    is_active: bool,
}

/// Transform entries into session blocks (5-hour blocks starting at hour boundary)
/// Matches Python's SessionAnalyzer.transform_to_blocks
fn transform_to_blocks(entries: &[UsageEntry]) -> Vec<SessionBlock> {
    if entries.is_empty() {
        return Vec::new();
    }

    let mut blocks: Vec<SessionBlock> = Vec::new();
    let session_duration = chrono::Duration::hours(5);

    let mut current_block: Option<SessionBlock> = None;

    for entry in entries {
        let should_create_new = match &current_block {
            None => true,
            Some(block) => {
                // Check if entry is past block's end time
                entry.timestamp >= block.start_time + session_duration
            }
        };

        if should_create_new {
            // Finalize current block
            if let Some(block) = current_block.take() {
                blocks.push(block);
            }

            // Create new block - round start time to hour boundary
            let start_time = entry.timestamp
                .with_minute(0).unwrap()
                .with_second(0).unwrap()
                .with_nanosecond(0).unwrap();

            current_block = Some(SessionBlock {
                start_time,
                actual_end_time: entry.timestamp,
                total_tokens: 0,
                total_cost: 0.0,
                is_active: false,
            });
        }

        // Add entry to current block
        if let Some(ref mut block) = current_block {
            // Python's totalTokens only includes input + output (no cache tokens)
            block.total_tokens += entry.input_tokens + entry.output_tokens;
            block.total_cost += entry.cost_usd;
            block.actual_end_time = entry.timestamp;
        }
    }

    // Finalize last block
    if let Some(mut block) = current_block {
        // Mark active if end_time is in the future
        let now = Utc::now();
        if block.start_time + session_duration > now {
            block.is_active = true;
        }
        blocks.push(block);
    }

    blocks
}

/// Calculate hourly burn rate using block-based proportional allocation
/// Matches Python's calculate_hourly_burn_rate in calculations.py
fn calculate_hourly_burn_rate(blocks: &[SessionBlock], current_time: &DateTime<Utc>) -> (f64, f64) {
    if blocks.is_empty() {
        return (0.0, 0.0);
    }

    let one_hour_ago = *current_time - chrono::Duration::hours(1);
    let mut total_tokens: f64 = 0.0;
    let mut total_cost: f64 = 0.0;

    for block in blocks {
        // Determine session end time (current time if active, actual_end_time otherwise)
        let session_actual_end = if block.is_active {
            *current_time
        } else {
            block.actual_end_time
        };

        // Skip if block ended before the hour window
        if session_actual_end < one_hour_ago {
            continue;
        }

        // Calculate overlap with the last hour
        let session_start_in_hour = if block.start_time > one_hour_ago {
            block.start_time
        } else {
            one_hour_ago
        };

        let session_end_in_hour = if session_actual_end < *current_time {
            session_actual_end
        } else {
            *current_time
        };

        if session_end_in_hour <= session_start_in_hour {
            continue;
        }

        // Calculate proportional tokens
        let total_session_duration = (session_actual_end - block.start_time).num_seconds() as f64 / 60.0;
        let hour_duration = (session_end_in_hour - session_start_in_hour).num_seconds() as f64 / 60.0;

        if total_session_duration > 0.0 {
            let proportion = hour_duration / total_session_duration;
            total_tokens += block.total_tokens as f64 * proportion;
            total_cost += block.total_cost * proportion;
        }
    }

    // Return tokens per minute (divide by 60)
    if total_tokens > 0.0 {
        (total_tokens / 60.0, total_cost / 60.0 * 60.0) // tokens/min, cost/hour
    } else {
        (0.0, 0.0)
    }
}

/// Calculate time to reset based on session start time
fn calculate_time_to_reset(session_start: Option<&DateTime<Utc>>, now: &DateTime<Utc>) -> u32 {
    match session_start {
        Some(start) => {
            let elapsed_minutes = (*now - *start).num_minutes();
            if elapsed_minutes < 0 {
                return SESSION_DURATION_MINUTES as u32;
            }
            let remaining = SESSION_DURATION_MINUTES - (elapsed_minutes % SESSION_DURATION_MINUTES);
            remaining.max(0) as u32
        }
        None => SESSION_DURATION_MINUTES as u32,
    }
}

/// Calculate project statistics from entries
fn calculate_project_stats(project: &ProjectData, entries: &[UsageEntry]) -> ProjectStats {
    let mut stats = ProjectStats {
        project_path: project.decoded_path.clone(),
        display_name: project.display_name.clone(),
        session_count: project.session_files.len() as u32,
        ..Default::default()
    };

    for entry in entries {
        stats.total_input_tokens += entry.input_tokens;
        stats.total_output_tokens += entry.output_tokens;
        stats.cache_creation_tokens += entry.cache_creation_tokens;
        stats.cache_read_tokens += entry.cache_read_tokens;
        stats.total_cost_usd += entry.cost_usd;
        stats.message_count += 1;

        // Update activity timestamps
        let ts = entry.timestamp.to_rfc3339();
        match &stats.first_activity {
            None => stats.first_activity = Some(ts.clone()),
            Some(first) if ts < *first => stats.first_activity = Some(ts.clone()),
            _ => {}
        }
        match &stats.last_activity {
            None => stats.last_activity = Some(ts.clone()),
            Some(last) if ts > *last => stats.last_activity = Some(ts.clone()),
            _ => {}
        }
    }

    // Round cost
    stats.total_cost_usd = (stats.total_cost_usd * 1_000_000.0).round() / 1_000_000.0;

    stats
}

/// Calculate daily usage from entries
fn calculate_daily_usage(entries: &[UsageEntry]) -> Vec<DailyUsage> {
    let mut daily_map: HashMap<String, DailyUsage> = HashMap::new();

    for entry in entries {
        let date_key = format!(
            "{:04}-{:02}-{:02}",
            entry.timestamp.year(),
            entry.timestamp.month(),
            entry.timestamp.day()
        );

        let daily = daily_map.entry(date_key.clone()).or_insert_with(|| DailyUsage {
            date: date_key,
            ..Default::default()
        });

        daily.input_tokens += entry.input_tokens;
        daily.output_tokens += entry.output_tokens;
        daily.cache_creation_tokens += entry.cache_creation_tokens;
        daily.cache_read_tokens += entry.cache_read_tokens;
        daily.cost_usd += entry.cost_usd;
        daily.message_count += 1;
    }

    // Round costs and sort by date
    let mut daily_list: Vec<_> = daily_map
        .into_values()
        .map(|mut d| {
            d.cost_usd = (d.cost_usd * 1_000_000.0).round() / 1_000_000.0;
            d
        })
        .collect();

    daily_list.sort_by(|a, b| a.date.cmp(&b.date));
    daily_list
}

/// Calculate overall statistics with advanced metrics
fn calculate_overall_stats(projects: &[ProjectStats], all_entries: &[UsageEntry]) -> OverallStats {
    let mut stats = OverallStats {
        project_count: projects.len() as u32,
        ..Default::default()
    };

    for project in projects {
        stats.total_input_tokens += project.total_input_tokens;
        stats.total_output_tokens += project.total_output_tokens;
        stats.cache_creation_tokens += project.cache_creation_tokens;
        stats.cache_read_tokens += project.cache_read_tokens;
        stats.total_cost_usd += project.total_cost_usd;
        stats.total_messages += project.message_count;
        stats.total_sessions += project.session_count;
    }

    // Round cost
    stats.total_cost_usd = (stats.total_cost_usd * 1_000_000.0).round() / 1_000_000.0;

    // Calculate model distribution
    stats.model_distribution = calculate_model_distribution(all_entries);

    // Calculate session timing and burn rate
    // Session timing uses 5-hour blocks, burn rate uses block-based proportional allocation (like Python CLI)
    if !all_entries.is_empty() {
        let now = Utc::now();

        // Get the last 5 hours window to identify recent activity for session timing
        let window_start = now - chrono::Duration::minutes(SESSION_DURATION_MINUTES);

        // Get entries within the 5-hour window
        let recent_entries: Vec<_> = all_entries
            .iter()
            .filter(|e| e.timestamp >= window_start)
            .collect();

        if !recent_entries.is_empty() {
            // Find the first entry in this window
            let first_entry_time = recent_entries.iter().map(|e| e.timestamp).min().unwrap();

            // Round to hour boundary like Python: start_time = round_to_hour(first_entry.timestamp)
            let session_block_start = first_entry_time
                .with_minute(0).unwrap()
                .with_second(0).unwrap()
                .with_nanosecond(0).unwrap();

            stats.session_start_time = Some(session_block_start.to_rfc3339());
            stats.time_to_reset_minutes = calculate_time_to_reset(Some(&session_block_start), &now);

            // Calculate HOURLY burn rate using block-based proportional allocation
            // Matches Python CLI's calculate_hourly_burn_rate in calculations.py

            // Transform all entries into session blocks (not just recent ones)
            // Python uses all blocks that overlap with the last hour
            let blocks = transform_to_blocks(all_entries);

            // Calculate proportional burn rate
            let (tokens_per_min, cost_per_hour) = calculate_hourly_burn_rate(&blocks, &now);

            if tokens_per_min > 0.0 {
                stats.burn_rate = Some(BurnRate {
                    tokens_per_minute: (tokens_per_min * 100.0).round() / 100.0,
                    cost_per_hour: (cost_per_hour * 10000.0).round() / 10000.0,
                });
            }
        } else {
            stats.time_to_reset_minutes = SESSION_DURATION_MINUTES as u32;
        }
    } else {
        stats.time_to_reset_minutes = SESSION_DURATION_MINUTES as u32;
    }

    stats
}

/// Get complete usage data
pub fn get_usage_data(
    custom_path: Option<&str>,
    filter: &FilterOptions,
) -> Result<UsageData, ReaderError> {
    let pricing = PricingCalculator::new();
    let all_data = load_all_entries(custom_path, &pricing)?;

    let mut all_entries: Vec<UsageEntry> = Vec::new();
    let mut projects: Vec<ProjectStats> = Vec::new();

    for (project, entries) in all_data {
        // Apply filter
        let filtered_entries: Vec<_> = entries
            .into_iter()
            .filter(|e| filter.matches(e, Some(&project.decoded_path)))
            .collect();

        if !filtered_entries.is_empty() {
            all_entries.extend(filtered_entries.clone());
            projects.push(calculate_project_stats(&project, &filtered_entries));
        }
    }

    // Sort entries by timestamp for daily calculation
    all_entries.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

    let daily_usage = calculate_daily_usage(&all_entries);
    let overall_stats = calculate_overall_stats(&projects, &all_entries);

    // Sort projects by last activity (most recent first)
    projects.sort_by(|a, b| {
        let a_time = a.last_activity.as_deref().unwrap_or("");
        let b_time = b.last_activity.as_deref().unwrap_or("");
        b_time.cmp(a_time)
    });

    Ok(UsageData {
        projects,
        daily_usage,
        overall_stats,
        data_source: None, // Will be set by command layer
    })
}

/// Get usage data for a specific project
pub fn get_project_usage(
    custom_path: Option<&str>,
    project_path: &str,
) -> Result<Option<ProjectStats>, ReaderError> {
    let filter = FilterOptions::new().with_project(Some(project_path.to_string()));
    let data = get_usage_data(custom_path, &filter)?;

    Ok(data.projects.into_iter().next())
}

/// Get daily usage for a specific date range
pub fn get_daily_usage_range(
    custom_path: Option<&str>,
    start_date: Option<DateTime<Utc>>,
    end_date: Option<DateTime<Utc>>,
) -> Result<Vec<DailyUsage>, ReaderError> {
    let filter = FilterOptions::new().with_date_range(start_date, end_date);
    let data = get_usage_data(custom_path, &filter)?;

    Ok(data.daily_usage)
}
