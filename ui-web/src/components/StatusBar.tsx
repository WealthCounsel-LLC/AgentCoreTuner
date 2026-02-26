import { Show } from 'solid-js';
import type { Component } from 'solid-js';
import { store } from '../stores/app';

const StatusBar: Component = () => {
  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(store.statusMessage());
      // Could show a brief "copied" feedback here
    } catch {
      // Clipboard API may not be available
    }
  };

  const handleDismiss = () => {
    store.setStatusMessage('');
  };

  return (
    <Show when={store.statusMessage()}>
      <div
        class={`fixed bottom-4 right-4 max-w-md p-4 rounded-standard shadow-lg border
                flex items-start gap-3 z-40 ${
                  store.statusIsError()
                    ? 'bg-error/10 border-error text-error'
                    : 'bg-success/10 border-success text-success'
                }`}
      >
        <span class="material-icons text-xl">
          {store.statusIsError() ? 'error' : 'check_circle'}
        </span>

        <p class="flex-1 text-sm">{store.statusMessage()}</p>

        <div class="flex gap-1">
          <button
            onClick={handleCopy}
            title="Copy to clipboard"
            class="p-1 rounded hover:bg-black/10 transition-colors"
          >
            <span class="material-icons text-sm">content_copy</span>
          </button>
          <button
            onClick={handleDismiss}
            title="Dismiss"
            class="p-1 rounded hover:bg-black/10 transition-colors"
          >
            <span class="material-icons text-sm">close</span>
          </button>
        </div>
      </div>
    </Show>
  );
};

export default StatusBar;
