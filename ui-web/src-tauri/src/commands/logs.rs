//! CloudWatch log commands.

use crate::aws;
use crate::state::AppState;
use tauri::State;

/// List CloudWatch log groups matching the AgentCore prefix.
#[tauri::command]
pub async fn list_log_groups(
    state: State<'_, AppState>,
    profile: String,
    region: String,
) -> Result<Vec<String>, String> {
    let log_groups = aws::list_log_groups(&profile, &region)?;

    // Update state cache
    *state.log_groups.lock().unwrap() = log_groups.clone();

    Ok(log_groups)
}

/// Fetch logs from a specific log group with optional filters.
#[tauri::command]
pub async fn get_logs(
    profile: String,
    region: String,
    log_group: String,
    stream_filter: Option<String>,
    filter_pattern: Option<String>,
) -> Result<(String, String), String> {
    // Returns (metrics, content)
    aws::get_logs(
        &profile,
        &region,
        &log_group,
        stream_filter.as_deref().unwrap_or(""),
        filter_pattern.as_deref().unwrap_or(""),
    )
}
