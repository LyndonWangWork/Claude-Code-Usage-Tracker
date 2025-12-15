//! JSONL file reading and parsing

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use glob::glob;
use log::{debug, warn};

use crate::usage::config::{decode_project_path, get_display_name, get_projects_dir};
use crate::usage::models::{SessionEvent, Usage, UsageEntry};
use crate::usage::pricing::PricingCalculator;

/// Error type for reader operations
#[derive(Debug, thiserror::Error)]
pub enum ReaderError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Directory not found: {0}")]
    DirNotFound(String),
    #[error("Invalid path: {0}")]
    InvalidPath(String),
}

/// Project with its sessions
#[derive(Debug)]
pub struct ProjectData {
    pub encoded_path: String,
    pub decoded_path: String,
    pub display_name: String,
    pub session_files: Vec<PathBuf>,
}

/// List all projects in the Claude data directory
pub fn list_projects(custom_path: Option<&str>) -> Result<Vec<ProjectData>, ReaderError> {
    let projects_dir = get_projects_dir(custom_path);

    if !projects_dir.exists() {
        return Err(ReaderError::DirNotFound(
            projects_dir.to_string_lossy().to_string(),
        ));
    }

    let mut projects = Vec::new();

    // Read all subdirectories in the projects folder
    for entry in fs::read_dir(&projects_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let encoded_path = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            let decoded_path = decode_project_path(&encoded_path);
            let display_name = get_display_name(&decoded_path);

            // Find all JSONL files in this project directory
            let pattern = path.join("*.jsonl");
            let session_files: Vec<PathBuf> = glob(pattern.to_string_lossy().as_ref())
                .map(|paths| paths.filter_map(Result::ok).collect())
                .unwrap_or_default();

            if !session_files.is_empty() {
                projects.push(ProjectData {
                    encoded_path,
                    decoded_path,
                    display_name,
                    session_files,
                });
            }
        }
    }

    Ok(projects)
}

/// Read all usage entries from a JSONL file
pub fn read_jsonl_file(
    path: &Path,
    pricing: &PricingCalculator,
) -> Result<Vec<UsageEntry>, ReaderError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    // Use HashMap to deduplicate by message.id, keeping the last entry
    let mut entries_by_id: HashMap<String, UsageEntry> = HashMap::new();

    for (line_num, line_result) in reader.lines().enumerate() {
        let line = match line_result {
            Ok(l) => l,
            Err(e) => {
                debug!("Failed to read line {} in {:?}: {}", line_num, path, e);
                continue;
            }
        };

        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        match serde_json::from_str::<SessionEvent>(line) {
            Ok(event) => {
                if let Some(entry) = process_event(&event, pricing) {
                    // Get unique key - only deduplicate if BOTH message_id and request_id present
                    // Python: return f"{message_id}:{request_id}" if message_id and request_id else None
                    // Entries without both IDs are NOT deduplicated (all included)
                    if let Some(key) = get_dedup_key(&event) {
                        // Has valid dedup key - use HashMap to keep last entry
                        entries_by_id.insert(key, entry);
                    } else {
                        // No dedup key - include entry directly (matches Python behavior)
                        // Use a unique key to prevent any deduplication
                        let unique_key = format!("no_dedup_{}_{}", line_num, entry.timestamp);
                        entries_by_id.insert(unique_key, entry);
                    }
                }
            }
            Err(e) => {
                debug!(
                    "Failed to parse JSON at line {} in {:?}: {}",
                    line_num, path, e
                );
            }
        }
    }

    Ok(entries_by_id.into_values().collect())
}

/// Process a session event into a usage entry
fn process_event(
    event: &SessionEvent,
    pricing: &PricingCalculator,
) -> Option<UsageEntry> {
    // Parse timestamp
    let timestamp = parse_timestamp(event.timestamp.as_deref()?)?;

    // Extract tokens based on event type priority
    let (tokens, model) = extract_tokens_and_model(event)?;

    // Calculate cost
    let cost_usd = event.cost.unwrap_or_else(|| {
        pricing.calculate_cost(
            &model,
            tokens.input_tokens.unwrap_or(0),
            tokens.output_tokens.unwrap_or(0),
            tokens.cache_creation_tokens.unwrap_or(0),
            tokens.cache_read_tokens.unwrap_or(0),
        )
    });

    let message_id = event
        .message_id
        .clone()
        .or_else(|| event.message.as_ref()?.id.clone())
        .unwrap_or_default();

    let request_id = event.request_id.clone().unwrap_or_else(|| "unknown".to_string());

    Some(UsageEntry {
        timestamp,
        input_tokens: tokens.input_tokens.unwrap_or(0),
        output_tokens: tokens.output_tokens.unwrap_or(0),
        cache_creation_tokens: tokens.cache_creation_tokens.unwrap_or(0),
        cache_read_tokens: tokens.cache_read_tokens.unwrap_or(0),
        cost_usd,
        model,
        message_id,
        request_id,
    })
}

