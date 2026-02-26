import { For, Show, createMemo } from 'solid-js';
import type { Component } from 'solid-js';
import type { LogEntry } from '../../types';

interface LogDisplayProps {
  logs: LogEntry[];
  loading: boolean;
  searchQuery: string;
}

const LogDisplay: Component<LogDisplayProps> = (props) => {
  const filteredLogs = createMemo(() => {
    if (!props.searchQuery.trim()) return props.logs;

    const query = props.searchQuery.toLowerCase();
    return props.logs.filter(log =>
      log.message.toLowerCase().includes(query)
    );
  });

  const formatTimestamp = (ts: string) => {
    try {
      const date = new Date(ts);
      return date.toLocaleString();
    } catch {
      return ts;
    }
  };

  const getLogLevel = (message: string) => {
    const lowerMessage = message.toLowerCase();
    if (lowerMessage.includes('error') || lowerMessage.includes('exception') || lowerMessage.includes('fatal')) {
      return 'error';
    }
    if (lowerMessage.includes('warn')) {
      return 'warning';
    }
    if (lowerMessage.includes('info')) {
      return 'info';
    }
    if (lowerMessage.includes('debug')) {
      return 'debug';
    }
    return 'default';
  };

  const getLevelColor = (level: string) => {
    switch (level) {
      case 'error':
        return 'text-error';
      case 'warning':
        return 'text-warning';
      case 'info':
        return 'text-accent';
      case 'debug':
        return 'text-text-dim';
      default:
        return 'text-text';
    }
  };

  return (
    <div class="flex-1 overflow-auto">
      <Show when={props.loading}>
        <div class="flex items-center justify-center py-8">
          <span class="material-icons animate-spin text-accent">sync</span>
          <span class="ml-2 text-text-dim">Loading logs...</span>
        </div>
      </Show>

      <Show when={!props.loading && filteredLogs().length === 0}>
        <div class="flex flex-col items-center justify-center h-full text-text-dim py-8">
          <span class="material-icons text-4xl mb-2">article</span>
          <p>No logs to display</p>
          <Show when={props.searchQuery}>
            <p class="text-sm mt-1">Try adjusting your search</p>
          </Show>
          <Show when={!props.searchQuery && props.logs.length === 0}>
            <p class="text-sm mt-1">Select a log group and click "Fetch Logs"</p>
          </Show>
        </div>
      </Show>

      <Show when={!props.loading && filteredLogs().length > 0}>
        <div class="space-y-1">
          <For each={filteredLogs()}>
            {(log) => {
              const level = getLogLevel(log.message);
              return (
                <div class="group px-3 py-2 hover:bg-surface-2 rounded font-mono text-xs">
                  <div class="flex items-start gap-3">
                    <span class="text-text-dim whitespace-nowrap flex-shrink-0">
                      {formatTimestamp(log.timestamp)}
                    </span>
                    <span class={`flex-1 whitespace-pre-wrap break-all ${getLevelColor(level)}`}>
                      {log.message}
                    </span>
                  </div>
                </div>
              );
            }}
          </For>
        </div>
      </Show>
    </div>
  );
};

export default LogDisplay;
