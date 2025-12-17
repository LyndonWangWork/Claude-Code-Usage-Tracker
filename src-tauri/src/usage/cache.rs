//! Cache manager for incremental data refresh

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Instant, SystemTime};

use crate::usage::models::{UsageData, UsageDataDelta, UsageEntry};
use crate::usage::pricing::PricingCalculator;
use crate::usage::reader::{list_projects, read_jsonl_file, ProjectData, ReaderError};

/// Cached data for a single file
#[derive(Debug, Clone)]
struct FileCacheEntry {
    /// File modification time when cached
    mtime: SystemTime,
    /// Parsed entries from this file
    entries: Vec<UsageEntry>,
}

/// Cache manager for incremental data refresh
#[derive(Debug, Default)]
pub struct CacheManager {
    /// Cached data per file path
    file_cache: HashMap<PathBuf, FileCacheEntry>,
    /// Cached project list
    cached_projects: Vec<ProjectData>,
    /// Last full refresh time
    last_full_refresh: Option<Instant>,
    /// Last directory scan time (for detecting new projects)
    last_dir_scan: Option<Instant>,
    /// Cached usage data from last calculation (for quick access when no changes)
    cached_usage_data: Option<UsageData>,
}

/// Result of checking file changes
#[derive(Debug, Default)]
pub struct FileChanges {
    /// Files that have been modified
    pub modified: Vec<PathBuf>,
    /// New files not in cache
    pub new_files: Vec<PathBuf>,
    /// Files that were deleted (in cache but not on disk)
    pub deleted: Vec<PathBuf>,
}

impl CacheManager {
    /// Create a new cache manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear all cached data
    pub fn clear(&mut self) {
        self.file_cache.clear();
        self.cached_projects.clear();
        self.last_full_refresh = None;
        self.last_dir_scan = None;
        self.cached_usage_data = None;
    }

    /// Get cached usage data without any file system operations
    /// Returns None if cache is empty or no data has been calculated yet
    pub fn get_cached_data(&self) -> Option<&UsageData> {
        self.cached_usage_data.as_ref()
    }

    /// Check if cache is empty (first load)
    pub fn is_empty(&self) -> bool {
        self.file_cache.is_empty()
    }

    /// Get time since last full refresh in seconds
    pub fn seconds_since_full_refresh(&self) -> Option<u64> {
        self.last_full_refresh.map(|t| t.elapsed().as_secs())
    }

    /// Check if we should rescan directories for new projects
    pub fn should_rescan_dirs(&self) -> bool {
        match self.last_dir_scan {
            None => true,
            Some(t) => t.elapsed().as_secs() >= 60, // Rescan every 60 seconds
        }
    }

    /// Detect file changes compared to cache
    pub fn check_file_changes(&self, current_files: &[PathBuf]) -> Result<FileChanges, ReaderError> {
        let mut changes = FileChanges::default();

        // Check current files against cache
        for file in current_files {
            let current_mtime = match std::fs::metadata(file) {
                Ok(meta) => match meta.modified() {
                    Ok(t) => t,
                    Err(_) => {
                        // Can't get mtime, treat as modified
                        changes.modified.push(file.clone());
                        continue;
                    }
                },
                Err(_) => continue, // File might have been deleted
            };

            match self.file_cache.get(file) {
                Some(cached) => {
                    if current_mtime > cached.mtime {
                        changes.modified.push(file.clone());
                    }
                }
                None => {
                    changes.new_files.push(file.clone());
                }
            }
        }

        // Check for deleted files
        let current_set: std::collections::HashSet<_> = current_files.iter().collect();
        for cached_path in self.file_cache.keys() {
            if !current_set.contains(cached_path) {
                changes.deleted.push(cached_path.clone());
            }
        }

        Ok(changes)
    }

    /// Update cache with new file data
    pub fn update_file_cache(
        &mut self,
        file: &PathBuf,
        entries: Vec<UsageEntry>,
    ) -> Result<(), ReaderError> {
        let mtime = std::fs::metadata(file)
            .and_then(|m| m.modified())
            .unwrap_or_else(|_| SystemTime::now());

        self.file_cache.insert(
            file.clone(),
            FileCacheEntry { mtime, entries },
        );

        Ok(())
    }

    /// Remove a file from cache
    pub fn remove_file(&mut self, file: &PathBuf) {
        self.file_cache.remove(file);
    }

