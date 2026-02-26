import { createSignal, Show, onMount } from 'solid-js';
import type { Component } from 'solid-js';
import { store } from '../../stores/app';
import * as tauri from '../../lib/tauri';
import type { LogEntry } from '../../types';
import LogGroupSelector from './LogGroupSelector';
import LogFilters from './LogFilters';
import LogDisplay from './LogDisplay';

const DebugTab: Component = () => {
  const [selectedLogGroup, setSelectedLogGroup] = createSignal('');
  const [streamFilter, setStreamFilter] = createSignal('');
  const [filterPattern, setFilterPattern] = createSignal('');
  const [searchQuery, setSearchQuery] = createSignal('');
  const [logs, setLogs] = createSignal<LogEntry[]>([]);
  const [loadingLogs, setLoadingLogs] = createSignal(false);
  const [logMetrics, setLogMetrics] = createSignal({ count: 0, timeRange: '' });

  onMount(async () => {
    // Load log groups on mount if not already loaded
    if (store.logGroups().length === 0 && store.authStatus() === 'valid') {
      await store.fetchLogGroups();
    }
  });

  const parseLogContent = (content: string): LogEntry[] => {
    // Parse log content - assume each line is a log entry
    // Format: timestamp message
    const lines = content.split('\n').filter(line => line.trim());
    return lines.map(line => {
      // Try to parse timestamp from beginning of line
      const match = line.match(/^(\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})?)\s+(.*)$/);
      if (match) {
        return { timestamp: match[1], message: match[2] };
      }
      // Fallback: use current time and whole line as message
      return { timestamp: new Date().toISOString(), message: line };
    });
  };

  const handleFetchLogs = async () => {
    if (!selectedLogGroup()) {
      store.showStatus('Please select a log group', true);
      return;
    }

    setLoadingLogs(true);
    try {
      const [metrics, content] = await tauri.getLogs(
        store.selectedProfile(),
        store.selectedRegion(),
        selectedLogGroup(),
        streamFilter() || undefined,
        filterPattern() || undefined
      );

      const fetchedLogs = parseLogContent(content);
      setLogs(fetchedLogs);

      // Set metrics from returned data
      setLogMetrics({
        count: fetchedLogs.length,
        timeRange: metrics || '',
      });

      store.showStatus(`Fetched ${fetchedLogs.length} log entries`, false);
    } catch (err) {
      store.showStatus(`Failed to fetch logs: ${err}`, true);
      setLogs([]);
    } finally {
      setLoadingLogs(false);
    }
  };

  const handleRefreshLogGroups = async () => {
    await store.fetchLogGroups();
  };

  const handleClearLogs = () => {
    setLogs([]);
    setLogMetrics({ count: 0, timeRange: '' });
  };

  return (
    <div class="h-full flex flex-col">
      {/* Header */}
      <div class="flex items-center justify-between p-4 border-b border-border bg-surface">
        <div>
          <h2 class="text-lg font-semibold">Debug & Logging</h2>
          <p class="text-sm text-text-dim">
            View CloudWatch logs and debug information
          </p>
        </div>
        <div class="flex items-center gap-4">
          {/* Dev Mode Toggle */}
          <label class="flex items-center gap-2 cursor-pointer">
            <input
              type="checkbox"
              checked={store.devMode()}
              onChange={(e) => store.setDevMode(e.currentTarget.checked)}
              class="accent-accent"
            />
            <span class="text-sm">Dev Mode</span>
          </label>
        </div>
      </div>

      {/* Controls */}
      <div class="p-4 border-b border-border bg-surface space-y-4">
        {/* Log Group Selector Row */}
        <div class="flex items-end gap-4">
          <div class="flex-1">
            <LogGroupSelector
              value={selectedLogGroup()}
              onChange={setSelectedLogGroup}
            />
          </div>
          <button
            onClick={handleRefreshLogGroups}
            disabled={store.loading().logGroups}
            class="px-3 py-2 text-sm bg-surface-2 border border-border rounded-standard
                   hover:bg-surface-3 transition-colors flex items-center gap-1
                   disabled:opacity-50"
            title="Refresh log groups"
          >
            <span class={`material-icons text-sm ${store.loading().logGroups ? 'animate-spin' : ''}`}>
              refresh
            </span>
          </button>
        </div>

        {/* Filters */}
        <LogFilters
          streamFilter={streamFilter()}
          onStreamFilterChange={setStreamFilter}
          filterPattern={filterPattern()}
          onFilterPatternChange={setFilterPattern}
        />

        {/* Action buttons */}
        <div class="flex items-center gap-4">
          <button
            onClick={handleFetchLogs}
            disabled={!selectedLogGroup() || loadingLogs()}
            class="px-4 py-2 text-sm bg-accent text-crust rounded-standard
                   hover:bg-accent-hover transition-colors flex items-center gap-1
                   disabled:opacity-50"
          >
            <span class={`material-icons text-sm ${loadingLogs() ? 'animate-spin' : ''}`}>
              {loadingLogs() ? 'sync' : 'download'}
            </span>
            Fetch Logs
          </button>

          <button
            onClick={handleClearLogs}
            disabled={logs().length === 0}
            class="px-4 py-2 text-sm bg-surface-2 border border-border rounded-standard
                   hover:bg-surface-3 transition-colors disabled:opacity-50"
          >
            Clear
          </button>

          {/* Search */}
          <div class="flex-1">
            <div class="relative">
              <span class="material-icons absolute left-3 top-1/2 -translate-y-1/2 text-text-dim text-sm">
                search
              </span>
              <input
                type="text"
                value={searchQuery()}
                onInput={(e) => setSearchQuery(e.currentTarget.value)}
                placeholder="Search in logs..."
                class="w-full pl-10 pr-4 py-2 bg-surface-2 border border-border rounded-standard
                       text-sm focus:outline-none focus:ring-2 focus:ring-accent
                       placeholder:text-text-dim/50"
              />
            </div>
          </div>
        </div>

        {/* Metrics */}
        <Show when={logMetrics().count > 0}>
          <div class="flex items-center gap-4 text-xs text-text-dim">
            <span>
              <strong>{logMetrics().count}</strong> entries
            </span>
            <span>
              Time range: <strong>{logMetrics().timeRange}</strong>
            </span>
          </div>
        </Show>
      </div>

      {/* Log Display */}
      <div class="flex-1 overflow-hidden bg-surface-2 m-4 rounded-standard border border-border">
        <LogDisplay
          logs={logs()}
          loading={loadingLogs()}
          searchQuery={searchQuery()}
        />
      </div>

      {/* Dev Mode Info */}
      <Show when={store.devMode()}>
        <div class="mx-4 mb-4 p-4 bg-surface rounded-standard border border-accent/50">
          <h3 class="text-sm font-semibold text-accent mb-2 flex items-center gap-2">
            <span class="material-icons text-sm">code</span>
            Dev Mode Enabled
          </h3>
          <p class="text-xs text-text-dim">
            Debug logs from AWS CLI commands will appear in the browser console.
            Check the developer tools for detailed traces.
          </p>
        </div>
      </Show>
    </div>
  );
};

export default DebugTab;
