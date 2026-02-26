import { createSignal, createEffect, createMemo, Show, For } from 'solid-js';
import type { Component } from 'solid-js';
import { store } from '../../stores/app';
import * as tauri from '../../lib/tauri';
import type { SessionSummary } from '../../lib/tauri';

const SessionIdInput: Component = () => {
  const [sessions, setSessions] = createSignal<SessionSummary[]>([]);
  const [loading, setLoading] = createSignal(false);
  const [manualMode, setManualMode] = createSignal(false);
  const [manualValue, setManualValue] = createSignal('');

  // Check if current session ID is in the fetched list
  const currentSessionInList = createMemo(() => {
    const currentId = store.sessionId();
    if (!currentId) return true; // empty means "(new session)" which is always shown
    return sessions().some(s => s.session_id === currentId);
  });

  // Fetch sessions when memory or runtimeUserId changes
  createEffect(() => {
    const memoryIdx = store.selectedMemoryIdx();
    const actorId = store.runtimeUserId();

    if (memoryIdx > 0 && actorId) {
      fetchSessions();
    } else {
      setSessions([]);
    }
  });

  const handleInput = (e: Event) => {
    const target = e.target as HTMLInputElement;
    setManualValue(target.value);
    store.setSessionId(target.value);
  };

  const handleSelectChange = (e: Event) => {
    const target = e.target as HTMLSelectElement;
    store.setSessionId(target.value);
  };

  const enterManualMode = () => {
    setManualValue(store.sessionId());
    setManualMode(true);
  };

  const exitManualMode = () => {
    setManualMode(false);
    // Keep current session ID if valid, otherwise clear
    if (!manualValue().trim()) {
      store.setSessionId('');
    }
  };

  const fetchSessions = async () => {
    const memoryIdx = store.selectedMemoryIdx();
    const actorId = store.runtimeUserId();

    if (memoryIdx <= 0 || !actorId) {
      setSessions([]);
      return;
    }

    const memory = store.memories()[memoryIdx - 1];
    if (!memory) {
      setSessions([]);
      return;
    }

    setLoading(true);

    try {
      const result = await tauri.listSessions(
        store.selectedProfile(),
        store.selectedRegion(),
        memory.id,
        actorId
      );
      setSessions(result);
    } catch (err) {
      // Silently fail - user may not have sessions yet
      setSessions([]);
    } finally {
      setLoading(false);
    }
  };

  const formatDate = (iso: string) => {
    if (!iso) return '';
    const date = new Date(iso);
    return date.toLocaleString(undefined, {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  return (
    <div class="flex flex-col gap-1">
      <label class="text-xs text-text-dim uppercase tracking-wide">Session ID</label>
      <div class="flex items-center gap-2">
        <Show when={!manualMode()}>
          {/* Select mode */}
          <select
            value={store.sessionId()}
            onChange={handleSelectChange}
            disabled={loading()}
            class="flex-1 bg-surface-2 border border-border rounded-small px-3 py-1.5 text-sm
                   focus:outline-none focus:ring-2 focus:ring-accent disabled:opacity-50"
          >
            <option value="">(new session)</option>
            {/* Show current session if it's not in the fetched list (e.g., just created) */}
            <Show when={store.sessionId() && !currentSessionInList()}>
              <option value={store.sessionId()}>
                {store.sessionId().length > 30
                  ? store.sessionId().slice(0, 30) + '...'
                  : store.sessionId()
                } • (current)
              </option>
            </Show>
            <For each={sessions()}>
              {(session) => (
                <option value={session.session_id}>
                  {session.session_id.length > 30
                    ? session.session_id.slice(0, 30) + '...'
                    : session.session_id
                  } • {formatDate(session.created_at)}
                </option>
              )}
            </For>
          </select>
          <button
            onClick={enterManualMode}
            title="Enter session ID manually"
            class="p-1.5 rounded-small hover:bg-surface-2 transition-colors"
          >
            <span class="material-icons text-text-dim text-sm">add</span>
          </button>
        </Show>

        <Show when={manualMode()}>
          {/* Manual entry mode */}
          <input
            type="text"
            value={manualValue()}
            onInput={handleInput}
            placeholder="Enter session ID..."
            class="flex-1 bg-surface-2 border border-border rounded-small px-3 py-1.5 text-sm
                   focus:outline-none focus:ring-2 focus:ring-accent
                   placeholder:text-text-dim/50"
          />
          <button
            onClick={exitManualMode}
            title="Back to session list"
            class="p-1.5 rounded-small hover:bg-surface-2 transition-colors"
          >
            <span class="material-icons text-text-dim text-sm">close</span>
          </button>
        </Show>

        <Show when={loading()}>
          <span class="material-icons text-text-dim text-sm animate-spin">sync</span>
        </Show>
      </div>
    </div>
  );
};

export default SessionIdInput;
