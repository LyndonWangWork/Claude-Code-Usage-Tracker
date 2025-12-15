//! Data models for Claude Code usage monitoring

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Usage data from a single JSONL event
#[derive(Debug, Clone, Deserialize)]
pub struct SessionEvent {
    #[serde(rename = "type")]
    pub event_type: Option<String>,
    pub message: Option<Message>,
    pub timestamp: Option<String>,
    #[serde(alias = "costUSD", alias = "cost_usd")]
    pub cost: Option<f64>,
    pub usage: Option<Usage>,
    pub message_id: Option<String>,
    #[serde(alias = "requestId")]
    pub request_id: Option<String>,
    /// Unique identifier for each JSONL record
    pub uuid: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Message {
    pub role: Option<String>,
    pub content: Option<serde_json::Value>,
    pub id: Option<String>,
    pub model: Option<String>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Usage {
    #[serde(default, alias = "inputTokens", alias = "prompt_tokens")]
    pub input_tokens: Option<u64>,
    #[serde(default, alias = "outputTokens", alias = "completion_tokens")]
    pub output_tokens: Option<u64>,
    #[serde(default, alias = "cache_creation_input_tokens", alias = "cacheCreationInputTokens")]
    pub cache_creation_tokens: Option<u64>,
    #[serde(default, alias = "cache_read_input_tokens", alias = "cacheReadInputTokens")]
    pub cache_read_tokens: Option<u64>,
}

/// Processed usage entry with extracted token counts
#[derive(Debug, Clone, Serialize)]
pub struct UsageEntry {
    pub timestamp: DateTime<Utc>,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cache_read_tokens: u64,
    pub cost_usd: f64,
    pub model: String,
    pub message_id: String,
    pub request_id: String,
}

/// Statistics for a single project
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProjectStats {
    pub project_path: String,
    pub display_name: String,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cache_read_tokens: u64,
    pub total_cost_usd: f64,
    pub message_count: u32,
    pub session_count: u32,
    pub first_activity: Option<String>,
    pub last_activity: Option<String>,
}

/// Daily usage statistics
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DailyUsage {
    pub date: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cache_read_tokens: u64,
    pub cost_usd: f64,
    pub message_count: u32,
}

/// Statistics for a specific model
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ModelStats {
    pub model: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cache_read_tokens: u64,
    pub total_tokens: u64,
    pub cost_usd: f64,
    pub message_count: u32,
    pub percentage: f64,
}

/// Burn rate metrics for current session
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BurnRate {
    pub tokens_per_minute: f64,
    pub cost_per_hour: f64,
}

/// Today's usage statistics (since local midnight)
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TodayStats {
    pub cost_usd: f64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
    pub message_count: u32,
}

/// Overall statistics across all projects
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct OverallStats {
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cache_read_tokens: u64,
    pub total_cost_usd: f64,
    pub total_messages: u32,
    pub total_sessions: u32,
    pub project_count: u32,
    // Advanced metrics
    pub model_distribution: Vec<ModelStats>,
    pub session_start_time: Option<String>,
    pub time_to_reset_minutes: u32,
    pub burn_rate: Option<BurnRate>,
    pub today_stats: TodayStats,
}

/// Complete usage data response
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UsageData {
    pub projects: Vec<ProjectStats>,
    pub daily_usage: Vec<DailyUsage>,
    pub overall_stats: OverallStats,
}

/// Incremental update payload for push notifications
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UsageDataDelta {
    /// Whether there are actual data changes (triggers animation)
    pub has_changes: bool,
    /// Whether frontend should do a full refresh
    pub full_refresh: bool,
    /// Projects that have been updated
    pub updated_projects: Vec<ProjectStats>,
    /// Updated overall statistics (if changed)
    pub overall_stats: Option<OverallStats>,
    /// Updated daily usage (if changed)
    pub daily_usage: Option<Vec<DailyUsage>>,
}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    #[serde(default = "default_data_path")]
    pub data_path: Option<String>,
    #[serde(default = "default_refresh_interval")]
    pub refresh_interval_seconds: u32,
    #[serde(default = "default_plan_type")]
    pub plan_type: String,
}

fn default_data_path() -> Option<String> {
    None
}

fn default_refresh_interval() -> u32 {
    300 // 5 minutes
}

fn default_plan_type() -> String {
    "pro".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            data_path: None,
            refresh_interval_seconds: 300,
            plan_type: "pro".to_string(),
        }
    }
}
