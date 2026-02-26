import { onMount, Show } from 'solid-js';
import type { Component } from 'solid-js';
import { store } from './stores/app';
import {
  TopBar,
  TabBar,
  StatusBar,
  MfaDialog,
  AwsCliDialog,
  AwsSetupDialog,
} from './components';
import { ChatTab } from './components/chat';
import { SnapshotsTab } from './components/snapshots';
import { PromptsTab } from './components/prompts';
import { RuntimesTab } from './components/runtimes';
import { DebugTab } from './components/debug';

const App: Component = () => {
  onMount(() => {
    // Initialize the app on mount
    store.initialize();
  });

  return (
    <div class="min-h-screen bg-bg text-text flex flex-col">
      {/* Top Bar */}
      <TopBar />

      {/* Tab Bar */}
      <TabBar />

      {/* Main Content */}
      <main class="flex-1 p-4 overflow-hidden">
        <Show when={store.activeTab() === 'chat'}>
          <ChatTab />
        </Show>
        <Show when={store.activeTab() === 'debug'}>
          <DebugTab />
        </Show>
        <Show when={store.activeTab() === 'snapshots'}>
          <SnapshotsTab />
        </Show>
        <Show when={store.activeTab() === 'prompts'}>
          <PromptsTab />
        </Show>
        <Show when={store.activeTab() === 'runtimes'}>
          <RuntimesTab />
        </Show>
      </main>

      {/* Footer */}
      <footer class="bg-surface border-t border-border px-4 py-2 text-xs text-text-dim flex items-center justify-between">
        <span>
          {store.authStatus() === 'valid'
            ? `Authenticated as ${store.userId()}`
            : 'Not authenticated'}
        </span>
        <span>
          {store.selectedProfile()} / {store.selectedRegion()}
        </span>
      </footer>

      {/* Dialogs */}
      <MfaDialog />
      <AwsCliDialog />
      <AwsSetupDialog />

      {/* Status Snackbar */}
      <StatusBar />
    </div>
  );
};

export default App;
