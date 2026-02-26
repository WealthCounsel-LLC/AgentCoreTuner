import { createSignal, Show } from 'solid-js';
import type { Component } from 'solid-js';
import { store } from '../stores/app';

const MfaDialog: Component = () => {
  const [mfaCode, setMfaCode] = createSignal('');
  const [error, setError] = createSignal('');

  const handleSubmit = (e: Event) => {
    e.preventDefault();
    const code = mfaCode().trim();

    // Validate: exactly 6 digits
    if (!/^\d{6}$/.test(code)) {
      setError('MFA code must be exactly 6 digits');
      return;
    }

    // TODO: Submit MFA code via Tauri command
    // For now, just close the dialog
    store.setShowMfaDialog(false);
    setMfaCode('');
    setError('');
  };

  const handleCancel = () => {
    store.setShowMfaDialog(false);
    setMfaCode('');
    setError('');
  };

  return (
    <Show when={store.showMfaDialog()}>
      <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
        <div class="bg-surface rounded-standard border border-border p-6 w-full max-w-md shadow-xl">
          <h2 class="text-lg font-semibold mb-4">MFA Required</h2>
          <p class="text-text-dim text-sm mb-4">
            Enter the 6-digit code from your authenticator app for:
          </p>
          <p class="text-sm font-mono bg-surface-2 p-2 rounded mb-4 break-all">
            {store.mfaSerial()}
          </p>

          <form onSubmit={handleSubmit}>
            <input
              type="text"
              inputMode="numeric"
              maxLength={6}
              placeholder="000000"
              value={mfaCode()}
              onInput={(e) => {
                setMfaCode(e.currentTarget.value.replace(/\D/g, ''));
                setError('');
              }}
              class="w-full bg-surface-2 border border-border rounded-small px-4 py-2 text-center
                     text-2xl font-mono tracking-widest
                     focus:outline-none focus:ring-2 focus:ring-accent"
              autofocus
            />

            <Show when={error()}>
              <p class="text-error text-sm mt-2">{error()}</p>
            </Show>

            <div class="flex justify-end gap-3 mt-6">
              <button
                type="button"
                onClick={handleCancel}
                class="px-4 py-2 bg-surface-2 border border-border rounded-small
                       font-medium hover:bg-overlay-0 transition-colors"
              >
                Cancel
              </button>
              <button
                type="submit"
                class="px-4 py-2 bg-accent text-crust rounded-small
                       font-medium hover:bg-accent-hover transition-colors"
              >
                Submit
              </button>
            </div>
          </form>
        </div>
      </div>
    </Show>
  );
};

export default MfaDialog;
