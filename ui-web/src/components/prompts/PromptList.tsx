import { createSignal, For, Show, createMemo } from 'solid-js';
import type { Component } from 'solid-js';
import { store } from '../../stores/app';
import PromptCard from './PromptCard';
import type { PromptEntry } from '../../types';

interface PromptListProps {
  selectedId: string | null;
  onSelect: (prompt: PromptEntry) => void;
  onEdit: (prompt: PromptEntry) => void;
  onDelete: (prompt: PromptEntry) => void;
  onHistory: (prompt: PromptEntry) => void;
}

const PromptList: Component<PromptListProps> = (props) => {
  const [searchQuery, setSearchQuery] = createSignal('');
  const [sortBy, setSortBy] = createSignal<'name' | 'updated'>('updated');

  const filteredPrompts = createMemo(() => {
    let prompts = [...store.prompts()];

    // Apply search filter
    const query = searchQuery().toLowerCase().trim();
    if (query) {
      prompts = prompts.filter(p =>
        p.name.toLowerCase().includes(query) ||
        p.content.toLowerCase().includes(query) ||
        p.owner.toLowerCase().includes(query)
      );
    }

    // Apply sorting
    switch (sortBy()) {
      case 'name':
        prompts.sort((a, b) => a.name.localeCompare(b.name));
        break;
      case 'updated':
        prompts.sort((a, b) => new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime());
        break;
    }

    return prompts;
  });

  return (
    <div class="flex flex-col h-full">
      {/* Search and filters */}
      <div class="flex flex-col gap-3 p-4 border-b border-border">
        <div class="relative">
          <span class="material-icons absolute left-3 top-1/2 -translate-y-1/2 text-text-dim text-sm">
            search
          </span>
          <input
            type="text"
            placeholder="Search prompts..."
            value={searchQuery()}
            onInput={(e) => setSearchQuery(e.currentTarget.value)}
            class="w-full pl-10 pr-4 py-2 bg-surface-2 border border-border rounded-standard
                   text-sm focus:outline-none focus:ring-2 focus:ring-accent
                   placeholder:text-text-dim/50"
          />
        </div>

        <div class="flex items-center gap-4">
          <div class="flex items-center gap-2">
            <label class="text-xs text-text-dim">Sort:</label>
            <select
              value={sortBy()}
              onChange={(e) => setSortBy(e.currentTarget.value as 'name' | 'updated')}
              class="bg-surface-2 border border-border rounded px-2 py-1 text-xs
                     focus:outline-none focus:ring-2 focus:ring-accent"
            >
              <option value="updated">Last Updated</option>
              <option value="name">Name</option>
            </select>
          </div>

          <div class="ml-auto text-xs text-text-dim">
            {filteredPrompts().length} of {store.prompts().length} prompts
          </div>
        </div>
      </div>

      {/* Prompt list */}
      <div class="flex-1 overflow-auto p-4">
        <Show
          when={filteredPrompts().length > 0}
          fallback={
            <div class="flex flex-col items-center justify-center h-full text-text-dim">
              <span class="material-icons text-4xl mb-2">description</span>
              <p>No prompts found</p>
              <Show when={searchQuery()}>
                <p class="text-sm mt-1">Try adjusting your search</p>
              </Show>
            </div>
          }
        >
          <div class="flex flex-col gap-3">
            <For each={filteredPrompts()}>
              {(prompt) => (
                <PromptCard
                  prompt={prompt}
                  isSelected={props.selectedId === prompt.id}
                  onClick={() => props.onSelect(prompt)}
                  onEdit={() => props.onEdit(prompt)}
                  onDelete={() => props.onDelete(prompt)}
                  onHistory={() => props.onHistory(prompt)}
                />
              )}
            </For>
          </div>
        </Show>
      </div>
    </div>
  );
};

export default PromptList;
