// Types that mirror Rust structs for Tauri IPC

export interface ModelInfo {
  id: string;
  label: string;
}

export interface Agent {
  id: string;
  name: string;
  arn: string;
  version: string;
  status: string;
  last_updated: string;
}

export interface AgentRuntimeDetail {
  id: string;
  name: string;
  arn: string;
  version: string;
  status: string;
  description: string;
  role_arn: string;
  network_mode: string;
  idle_session_timeout: number;
  max_lifetime: number;
  protocol: string;
  code_s3_bucket: string;
  code_s3_prefix: string;
  code_runtime: string;
  code_entry_point: string;
  environment_variables: Record<string, string>;
  created_at: string;
  last_updated_at: string;
}

export interface CreateAgentRuntimeParams {
  profile: string;
  region: string;
  name: string;
  agent_runtime_id: string;
  description: string;
  role_arn: string;
  network_mode: string;
  protocol: string;
  idle_session_timeout: number;
  max_lifetime: number;
  code_s3_bucket: string;
  code_s3_prefix: string;
  code_runtime: string;
  code_entry_point: string;
  environment_variables: Record<string, string>;
}

export interface MemoryInfo {
  id: string;
  name: string;
}

export interface MemoryStrategy {
  strategy_type: string;
  name: string;
  namespace: string;
}

export interface CreateMemoryParams {
  profile: string;
  region: string;
  name: string;
  description: string;
  event_expiry_duration: number;
  strategies: MemoryStrategy[];
}

export interface RecipeEntry {
  id: string;
  name: string;
  description: string;
  model_id: string;
  system_prompt: string;
  prompt_id: string;
  memory_id: string;
  namespace_pattern: string;
  qualifier: string;
  extra_params: Record<string, unknown>;
  owner: string;
  visibility: string;
  personality_rating: number | null;
  accuracy_rating: number | null;
  forked_from: string | null;
  created_at: string;
  updated_at: string;
  content_hash?: string;
  version?: string;
  source_id?: string;
}

export interface PromptEntry {
  id: string;
  name: string;
  content: string;
  owner: string;
  created_at: string;
  updated_at: string;
  content_hash?: string;
  version?: string;
  source_id?: string;
}

export interface ChatMessage {
  role: string;
  content: string;
}

export interface LogEntry {
  timestamp: string;
  message: string;
}

export interface InvokeParams {
  profile: string;
  region: string;
  agent_arn: string;
  session_id: string;
  qualifier: string;
  runtime_user_id: string;
  input: string;
  model_id: string;
  system_prompt: string;
  memory_id: string;
  extra_payload: Record<string, unknown>;
}

export interface UserPrefs {
  selected_profile: string;
  selected_region: string;
  selected_model_id: string;
  selected_agent_id: string;
  selected_memory_id: string;
  selected_recipe_id: string;
}

// Auth status for the cloud icon
export type AuthStatus =
  | 'valid'           // Green - authenticated
  | 'expiring'        // Yellow - credentials expiring soon
  | 'expired'         // Red - credentials expired
  | 'not-authenticated' // Gray - no auth attempted
  | 'refreshing'      // Spinner - checking status
  | 'error';          // Red - auth error

// AWS regions (hardcoded list)
export const AWS_REGIONS = [
  'us-east-1',
  'us-east-2',
  'us-west-1',
  'us-west-2',
  'eu-west-1',
  'eu-central-1',
  'ap-southeast-1',
  'ap-northeast-1',
] as const;

export type AwsRegion = typeof AWS_REGIONS[number];
