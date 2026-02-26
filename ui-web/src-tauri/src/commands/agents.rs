//! Agent runtime management commands.

use crate::aws;
use crate::cache;
use crate::state::AppState;
use crate::types::{Agent, AgentRuntimeDetail, CreateAgentRuntimeParams, InvokeParams};
use tauri::State;

/// List all AgentCore Runtime agents.
#[tauri::command]
pub async fn list_agents(
    state: State<'_, AppState>,
    profile: String,
    region: String,
) -> Result<Vec<Agent>, String> {
    let agents = aws::list_agents(&profile, &region)?;

    // Update state cache
    *state.agents.lock().unwrap() = agents.clone();

    // Persist to disk cache
    cache::write_agents(&agents);

    Ok(agents)
}

/// Get detailed information about a specific agent runtime.
#[tauri::command]
pub async fn get_agent_runtime(
    profile: String,
    region: String,
    agent_id: String,
) -> Result<AgentRuntimeDetail, String> {
    aws::get_agent_runtime(&profile, &region, &agent_id)
}

/// Create a new AgentCore Runtime.
#[tauri::command]
pub async fn create_agent_runtime(
    params: CreateAgentRuntimeParams,
) -> Result<AgentRuntimeDetail, String> {
    aws::create_agent_runtime(&params)
}

/// Update an existing AgentCore Runtime.
#[tauri::command]
pub async fn update_agent_runtime(
    params: CreateAgentRuntimeParams,
) -> Result<AgentRuntimeDetail, String> {
    aws::update_agent_runtime(&params)
}

/// Delete an AgentCore Runtime.
#[tauri::command]
pub async fn delete_agent_runtime(
    profile: String,
    region: String,
    agent_id: String,
) -> Result<(), String> {
    aws::delete_agent_runtime(&profile, &region, &agent_id)
}

/// Invoke an agent runtime and return the response.
#[tauri::command]
pub async fn invoke_agent(
    params: InvokeParams,
) -> Result<(String, String, String), String> {
    // Returns (thinking, response, debug_log)
    aws::invoke_agent(&params)
}

/// List available versions/qualifiers for a specific agent runtime.
#[tauri::command]
pub async fn list_agent_versions(
    profile: String,
    region: String,
    agent_id: String,
) -> Result<Vec<String>, String> {
    aws::list_agent_runtime_versions(&profile, &region, &agent_id)
}
