//! Application state managed by Tauri.

use std::sync::Mutex;

use crate::cache::UserPrefs;
use crate::credentials::CredentialManager;
use crate::types::{Agent, MemoryInfo, ModelInfo, PromptEntry, RecipeEntry, SyncConfig};

/// Application state shared across all Tauri commands.
/// Uses Mutex for interior mutability in a thread-safe way.
pub struct AppState {
    /// Cached list of AgentCore Runtime agents.
    pub agents: Mutex<Vec<Agent>>,
    /// Cached list of Bedrock foundation models and inference profiles.
    pub models: Mutex<Vec<ModelInfo>>,
    /// Cached list of Bedrock AgentCore memories.
    pub memories: Mutex<Vec<MemoryInfo>>,
    /// Cached list of CloudWatch log groups.
    pub log_groups: Mutex<Vec<String>>,
    /// Cached list of S3 buckets (for code upload).
    pub s3_buckets: Mutex<Vec<String>>,
    /// Cached list of IAM roles (for AgentCore).
    pub iam_roles: Mutex<Vec<String>>,
    /// Recipes loaded from the git-backed registry.
    pub recipes: Mutex<Vec<RecipeEntry>>,
    /// Prompts loaded from the git-backed registry.
    pub prompts: Mutex<Vec<PromptEntry>>,
    /// User preferences (selected profile, region, etc.).
    pub user_prefs: Mutex<UserPrefs>,
    /// Credential manager for AWS auth.
    pub credential_manager: Mutex<Option<CredentialManager>>,
    /// S3 sync configuration.
    pub sync_config: Mutex<SyncConfig>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            agents: Mutex::new(Vec::new()),
            models: Mutex::new(Vec::new()),
            memories: Mutex::new(Vec::new()),
            log_groups: Mutex::new(Vec::new()),
            s3_buckets: Mutex::new(Vec::new()),
            iam_roles: Mutex::new(Vec::new()),
            recipes: Mutex::new(Vec::new()),
            prompts: Mutex::new(Vec::new()),
            user_prefs: Mutex::new(UserPrefs::default()),
            credential_manager: Mutex::new(None),
            sync_config: Mutex::new(SyncConfig::default()),
        }
    }
}
