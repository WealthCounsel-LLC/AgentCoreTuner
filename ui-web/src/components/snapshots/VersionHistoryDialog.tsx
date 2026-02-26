import { createSignal, createEffect, For, Show } from 'solid-js';
import type { Component } from 'solid-js';
import * as tauri from '../../lib/tauri';

interface VersionHistoryDialogProps {
  isOpen: boolean;
  recipeId: string | null;
  recipeName: string | null;
  onClose: () => void;
}

const VersionHistoryDialog: Component<VersionHistoryDialogProps> = (props) => {
  const [history, setHistory] = createSignal<string[]>([]);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  createEffect(async () => {
    if (props.isOpen && props.recipeId) {
      setLoading(true);
      setError(null);
      try {
        const commits = await tauri.recipeHistory(props.recipeId);
        setHistory(commits);
      } catch (err) {
        setError(String(err));
        setHistory([]);
      } finally {
        setLoading(false);
      }
    }
  });

  return (
    <Show when={props.isOpen}>
      <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
        <div class="bg-surface rounded-standard border border-border w-full max-w-lg max-h-[80vh] flex flex-col">
          <div class="flex items-center justify-between p-4 border-b border-border">
            <div>
              <h2 class="text-lg font-semibold">Version History</h2>
              <p class="text-sm text-text-dim">{props.recipeName}</p>
            </div>
            <button
              onClick={props.onClose}
              class="p-1 hover:bg-surface-2 rounded transition-colors"
            >
              <span class="material-icons">close</span>
            </button>
          </div>

          <div class="flex-1 overflow-auto p-4">
            <Show when={loading()}>
              <div class="flex items-center justify-center py-8">
                <span class="material-icons animate-spin text-accent">sync</span>
                <span class="ml-2 text-text-dim">Loading history...</span>
              </div>
            </Show>

            <Show when={error()}>
              <div class="flex items-center gap-2 text-error py-4">
                <span class="material-icons">error</span>
                <span>{error()}</span>
              </div>
            </Show>

            <Show when={!loading() && !error() && history().length === 0}>
              <div class="text-center py-8 text-text-dim">
                <span class="material-icons text-3xl mb-2">history</span>
                <p>No version history available</p>
              </div>
            </Show>

            <Show when={!loading() && !error() && history().length > 0}>
              <div class="space-y-2">
                <For each={history()}>
                  {(commit, index) => (
                    <div class={`p-3 rounded-standard border ${
                      index() === 0
                        ? 'bg-accent/10 border-accent'
                        : 'bg-surface-2 border-border'
                    }`}>
                      <div class="flex items-center gap-2">
                        <span class="material-icons text-sm text-text-dim">commit</span>
                        <span class="font-mono text-xs text-text-dim">
                          {commit.substring(0, 7)}
                        </span>
                        <Show when={index() === 0}>
                          <span class="px-2 py-0.5 text-xs bg-accent text-crust rounded">
                            Latest
                          </span>
                        </Show>
                      </div>
                      <p class="text-sm mt-1 ml-6">{commit.substring(8) || 'No message'}</p>
                    </div>
                  )}
                </For>
              </div>
            </Show>
          </div>

          <div class="flex justify-end p-4 border-t border-border">
            <button
              onClick={props.onClose}
              class="px-4 py-2 text-sm rounded-standard hover:bg-surface-2 transition-colors"
            >
              Close
            </button>
          </div>
        </div>
      </div>
    </Show>
  );
};

export default VersionHistoryDialog;
