import { Show, createSignal } from 'solid-js';
import type { Component } from 'solid-js';
import { store } from '../stores/app';

const AwsCliDialog: Component = () => {
  const [installing, setInstalling] = createSignal(false);
  const [installOutput, setInstallOutput] = createSignal('');

  const handleInstall = async () => {
    setInstalling(true);
    setInstallOutput('Installing AWS CLI via Homebrew...\n');

    // TODO: Use Tauri shell to run brew install
    // For now, simulate the process
    try {
      // This would use tauri shell plugin to run: brew install awscli
      setInstallOutput(prev => prev + '$ brew install awscli\n');
      // Simulate installation output
      await new Promise(resolve => setTimeout(resolve, 2000));
      setInstallOutput(prev => prev + 'AWS CLI installed successfully!\n');

      // Re-check AWS CLI
      await store.checkAwsCli();
      if (store.awsCliInstalled()) {
        store.setShowAwsCliDialog(false);
        store.showStatus('AWS CLI installed successfully');
      }
    } catch (err) {
      setInstallOutput(prev => prev + `Error: ${err}\n`);
    } finally {
      setInstalling(false);
    }
  };

  const handleRecheck = async () => {
    const installed = await store.checkAwsCli();
    if (installed) {
      store.setShowAwsCliDialog(false);
      store.showStatus(`AWS CLI detected: ${store.awsCliVersion()}`);
    }
  };

  const handleDismiss = () => {
    store.setShowAwsCliDialog(false);
  };

  return (
    <Show when={store.showAwsCliDialog()}>
      <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
        <div class="bg-surface rounded-standard border border-border p-6 w-full max-w-lg shadow-xl">
          <div class="flex items-center gap-3 mb-4">
            <span class="material-icons text-warning text-3xl">warning</span>
            <h2 class="text-lg font-semibold">AWS CLI Not Found</h2>
          </div>

          <p class="text-text-dim text-sm mb-4">
            The AWS Command Line Interface (CLI) is required to use this application.
            It was not detected on your system.
          </p>

          <Show when={installOutput()}>
            <pre class="bg-crust p-3 rounded text-sm font-mono text-text-dim overflow-auto max-h-40 mb-4">
              {installOutput()}
            </pre>
          </Show>

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
              disabled={installing()}
              class="px-4 py-2 bg-surface-2 border border-border rounded-small
                     font-medium hover:bg-overlay-0 transition-colors disabled:opacity-50"
            >
              Re-check
            </button>
            <button
              type="button"
              onClick={handleInstall}
              disabled={installing()}
              class="px-4 py-2 bg-accent text-crust rounded-small
                     font-medium hover:bg-accent-hover transition-colors disabled:opacity-50
                     flex items-center gap-2"
            >
              <Show when={installing()}>
                <span class="material-icons text-sm animate-spin">sync</span>
              </Show>
              Install (Homebrew)
            </button>
          </div>
        </div>
      </div>
    </Show>
  );
};

export default AwsCliDialog;
