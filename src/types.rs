use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub label: String, // "{Provider} – {Name}" for display
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    /// Full ARN required by invoke-agent-runtime.
    pub arn: String,
    /// Current version number from list-agent-runtimes.
    #[serde(default)]
    pub version: String,
    /// Runtime status (e.g. "READY", "CREATING", "UPDATING").
    #[serde(default)]
    pub status: String,
    /// ISO-8601 timestamp of last update.
    #[serde(default)]
    pub last_updated: String,
}

/// Full configuration detail for an AgentCore Runtime.
#[derive(Debug, Clone)]
pub struct AgentRuntimeDetail {
    pub id: String,
    pub name: String,
    pub arn: String,
    pub version: String,
    pub status: String,
    pub description: String,
    pub role_arn: String,
    pub network_mode: String,
    pub idle_session_timeout: u32,
    pub max_lifetime: u32,
    pub protocol: String,
    pub code_s3_bucket: String,
    pub code_s3_prefix: String,
    pub code_runtime: String,
    pub code_entry_point: String,
    pub environment_variables: HashMap<String, String>,
    pub created_at: String,
    pub last_updated_at: String,
}

/// Parameters for creating or updating an AgentCore Runtime.
#[derive(Debug, Clone, Default)]
pub struct CreateAgentRuntimeParams {
    pub profile: String,
    pub region: String,
    /// Required for create; ignored for update.
    pub name: String,
    /// Set only for update (empty for create).
    pub agent_runtime_id: String,
    pub description: String,
    pub role_arn: String,
    /// "PUBLIC" (default).
    pub network_mode: String,
    /// "HTTP" (default), "MCP", or "A2A".
    pub protocol: String,
    /// Idle session timeout in seconds (60–28800, default 900).
    pub idle_session_timeout: u32,
    /// Max lifetime in seconds (60–28800, default 28800).
    pub max_lifetime: u32,
    pub code_s3_bucket: String,
    pub code_s3_prefix: String,
    /// e.g. "PYTHON_3_13".
    pub code_runtime: String,
    /// e.g. "main.py".
    pub code_entry_point: String,
    pub environment_variables: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryStrategy {
    pub strategy_type: String,
    pub name: String,
    pub namespace: String,
}

#[derive(Debug, Clone, Default)]
pub struct CreateMemoryParams {
    pub profile: String,
    pub region: String,
    /// Required. Must match `[a-zA-Z][a-zA-Z0-9_]{0,47}`.
    pub name: String,
    pub description: String,
    /// Required. Number of days before memory events expire (3–365).
    pub event_expiry_duration: u32,
    pub strategies: Vec<MemoryStrategy>,
}

/// A saved agent configuration that bundles model, prompt, memory, and tuning
/// parameters into a shareable, version-tracked recipe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    /// Bedrock model ID. Empty = let the agent use its default.
    #[serde(default)]
    pub model_id: String,
    /// System prompt override. Empty = agent's built-in prompt.
    #[serde(default)]
    pub system_prompt: String,
    /// Reference to a PromptEntry by ID. When set, the prompt's content
    /// overrides system_prompt at invocation time. Empty = use inline system_prompt.
    #[serde(default)]
    pub prompt_id: String,
    /// AWS memory pool ID to attach. Empty = no memory.
    #[serde(default)]
    pub memory_id: String,
    /// Memory namespace pattern (e.g. "/strategy/{id}/actor/{actorId}/").
    #[serde(default)]
    pub namespace_pattern: String,
    /// Agent version qualifier (e.g. "v1", "LATEST"). Empty = default.
    #[serde(default)]
    pub qualifier: String,
    /// Extra invoke parameters (temperature, max_tokens, custom keys).
    #[serde(default)]
    pub extra_params: HashMap<String, Value>,
    pub owner: String,
    pub visibility: String,
    /// How well the recipe's personality matches intent (1–5). None = not rated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub personality_rating: Option<i32>,
    /// How accurately the recipe's responses match factual expectations (1–5). None = not rated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accuracy_rating: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forked_from: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// A reusable, versioned system prompt that can be referenced by recipes.
/// Each save creates a new git commit, enabling version history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptEntry {
    pub id: String,
    pub name: String,
    /// The actual prompt text (may be multi-line).
    pub content: String,
    pub owner: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub message: String,
}

/// All knobs for a single invoke-agent-runtime call.
///
/// `extra_payload` is merged into the JSON body last, so any field your agent
/// handler reads (e.g. `"temperature"`, `"max_tokens"`, custom keys) can be
/// injected without changing this struct.
#[derive(Debug, Clone, Default)]
pub struct InvokeParams {
    // ── AWS request parameters ──────────────────────────────────────────────
    pub profile: String,
    pub region: String,
    pub agent_arn: String,
    /// 33–256 chars. Auto-generated from nanoseconds if empty.
    pub session_id: String,
    /// Specific agent version or named endpoint. Empty = default version.
    pub qualifier: String,
    /// Passed as X-Amzn-Bedrock-AgentCore-Runtime-User-Id. Empty = omitted.
    pub runtime_user_id: String,

    // ── Payload body fields ─────────────────────────────────────────────────
    /// The user's message / prompt input.
    pub input: String,
    /// Bedrock model ID to use. Empty = agent's configured default.
    pub model_id: String,
    /// Overrides the agent's system prompt for this request.
    /// Only takes effect if your agent handler reads this field.
    pub system_prompt: String,
    /// Bedrock AgentCore memory ID. Empty = no memory attached.
    pub memory_id: String,
    /// Any additional key/value pairs merged into the payload JSON.
    /// Use this for agent-specific fields (e.g. temperature, max_tokens).
    pub extra_payload: HashMap<String, Value>,
}
