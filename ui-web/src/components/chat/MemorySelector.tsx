import { For, Show } from 'solid-js';
import type { Component } from 'solid-js';
import { store } from '../../stores/app';

const MemorySelector: Component = () => {
  const handleChange = (e: Event) => {
    const target = e.target as HTMLSelectElement;
    store.setSelectedMemoryIdx(parseInt(target.value, 10));
  };

  return (
    <div class="flex flex-col gap-1">
      <label class="text-xs text-text-dim uppercase tracking-wide">Memory</label>
      <div class="flex items-center gap-2">
        <select
          value={store.selectedMemoryIdx()}
          onChange={handleChange}
          disabled={store.loading().memories}
          class="flex-1 bg-surface-2 border border-border rounded-small px-3 py-1.5 text-sm
                 focus:outline-none focus:ring-2 focus:ring-accent disabled:opacity-50"
        >
          <option value={0}>(none)</option>
          <For each={store.memories()}>
            {(memory, idx) => (
              <option value={idx() + 1}>{memory.name}</option>
            )}
          </For>
        </select>
        <Show when={store.loading().memories}>
          <span class="material-icons text-accent animate-spin text-sm">sync</span>
        </Show>
      </div>
    </div>
  );
};

export default MemorySelector;