    /// Get cached entries for a file
    pub fn get_file_entries(&self, file: &PathBuf) -> Option<&Vec<UsageEntry>> {
        self.file_cache.get(file).map(|entry| &entry.entries)
    }

    /// Update cached project list
    pub fn update_projects(&mut self, projects: Vec<ProjectData>) {
        self.cached_projects = projects;
        self.last_dir_scan = Some(Instant::now());
    }

    /// Get cached project list
    pub fn get_projects(&self) -> &[ProjectData] {
        &self.cached_projects
    }

    /// Mark full refresh completed
    pub fn mark_full_refresh(&mut self) {
        self.last_full_refresh = Some(Instant::now());
    }

    /// Check if there are any file changes without processing
    pub fn has_changes(&self, custom_path: Option<&str>) -> bool {
        // If cache is empty, there are changes (need initial load)
        if self.is_empty() {
            return true;
        }

        // Get current files
        let projects = match list_projects(custom_path) {
            Ok(p) => p,
            Err(_) => return false,
        };

        let all_files: Vec<PathBuf> = projects
            .iter()
            .flat_map(|p| p.session_files.iter().cloned())
            .collect();

        // Check for changes
        match self.check_file_changes(&all_files) {
            Ok(changes) => {
                !changes.modified.is_empty()
                    || !changes.new_files.is_empty()
                    || !changes.deleted.is_empty()
            }
            Err(_) => false,
        }
    }

    /// Perform incremental load and return delta (only changed data)
    pub fn incremental_load_with_delta(
        &mut self,
        custom_path: Option<&str>,
        pricing: &PricingCalculator,
    ) -> Result<(UsageData, UsageDataDelta), ReaderError> {
        // If cache is empty, do full load
        if self.is_empty() {
            let data = self.full_load(custom_path, pricing)?;
            let delta = UsageDataDelta {
                has_changes: true,
                full_refresh: true,
                updated_projects: data.projects.clone(),
                overall_stats: Some(data.overall_stats.clone()),
                daily_usage: Some(data.daily_usage.clone()),
            };
            return Ok((data, delta));
        }

        // Track which projects had changes
        let mut changed_project_paths: std::collections::HashSet<String> = std::collections::HashSet::new();

        // Check if we should rescan directories
        let projects = if self.should_rescan_dirs() {
            let new_projects = list_projects(custom_path)?;
            self.update_projects(
                new_projects
                    .iter()
                    .map(|p| ProjectData {
                        encoded_path: p.encoded_path.clone(),
                        decoded_path: p.decoded_path.clone(),
                        display_name: p.display_name.clone(),
                        session_files: p.session_files.clone(),
                    })
                    .collect(),
            );
            new_projects
        } else {
            list_projects(custom_path)?
        };

        // Collect all current files
        let all_files: Vec<PathBuf> = projects
            .iter()
            .flat_map(|p| p.session_files.iter().cloned())
            .collect();

        // Check for changes
        let changes = self.check_file_changes(&all_files)?;

        // Track which project paths had file changes
        for file in changes.modified.iter().chain(changes.new_files.iter()) {
            // Find which project this file belongs to
            for project in &projects {
                if project.session_files.contains(file) {
                    changed_project_paths.insert(project.decoded_path.clone());
                    break;
                }
            }
        }

        for deleted in &changes.deleted {
            // For deleted files, we need to check cached projects
            for project in self.get_projects() {
                if project.session_files.contains(deleted) {
                    changed_project_paths.insert(project.decoded_path.clone());
                    break;
                }
            }
            self.remove_file(deleted);
        }

        // Process modified and new files
        for file in changes.modified.iter().chain(changes.new_files.iter()) {
            match read_jsonl_file(file, pricing) {
                Ok(entries) => {
                    self.update_file_cache(file, entries)?;
                }
                Err(e) => {
                    log::warn!("Failed to read file {:?}: {}", file, e);
                }
            }
        }

        // Build usage data from cache
        let mut all_data: Vec<(ProjectData, Vec<UsageEntry>)> = Vec::new();

        for project in &projects {
            let mut project_entries = Vec::new();

            for session_file in &project.session_files {
                if let Some(entries) = self.get_file_entries(session_file) {
                    project_entries.extend(entries.clone());
                }
            }

            all_data.push((
                ProjectData {
                    encoded_path: project.encoded_path.clone(),
                    decoded_path: project.decoded_path.clone(),
                    display_name: project.display_name.clone(),
                    session_files: project.session_files.clone(),
                },
                project_entries,
            ));
        }

        let data = calculate_usage_data(all_data)?;

        // Build delta with only changed projects
        let updated_projects: Vec<_> = data
            .projects
            .iter()
            .filter(|p| changed_project_paths.contains(&p.project_path))
            .cloned()
            .collect();

        let has_changes = !updated_projects.is_empty();

        let delta = UsageDataDelta {
            has_changes,
            full_refresh: false,
            updated_projects,
            overall_stats: if has_changes {
                Some(data.overall_stats.clone())
            } else {
                None
            },
            daily_usage: if has_changes {
                Some(data.daily_usage.clone())
            } else {
                None
            },
        };

        Ok((data, delta))
    }

