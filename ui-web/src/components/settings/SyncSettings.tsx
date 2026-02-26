import { createSignal, createEffect, Show, For } from 'solid-js';
import type { Component } from 'solid-js';
import { store } from '../../stores/app';
import * as tauri from '../../lib/tauri';
import type { SyncConfig } from '../../lib/tauri';

const SyncSettings: Component = () => {
  const [config, setConfig] = createSignal<SyncConfig>({
    profile: '',
    region: '',
    bucket: '',
    prefix: 'agentcore-team/',
    auto_sync: false,
  });
  const [buckets, setBuckets] = createSignal<string[]>([]);
  const [loadingBuckets, setLoadingBuckets] = createSignal(false);
  const [saving, setSaving] = createSignal(false);

  // Load sync config on mount
  createEffect(async () => {
    try {
      const savedConfig = await tauri.getSyncConfig();
      if (savedConfig.bucket) {
        setConfig(savedConfig);
      } else {
        // Default to current profile/region
        setConfig({
          ...savedConfig,
          profile: store.selectedProfile(),
          region: store.selectedRegion(),
          prefix: 'agentcore-team/',
        });
      }
    } catch (err) {
      console.error('Failed to load sync config:', err);
    }
  });

  const fetchBuckets = async () => {
    const currentConfig = config();
    if (!currentConfig.profile || !currentConfig.region) return;

    setLoadingBuckets(true);
    try {
      const bucketList = await tauri.listSyncBuckets(
        currentConfig.profile,
        currentConfig.region
      );
      setBuckets(bucketList);
    } catch (err) {
      console.error('Failed to fetch buckets:', err);
      store.showStatus(`Failed to fetch buckets: ${err}`, true);
    } finally {
      setLoadingBuckets(false);
    }
  };

  const handleSaveConfig = async () => {
    setSaving(true);
    try {
      await tauri.setSyncConfig(config());
      store.showStatus('Sharing settings saved');
    } catch (err) {
      store.showStatus(`Failed to save settings: ${err}`, true);
    } finally {
      setSaving(false);
    }
  };

  const updateConfig = (updates: Partial<SyncConfig>) => {
    setConfig(prev => ({ ...prev, ...updates }));
  };

  return (
    <div class="p-4 space-y-4">
      <div class="flex items-center gap-2 mb-4">
        <span class="material-icons text-accent">share</span>
        <h3 class="text-lg font-semibold">Team Sharing</h3>
      </div>

      <p class="text-sm text-text-dim">
        Share recipes and prompts with your team via S3.
      </p>

      {/* AWS Profile */}
      <div class="space-y-1">
        <label class="text-xs text-text-dim uppercase tracking-wide">AWS Profile</label>
        <select
          value={config().profile}
          onChange={(e) => updateConfig({ profile: e.currentTarget.value })}
          class="w-full bg-surface-2 border border-border rounded-standard px-3 py-2 text-sm
                 focus:outline-none focus:ring-2 focus:ring-accent"
        >
          <option value="">Select profile...</option>
          <For each={store.profiles()}>
            {(profile) => <option value={profile}>{profile}</option>}
          </For>
        </select>
      </div>

      {/* Region */}
      <div class="space-y-1">
        <label class="text-xs text-text-dim uppercase tracking-wide">Region</label>
        <input
          type="text"
          value={config().region}
          onInput={(e) => updateConfig({ region: e.currentTarget.value })}
          placeholder="us-east-1"
          class="w-full bg-surface-2 border border-border rounded-standard px-3 py-2 text-sm
                 focus:outline-none focus:ring-2 focus:ring-accent"
        />
      </div>

      {/* Bucket */}
      <div class="space-y-1">
        <label class="text-xs text-text-dim uppercase tracking-wide flex items-center gap-2">
          Shared S3 Bucket
          <button
            onClick={fetchBuckets}
            disabled={loadingBuckets() || !config().profile}
            class="text-accent hover:text-accent-hover disabled:opacity-50"
            title="Refresh bucket list"
          >
            <span class={`material-icons text-sm ${loadingBuckets() ? 'animate-spin' : ''}`}>
              refresh
            </span>
          </button>
        </label>
        <select
          value={config().bucket}
          onChange={(e) => updateConfig({ bucket: e.currentTarget.value })}
          class="w-full bg-surface-2 border border-border rounded-standard px-3 py-2 text-sm
                 focus:outline-none focus:ring-2 focus:ring-accent"
        >
          <option value="">Select bucket...</option>
          <For each={buckets()}>
            {(bucket) => <option value={bucket}>{bucket}</option>}
          </For>
        </select>
        <Show when={!buckets().length}>
          <input
            type="text"
            value={config().bucket}
            onInput={(e) => updateConfig({ bucket: e.currentTarget.value })}
            placeholder="Or enter bucket name"
            class="w-full bg-surface-2 border border-border rounded-standard px-3 py-2 text-sm mt-1
                   focus:outline-none focus:ring-2 focus:ring-accent"
          />
        </Show>
      </div>

      {/* Prefix */}
      <div class="space-y-1">
        <label class="text-xs text-text-dim uppercase tracking-wide">Key Prefix</label>
        <input
          type="text"
          value={config().prefix}
          onInput={(e) => updateConfig({ prefix: e.currentTarget.value })}
          placeholder="agentcore-team/"
          class="w-full bg-surface-2 border border-border rounded-standard px-3 py-2 text-sm
                 focus:outline-none focus:ring-2 focus:ring-accent"
        />
      </div>

      {/* Save button */}
      <button
        onClick={handleSaveConfig}
        disabled={saving() || !config().bucket}
        class="w-full px-4 py-2 bg-accent text-crust rounded-standard font-medium
               hover:bg-accent-hover transition-colors disabled:opacity-50"
      >
        {saving() ? 'Saving...' : 'Save Settings'}
      </button>

      <Show when={config().bucket}>
        <div class="mt-4 p-3 bg-surface-2 rounded-standard text-sm">
          <p class="text-text-dim">
            Shared items will be stored at:
          </p>
          <code class="text-xs text-accent block mt-1">
            s3://{config().bucket}/{config().prefix}recipes/
          </code>
          <code class="text-xs text-accent block">
            s3://{config().bucket}/{config().prefix}prompts/
          </code>
        </div>
      </Show>
    </div>
  );
};

export default SyncSettings;
