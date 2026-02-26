//! Model listing commands.

use crate::aws;
use crate::cache;
use crate::state::AppState;
use crate::types::ModelInfo;
use tauri::State;

/// List all available Bedrock foundation models and inference profiles.
#[tauri::command]
pub async fn list_models(
    state: State<'_, AppState>,
    profile: String,
    region: String,
) -> Result<Vec<ModelInfo>, String> {
    let models = aws::list_models(&profile, &region)?;

    // Update state cache
    *state.models.lock().unwrap() = models.clone();

    // Persist to disk cache
    cache::write_models(&models);

    Ok(models)
}
