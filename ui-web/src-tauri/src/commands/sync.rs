//! S3 sharing commands for recipes and prompts.
//! Simple team library: publish to share, browse & import from team.

use crate::aws::{RealCli, CliRunner};
use crate::registry;
use crate::state::AppState;
use crate::types::{PromptEntry, RecipeEntry, SyncConfig, SharedItem, VersionInfo};
use std::collections::HashMap;
use tauri::State;

/// Get the current sync configuration.
#[tauri::command]
pub async fn get_sync_config(state: State<'_, AppState>) -> Result<SyncConfig, String> {
    let config = state.sync_config.lock().unwrap();
    Ok(config.clone())
}

/// Set the sync configuration.
#[tauri::command]
pub async fn set_sync_config(
    state: State<'_, AppState>,
    config: SyncConfig,
) -> Result<(), String> {
    let mut current = state.sync_config.lock().unwrap();
    *current = config;
    crate::cache::write_sync_config(&current);
    Ok(())
}

/// Publish a recipe to the shared team bucket.
/// If version is empty, auto-increments from last published version.
#[tauri::command]
pub async fn publish_recipe(
    state: State<'_, AppState>,
    recipe_id: String,
    username: String,
    version: Option<String>,
) -> Result<(), String> {
    let config = state.sync_config.lock().unwrap().clone();
    if config.bucket.is_empty() {
        return Err("S3 bucket not configured. Go to Settings to configure.".to_string());
    }

    let mut recipe = registry::get_entry(&recipe_id)?
        .ok_or_else(|| format!("Recipe '{}' not found", recipe_id))?;

    // Compute content hash before publishing
    recipe.content_hash = registry::compute_recipe_hash(&recipe);

    // Set source_id for tracking (owner-id pattern)
    let source_id = format!("{}-{}", username, recipe.id);
    recipe.source_id = source_id.clone();

    // Determine version: use provided, auto-increment, or start at 1.0.0
    let final_version = match version {
        Some(v) if !v.is_empty() => v,
        _ => {
            // Find existing versions for this source_id
            let prefix = format!("{}recipes/{}-{}-", config.prefix, username, recipe.id);
            let keys = list_s3_objects(&config.profile, &config.region, &config.bucket, &prefix)?;
            let latest = find_latest_version(&keys);
            increment_version(&latest.unwrap_or_default())
        }
    };
    recipe.version = final_version.clone();

    // Upload with version in key: username-recipeid-version.json
    let key = format!("{}recipes/{}-{}-{}.json", config.prefix, username, recipe.id, final_version);
    let json = serde_json::to_string_pretty(&recipe)
        .map_err(|e| format!("Failed to serialize recipe: {e}"))?;

    upload_to_s3(&config.profile, &config.region, &config.bucket, &key, &json)?;

    // Also update local recipe with source_id and version
    registry::save_entry(&recipe, &format!("Published v{}", final_version))?;

    Ok(())
}

/// Publish a prompt to the shared team bucket.
/// If version is empty, auto-increments from last published version.
#[tauri::command]
pub async fn publish_prompt(
    state: State<'_, AppState>,
    prompt_id: String,
    username: String,
    version: Option<String>,
) -> Result<(), String> {
    let config = state.sync_config.lock().unwrap().clone();
    if config.bucket.is_empty() {
        return Err("S3 bucket not configured. Go to Settings to configure.".to_string());
    }

    let mut prompt = registry::get_prompt(&prompt_id)?
        .ok_or_else(|| format!("Prompt '{}' not found", prompt_id))?;

    // Compute content hash before publishing
    prompt.content_hash = registry::compute_prompt_hash(&prompt);

    // Set source_id for tracking
    let source_id = format!("{}-{}", username, prompt.id);
    prompt.source_id = source_id.clone();

    // Determine version
    let final_version = match version {
        Some(v) if !v.is_empty() => v,
        _ => {
            let prefix = format!("{}prompts/{}-{}-", config.prefix, username, prompt.id);
            let keys = list_s3_objects(&config.profile, &config.region, &config.bucket, &prefix)?;
            let latest = find_latest_version(&keys);
            increment_version(&latest.unwrap_or_default())
        }
    };
    prompt.version = final_version.clone();

    // Upload with version in key
    let key = format!("{}prompts/{}-{}-{}.json", config.prefix, username, prompt.id, final_version);
    let json = serde_json::to_string_pretty(&prompt)
        .map_err(|e| format!("Failed to serialize prompt: {e}"))?;

    upload_to_s3(&config.profile, &config.region, &config.bucket, &key, &json)?;

    // Update local with source_id and version
    registry::save_prompt(&prompt, &format!("Published v{}", final_version))?;

    Ok(())
}

