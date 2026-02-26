// Tauri IPC wrapper functions
import { invoke } from '@tauri-apps/api/core';
import type {
  Agent,
  AgentRuntimeDetail,
  CreateAgentRuntimeParams,
  MemoryInfo,
  CreateMemoryParams,
  ModelInfo,
  RecipeEntry,
  PromptEntry,
  InvokeParams,
  UserPrefs,
} from '../types';

// ============================================================================
// Auth Commands
// ============================================================================

export async function listProfiles(): Promise<string[]> {
  return invoke('list_profiles');
}

export async function checkAwsCli(): Promise<string> {
  return invoke('check_aws_cli');
}

export async function getCallerIdentity(profile: string, region: string): Promise<string> {
  return invoke('get_caller_identity', { profile, region });
}

export async function ssoLogin(profile: string): Promise<void> {
  return invoke('sso_login', { profile });
}

// ============================================================================
// Agent Commands
// ============================================================================

export async function listAgents(profile: string, region: string): Promise<Agent[]> {
  return invoke('list_agents', { profile, region });
}

export async function getAgentRuntime(
  profile: string,
  region: string,
  agentId: string
): Promise<AgentRuntimeDetail> {
  return invoke('get_agent_runtime', { profile, region, agentId });
}

export async function createAgentRuntime(
  params: CreateAgentRuntimeParams
): Promise<AgentRuntimeDetail> {
  return invoke('create_agent_runtime', { params });
}

export async function updateAgentRuntime(
  params: CreateAgentRuntimeParams
): Promise<AgentRuntimeDetail> {
  return invoke('update_agent_runtime', { params });
}

export async function deleteAgentRuntime(
  profile: string,
  region: string,
  agentId: string
): Promise<void> {
  return invoke('delete_agent_runtime', { profile, region, agentId });
}

export async function invokeAgent(
  params: InvokeParams
): Promise<[string, string, string]> {
  // Returns [thinking, response, debug_log]
  return invoke('invoke_agent', { params });
}

export async function listAgentVersions(
  profile: string,
  region: string,
  agentId: string
): Promise<string[]> {
  return invoke('list_agent_versions', { profile, region, agentId });
}

// ============================================================================
// Model Commands
// ============================================================================

export async function listModels(profile: string, region: string): Promise<ModelInfo[]> {
  return invoke('list_models', { profile, region });
}

// ============================================================================
// Memory Commands
// ============================================================================

export async function listMemories(profile: string, region: string): Promise<MemoryInfo[]> {
  return invoke('list_memories', { profile, region });
}

export async function createMemory(params: CreateMemoryParams): Promise<MemoryInfo> {
  return invoke('create_memory', { params });
}

export async function deleteMemory(
  profile: string,
  region: string,
  memoryId: string
): Promise<void> {
  return invoke('delete_memory', { profile, region, memoryId });
}

export interface SessionSummary {
  session_id: string;
  actor_id: string;
  created_at: string;
}

export async function listSessions(
  profile: string,
  region: string,
  memoryId: string,
  actorId: string
): Promise<SessionSummary[]> {
  return invoke('list_sessions', { profile, region, memoryId, actorId });
}

// ============================================================================
// Recipe Commands
// ============================================================================

export async function listRecipes(): Promise<RecipeEntry[]> {
  return invoke('list_recipes');
}

export async function getRecipe(id: string): Promise<RecipeEntry | null> {
  return invoke('get_recipe', { id });
}

export async function saveRecipe(entry: RecipeEntry, message?: string): Promise<void> {
  return invoke('save_recipe', { entry, message });
}

export async function deleteRecipe(id: string): Promise<void> {
  return invoke('delete_recipe', { id });
}

export async function recipeHistory(id: string): Promise<string[]> {
  return invoke('recipe_history', { id });
}

// ============================================================================
// Prompt Commands
// ============================================================================

export async function listPrompts(): Promise<PromptEntry[]> {
  return invoke('list_prompts');
}

export async function getPrompt(id: string): Promise<PromptEntry | null> {
  return invoke('get_prompt', { id });
}

export async function savePrompt(entry: PromptEntry, message?: string): Promise<void> {
  return invoke('save_prompt', { entry, message });
}

export async function deletePrompt(id: string): Promise<void> {
  return invoke('delete_prompt', { id });
}

export async function promptHistory(id: string): Promise<string[]> {
  return invoke('prompt_history', { id });
}

// ============================================================================
// Log Commands
// ============================================================================

export async function listLogGroups(profile: string, region: string): Promise<string[]> {
  return invoke('list_log_groups', { profile, region });
}

export async function getLogs(
  profile: string,
  region: string,
  logGroup: string,
  streamFilter?: string,
  filterPattern?: string
): Promise<[string, string]> {
  // Returns [metrics, content]
  return invoke('get_logs', { profile, region, logGroup, streamFilter, filterPattern });
}

// ============================================================================
// Cache Commands
// ============================================================================

export async function readPrefs(): Promise<UserPrefs> {
  return invoke('read_prefs');
}

export async function writePrefs(prefs: UserPrefs): Promise<void> {
  return invoke('write_prefs', { prefs });
}

// ============================================================================
// Sharing Commands
// ============================================================================

export interface SyncConfig {
  profile: string;
  region: string;
  bucket: string;
  prefix: string;
  auto_sync: boolean;
}

export interface VersionInfo {
  version: string;
  key: string;
  updated_at: string;
  content_hash: string;
}

export interface SharedItem {
  key: string;
  id: string;
  name: string;
  description: string;
  owner: string;
  item_type: string;
  created_at: string;
  updated_at: string;
  content_hash: string;
  version: string;
  source_id: string;
  available_versions: VersionInfo[];
}

export async function getSyncConfig(): Promise<SyncConfig> {
  return invoke('get_sync_config');
}

export async function setSyncConfig(config: SyncConfig): Promise<void> {
  return invoke('set_sync_config', { config });
}

export async function publishRecipe(recipeId: string, username: string, version?: string): Promise<void> {
  return invoke('publish_recipe', { recipeId, username, version });
}

export async function publishPrompt(promptId: string, username: string, version?: string): Promise<void> {
  return invoke('publish_prompt', { promptId, username, version });
}

export async function listSharedRecipes(): Promise<SharedItem[]> {
  return invoke('list_shared_recipes');
}

export async function listSharedPrompts(): Promise<SharedItem[]> {
  return invoke('list_shared_prompts');
}

export async function importSharedRecipe(key: string): Promise<RecipeEntry> {
  return invoke('import_shared_recipe', { key });
}

export async function importSharedPrompt(key: string): Promise<PromptEntry> {
  return invoke('import_shared_prompt', { key });
}

export async function listSyncBuckets(profile: string, region: string): Promise<string[]> {
  return invoke('list_sync_buckets', { profile, region });
}

export interface UpdateAvailable {
  item_type: string;
  item_id: string;
  item_name: string;
  source_id: string;
  local_version: string;
  latest_version: string;
}

export async function checkForUpdates(): Promise<UpdateAvailable[]> {
  return invoke('check_for_updates');
}
