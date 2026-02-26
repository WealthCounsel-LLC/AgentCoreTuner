import { createSignal, createEffect, Show } from 'solid-js';
import type { Component } from 'solid-js';
import type { RecipeEntry } from '../../types';
import { store } from '../../stores/app';

interface ForkDialogProps {
  isOpen: boolean;
  sourceRecipe: RecipeEntry | null;
  onClose: () => void;
  onFork: (newRecipe: RecipeEntry) => void;
}

const generateId = (name: string) => {
  return name
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-|-$/g, '')
    .substring(0, 64);
};

const ForkDialog: Component<ForkDialogProps> = (props) => {
  const [name, setName] = createSignal('');
  const [description, setDescription] = createSignal('');

  createEffect(() => {
    if (props.isOpen && props.sourceRecipe) {
      setName(`${props.sourceRecipe.name}-fork`);
      setDescription(`Forked from ${props.sourceRecipe.name}: ${props.sourceRecipe.description}`);
    }
  });

  const handleSubmit = (e: Event) => {
    e.preventDefault();
    if (!props.sourceRecipe) return;

    const now = new Date().toISOString();
    const newRecipe: RecipeEntry = {
      ...props.sourceRecipe,
      id: generateId(name()),
      name: name(),
      description: description(),
      owner: store.userId() || 'unknown',
      forked_from: props.sourceRecipe.id,
      created_at: now,
      updated_at: now,
      // Reset ratings for fork
      personality_rating: null,
      accuracy_rating: null,
    };

    props.onFork(newRecipe);
  };

  const canSubmit = () => name().trim().length > 0;

  return (
    <Show when={props.isOpen && props.sourceRecipe}>
      <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
        <div class="bg-surface rounded-standard border border-border w-full max-w-md">
          <div class="flex items-center justify-between p-4 border-b border-border">
            <h2 class="text-lg font-semibold">Fork Snapshot</h2>
            <button
              onClick={props.onClose}
              class="p-1 hover:bg-surface-2 rounded transition-colors"
            >
              <span class="material-icons">close</span>
            </button>
          </div>

          <form onSubmit={handleSubmit} class="p-4">
            <p class="text-sm text-text-dim mb-4">
              Create a new snapshot based on <strong>{props.sourceRecipe?.name}</strong>
            </p>

            <div class="space-y-4">
              <div>
                <label class="block text-sm font-medium mb-1">
                  New Name <span class="text-error">*</span>
                </label>
                <input
                  type="text"
                  value={name()}
                  onInput={(e) => setName(e.currentTarget.value)}
                  placeholder="my-snapshot-fork"
                  class="w-full bg-surface-2 border border-border rounded-standard px-3 py-2 text-sm
                         focus:outline-none focus:ring-2 focus:ring-accent"
                />
                <Show when={name()}>
                  <p class="text-xs text-text-dim mt-1">ID: {generateId(name())}</p>
                </Show>
              </div>

              <div>
                <label class="block text-sm font-medium mb-1">Description</label>
                <textarea
                  value={description()}
                  onInput={(e) => setDescription(e.currentTarget.value)}
                  rows={2}
                  class="w-full bg-surface-2 border border-border rounded-standard px-3 py-2 text-sm
                         resize-none focus:outline-none focus:ring-2 focus:ring-accent"
                />
              </div>
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
              <span class="material-icons text-sm mr-1">call_split</span>
              Fork
            </button>
          </div>
        </div>
      </div>
    </Show>
  );
};

export default ForkDialog;
