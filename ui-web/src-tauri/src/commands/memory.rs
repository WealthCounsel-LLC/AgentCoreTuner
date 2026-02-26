//! Memory management commands.

use crate::aws;
use crate::cache;
use crate::state::AppState;
use crate::types::{CreateMemoryParams, MemoryInfo, SessionSummary};
use tauri::State;

/// List all Bedrock AgentCore memories.
#[tauri::command]
pub async fn list_memories(
    state: State<'_, AppState>,
    profile: String,
    region: String,
) -> Result<Vec<MemoryInfo>, String> {
    let memories = aws::list_memories(&profile, &region)?;

    // Update state cache
    *state.memories.lock().unwrap() = memories.clone();

    // Persist to disk cache
    cache::write_memories(&memories);

    Ok(memories)
}

/// Create a new Bedrock AgentCore memory.
#[tauri::command]
pub async fn create_memory(params: CreateMemoryParams) -> Result<MemoryInfo, String> {
    aws::create_memory(&params)
}

/// Delete a Bedrock AgentCore memory.
#[tauri::command]
pub async fn delete_memory(
    profile: String,
    region: String,
    memory_id: String,
) -> Result<(), String> {
    aws::delete_memory(&profile, &region, &memory_id)
}

/// List sessions for an actor in a specific memory pool.
#[tauri::command]
pub async fn list_sessions(
    profile: String,
    region: String,
    memory_id: String,
    actor_id: String,
) -> Result<Vec<SessionSummary>, String> {
    aws::list_sessions(&profile, &region, &memory_id, &actor_id)
}
