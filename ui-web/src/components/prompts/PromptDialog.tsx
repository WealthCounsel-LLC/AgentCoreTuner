import { createSignal, createEffect, Show } from 'solid-js';
import type { Component } from 'solid-js';
import type { PromptEntry } from '../../types';
import { store } from '../../stores/app';

interface PromptDialogProps {
  isOpen: boolean;
  mode: 'create' | 'edit';
  prompt: PromptEntry | null;
  onClose: () => void;
  onSave: (prompt: PromptEntry, message?: string) => void;
}

const generateId = (name: string) => {
  return name
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-|-$/g, '')
    .substring(0, 64);
};

const PromptDialog: Component<PromptDialogProps> = (props) => {
  const [name, setName] = createSignal('');
  const [content, setContent] = createSignal('');
  const [commitMessage, setCommitMessage] = createSignal('');

  // Reset form when dialog opens
  createEffect(() => {
    if (props.isOpen) {
      if (props.mode === 'edit' && props.prompt) {
        setName(props.prompt.name);
        setContent(props.prompt.content);
        setCommitMessage('');
      } else {
        setName('');
        setContent('');
        setCommitMessage('');
      }
    }
  });

  const handleSubmit = (e: Event) => {
    e.preventDefault();

    const now = new Date().toISOString();
    const prompt: PromptEntry = {
      id: props.mode === 'edit' && props.prompt ? props.prompt.id : generateId(name()),
      name: name(),
      content: content(),
      owner: props.prompt?.owner || store.userId() || 'unknown',
      created_at: props.prompt?.created_at || now,
      updated_at: now,
    };

    props.onSave(prompt, commitMessage() || undefined);
  };

  const canSubmit = () => name().trim().length > 0 && content().trim().length > 0;

  return (
    <Show when={props.isOpen}>
      <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
        <div class="bg-surface rounded-standard border border-border w-full max-w-2xl max-h-[90vh] flex flex-col">
          <div class="flex items-center justify-between p-4 border-b border-border">
            <h2 class="text-lg font-semibold">
              {props.mode === 'create' ? 'Create Prompt' : 'Edit Prompt'}
            </h2>
            <button
              onClick={props.onClose}
              class="p-1 hover:bg-surface-2 rounded transition-colors"
            >
              <span class="material-icons">close</span>
            </button>
          </div>

          <form onSubmit={handleSubmit} class="flex-1 overflow-auto p-4">
            <div class="grid gap-4">
              {/* Name */}
              <div>
                <label class="block text-sm font-medium mb-1">
                  Name <span class="text-error">*</span>
                </label>
                <input
                  type="text"
                  value={name()}
                  onInput={(e) => setName(e.currentTarget.value)}
                  placeholder="My Prompt"
                  class="w-full bg-surface-2 border border-border rounded-standard px-3 py-2 text-sm
                         focus:outline-none focus:ring-2 focus:ring-accent"
                />
                <Show when={props.mode === 'create' && name()}>
                  <p class="text-xs text-text-dim mt-1">ID: {generateId(name())}</p>
                </Show>
              </div>

              {/* Content */}
              <div>
                <label class="block text-sm font-medium mb-1">
                  Content <span class="text-error">*</span>
                </label>
                <textarea
                  value={content()}
                  onInput={(e) => setContent(e.currentTarget.value)}
                  placeholder="Enter your system prompt here..."
                  rows={12}
                  class="w-full bg-surface-2 border border-border rounded-standard px-3 py-2 text-sm
                         font-mono resize-none focus:outline-none focus:ring-2 focus:ring-accent"
                />
                <p class="text-xs text-text-dim mt-1">
                  {content().length} characters
                </p>
              </div>

              {/* Commit Message (for edits) */}
              <Show when={props.mode === 'edit'}>
                <div>
                  <label class="block text-sm font-medium mb-1">Version Description</label>
                  <input
                    type="text"
                    value={commitMessage()}
                    onInput={(e) => setCommitMessage(e.currentTarget.value)}
                    placeholder="Describe what changed (optional)"
                    class="w-full bg-surface-2 border border-border rounded-standard px-3 py-2 text-sm
                           focus:outline-none focus:ring-2 focus:ring-accent"
                  />
                </div>
              </Show>
            </div>
          </form>

          <div class="flex justify-end gap-3 p-4 border-t border-border">
            <button
              type="button"
              onClick={props.onClose}
              class="px-4 py-2 text-sm rounded-standard hover:bg-surface-2 transition-colors"
            >
              Cancel
            </button>
            <button
              type="submit"
              onClick={handleSubmit}
              disabled={!canSubmit()}
              class="px-4 py-2 text-sm bg-accent text-crust rounded-standard font-medium
                     hover:bg-accent-hover transition-colors disabled:opacity-50"
            >
              {props.mode === 'create' ? 'Create' : 'Save'}
            </button>
          </div>
        </div>
      </div>
    </Show>
  );
};

export default PromptDialog;