/// List shared recipes from the team bucket (with full metadata).
/// Groups items by source_id and collects all available versions.
#[tauri::command]
pub async fn list_shared_recipes(
    state: State<'_, AppState>,
) -> Result<Vec<SharedItem>, String> {
    let config = state.sync_config.lock().unwrap().clone();
    if config.bucket.is_empty() {
        return Err("S3 bucket not configured".to_string());
    }

    let prefix = format!("{}recipes/", config.prefix);
    let keys = list_s3_objects(&config.profile, &config.region, &config.bucket, &prefix)?;

    // Group items by source_id
    let mut groups: HashMap<String, Vec<(String, RecipeEntry)>> = HashMap::new();

    for key in keys {
        if !key.ends_with(".json") {
            continue;
        }

        match download_from_s3(&config.profile, &config.region, &config.bucket, &key) {
            Ok(content) => {
                if let Ok(recipe) = serde_json::from_str::<RecipeEntry>(&content) {
                    // Derive source_id from key if not set in recipe
                    let source_id = if !recipe.source_id.is_empty() {
                        recipe.source_id.clone()
                    } else {
                        extract_source_id(&key).unwrap_or_else(|| format!("{}-{}", recipe.owner, recipe.id))
                    };
                    groups.entry(source_id).or_default().push((key.clone(), recipe));
                }
            }
            Err(e) => {
                eprintln!("Failed to download {}: {}", key, e);
            }
        }
    }

    // Build SharedItems from groups
    let mut items = Vec::new();
    for (source_id, mut versions) in groups {
        // Sort by version descending
        versions.sort_by(|(_, a), (_, b)| compare_versions(&b.version, &a.version));

        if let Some((latest_key, latest)) = versions.first() {
            let available_versions: Vec<VersionInfo> = versions
                .iter()
                .map(|(key, r)| VersionInfo {
                    version: r.version.clone(),
                    key: key.clone(),
                    updated_at: r.updated_at.clone(),
                    content_hash: r.content_hash.clone(),
                })
                .collect();

            items.push(SharedItem {
                key: latest_key.clone(),
                id: latest.id.clone(),
                name: latest.name.clone(),
                description: latest.description.clone(),
                owner: latest.owner.clone(),
                item_type: "recipe".to_string(),
                created_at: latest.created_at.clone(),
                updated_at: latest.updated_at.clone(),
                content_hash: latest.content_hash.clone(),
                version: latest.version.clone(),
                source_id,
                available_versions,
            });
        }
    }

    // Sort by updated_at descending (newest first)
    items.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(items)
}

/// List shared prompts from the team bucket (with full metadata).
/// Groups items by source_id and collects all available versions.
#[tauri::command]
pub async fn list_shared_prompts(
    state: State<'_, AppState>,
) -> Result<Vec<SharedItem>, String> {
    let config = state.sync_config.lock().unwrap().clone();
    if config.bucket.is_empty() {
        return Err("S3 bucket not configured".to_string());
    }

    let prefix = format!("{}prompts/", config.prefix);
    let keys = list_s3_objects(&config.profile, &config.region, &config.bucket, &prefix)?;

    // Group items by source_id
    let mut groups: HashMap<String, Vec<(String, PromptEntry)>> = HashMap::new();

    for key in keys {
        if !key.ends_with(".json") {
            continue;
        }

        match download_from_s3(&config.profile, &config.region, &config.bucket, &key) {
            Ok(content) => {
                if let Ok(prompt) = serde_json::from_str::<PromptEntry>(&content) {
                    let source_id = if !prompt.source_id.is_empty() {
                        prompt.source_id.clone()
                    } else {
                        extract_source_id(&key).unwrap_or_else(|| format!("{}-{}", prompt.owner, prompt.id))
                    };
                    groups.entry(source_id).or_default().push((key.clone(), prompt));
                }
            }
            Err(e) => {
                eprintln!("Failed to download {}: {}", key, e);
            }
        }
    }

    // Build SharedItems from groups
    let mut items = Vec::new();
    for (source_id, mut versions) in groups {
        versions.sort_by(|(_, a), (_, b)| compare_versions(&b.version, &a.version));

        if let Some((latest_key, latest)) = versions.first() {
            let available_versions: Vec<VersionInfo> = versions
                .iter()
                .map(|(key, p)| VersionInfo {
                    version: p.version.clone(),
                    key: key.clone(),
                    updated_at: p.updated_at.clone(),
                    content_hash: p.content_hash.clone(),
                })
                .collect();

            items.push(SharedItem {
                key: latest_key.clone(),
                id: latest.id.clone(),
                name: latest.name.clone(),
                description: String::new(),
                owner: latest.owner.clone(),
                item_type: "prompt".to_string(),
                created_at: latest.created_at.clone(),
                updated_at: latest.updated_at.clone(),
                content_hash: latest.content_hash.clone(),
                version: latest.version.clone(),
                source_id,
                available_versions,
            });
        }
    }

    // Sort by updated_at descending (newest first)
    items.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(items)
}

