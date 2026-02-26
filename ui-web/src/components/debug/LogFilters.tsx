import type { Component } from 'solid-js';

interface LogFiltersProps {
  streamFilter: string;
  onStreamFilterChange: (value: string) => void;
  filterPattern: string;
  onFilterPatternChange: (value: string) => void;
}

const LogFilters: Component<LogFiltersProps> = (props) => {
  return (
    <div class="grid grid-cols-2 gap-4">
      <div class="flex flex-col gap-1">
        <label class="text-xs text-text-dim uppercase tracking-wide">Stream Filter</label>
        <input
          type="text"
          value={props.streamFilter}
          onInput={(e) => props.onStreamFilterChange(e.currentTarget.value)}
          placeholder="Filter by stream name..."
          class="bg-surface-2 border border-border rounded-standard px-3 py-2 text-sm
                 focus:outline-none focus:ring-2 focus:ring-accent"
        />
        <p class="text-xs text-text-dim">Filter logs by stream name pattern</p>
      </div>

      <div class="flex flex-col gap-1">
        <label class="text-xs text-text-dim uppercase tracking-wide">Filter Pattern</label>
        <input
          type="text"
          value={props.filterPattern}
          onInput={(e) => props.onFilterPatternChange(e.currentTarget.value)}
          placeholder='e.g., "ERROR" or "level = error"'
          class="bg-surface-2 border border-border rounded-standard px-3 py-2 text-sm
                 focus:outline-none focus:ring-2 focus:ring-accent"
        />
        <p class="text-xs text-text-dim">CloudWatch Logs filter pattern</p>
      </div>
    </div>
  );
};

export default LogFilters;
