use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::types::{Agent, MemoryInfo, ModelInfo, SyncConfig};

/// Time-to-live for agent list cache (5 minutes).
const AGENTS_TTL: u64 = 300;
/// Time-to-live for model list cache (24 hours).
const MODELS_TTL: u64 = 86_400;
/// Time-to-live for memory list cache (5 minutes).
const MEMORIES_TTL: u64 = 300;

#[derive(Serialize, Deserialize)]
struct CacheEntry<T> {
    fetched_at: u64,
    data: T,
}

fn cache_dir() -> Option<PathBuf> {
    dirs::cache_dir().map(|d| d.join("agentcore-manager"))
}

fn cache_file(name: &str) -> Option<PathBuf> {
    cache_dir().map(|d| d.join(name))
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn read_cache<T: DeserializeOwned>(name: &str, ttl: u64) -> Option<T> {
    let path = cache_file(name)?;
    let bytes = fs::read(path).ok()?;
    let entry: CacheEntry<T> = serde_json::from_slice(&bytes).ok()?;
    if now_secs().saturating_sub(entry.fetched_at) > ttl {
        return None;
    }
    Some(entry.data)
}

fn write_cache<T: Serialize>(name: &str, data: &T) {
    let Some(path) = cache_file(name) else {
        return;
    };
    if let Some(dir) = path.parent() {
        let _ = fs::create_dir_all(dir);
    }
    let entry = CacheEntry {
        fetched_at: now_secs(),
        data,
    };
    if let Ok(json) = serde_json::to_string(&entry) {
        let _ = fs::write(path, json);
    }
}

pub fn read_agents() -> Option<Vec<Agent>> {
    read_cache("agents.json", AGENTS_TTL)
}

pub fn write_agents(agents: &[Agent]) {
    write_cache("agents.json", &agents);
}

pub fn read_models() -> Option<Vec<ModelInfo>> {
    read_cache("models.json", MODELS_TTL)
}

pub fn write_models(models: &[ModelInfo]) {
    write_cache("models.json", &models);
}

pub fn read_memories() -> Option<Vec<MemoryInfo>> {
    read_cache("memories.json", MEMORIES_TTL)
}

pub fn write_memories(memories: &[MemoryInfo]) {
    write_cache("memories.json", &memories);
}

/// User preferences — persisted indefinitely (no TTL).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserPrefs {
    pub selected_profile: String,
    pub selected_region: String,
    pub selected_model_id: String,
    pub selected_agent_id: String,
    #[serde(default)]
    pub selected_memory_id: String,
    /// ID of the last selected recipe. Empty = none.
    #[serde(default)]
    pub selected_recipe_id: String,
}

pub fn read_prefs() -> UserPrefs {
    let Some(path) = cache_file("prefs.json") else {
        return UserPrefs::default();
    };
    let Ok(bytes) = fs::read(path) else {
        return UserPrefs::default();
    };
    serde_json::from_slice(&bytes).unwrap_or_default()
}

pub fn write_prefs(prefs: &UserPrefs) {
    let Some(path) = cache_file("prefs.json") else {
        return;
    };
    if let Some(dir) = path.parent() {
        let _ = fs::create_dir_all(dir);
    }
    if let Ok(json) = serde_json::to_string(prefs) {
        let _ = fs::write(path, json);
    }
}

/// Read S3 sync configuration from cache.
pub fn read_sync_config() -> SyncConfig {
    let Some(path) = cache_file("sync_config.json") else {
        return SyncConfig::default();
    };
    let Ok(bytes) = fs::read(path) else {
        return SyncConfig::default();
    };
    serde_json::from_slice(&bytes).unwrap_or_default()
}

/// Write S3 sync configuration to cache.
pub fn write_sync_config(config: &SyncConfig) {
    let Some(path) = cache_file("sync_config.json") else {
        return;
    };
    if let Some(dir) = path.parent() {
        let _ = fs::create_dir_all(dir);
    }
    if let Ok(json) = serde_json::to_string(config) {
        let _ = fs::write(path, json);
    }
}