/// Extract tokens and model from event based on type priority
fn extract_tokens_and_model(event: &SessionEvent) -> Option<(Usage, String)> {
    let is_assistant = event.event_type.as_deref() == Some("assistant");

    // Get token sources in priority order based on event type
    let token_sources: Vec<Option<&Usage>> = if is_assistant {
        vec![
            event.message.as_ref().and_then(|m| m.usage.as_ref()),
            event.usage.as_ref(),
        ]
    } else {
        vec![
            event.usage.as_ref(),
            event.message.as_ref().and_then(|m| m.usage.as_ref()),
        ]
    };

    // Find first valid token source
    for source in token_sources.into_iter().flatten() {
        let has_tokens = source.input_tokens.unwrap_or(0) > 0
            || source.output_tokens.unwrap_or(0) > 0;

        if has_tokens {
            let model = extract_model(event);
            return Some((source.clone(), model));
        }
    }

    None
}

/// Extract model name from event
fn extract_model(event: &SessionEvent) -> String {
    // Try various locations for model name
    event
        .message
        .as_ref()
        .and_then(|m| m.model.clone())
        .unwrap_or_else(|| "claude-3-5-sonnet".to_string())
}

/// Parse ISO timestamp to DateTime<Utc>
fn parse_timestamp(ts: &str) -> Option<DateTime<Utc>> {
    // Handle 'Z' suffix
    let ts = if ts.ends_with('Z') {
        &ts[..ts.len() - 1]
    } else {
        ts
    };

    // Try parsing with various formats
    DateTime::parse_from_rfc3339(&format!("{}+00:00", ts))
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
        .or_else(|| {
            chrono::NaiveDateTime::parse_from_str(ts, "%Y-%m-%dT%H:%M:%S%.f")
                .ok()
                .map(|ndt| ndt.and_utc())
        })
        .or_else(|| {
            chrono::NaiveDateTime::parse_from_str(ts, "%Y-%m-%dT%H:%M:%S")
                .ok()
                .map(|ndt| ndt.and_utc())
        })
}

/// Get deduplication key for an event
/// Uses message_id:request_id format like Python version for global deduplication
/// Python only deduplicates when BOTH message_id AND request_id are present
fn get_dedup_key(event: &SessionEvent) -> Option<String> {
    // Get message_id: prefer message.id, fallback to top-level message_id
    let message_id = event
        .message
        .as_ref()
        .and_then(|m| m.id.clone())
        .or_else(|| event.message_id.clone());

    // Get request_id: prefer requestId, fallback to request_id
    let request_id = event
        .request_id
        .clone()
        .or_else(|| event.request_id.clone());

    // Create composite key like Python: only when BOTH are present
    // Python: return f"{message_id}:{request_id}" if message_id and request_id else None
    match (message_id, request_id) {
        (Some(mid), Some(rid)) => Some(format!("{}:{}", mid, rid)),
        _ => None, // Don't deduplicate if either is missing (match Python behavior)
    }
}

/// Load all usage entries from a project with global deduplication
/// Python only deduplicates when BOTH message_id AND request_id are non-empty
pub fn load_project_entries(
    project: &ProjectData,
    pricing: &PricingCalculator,
) -> Vec<UsageEntry> {
    // Use HashMap to deduplicate across all session files
    let mut entries_by_key: HashMap<String, UsageEntry> = HashMap::new();
    let mut entry_counter: usize = 0;

    for session_file in &project.session_files {
        match read_jsonl_file(session_file, pricing) {
            Ok(entries) => {
                for entry in entries {
                    // Python only deduplicates when BOTH message_id and request_id are present
                    // Python: return f"{message_id}:{request_id}" if message_id and request_id else None
                    let has_message_id = !entry.message_id.is_empty();
                    let has_request_id = !entry.request_id.is_empty() && entry.request_id != "unknown";

                    let key = if has_message_id && has_request_id {
                        format!("{}:{}", entry.message_id, entry.request_id)
                    } else {
                        // No deduplication - use unique key
                        entry_counter += 1;
                        format!("no_dedup_{}_{}", entry_counter, entry.timestamp)
                    };

                    // Keep the later entry (last one has final token counts)
                    entries_by_key.insert(key, entry);
                }
            }
            Err(e) => {
                warn!("Failed to read session file {:?}: {}", session_file, e);
            }
        }
    }

    // Convert to vector and sort by timestamp
    let mut entries: Vec<_> = entries_by_key.into_values().collect();
    entries.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

    entries
}

/// Load all usage entries from all projects
pub fn load_all_entries(
    custom_path: Option<&str>,
    pricing: &PricingCalculator,
) -> Result<Vec<(ProjectData, Vec<UsageEntry>)>, ReaderError> {
    let projects = list_projects(custom_path)?;

    let results: Vec<_> = projects
        .into_iter()
        .map(|project| {
            let entries = load_project_entries(&project, pricing);
            (project, entries)
        })
        .collect();

    Ok(results)
}
