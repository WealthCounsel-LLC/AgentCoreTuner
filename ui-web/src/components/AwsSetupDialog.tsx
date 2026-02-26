import { Show } from 'solid-js';
import type { Component } from 'solid-js';
import { store } from '../stores/app';

const AwsSetupDialog: Component = () => {
  const handleOpenDocs = () => {
    // Open AWS CLI configuration docs in browser
    window.open('https://docs.aws.amazon.com/cli/latest/userguide/cli-configure-files.html', '_blank');
  };

  const handleDismiss = () => {
    store.setShowAwsSetupDialog(false);
  };

  const handleRecheck = async () => {
    await store.loadProfiles();
    if (store.profiles().length > 0) {
      store.setShowAwsSetupDialog(false);
    }
  };

  return (
    <Show when={store.showAwsSetupDialog()}>
      <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
        <div class="bg-surface rounded-standard border border-border p-6 w-full max-w-lg shadow-xl">
          <div class="flex items-center gap-3 mb-4">
            <span class="material-icons text-accent text-3xl">help</span>
            <h2 class="text-lg font-semibold">AWS Configuration Required</h2>
          </div>

          <p class="text-text-dim text-sm mb-4">
            No AWS profiles were found. You need to configure the AWS CLI before using this application.
          </p>

          <div class="bg-surface-2 rounded p-4 mb-4">
            <h3 class="font-medium text-sm mb-2">Quick Setup</h3>
            <p class="text-text-dim text-sm mb-3">
              Run this command in your terminal to configure AWS:
            </p>
            <code class="block bg-crust p-2 rounded text-sm font-mono">
              aws configure
            </code>
          </div>

          <div class="bg-surface-2 rounded p-4 mb-4">
            <h3 class="font-medium text-sm mb-2">For SSO Users</h3>
            <p class="text-text-dim text-sm mb-3">
              If your organization uses AWS SSO, run:
            </p>
            <code class="block bg-crust p-2 rounded text-sm font-mono">
              aws configure sso
            </code>
          </div>

          <div class="flex justify-end gap-3">
            <button
              type="button"
              onClick={handleDismiss}
              class="px-4 py-2 bg-surface-2 border border-border rounded-small
                     font-medium hover:bg-overlay-0 transition-colors"
            >
              Dismiss
            </button>
            <button
              type="button"
              onClick={handleRecheck}
              class="px-4 py-2 bg-surface-2 border border-border rounded-small
                     font-medium hover:bg-overlay-0 transition-colors"
            >
              Re-check
            </button>
            <button
              type="button"
              onClick={handleOpenDocs}
              class="px-4 py-2 bg-accent text-crust rounded-small
                     font-medium hover:bg-accent-hover transition-colors
                     flex items-center gap-2"
            >
              <span class="material-icons text-sm">open_in_new</span>
              View Documentation
            </button>
          </div>
        </div>
      </div>
    </Show>
  );
};

export default AwsSetupDialog;