/// Import a shared recipe to local registry.
/// If updating an existing import, use the same local ID.
/// Also imports the referenced prompt if not already present locally.
#[tauri::command]
pub async fn import_shared_recipe(
    state: State<'_, AppState>,
    key: String,
) -> Result<RecipeEntry, String> {
    let config = state.sync_config.lock().unwrap().clone();
    if config.bucket.is_empty() {
        return Err("S3 bucket not configured".to_string());
    }

    let content = download_from_s3(&config.profile, &config.region, &config.bucket, &key)?;
    let mut recipe: RecipeEntry = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse recipe: {e}"))?;

    // Ensure source_id is set from key if not in the recipe
    if recipe.source_id.is_empty() {
        recipe.source_id = extract_source_id(&key)
            .unwrap_or_else(|| format!("{}-{}", recipe.owner, recipe.id));
    }

    // Auto-import referenced prompt if needed
    if !recipe.prompt_id.is_empty() {
        let has_prompt_locally = registry::get_prompt(&recipe.prompt_id)?.is_some();
        if !has_prompt_locally {
            // Search shared prompts for one with matching ID
            if let Some(prompt_key) = find_shared_prompt_by_id(&config, &recipe.prompt_id)? {
                let prompt_content = download_from_s3(&config.profile, &config.region, &config.bucket, &prompt_key)?;
                if let Ok(mut prompt) = serde_json::from_str::<PromptEntry>(&prompt_content) {
                    if prompt.source_id.is_empty() {
                        prompt.source_id = extract_source_id(&prompt_key)
                            .unwrap_or_else(|| format!("{}-{}", prompt.owner, prompt.id));
                    }
                    registry::save_prompt(&prompt, &format!("Auto-imported with recipe: {}", recipe.name))?;
                }
            }
        }
    }

    // Check if we have a local item with this source_id (update scenario)
    let existing = registry::list_entries()?
        .into_iter()
        .find(|r| r.source_id == recipe.source_id);

    if let Some(local) = existing {
        // Update existing local recipe, keep local ID
        recipe.id = local.id;
        registry::save_entry(&recipe, &format!("Updated to v{}: {}", recipe.version, recipe.name))?;
    } else {
        // Check for ID collision
        if registry::get_entry(&recipe.id)?.is_some() {
            return Err(format!("Recipe '{}' already exists locally with different source", recipe.id));
        }
        registry::save_entry(&recipe, &format!("Imported v{}: {}", recipe.version, recipe.name))?;
    }

    Ok(recipe)
}

/// Import a shared prompt to local registry.
/// If updating an existing import, use the same local ID.
#[tauri::command]
pub async fn import_shared_prompt(
    state: State<'_, AppState>,
    key: String,
) -> Result<PromptEntry, String> {
    let config = state.sync_config.lock().unwrap().clone();
    if config.bucket.is_empty() {
        return Err("S3 bucket not configured".to_string());
    }

    let content = download_from_s3(&config.profile, &config.region, &config.bucket, &key)?;
    let mut prompt: PromptEntry = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse prompt: {e}"))?;

    // Ensure source_id is set
    if prompt.source_id.is_empty() {
        prompt.source_id = extract_source_id(&key)
            .unwrap_or_else(|| format!("{}-{}", prompt.owner, prompt.id));
    }

    // Check for existing import with this source_id
    let existing = registry::list_prompts()?
        .into_iter()
        .find(|p| p.source_id == prompt.source_id);

    if let Some(local) = existing {
        // Update existing, keep local ID
        prompt.id = local.id;
        registry::save_prompt(&prompt, &format!("Updated to v{}: {}", prompt.version, prompt.name))?;
    } else {
        if registry::get_prompt(&prompt.id)?.is_some() {
            return Err(format!("Prompt '{}' already exists locally with different source", prompt.id));
        }
        registry::save_prompt(&prompt, &format!("Imported v{}: {}", prompt.version, prompt.name))?;
    }

    Ok(prompt)
}

