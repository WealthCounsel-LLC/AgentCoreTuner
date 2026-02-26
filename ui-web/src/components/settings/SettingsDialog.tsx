import { Show } from 'solid-js';
import type { Component } from 'solid-js';
import SyncSettings from './SyncSettings';

interface SettingsDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

const SettingsDialog: Component<SettingsDialogProps> = (props) => {
  return (
    <Show when={props.isOpen}>
      <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
        <div class="bg-surface rounded-standard border border-border w-full max-w-lg max-h-[80vh] flex flex-col">
          {/* Header */}
          <div class="flex items-center justify-between p-4 border-b border-border">
            <h2 class="text-lg font-semibold flex items-center gap-2">
              <span class="material-icons">settings</span>
              Settings
            </h2>
            <button
              onClick={props.onClose}
              class="p-1 hover:bg-surface-2 rounded transition-colors"
            >
              <span class="material-icons">close</span>
            </button>
          </div>

          {/* Content */}
          <div class="flex-1 overflow-y-auto">
            <SyncSettings />
          </div>
        </div>
      </div>
    </Show>
  );
};

export default SettingsDialog;
