//! Cache and user preferences commands.

use crate::cache as cache_module;
use crate::cache::UserPrefs;
use crate::state::AppState;
use tauri::State;

/// Read user preferences from disk cache.
#[tauri::command]
pub async fn read_prefs(state: State<'_, AppState>) -> Result<UserPrefs, String> {
    let prefs = cache_module::read_prefs();

    // Update state cache
    *state.user_prefs.lock().unwrap() = prefs.clone();

    Ok(prefs)
}

/// Write user preferences to disk cache.
#[tauri::command]
pub async fn write_prefs(state: State<'_, AppState>, prefs: UserPrefs) -> Result<(), String> {
    cache_module::write_prefs(&prefs);

    // Update state cache
    *state.user_prefs.lock().unwrap() = prefs;

    Ok(())
}
