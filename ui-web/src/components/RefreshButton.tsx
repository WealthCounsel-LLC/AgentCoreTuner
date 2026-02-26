import { createMemo } from 'solid-js';
import type { Component } from 'solid-js';
import { store } from '../stores/app';

const RefreshButton: Component = () => {
  const isLoading = createMemo(() => {
    const l = store.loading();
    return l.agents || l.models || l.memories || l.logGroups;
  });

  const handleClick = () => {
    if (store.authStatus() === 'valid') {
      store.refreshAll();
    } else {
      store.verifyAuth();
    }
  };

  return (
    <button
      onClick={handleClick}
      disabled={isLoading()}
      title="Refresh all data"
      class="p-2 rounded-small hover:bg-surface-2 transition-colors disabled:opacity-50"
    >
      <span
        class={`material-icons text-text-dim text-xl ${isLoading() ? 'animate-spin' : ''}`}
      >
        refresh
      </span>
    </button>
  );
};

export default RefreshButton;