    /// Perform full data load and populate cache
    pub fn full_load(
        &mut self,
        custom_path: Option<&str>,
        pricing: &PricingCalculator,
    ) -> Result<UsageData, ReaderError> {
        // Clear existing cache
        self.clear();

        // Load projects
        let projects = list_projects(custom_path)?;

        // Load all files and populate cache
        let mut all_data: Vec<(ProjectData, Vec<UsageEntry>)> = Vec::new();

        for project in projects {
            let mut project_entries = Vec::new();

            for session_file in &project.session_files {
                match read_jsonl_file(session_file, pricing) {
                    Ok(entries) => {
                        self.update_file_cache(session_file, entries.clone())?;
                        project_entries.extend(entries);
                    }
                    Err(e) => {
                        log::warn!("Failed to read session file {:?}: {}", session_file, e);
                    }
                }
            }

            all_data.push((project, project_entries));
        }

        // Update project cache
        let projects: Vec<ProjectData> = all_data.iter().map(|(p, _)| {
            ProjectData {
                encoded_path: p.encoded_path.clone(),
                decoded_path: p.decoded_path.clone(),
                display_name: p.display_name.clone(),
                session_files: p.session_files.clone(),
            }
        }).collect();
        self.update_projects(projects);
        self.mark_full_refresh();

        // Calculate statistics
        let data = calculate_usage_data(all_data)?;

        // Cache the result for quick access
        self.cached_usage_data = Some(data.clone());

        Ok(data)
    }

    /// Perform incremental load (only read changed files)
    pub fn incremental_load(
        &mut self,
        custom_path: Option<&str>,
        pricing: &PricingCalculator,
    ) -> Result<UsageData, ReaderError> {
        // If cache is empty, do full load
        if self.is_empty() {
            return self.full_load(custom_path, pricing);
        }

        // Check if we should rescan directories
        let projects = if self.should_rescan_dirs() {
            let new_projects = list_projects(custom_path)?;
            self.update_projects(new_projects.iter().map(|p| ProjectData {
                encoded_path: p.encoded_path.clone(),
                decoded_path: p.decoded_path.clone(),
                display_name: p.display_name.clone(),
                session_files: p.session_files.clone(),
            }).collect());
            new_projects
        } else {
            // Use cached projects but refresh session file list
            list_projects(custom_path)?
        };

        // Collect all current files
        let all_files: Vec<PathBuf> = projects
            .iter()
            .flat_map(|p| p.session_files.iter().cloned())
            .collect();

        // Check for changes
        let changes = self.check_file_changes(&all_files)?;

        // Process deleted files
        for deleted in &changes.deleted {
            self.remove_file(deleted);
        }

        // Process modified and new files
        for file in changes.modified.iter().chain(changes.new_files.iter()) {
            match read_jsonl_file(file, pricing) {
                Ok(entries) => {
                    self.update_file_cache(file, entries)?;
                }
                Err(e) => {
                    log::warn!("Failed to read file {:?}: {}", file, e);
                }
            }
        }

        // Build usage data from cache
        let mut all_data: Vec<(ProjectData, Vec<UsageEntry>)> = Vec::new();

        for project in &projects {
            let mut project_entries = Vec::new();

            for session_file in &project.session_files {
                if let Some(entries) = self.get_file_entries(session_file) {
                    project_entries.extend(entries.clone());
                }
            }

            all_data.push((
                ProjectData {
                    encoded_path: project.encoded_path.clone(),
                    decoded_path: project.decoded_path.clone(),
                    display_name: project.display_name.clone(),
                    session_files: project.session_files.clone(),
                },
                project_entries,
            ));
        }

        let data = calculate_usage_data(all_data)?;

        // Cache the result for quick access
        self.cached_usage_data = Some(data.clone());

        Ok(data)
    }
}

