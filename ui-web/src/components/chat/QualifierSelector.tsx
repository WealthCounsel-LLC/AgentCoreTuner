import { For, Show, createSignal, createEffect } from 'solid-js';
import type { Component } from 'solid-js';
import { store } from '../../stores/app';
import * as tauri from '../../lib/tauri';

// Display label for when no qualifier is specified (uses the active/default version)
const DEFAULT_LABEL = '(default)';
const DEFAULT_VALUE = '';

const QualifierSelector: Component = () => {
  // Each entry: { value: string, label: string }
  const [qualifiers, setQualifiers] = createSignal<Array<{ value: string; label: string }>>([
    { value: DEFAULT_VALUE, label: DEFAULT_LABEL }
  ]);
  const [loading, setLoading] = createSignal(false);

  // Fetch qualifiers when agent selection changes
  createEffect(async () => {
    const agentIdx = store.selectedAgentIdx();
    const agent = store.agents()[agentIdx];

    if (!agent || store.authStatus() !== 'valid') {
      setQualifiers([{ value: DEFAULT_VALUE, label: DEFAULT_LABEL }]);
      store.setSelectedQualifier(DEFAULT_VALUE);
      return;
    }

    setLoading(true);
    try {
      const versions = await tauri.listAgentVersions(
        store.selectedProfile(),
        store.selectedRegion(),
        agent.id
      );

      // Build options: default first, then actual versions
      const options: Array<{ value: string; label: string }> = [
        { value: DEFAULT_VALUE, label: DEFAULT_LABEL }
      ];

      // Add fetched versions (they come sorted descending: newest first)
      for (const v of versions) {
        options.push({ value: v, label: `Version ${v}` });
      }

      setQualifiers(options);

      // If current selection isn't in the list, reset to default
      const currentValue = store.selectedQualifier();
      const validValues = options.map(o => o.value);
      if (!validValues.includes(currentValue)) {
        store.setSelectedQualifier(DEFAULT_VALUE);
      }
    } catch (err) {
      console.error('Failed to fetch agent versions:', err);
      setQualifiers([{ value: DEFAULT_VALUE, label: DEFAULT_LABEL }]);
      store.setSelectedQualifier(DEFAULT_VALUE);
    } finally {
      setLoading(false);
    }
  });

  const handleChange = (e: Event) => {
    const target = e.target as HTMLSelectElement;
    store.setSelectedQualifier(target.value);
  };

  return (
    <div class="flex flex-col gap-1">
      <label class="text-xs text-text-dim uppercase tracking-wide flex items-center gap-1">
        Qualifier
        <Show when={loading()}>
          <span class="material-icons animate-spin text-xs">sync</span>
        </Show>
      </label>
      <select
        value={store.selectedQualifier()}
        onChange={handleChange}
        disabled={loading()}
        class="bg-surface-2 border border-border rounded-small px-3 py-1.5 text-sm
               focus:outline-none focus:ring-2 focus:ring-accent disabled:opacity-50"
      >
        <For each={qualifiers()}>
          {(qualifier) => (
            <option value={qualifier.value}>{qualifier.label}</option>
          )}
        </For>
      </select>
      <Show when={qualifiers().length > 1}>
        <p class="text-xs text-text-dim">{qualifiers().length - 1} version(s) available</p>
      </Show>
    </div>
  );
};

export default QualifierSelector;
