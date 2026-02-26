import { Show } from 'solid-js';
import type { Component } from 'solid-js';
import type { Agent } from '../../types';

interface DeleteRuntimeDialogProps {
  isOpen: boolean;
  agent: Agent | null;
  onClose: () => void;
  onDelete: (id: string) => void;
}

const DeleteRuntimeDialog: Component<DeleteRuntimeDialogProps> = (props) => {
  const handleDelete = () => {
    if (props.agent) {
      props.onDelete(props.agent.id);
    }
  };

  return (
    <Show when={props.isOpen && props.agent}>
      <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
        <div class="bg-surface rounded-standard border border-border w-full max-w-md">
          <div class="flex items-center justify-between p-4 border-b border-border">
            <h2 class="text-lg font-semibold text-error">Delete Runtime</h2>
            <button
              onClick={props.onClose}
              class="p-1 hover:bg-surface-2 rounded transition-colors"
            >
              <span class="material-icons">close</span>
            </button>
          </div>

          <div class="p-4">
            <div class="flex items-start gap-3">
              <span class="material-icons text-error">warning</span>
              <div>
                <p class="text-sm">
                  Are you sure you want to delete <strong>{props.agent?.name}</strong>?
                </p>
                <p class="text-xs text-text-dim font-mono mt-1">
                  {props.agent?.id}
                </p>
                <p class="text-sm text-text-dim mt-3">
                  This action cannot be undone. The runtime will be permanently deleted from AWS.
                </p>
                <p class="text-sm text-warning mt-2">
                  Any active sessions using this runtime will be terminated.
                </p>
              </div>
            </div>
          </div>

          <div class="flex justify-end gap-3 p-4 border-t border-border">
            <button
              type="button"
              onClick={props.onClose}
              class="px-4 py-2 text-sm rounded-standard hover:bg-surface-2 transition-colors"
            >
              Cancel
            </button>
            <button
              type="button"
              onClick={handleDelete}
              class="px-4 py-2 text-sm bg-error text-crust rounded-standard font-medium
                     hover:bg-error/80 transition-colors"
            >
              <span class="material-icons text-sm mr-1">delete</span>
              Delete
            </button>
          </div>
        </div>
      </div>
    </Show>
  );
};

export default DeleteRuntimeDialog;