/// Session duration in minutes (5 hours)
const SESSION_DURATION_MINUTES: i64 = 300;

/// Session block for proportional burn rate calculation
#[derive(Debug)]
struct SessionBlock {
    start_time: chrono::DateTime<chrono::Utc>,
    actual_end_time: chrono::DateTime<chrono::Utc>,
    total_tokens: u64,
    total_cost: f64,
    is_active: bool,
}

/// Transform entries into session blocks (5-hour blocks starting at hour boundary)
fn transform_to_blocks(entries: &[UsageEntry]) -> Vec<SessionBlock> {
    use chrono::{Duration, Timelike, Utc};

    if entries.is_empty() {
        return Vec::new();
    }

    let mut blocks: Vec<SessionBlock> = Vec::new();
    let session_duration = Duration::hours(5);
    let mut current_block: Option<SessionBlock> = None;

    for entry in entries {
        let should_create_new = match &current_block {
            None => true,
            Some(block) => entry.timestamp >= block.start_time + session_duration,
        };

        if should_create_new {
            if let Some(block) = current_block.take() {
                blocks.push(block);
            }

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

        if let Some(ref mut block) = current_block {
            block.total_tokens += entry.input_tokens + entry.output_tokens;
            block.total_cost += entry.cost_usd;
            block.actual_end_time = entry.timestamp;
        }
    }

    if let Some(mut block) = current_block {
        let now = Utc::now();
        if block.start_time + session_duration > now {
            block.is_active = true;
        }
        blocks.push(block);
    }

    blocks
}

/// Calculate hourly burn rate using block-based proportional allocation
fn calculate_hourly_burn_rate(blocks: &[SessionBlock], current_time: &chrono::DateTime<chrono::Utc>) -> (f64, f64) {
    use chrono::Duration;

    if blocks.is_empty() {
        return (0.0, 0.0);
    }

    let one_hour_ago = *current_time - Duration::hours(1);
    let mut total_tokens: f64 = 0.0;
    let mut total_cost: f64 = 0.0;

    for block in blocks {
        let session_actual_end = if block.is_active {
            *current_time
        } else {
            block.actual_end_time
        };

        if session_actual_end < one_hour_ago {
            continue;
        }

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

        let total_session_duration = (session_actual_end - block.start_time).num_seconds() as f64 / 60.0;
        let hour_duration = (session_end_in_hour - session_start_in_hour).num_seconds() as f64 / 60.0;

        if total_session_duration > 0.0 {
            let proportion = hour_duration / total_session_duration;
            total_tokens += block.total_tokens as f64 * proportion;
            total_cost += block.total_cost * proportion;
        }
    }

    if total_tokens > 0.0 {
        (total_tokens / 60.0, total_cost / 60.0 * 60.0)
    } else {
        (0.0, 0.0)
    }
}

/// Calculate time to reset based on session start time
fn calculate_time_to_reset(session_start: Option<&chrono::DateTime<chrono::Utc>>, now: &chrono::DateTime<chrono::Utc>) -> u32 {
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
fn calculate_model_distribution(entries: &[UsageEntry]) -> Vec<crate::usage::models::ModelStats> {
    use std::collections::HashMap;
    use crate::usage::models::ModelStats;

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

/// Calculate UsageData from project entries (reuse logic from stats.rs)
fn calculate_usage_data(
    all_data: Vec<(ProjectData, Vec<UsageEntry>)>,
) -> Result<UsageData, ReaderError> {
    use std::collections::HashMap;
    use chrono::{Datelike, Duration, Local, Timelike, Utc};
    use crate::usage::models::{BurnRate, DailyUsage, OverallStats, ProjectStats, TodayStats};

    let mut all_entries: Vec<UsageEntry> = Vec::new();
    let mut projects: Vec<ProjectStats> = Vec::new();

    for (project, entries) in all_data {
        if entries.is_empty() {
            continue;
        }

        all_entries.extend(entries.clone());

        // Calculate project stats
        let mut stats = ProjectStats {
            project_path: project.decoded_path.clone(),
            display_name: project.display_name.clone(),
            session_count: project.session_files.len() as u32,
            ..Default::default()
        };

        for entry in &entries {
            stats.total_input_tokens += entry.input_tokens;
            stats.total_output_tokens += entry.output_tokens;
            stats.cache_creation_tokens += entry.cache_creation_tokens;
            stats.cache_read_tokens += entry.cache_read_tokens;
            stats.total_cost_usd += entry.cost_usd;
            stats.message_count += 1;

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

        stats.total_cost_usd = (stats.total_cost_usd * 1_000_000.0).round() / 1_000_000.0;
        projects.push(stats);
    }

    // Calculate daily usage
    let mut daily_map: HashMap<String, DailyUsage> = HashMap::new();

    for entry in &all_entries {
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

    let mut daily_usage: Vec<_> = daily_map
        .into_values()
        .map(|mut d| {
            d.cost_usd = (d.cost_usd * 1_000_000.0).round() / 1_000_000.0;
            d
        })
        .collect();
    daily_usage.sort_by(|a, b| a.date.cmp(&b.date));

    // Calculate overall stats
    let mut overall_stats = OverallStats {
        project_count: projects.len() as u32,
        ..Default::default()
    };

    for project in &projects {
        overall_stats.total_input_tokens += project.total_input_tokens;
        overall_stats.total_output_tokens += project.total_output_tokens;
        overall_stats.cache_creation_tokens += project.cache_creation_tokens;
        overall_stats.cache_read_tokens += project.cache_read_tokens;
        overall_stats.total_cost_usd += project.total_cost_usd;
        overall_stats.total_messages += project.message_count;
        overall_stats.total_sessions += project.session_count;
    }
    overall_stats.total_cost_usd = (overall_stats.total_cost_usd * 1_000_000.0).round() / 1_000_000.0;

    // Calculate model distribution
    overall_stats.model_distribution = calculate_model_distribution(&all_entries);

    // Calculate today's stats (since local midnight)
    let today_local = Local::now().date_naive();
    let mut today_stats = TodayStats::default();

    for entry in &all_entries {
        // Convert UTC timestamp to local date for comparison
        let entry_local_date = entry.timestamp.with_timezone(&Local).date_naive();
        if entry_local_date == today_local {
            today_stats.input_tokens += entry.input_tokens;
            today_stats.output_tokens += entry.output_tokens;
            today_stats.cost_usd += entry.cost_usd;
            today_stats.message_count += 1;
        }
    }
    today_stats.total_tokens = today_stats.input_tokens + today_stats.output_tokens;
    today_stats.cost_usd = (today_stats.cost_usd * 1_000_000.0).round() / 1_000_000.0;
    overall_stats.today_stats = today_stats;

    // Calculate session timing and burn rate (matches stats.rs logic)
    if !all_entries.is_empty() {
        let now = Utc::now();
        let window_start = now - Duration::minutes(SESSION_DURATION_MINUTES);

        // Sort entries by timestamp for proper processing
        all_entries.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        let recent_entries: Vec<_> = all_entries
            .iter()
            .filter(|e| e.timestamp >= window_start)
            .collect();

        if !recent_entries.is_empty() {
            let first_entry_time = recent_entries.iter().map(|e| e.timestamp).min().unwrap();

            let session_block_start = first_entry_time
                .with_minute(0).unwrap()
                .with_second(0).unwrap()
                .with_nanosecond(0).unwrap();

            overall_stats.session_start_time = Some(session_block_start.to_rfc3339());
            overall_stats.time_to_reset_minutes = calculate_time_to_reset(Some(&session_block_start), &now);

            // Calculate hourly burn rate using block-based proportional allocation
            let blocks = transform_to_blocks(&all_entries);
            let (tokens_per_min, cost_per_hour) = calculate_hourly_burn_rate(&blocks, &now);

            if tokens_per_min > 0.0 {
                overall_stats.burn_rate = Some(BurnRate {
                    tokens_per_minute: (tokens_per_min * 100.0).round() / 100.0,
                    cost_per_hour: (cost_per_hour * 10000.0).round() / 10000.0,
                });
            }
        } else {
            overall_stats.time_to_reset_minutes = SESSION_DURATION_MINUTES as u32;
        }
    } else {
        overall_stats.time_to_reset_minutes = SESSION_DURATION_MINUTES as u32;
    }

    // Sort projects by last activity
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