/// List available S3 buckets for the current profile.
#[tauri::command]
pub async fn list_sync_buckets(
    profile: String,
    region: String,
) -> Result<Vec<String>, String> {
    crate::aws::list_s3_buckets(&profile, &region)
}

/// Check for available updates to imported items.
/// Returns a list of source_ids that have newer versions available.
/// Call this on app startup to show update notifications.
#[tauri::command]
pub async fn check_for_updates(
    state: State<'_, AppState>,
) -> Result<Vec<UpdateAvailable>, String> {
    let config = state.sync_config.lock().unwrap().clone();
    if config.bucket.is_empty() {
        return Ok(Vec::new()); // No sync configured, no updates
    }

    // Get local items with source_ids
    let local_recipes: Vec<_> = registry::list_entries()?
        .into_iter()
        .filter(|r| !r.source_id.is_empty())
        .collect();
    let local_prompts: Vec<_> = registry::list_prompts()?
        .into_iter()
        .filter(|p| !p.source_id.is_empty())
        .collect();

    let mut updates = Vec::new();

    // Check recipes
    let recipe_prefix = format!("{}recipes/", config.prefix);
    let recipe_keys = list_s3_objects(&config.profile, &config.region, &config.bucket, &recipe_prefix)?;

    for local in &local_recipes {
        // Find matching keys by source_id prefix
        let matching_keys: Vec<_> = recipe_keys.iter()
            .filter(|k| {
                extract_source_id(k).map(|sid| sid == local.source_id).unwrap_or(false)
            })
            .collect();

        if let Some(latest_version) = find_latest_version(&matching_keys.iter().map(|s| s.to_string()).collect::<Vec<_>>()) {
            if compare_versions(&latest_version, &local.version) == std::cmp::Ordering::Greater {
                updates.push(UpdateAvailable {
                    item_type: "recipe".to_string(),
                    item_id: local.id.clone(),
                    item_name: local.name.clone(),
                    source_id: local.source_id.clone(),
                    local_version: local.version.clone(),
                    latest_version,
                });
            }
        }
    }

    // Check prompts
    let prompt_prefix = format!("{}prompts/", config.prefix);
    let prompt_keys = list_s3_objects(&config.profile, &config.region, &config.bucket, &prompt_prefix)?;

    for local in &local_prompts {
        let matching_keys: Vec<_> = prompt_keys.iter()
            .filter(|k| {
                extract_source_id(k).map(|sid| sid == local.source_id).unwrap_or(false)
            })
            .collect();

        if let Some(latest_version) = find_latest_version(&matching_keys.iter().map(|s| s.to_string()).collect::<Vec<_>>()) {
            if compare_versions(&latest_version, &local.version) == std::cmp::Ordering::Greater {
                updates.push(UpdateAvailable {
                    item_type: "prompt".to_string(),
                    item_id: local.id.clone(),
                    item_name: local.name.clone(),
                    source_id: local.source_id.clone(),
                    local_version: local.version.clone(),
                    latest_version,
                });
            }
        }
    }

    Ok(updates)
}

/// Info about an available update for a local item.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpdateAvailable {
    pub item_type: String,
    pub item_id: String,
    pub item_name: String,
    pub source_id: String,
    pub local_version: String,
    pub latest_version: String,
}

// ── Internal S3 helpers ─────────────────────────────────────────────────────

