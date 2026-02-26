import { For } from 'solid-js';
import type { Component } from 'solid-js';
import { store } from '../stores/app';

type Tab = 'chat' | 'debug' | 'snapshots' | 'prompts' | 'runtimes';

const tabs: { id: Tab; label: string }[] = [
  { id: 'chat', label: 'Chat' },
  { id: 'debug', label: 'Debug' },
  { id: 'snapshots', label: 'Snapshots' },
  { id: 'prompts', label: 'Prompts' },
  { id: 'runtimes', label: 'Runtimes' },
];

const TabBar: Component = () => {
  return (
    <nav class="bg-surface border-b border-border px-4">
      <div class="flex gap-1">
        <For each={tabs}>
          {(tab) => (
            <button
              onClick={() => store.setActiveTab(tab.id)}
              class={`px-4 py-2.5 text-sm font-medium transition-colors ${
                store.activeTab() === tab.id
                  ? 'text-accent border-b-2 border-accent'
                  : 'text-text-dim hover:text-text'
              }`}
            >
              {tab.label}
            </button>
          )}
        </For>
      </div>
    </nav>
  );
};

export default TabBar;
