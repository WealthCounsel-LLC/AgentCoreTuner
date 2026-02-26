//! Prompt management commands.

use crate::registry;
use crate::state::AppState;
use crate::types::PromptEntry;
use tauri::State;

/// List all prompts from the git-backed registry.
#[tauri::command]
pub async fn list_prompts(state: State<'_, AppState>) -> Result<Vec<PromptEntry>, String> {
    let prompts = registry::list_prompts().map_err(|e| e.to_string())?;

    // Update state cache
    *state.prompts.lock().unwrap() = prompts.clone();

    Ok(prompts)
}

/// Get a specific prompt by ID.
#[tauri::command]
pub async fn get_prompt(id: String) -> Result<Option<PromptEntry>, String> {
    registry::get_prompt(&id).map_err(|e| e.to_string())
}

/// Save a prompt to the git-backed registry.
#[tauri::command]
pub async fn save_prompt(
    state: State<'_, AppState>,
    entry: PromptEntry,
    message: Option<String>,
) -> Result<(), String> {
    let msg = message.as_deref().unwrap_or("Update prompt");
    registry::save_prompt(&entry, msg).map_err(|e| e.to_string())?;

    // Refresh the cache
    let prompts = registry::list_prompts().map_err(|e| e.to_string())?;
    *state.prompts.lock().unwrap() = prompts;

    Ok(())
}

/// Delete a prompt from the git-backed registry.
#[tauri::command]
pub async fn delete_prompt(state: State<'_, AppState>, id: String) -> Result<(), String> {
    registry::remove_prompt(&id, "Delete prompt").map_err(|e| e.to_string())?;

    // Refresh the cache
    let prompts = registry::list_prompts().map_err(|e| e.to_string())?;
    *state.prompts.lock().unwrap() = prompts;

    Ok(())
}

/// Get the version history of a prompt.
#[tauri::command]
pub async fn prompt_history(id: String) -> Result<Vec<String>, String> {
    registry::prompt_history(&id).map_err(|e| e.to_string())
}