fn list_s3_objects(
    profile: &str,
    region: &str,
    bucket: &str,
    prefix: &str,
) -> Result<Vec<String>, String> {
    let cli = RealCli;
    let out = cli.run(
        "aws",
        &[
            "s3api",
            "list-objects-v2",
            "--bucket",
            bucket,
            "--prefix",
            prefix,
            "--query",
            "Contents[].Key",
            "--profile",
            profile,
            "--region",
            region,
            "--output",
            "json",
        ],
    )?;

    if !out.success {
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        return Err(format!("Failed to list S3 objects: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    if stdout.trim() == "null" || stdout.trim().is_empty() {
        return Ok(Vec::new());
    }

    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .map_err(|e| format!("Failed to parse S3 list response: {e}"))?;

    let keys = parsed
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    Ok(keys)
}

fn upload_to_s3(
    profile: &str,
    region: &str,
    bucket: &str,
    key: &str,
    content: &str,
) -> Result<(), String> {
    let temp_path = std::env::temp_dir().join("agentcore-upload.json");
    std::fs::write(&temp_path, content)
        .map_err(|e| format!("Failed to write temp file: {e}"))?;

    let s3_uri = format!("s3://{}/{}", bucket, key);
    let cli = RealCli;
    let out = cli.run(
        "aws",
        &[
            "s3",
            "cp",
            temp_path.to_str().unwrap(),
            &s3_uri,
            "--profile",
            profile,
            "--region",
            region,
        ],
    )?;

    let _ = std::fs::remove_file(&temp_path);

    if !out.success {
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        return Err(format!("S3 upload failed: {}", stderr));
    }

    Ok(())
}

fn download_from_s3(
    profile: &str,
    region: &str,
    bucket: &str,
    key: &str,
) -> Result<String, String> {
    let temp_path = std::env::temp_dir().join("agentcore-download.json");
    let s3_uri = format!("s3://{}/{}", bucket, key);

    let cli = RealCli;
    let out = cli.run(
        "aws",
        &[
            "s3",
            "cp",
            &s3_uri,
            temp_path.to_str().unwrap(),
            "--profile",
            profile,
            "--region",
            region,
        ],
    )?;

    if !out.success {
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        return Err(format!("S3 download failed: {}", stderr));
    }

    let content = std::fs::read_to_string(&temp_path)
        .map_err(|e| format!("Failed to read downloaded file: {e}"))?;
    let _ = std::fs::remove_file(&temp_path);

    Ok(content)
}

// ── Version helpers ─────────────────────────────────────────────────────────

/// Parse version from S3 key (e.g., "prefix/recipes/alice-myid-1.2.0.json" -> "1.2.0")
fn parse_version_from_key(key: &str) -> Option<String> {
    let filename = key.rsplit('/').next()?;
    let stem = filename.strip_suffix(".json")?;
    // Find last dash and extract version
    let version_start = stem.rfind('-')?;
    let version = &stem[version_start + 1..];
    // Validate it looks like semver (x.y.z)
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() >= 2 && parts.iter().all(|p| p.parse::<u32>().is_ok()) {
        Some(version.to_string())
    } else {
        None
    }
}

/// Find the latest version from a list of S3 keys
fn find_latest_version(keys: &[String]) -> Option<String> {
    keys.iter()
        .filter_map(|k| parse_version_from_key(k))
        .max_by(|a, b| compare_versions(a, b))
}

/// Compare two semver strings (e.g., "1.2.0" vs "1.10.0")
fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
    let parse = |v: &str| -> Vec<u32> {
        v.split('.').filter_map(|p| p.parse().ok()).collect()
    };
    let va = parse(a);
    let vb = parse(b);
    va.cmp(&vb)
}

/// Increment a semver version (patch bump by default)
fn increment_version(version: &str) -> String {
    if version.is_empty() {
        return "1.0.0".to_string();
    }
    let parts: Vec<u32> = version
        .split('.')
        .filter_map(|p| p.parse().ok())
        .collect();
    match parts.as_slice() {
        [major, minor, patch] => format!("{}.{}.{}", major, minor, patch + 1),
        [major, minor] => format!("{}.{}.1", major, minor),
        [major] => format!("{}.0.1", major),
        _ => "1.0.0".to_string(),
    }
}

/// Extract source_id from S3 key (e.g., "prefix/recipes/alice-myid-1.0.0.json" -> "alice-myid")
fn extract_source_id(key: &str) -> Option<String> {
    let filename = key.rsplit('/').next()?;
    let stem = filename.strip_suffix(".json")?;
    // Remove version suffix
    let version_start = stem.rfind('-')?;
    Some(stem[..version_start].to_string())
}

/// Find a shared prompt by its ID (searches S3 for latest version).
fn find_shared_prompt_by_id(config: &SyncConfig, prompt_id: &str) -> Result<Option<String>, String> {
    let prefix = format!("{}prompts/", config.prefix);
    let keys = list_s3_objects(&config.profile, &config.region, &config.bucket, &prefix)?;

    // Download each prompt and check if ID matches
    for key in keys {
        if !key.ends_with(".json") {
            continue;
        }
        match download_from_s3(&config.profile, &config.region, &config.bucket, &key) {
            Ok(content) => {
                if let Ok(prompt) = serde_json::from_str::<PromptEntry>(&content) {
                    if prompt.id == prompt_id {
                        return Ok(Some(key));
                    }
                }
            }
            Err(_) => continue,
        }
    }
    Ok(None)
}
