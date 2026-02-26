//! Recipe management commands.

use crate::registry;
use crate::state::AppState;
use crate::types::RecipeEntry;
use tauri::State;

/// List all recipes from the git-backed registry.
#[tauri::command]
pub async fn list_recipes(state: State<'_, AppState>) -> Result<Vec<RecipeEntry>, String> {
    let recipes = registry::list_entries().map_err(|e| e.to_string())?;

    // Update state cache
    *state.recipes.lock().unwrap() = recipes.clone();

    Ok(recipes)
}

/// Get a specific recipe by ID.
#[tauri::command]
pub async fn get_recipe(id: String) -> Result<Option<RecipeEntry>, String> {
    registry::get_entry(&id).map_err(|e| e.to_string())
}

/// Save a recipe to the git-backed registry.
#[tauri::command]
pub async fn save_recipe(
    state: State<'_, AppState>,
    entry: RecipeEntry,
    message: Option<String>,
) -> Result<(), String> {
    let msg = message.as_deref().unwrap_or("Update recipe");
    registry::save_entry(&entry, msg).map_err(|e| e.to_string())?;

    // Refresh the cache
    let recipes = registry::list_entries().map_err(|e| e.to_string())?;
    *state.recipes.lock().unwrap() = recipes;

    Ok(())
}

/// Delete a recipe from the git-backed registry.
#[tauri::command]
pub async fn delete_recipe(state: State<'_, AppState>, id: String) -> Result<(), String> {
    registry::remove_entry(&id, "Delete recipe").map_err(|e| e.to_string())?;

    // Refresh the cache
    let recipes = registry::list_entries().map_err(|e| e.to_string())?;
    *state.recipes.lock().unwrap() = recipes;

    Ok(())
}

/// Get the version history of a recipe.
#[tauri::command]
pub async fn recipe_history(id: String) -> Result<Vec<String>, String> {
    registry::entry_history(&id).map_err(|e| e.to_string())
}
