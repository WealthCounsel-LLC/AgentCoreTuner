// Global application state using Solid signals
import { createSignal, createRoot } from 'solid-js';
import type {
  Agent,
  ModelInfo,
  MemoryInfo,
  RecipeEntry,
  PromptEntry,
  ChatMessage,
  AuthStatus,
  UserPrefs,
} from '../types';
import { AWS_REGIONS } from '../types';
import * as tauri from '../lib/tauri';

// Create store in a root to allow usage outside components
function createAppStore() {
  // ============================================================================
  // Auth State
  // ============================================================================
  const [profiles, setProfiles] = createSignal<string[]>([]);
  const [selectedProfile, setSelectedProfile] = createSignal('');
  const [selectedRegion, setSelectedRegion] = createSignal<string>(AWS_REGIONS[0]);
  const [authStatus, setAuthStatus] = createSignal<AuthStatus>('not-authenticated');
  const [userId, setUserId] = createSignal('');
  const [awsCliInstalled, setAwsCliInstalled] = createSignal(true);
  const [awsCliVersion, setAwsCliVersion] = createSignal('');

  // ============================================================================
  // Data State
  // ============================================================================
  const [agents, setAgents] = createSignal<Agent[]>([]);
  const [models, setModels] = createSignal<ModelInfo[]>([]);
  const [memories, setMemories] = createSignal<MemoryInfo[]>([]);
  const [logGroups, setLogGroups] = createSignal<string[]>([]);
  const [recipes, setRecipes] = createSignal<RecipeEntry[]>([]);
  const [prompts, setPrompts] = createSignal<PromptEntry[]>([]);

  // ============================================================================
  // Chat State
  // ============================================================================
  const [messages, setMessages] = createSignal<ChatMessage[]>([]);
  const [selectedAgentIdx, setSelectedAgentIdx] = createSignal(0);
  const [selectedModelIdx, setSelectedModelIdx] = createSignal(0);
  const [selectedMemoryIdx, setSelectedMemoryIdx] = createSignal(0);
  const [selectedPromptIdx, setSelectedPromptIdx] = createSignal(0);
  const [selectedQualifierIdx, setSelectedQualifierIdx] = createSignal(0);
  const [selectedQualifier, setSelectedQualifier] = createSignal('');
  const [sessionId, setSessionId] = createSignal('');
  const [activeRecipeId, setActiveRecipeId] = createSignal('');
  const [systemPrompt, setSystemPrompt] = createSignal('');
  const [runtimeUserId, setRuntimeUserId] = createSignal('');

  // ============================================================================
  // UI State
  // ============================================================================
  const [loading, setLoading] = createSignal({
    agents: false,
    models: false,
    memories: false,
    logs: false,
    logGroups: false,
    sending: false,
    deploy: false,
    auth: false,
  });
  const [statusMessage, setStatusMessage] = createSignal('');
  const [statusIsError, setStatusIsError] = createSignal(false);
  const [devMode, setDevMode] = createSignal(false);
  const [activeTab, setActiveTab] = createSignal<'chat' | 'debug' | 'snapshots' | 'prompts' | 'runtimes'>('chat');

  // ============================================================================
  // Dialog State
  // ============================================================================
  const [showMfaDialog, setShowMfaDialog] = createSignal(false);
  const [mfaSerial, setMfaSerial] = createSignal('');
  const [showAwsCliDialog, setShowAwsCliDialog] = createSignal(false);
  const [showAwsSetupDialog, setShowAwsSetupDialog] = createSignal(false);

  // ============================================================================
  // Helper Functions
  // ============================================================================

  function updateLoading(key: keyof ReturnType<typeof loading>, value: boolean) {
    setLoading(prev => ({ ...prev, [key]: value }));
  }

  function showStatus(message: string, isError = false) {
    setStatusMessage(message);
    setStatusIsError(isError);
    // Auto-dismiss success messages after 3 seconds
    if (!isError) {
      setTimeout(() => {
        if (statusMessage() === message) {
          setStatusMessage('');
        }
      }, 3000);
    }
  }

  // ============================================================================
  // Actions
  // ============================================================================

  async function checkAwsCli() {
    try {
      const version = await tauri.checkAwsCli();
      setAwsCliInstalled(true);
      setAwsCliVersion(version);
      return true;
    } catch {
      setAwsCliInstalled(false);
      setShowAwsCliDialog(true);
      return false;
    }
  }

  async function loadProfiles() {
    try {
      const profileList = await tauri.listProfiles();
      setProfiles(profileList);
      if (profileList.length > 0 && !selectedProfile()) {
        setSelectedProfile(profileList[0]);
      }
      if (profileList.length === 0) {
        setShowAwsSetupDialog(true);
      }
    } catch (err) {
      showStatus(`Failed to load profiles: ${err}`, true);
    }
  }

  async function verifyAuth() {
    const profile = selectedProfile();
    const region = selectedRegion();
    if (!profile) return;

    updateLoading('auth', true);
    setAuthStatus('refreshing');

    try {
      const arn = await tauri.getCallerIdentity(profile, region);
      setAuthStatus('valid');
      // Extract user ID from ARN (last part after /)
      const parts = arn.split('/');
      setUserId(parts[parts.length - 1] || arn);
      setRuntimeUserId(parts[parts.length - 1] || '');
    } catch (err) {
      const errStr = String(err);
      if (errStr.includes('SSO') || errStr.includes('sso')) {
        setAuthStatus('expired');
      } else if (errStr.includes('expired')) {
        setAuthStatus('expired');
      } else {
        setAuthStatus('error');
      }
      showStatus(`Auth failed: ${err}`, true);
    } finally {
      updateLoading('auth', false);
    }
  }

  async function ssoLogin() {
    const profile = selectedProfile();
    if (!profile) return;

    updateLoading('auth', true);
    setAuthStatus('refreshing');

    try {
      await tauri.ssoLogin(profile);
      await verifyAuth();
      showStatus('SSO login successful');
    } catch (err) {
      setAuthStatus('error');
      showStatus(`SSO login failed: ${err}`, true);
    } finally {
      updateLoading('auth', false);
    }
  }

  async function fetchAgents() {
    const profile = selectedProfile();
    const region = selectedRegion();
    if (!profile) return;

    updateLoading('agents', true);
    try {
      const agentList = await tauri.listAgents(profile, region);
      setAgents(agentList);
    } catch (err) {
      showStatus(`Failed to fetch agents: ${err}`, true);
    } finally {
      updateLoading('agents', false);
    }
  }

  async function fetchModels() {
    const profile = selectedProfile();
    const region = selectedRegion();
    if (!profile) return;

    updateLoading('models', true);
    try {
      const modelList = await tauri.listModels(profile, region);
      setModels(modelList);
    } catch (err) {
      showStatus(`Failed to fetch models: ${err}`, true);
    } finally {
      updateLoading('models', false);
    }
  }

  async function fetchMemories() {
    const profile = selectedProfile();
    const region = selectedRegion();
    if (!profile) return;

    updateLoading('memories', true);
    try {
      const memoryList = await tauri.listMemories(profile, region);
      setMemories(memoryList);
    } catch (err) {
      showStatus(`Failed to fetch memories: ${err}`, true);
    } finally {
      updateLoading('memories', false);
    }
  }

  async function fetchLogGroups() {
    const profile = selectedProfile();
    const region = selectedRegion();
    if (!profile) return;

    updateLoading('logGroups', true);
    try {
      const groups = await tauri.listLogGroups(profile, region);
      setLogGroups(groups);
    } catch (err) {
      showStatus(`Failed to fetch log groups: ${err}`, true);
    } finally {
      updateLoading('logGroups', false);
    }
  }

  async function fetchRecipes() {
    try {
      const recipeList = await tauri.listRecipes();
      setRecipes(recipeList);
    } catch (err) {
      showStatus(`Failed to fetch snapshots: ${err}`, true);
    }
  }

  async function fetchPrompts() {
    try {
      const promptList = await tauri.listPrompts();
      setPrompts(promptList);
    } catch (err) {
      showStatus(`Failed to fetch prompts: ${err}`, true);
    }
  }

  async function refreshAll() {
    await Promise.all([
      fetchAgents(),
      fetchModels(),
      fetchMemories(),
      fetchLogGroups(),
    ]);
  }

  async function loadPrefs() {
    try {
      const prefs = await tauri.readPrefs();
      if (prefs.selected_profile) setSelectedProfile(prefs.selected_profile);
      if (prefs.selected_region) setSelectedRegion(prefs.selected_region);
      if (prefs.selected_recipe_id) setActiveRecipeId(prefs.selected_recipe_id);
    } catch {
      // Ignore - prefs may not exist yet
    }
  }

  async function savePrefs() {
    const prefs: UserPrefs = {
      selected_profile: selectedProfile(),
      selected_region: selectedRegion(),
      selected_model_id: models()[selectedModelIdx()]?.id || '',
      selected_agent_id: agents()[selectedAgentIdx()]?.id || '',
      selected_memory_id: memories()[selectedMemoryIdx()]?.id || '',
      selected_recipe_id: activeRecipeId(),
    };
    try {
      await tauri.writePrefs(prefs);
    } catch {
      // Ignore save errors
    }
  }

  async function initialize() {
    // Check AWS CLI first
    const cliOk = await checkAwsCli();
    if (!cliOk) return;

    // Load profiles and preferences
    await loadProfiles();
    await loadPrefs();
    await fetchRecipes();
    await fetchPrompts();

    // Verify auth and fetch data
    if (selectedProfile()) {
      await verifyAuth();
      if (authStatus() === 'valid') {
        await refreshAll();
      }
    }
  }

  return {
    // Auth state
    profiles,
    selectedProfile,
    setSelectedProfile,
    selectedRegion,
    setSelectedRegion,
    authStatus,
    setAuthStatus,
    userId,
    awsCliInstalled,
    awsCliVersion,

    // Data state
    agents,
    models,
    memories,
    logGroups,
    recipes,
    prompts,
    setRecipes,
    setPrompts,

    // Chat state
    messages,
    setMessages,
    selectedAgentIdx,
    setSelectedAgentIdx,
    selectedModelIdx,
    setSelectedModelIdx,
    selectedMemoryIdx,
    setSelectedMemoryIdx,
    selectedPromptIdx,
    setSelectedPromptIdx,
    selectedQualifierIdx,
    setSelectedQualifierIdx,
    selectedQualifier,
    setSelectedQualifier,
    sessionId,
    setSessionId,
    activeRecipeId,
    setActiveRecipeId,
    systemPrompt,
    setSystemPrompt,
    runtimeUserId,
    setRuntimeUserId,

    // UI state
    loading,
    statusMessage,
    statusIsError,
    setStatusMessage,
    devMode,
    setDevMode,
    activeTab,
    setActiveTab,
    showStatus,

    // Dialog state
    showMfaDialog,
    setShowMfaDialog,
    mfaSerial,
    setMfaSerial,
    showAwsCliDialog,
    setShowAwsCliDialog,
    showAwsSetupDialog,
    setShowAwsSetupDialog,

    // Actions
    checkAwsCli,
    loadProfiles,
    verifyAuth,
    ssoLogin,
    fetchAgents,
    fetchModels,
    fetchMemories,
    fetchLogGroups,
    fetchRecipes,
    fetchPrompts,
    refreshAll,
    loadPrefs,
    savePrefs,
    initialize,
  };
}

// Create singleton store
export const store = createRoot(createAppStore);
