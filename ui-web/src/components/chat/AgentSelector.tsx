import { For, Show } from 'solid-js';
import type { Component } from 'solid-js';
import { store } from '../../stores/app';

const AgentSelector: Component = () => {
  const handleChange = (e: Event) => {
    const target = e.target as HTMLSelectElement;
    store.setSelectedAgentIdx(parseInt(target.value, 10));
  };

  return (
    <div class="flex flex-col gap-1">
      <label class="text-xs text-text-dim uppercase tracking-wide">Agent</label>
      <div class="flex items-center gap-2">
        <select
          value={store.selectedAgentIdx()}
          onChange={handleChange}
          disabled={store.loading().agents}
          class="flex-1 bg-surface-2 border border-border rounded-small px-3 py-1.5 text-sm
                 focus:outline-none focus:ring-2 focus:ring-accent disabled:opacity-50"
        >
          <Show when={store.agents().length === 0}>
            <option value={0}>No agents available</option>
          </Show>
          <For each={store.agents()}>
            {(agent, idx) => (
              <option value={idx()}>
                {agent.name} ({agent.status})
              </option>
            )}
          </For>
        </select>
        <Show when={store.loading().agents}>
          <span class="material-icons text-accent animate-spin text-sm">sync</span>
        </Show>
      </div>
    </div>
  );
};

export default AgentSelector;
