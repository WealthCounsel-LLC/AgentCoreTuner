import { Show, createMemo } from 'solid-js';
import type { Component } from 'solid-js';
import { store } from '../stores/app';

const AuthStatusIcon: Component = () => {
  const statusColor = createMemo(() => {
    switch (store.authStatus()) {
      case 'valid':
        return 'text-success';
      case 'expiring':
        return 'text-warning';
      case 'expired':
      case 'error':
        return 'text-error';
      case 'refreshing':
        return 'text-accent animate-pulse';
      default:
        return 'text-text-dim';
    }
  });

  const statusTitle = createMemo(() => {
    switch (store.authStatus()) {
      case 'valid':
        return `Authenticated as ${store.userId()}`;
      case 'expiring':
        return 'Credentials expiring soon';
      case 'expired':
        return 'Credentials expired - click to login';
      case 'error':
        return 'Authentication error - click to retry';
      case 'refreshing':
        return 'Checking credentials...';
      default:
        return 'Not authenticated - click to login';
    }
  });

  const handleClick = () => {
    const status = store.authStatus();
    if (status === 'refreshing') return;

    if (status === 'valid') {
      // Re-verify when clicking on valid status
      store.verifyAuth();
    } else {
      // Try SSO login for expired/error/not-authenticated
      store.ssoLogin();
    }
  };

  return (
    <button
      onClick={handleClick}
      disabled={store.authStatus() === 'refreshing'}
      title={statusTitle()}
      class="p-2 rounded-small hover:bg-surface-2 transition-colors disabled:opacity-50"
    >
      <Show
        when={store.authStatus() !== 'refreshing'}
        fallback={
          <span class="material-icons text-accent animate-spin text-xl">sync</span>
        }
      >
        <span class={`material-icons text-xl ${statusColor()}`}>cloud</span>
      </Show>
    </button>
  );
};

export default AuthStatusIcon;
