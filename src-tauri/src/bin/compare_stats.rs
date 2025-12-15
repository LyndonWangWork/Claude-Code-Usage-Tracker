//! Compare CCM stats with Python version
//!
//! Run with: cargo run --bin compare_stats

use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use glob::glob;
use chrono::{Utc, Duration};

fn main() {
    // First, test deduplication on a specific file
    let test_file = PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "C:\\Users\\dev".to_string()))
        .join(".claude/projects/d--code-ccm/184a01eb-97fd-44a0-b5c0-b65888d3f215.jsonl");

    if test_file.exists() {
        let pricing = claude_code_usage_tracker_lib::usage::pricing::PricingCalculator::new();
        let entries = claude_code_usage_tracker_lib::usage::reader::read_jsonl_file(&test_file, &pricing).unwrap();

        let total_tokens: u64 = entries.iter()
            .map(|e| e.input_tokens + e.output_tokens + e.cache_creation_tokens + e.cache_read_tokens)
            .sum();

        println!("=== Single File Dedup Test ===");
        println!("File: {:?}", test_file.file_name());
        println!("Entries after dedup: {}", entries.len());
        println!("Total tokens after dedup: {}", total_tokens);
        println!("");
    }

    // Get projects dir
    let projects_dir = claude_code_usage_tracker_lib::usage::config::get_projects_dir(None);

    // Count raw lines vs parsed entries
    let mut total_jsonl_lines = 0u64;
    let mut total_valid_json = 0u64;
    let mut total_with_usage = 0u64;

    // Token counts before dedup
    let mut raw_tokens = 0u64;

    // Unique keys tracking (like Python's processed_hashes)
    let mut global_keys: HashSet<String> = HashSet::new();
    let mut dedup_entries = 0u64;
    let mut dedup_tokens = 0u64;

    // Session window (last 5 hours)
    let now = Utc::now();
    let session_start = now - Duration::minutes(300);
    let mut session_tokens = 0u64;
    let mut session_entries = 0u64;
    let mut session_first: Option<chrono::DateTime<Utc>> = None;
    let mut session_last: Option<chrono::DateTime<Utc>> = None;

    let pricing = claude_code_usage_tracker_lib::usage::pricing::PricingCalculator::new();

    // Process each project directory
    for entry in std::fs::read_dir(&projects_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let pattern = path.join("*.jsonl");
        let files: Vec<_> = glob(pattern.to_string_lossy().as_ref())
            .unwrap()
            .filter_map(Result::ok)
            .collect();

        for file_path in &files {
            // Count raw lines and tokens
            let file = File::open(file_path).unwrap();
            let reader = BufReader::new(file);
            for line in reader.lines() {
                let line = line.unwrap();
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                total_jsonl_lines += 1;

                if let Ok(event) = serde_json::from_str::<claude_code_usage_tracker_lib::usage::models::SessionEvent>(line) {
                    total_valid_json += 1;

                    // Get tokens from usage or message.usage
                    let (input, output, cache_create, cache_read) = get_tokens(&event);
                    let total = input + output + cache_create + cache_read;

                    if total > 0 {
                        total_with_usage += 1;
                        raw_tokens += total;
                    }
                }
            }

            // Get deduplicated entries using our reader
            let entries = claude_code_usage_tracker_lib::usage::reader::read_jsonl_file(file_path, &pricing).unwrap();
            for entry in &entries {
                // Use Python-style key: message_id:request_id
                let key = format!("{}:{}", entry.message_id, entry.request_id);

                if !global_keys.contains(&key) {
                    global_keys.insert(key);
                    dedup_entries += 1;
                    let total = entry.input_tokens + entry.output_tokens
                        + entry.cache_creation_tokens + entry.cache_read_tokens;
                    dedup_tokens += total;

                    // Check if in session window
                    if entry.timestamp >= session_start {
                        session_entries += 1;
                        session_tokens += total;

                        if session_first.is_none() || entry.timestamp < session_first.unwrap() {
                            session_first = Some(entry.timestamp);
                        }
                        if session_last.is_none() || entry.timestamp > session_last.unwrap() {
                            session_last = Some(entry.timestamp);
                        }
                    }
                }
            }
        }
    }

    println!("=== Raw Data Analysis ===");
    println!("Total JSONL lines: {}", total_jsonl_lines);
    println!("Valid JSON lines: {}", total_valid_json);
    println!("Lines with usage data: {}", total_with_usage);
    println!("Raw tokens (before dedup): {}", raw_tokens);
    println!("");
    println!("=== After Global Deduplication ===");
    println!("Unique entries: {}", dedup_entries);
    println!("Tokens after dedup: {}", dedup_tokens);
    println!("Dedup ratio: {:.2}% of original", (dedup_entries as f64 / total_with_usage as f64) * 100.0);
    println!("Token reduction: {:.2}x", raw_tokens as f64 / dedup_tokens as f64);
    println!("");
    println!("=== Session Window (last 5 hours) ===");
    println!("Entries in window: {}", session_entries);
    println!("Tokens in window: {}", session_tokens);

    if let (Some(first), Some(last)) = (session_first, session_last) {
        let duration_mins = (last - first).num_minutes() as f64;
        println!("Session duration: {:.1} minutes", duration_mins);

        if duration_mins >= 1.0 {
            let burn_rate = session_tokens as f64 / duration_mins;
            println!("Burn rate: {:.0} tokens/min", burn_rate);
        }
    }

    // Test stats.rs output (hour-aligned block approach like Python)
    println!("");
    println!("=== Stats.rs Output (Hour-Aligned Block) ===");
    let filter = claude_code_usage_tracker_lib::usage::stats::FilterOptions::new();
    match claude_code_usage_tracker_lib::usage::stats::get_usage_data(None, &filter) {
        Ok(data) => {
            println!("Total projects: {}", data.overall_stats.project_count);
            println!("Total messages: {}", data.overall_stats.total_messages);
            if let Some(start_time) = &data.overall_stats.session_start_time {
                println!("Session start time: {}", start_time);
            }
            println!("Time to reset: {} minutes", data.overall_stats.time_to_reset_minutes);
            if let Some(burn_rate) = &data.overall_stats.burn_rate {
                println!("Burn rate: {:.0} tokens/min", burn_rate.tokens_per_minute);
                println!("Cost per hour: ${:.4}", burn_rate.cost_per_hour);
            }
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}

fn get_tokens(event: &claude_code_usage_tracker_lib::usage::models::SessionEvent) -> (u64, u64, u64, u64) {
    // Try message.usage first for assistant events
    if let Some(msg) = &event.message {
        if let Some(usage) = &msg.usage {
            let input = usage.input_tokens.unwrap_or(0);
            let output = usage.output_tokens.unwrap_or(0);
            if input > 0 || output > 0 {
                return (
                    input,
                    output,
                    usage.cache_creation_tokens.unwrap_or(0),
                    usage.cache_read_tokens.unwrap_or(0),
                );
            }
        }
    }

    // Try top-level usage
    if let Some(usage) = &event.usage {
        return (
            usage.input_tokens.unwrap_or(0),
            usage.output_tokens.unwrap_or(0),
            usage.cache_creation_tokens.unwrap_or(0),
            usage.cache_read_tokens.unwrap_or(0),
        );
    }

    (0, 0, 0, 0)
}
