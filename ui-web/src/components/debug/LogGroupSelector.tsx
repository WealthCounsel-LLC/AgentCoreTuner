import { For, Show } from 'solid-js';
import type { Component } from 'solid-js';
import { store } from '../../stores/app';

interface LogGroupSelectorProps {
  value: string;
  onChange: (value: string) => void;
}

const LogGroupSelector: Component<LogGroupSelectorProps> = (props) => {
  return (
    <div class="flex flex-col gap-1">
      <label class="text-xs text-text-dim uppercase tracking-wide">Log Group</label>
      <select
        value={props.value}
        onChange={(e) => props.onChange(e.currentTarget.value)}
        disabled={store.loading().logGroups}
        class="bg-surface-2 border border-border rounded-standard px-3 py-2 text-sm
               focus:outline-none focus:ring-2 focus:ring-accent disabled:opacity-50"
      >
        <option value="">Select a log group...</option>
        <For each={store.logGroups()}>
          {(group) => <option value={group}>{group}</option>}
        </For>
      </select>
      <Show when={store.loading().logGroups}>
        <p class="text-xs text-text-dim flex items-center gap-1">
          <span class="material-icons animate-spin text-xs">sync</span>
          Loading log groups...
        </p>
      </Show>
      <Show when={!store.loading().logGroups && store.logGroups().length === 0}>
        <p class="text-xs text-text-dim">No log groups found</p>
      </Show>
    </div>
  );
};

export default LogGroupSelector;
