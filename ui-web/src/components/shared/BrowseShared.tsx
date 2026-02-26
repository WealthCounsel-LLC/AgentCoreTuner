import { createSignal, createMemo, Show, For } from 'solid-js';
import type { Component } from 'solid-js';
import { store } from '../../stores/app';
import * as tauri from '../../lib/tauri';
import type { SharedItem, VersionInfo } from '../../lib/tauri';

interface BrowseSharedProps {
  type: 'recipes' | 'prompts';
  onImport?: () => void;
}

const BrowseShared: Component<BrowseSharedProps> = (props) => {
  const [items, setItems] = createSignal<SharedItem[]>([]);
  const [loading, setLoading] = createSignal(false);
  const [importing, setImporting] = createSignal<string | null>(null);
  const [expanded, setExpanded] = createSignal(false);
  const [selectedVersions, setSelectedVersions] = createSignal<Record<string, string>>({});

  // Get local source_ids and versions for comparison
  const localSourceInfo = createMemo(() => {
    const info = new Map<string, { version: string; contentHash: string }>();
    if (props.type === 'recipes') {
      for (const recipe of store.recipes()) {
        if (recipe.source_id) {
          info.set(recipe.source_id, {
            version: recipe.version || '',
            contentHash: recipe.content_hash || '',
          });
        }
      }
    } else {
      for (const prompt of store.prompts()) {
        if (prompt.source_id) {
          info.set(prompt.source_id, {
            version: prompt.version || '',
            contentHash: prompt.content_hash || '',
          });
        }
      }
    }
    return info;
  });

  const getItemStatus = (item: SharedItem) => {
    const local = localSourceInfo().get(item.source_id);
    if (!local) {
      return 'not_imported'; // Never imported
    }
    // Compare versions
    if (local.version && item.version) {
      const comparison = compareVersions(item.version, local.version);
      if (comparison > 0) {
        return 'update_available'; // Remote is newer
      }
    }
    // Fallback to hash comparison
    if (local.contentHash === item.content_hash) {
      return 'up_to_date';
    }
    return 'update_available';
  };

  const compareVersions = (a: string, b: string): number => {
    const parseVersion = (v: string) => v.split('.').map(n => parseInt(n, 10) || 0);
    const va = parseVersion(a);
    const vb = parseVersion(b);
    for (let i = 0; i < Math.max(va.length, vb.length); i++) {
      const diff = (va[i] || 0) - (vb[i] || 0);
      if (diff !== 0) return diff;
    }
    return 0;
  };

  const fetchItems = async () => {
    setLoading(true);
    try {
      const shared = props.type === 'recipes'
        ? await tauri.listSharedRecipes()
        : await tauri.listSharedPrompts();
      setItems(shared);
      setExpanded(true);
    } catch (err) {
      store.showStatus(`Failed to load shared ${props.type}: ${err}`, true);
    } finally {
      setLoading(false);
    }
  };

  const handleImport = async (item: SharedItem, versionKey?: string) => {
    const keyToImport = versionKey || item.key;
    setImporting(keyToImport);
    try {
      if (props.type === 'recipes') {
        await tauri.importSharedRecipe(keyToImport);
        await store.fetchRecipes();
      } else {
        await tauri.importSharedPrompt(keyToImport);
        await store.fetchPrompts();
      }
      const version = item.available_versions.find(v => v.key === keyToImport)?.version || item.version;
      store.showStatus(`Imported ${item.name} v${version}`);
      props.onImport?.();
    } catch (err) {
      store.showStatus(`Import failed: ${err}`, true);
    } finally {
      setImporting(null);
    }
  };

  const formatDate = (iso: string) => {
    if (!iso) return '';
    const date = new Date(iso);
    return date.toLocaleDateString(undefined, {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
    });
  };

  const getSelectedVersion = (item: SharedItem): VersionInfo | undefined => {
    const selectedKey = selectedVersions()[item.source_id];
    if (selectedKey) {
      return item.available_versions.find(v => v.key === selectedKey);
    }
    return item.available_versions[0]; // Default to latest
  };

  return (
    <div class="border border-border rounded-standard">
      {/* Header */}
      <button
        onClick={() => expanded() ? setExpanded(false) : fetchItems()}
        class="w-full px-4 py-3 flex items-center justify-between hover:bg-surface-2 transition-colors"
      >
        <div class="flex items-center gap-2">
          <span class="material-icons text-accent">cloud_download</span>
          <span class="font-medium">Browse Team {props.type === 'recipes' ? 'Snapshots' : 'Prompts'}</span>
        </div>
        <div class="flex items-center gap-2">
          <Show when={loading()}>
            <span class="material-icons animate-spin text-sm">sync</span>
          </Show>
          <span class="material-icons text-text-dim">
            {expanded() ? 'expand_less' : 'expand_more'}
          </span>
        </div>
      </button>

      {/* Items list */}
      <Show when={expanded()}>
        <div class="border-t border-border max-h-80 overflow-y-auto">
          <Show when={items().length === 0 && !loading()}>
            <div class="p-4 text-center text-text-dim">
              No shared {props.type} found
            </div>
          </Show>

          <For each={items()}>
            {(item) => {
              const status = () => getItemStatus(item);
              const selectedVer = () => getSelectedVersion(item);
              const hasMultipleVersions = () => item.available_versions.length > 1;

              return (
                <div class="px-4 py-3 border-b border-border last:border-b-0 hover:bg-surface-2">
                  <div class="flex items-start justify-between gap-4">
                    <div class="flex-1 min-w-0">
                      <div class="font-medium truncate flex items-center gap-2">
                        {item.name}
                        <Show when={item.version}>
                          <span class="text-xs text-text-dim bg-surface-3 px-1.5 py-0.5 rounded">
                            v{item.version}
                          </span>
                        </Show>
                        <Show when={status() === 'up_to_date'}>
                          <span class="text-xs text-success bg-success/10 px-1.5 py-0.5 rounded">
                            imported
                          </span>
                        </Show>
                        <Show when={status() === 'update_available'}>
                          <span class="text-xs text-warning bg-warning/10 px-1.5 py-0.5 rounded flex items-center gap-1">
                            <span class="material-icons text-xs">upgrade</span>
                            update
                          </span>
                        </Show>
                      </div>
                      <div class="text-xs text-text-dim mt-0.5">
                        by {item.owner} • {formatDate(item.updated_at)}
                      </div>
                      <Show when={item.description}>
                        <div class="text-sm text-text-dim mt-1 line-clamp-2">
                          {item.description}
                        </div>
                      </Show>

                      {/* Version picker */}
                      <Show when={hasMultipleVersions()}>
                        <div class="mt-2 flex items-center gap-2">
                          <span class="text-xs text-text-dim">Version:</span>
                          <select
                            class="text-xs bg-surface-2 border border-border rounded px-2 py-1"
                            value={selectedVersions()[item.source_id] || item.key}
                            onChange={(e) => {
                              setSelectedVersions(prev => ({
                                ...prev,
                                [item.source_id]: e.currentTarget.value,
                              }));
                            }}
                          >
                            <For each={item.available_versions}>
                              {(ver) => (
                                <option value={ver.key}>
                                  v{ver.version} ({formatDate(ver.updated_at)})
                                </option>
                              )}
                            </For>
                          </select>
                        </div>
                      </Show>
                    </div>

                    <Show when={status() === 'up_to_date'}>
                      <div class="p-2 text-success" title="Up to date">
                        <span class="material-icons">check_circle</span>
                      </div>
                    </Show>

                    <Show when={status() === 'update_available'}>
                      <button
                        onClick={() => handleImport(item, selectedVer()?.key)}
                        disabled={importing() === (selectedVer()?.key || item.key)}
                        class="px-3 py-1.5 text-sm bg-warning text-crust rounded-small
                               hover:bg-warning/80 transition-colors disabled:opacity-50
                               flex items-center gap-1"
                      >
                        <Show when={importing() === (selectedVer()?.key || item.key)}>
                          <span class="material-icons animate-spin text-sm">sync</span>
                        </Show>
                        <Show when={importing() !== (selectedVer()?.key || item.key)}>
                          <span class="material-icons text-sm">upgrade</span>
                        </Show>
                        Update
                      </button>
                    </Show>

                    <Show when={status() === 'not_imported'}>
                      <button
                        onClick={() => handleImport(item, selectedVer()?.key)}
                        disabled={importing() === (selectedVer()?.key || item.key)}
                        class="px-3 py-1.5 text-sm bg-accent text-crust rounded-small
                               hover:bg-accent-hover transition-colors disabled:opacity-50
                               flex items-center gap-1"
                      >
                        <Show when={importing() === (selectedVer()?.key || item.key)}>
                          <span class="material-icons animate-spin text-sm">sync</span>
                        </Show>
                        <Show when={importing() !== (selectedVer()?.key || item.key)}>
                          <span class="material-icons text-sm">download</span>
                        </Show>
                        Import
                      </button>
                    </Show>
                  </div>
                </div>
              );
            }}
          </For>
        </div>

        {/* Refresh button */}
        <Show when={items().length > 0}>
          <div class="border-t border-border px-4 py-2">
            <button
              onClick={fetchItems}
              disabled={loading()}
              class="text-sm text-accent hover:text-accent-hover flex items-center gap-1"
            >
              <span class={`material-icons text-sm ${loading() ? 'animate-spin' : ''}`}>refresh</span>
              Refresh
            </button>
          </div>
        </Show>
      </Show>
    </div>
  );
};

export default BrowseShared;
