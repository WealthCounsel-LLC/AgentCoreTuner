import { createSignal, For, Show, createMemo } from 'solid-js';
import type { Component } from 'solid-js';
import { store } from '../../stores/app';
import RuntimeCard from './RuntimeCard';
import type { Agent } from '../../types';

interface RuntimeListProps {
  selectedId: string | null;
  onSelect: (agent: Agent) => void;
  onEdit: (agent: Agent) => void;
  onDelete: (agent: Agent) => void;
  onViewDetails: (agent: Agent) => void;
}

const RuntimeList: Component<RuntimeListProps> = (props) => {
  const [searchQuery, setSearchQuery] = createSignal('');
  const [statusFilter, setStatusFilter] = createSignal<'all' | 'active' | 'other'>('all');
  const [sortBy, setSortBy] = createSignal<'name' | 'updated' | 'status'>('updated');

  const filteredAgents = createMemo(() => {
    let agents = [...store.agents()];

    // Apply search filter
    const query = searchQuery().toLowerCase().trim();
    if (query) {
      agents = agents.filter(a =>
        a.name.toLowerCase().includes(query) ||
        a.id.toLowerCase().includes(query)
      );
    }

    // Apply status filter
    if (statusFilter() === 'active') {
      agents = agents.filter(a => a.status.toLowerCase() === 'active' || a.status.toLowerCase() === 'running');
    } else if (statusFilter() === 'other') {
      agents = agents.filter(a => a.status.toLowerCase() !== 'active' && a.status.toLowerCase() !== 'running');
    }

    // Apply sorting
    switch (sortBy()) {
      case 'name':
        agents.sort((a, b) => a.name.localeCompare(b.name));
        break;
      case 'updated':
        agents.sort((a, b) => new Date(b.last_updated).getTime() - new Date(a.last_updated).getTime());
        break;
      case 'status':
        agents.sort((a, b) => a.status.localeCompare(b.status));
        break;
    }

    return agents;
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
            placeholder="Search runtimes..."
            value={searchQuery()}
            onInput={(e) => setSearchQuery(e.currentTarget.value)}
            class="w-full pl-10 pr-4 py-2 bg-surface-2 border border-border rounded-standard
                   text-sm focus:outline-none focus:ring-2 focus:ring-accent
                   placeholder:text-text-dim/50"
          />
        </div>

        <div class="flex items-center gap-4">
          <div class="flex items-center gap-2">
            <label class="text-xs text-text-dim">Status:</label>
            <select
              value={statusFilter()}
              onChange={(e) => setStatusFilter(e.currentTarget.value as 'all' | 'active' | 'other')}
              class="bg-surface-2 border border-border rounded px-2 py-1 text-xs
                     focus:outline-none focus:ring-2 focus:ring-accent"
            >
              <option value="all">All</option>
              <option value="active">Active</option>
              <option value="other">Other</option>
            </select>
          </div>

          <div class="flex items-center gap-2">
            <label class="text-xs text-text-dim">Sort:</label>
            <select
              value={sortBy()}
              onChange={(e) => setSortBy(e.currentTarget.value as 'name' | 'updated' | 'status')}
              class="bg-surface-2 border border-border rounded px-2 py-1 text-xs
                     focus:outline-none focus:ring-2 focus:ring-accent"
            >
              <option value="updated">Last Updated</option>
              <option value="name">Name</option>
              <option value="status">Status</option>
            </select>
          </div>

          <div class="ml-auto text-xs text-text-dim">
            {filteredAgents().length} of {store.agents().length} runtimes
          </div>
        </div>
      </div>

      {/* Runtime list */}
      <div class="flex-1 overflow-auto p-4">
        <Show
          when={filteredAgents().length > 0}
          fallback={
            <div class="flex flex-col items-center justify-center h-full text-text-dim">
              <span class="material-icons text-4xl mb-2">dns</span>
              <p>No runtimes found</p>
              <Show when={searchQuery() || statusFilter() !== 'all'}>
                <p class="text-sm mt-1">Try adjusting your filters</p>
              </Show>
            </div>
          }
        >
          <div class="flex flex-col gap-3">
            <For each={filteredAgents()}>
              {(agent) => (
                <RuntimeCard
                  agent={agent}
                  isSelected={props.selectedId === agent.id}
                  onClick={() => props.onSelect(agent)}
                  onEdit={() => props.onEdit(agent)}
                  onDelete={() => props.onDelete(agent)}
                  onViewDetails={() => props.onViewDetails(agent)}
                />
              )}
            </For>
          </div>
        </Show>
      </div>
    </div>
  );
};

export default RuntimeList;
