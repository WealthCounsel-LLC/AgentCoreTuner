import { createSignal, createEffect, Show } from 'solid-js';
import type { Component } from 'solid-js';
import type { RecipeEntry } from '../../types';
import { store } from '../../stores/app';

interface SaveAsDialogProps {
  isOpen: boolean;
  baseName?: string;
  onClose: () => void;
  onSave: (recipe: RecipeEntry) => void;
}

const generateId = (name: string) => {
  return name
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-|-$/g, '')
    .substring(0, 64);
};

const SaveAsDialog: Component<SaveAsDialogProps> = (props) => {
  const [name, setName] = createSignal('');
  const [description, setDescription] = createSignal('');
  const [visibility, setVisibility] = createSignal<'public' | 'private'>('private');

  createEffect(() => {
    if (props.isOpen) {
      setName(props.baseName ? `${props.baseName}-copy` : '');
      setDescription('');
      setVisibility('private');
    }
  });

  const handleSubmit = (e: Event) => {
    e.preventDefault();

    const now = new Date().toISOString();
    const selectedModel = store.models()[store.selectedModelIdx()];
    const selectedPrompt = store.prompts()[store.selectedPromptIdx()];
    const selectedMemory = store.selectedMemoryIdx() > 0
      ? store.memories()[store.selectedMemoryIdx() - 1]
      : null;

    const newRecipe: RecipeEntry = {
      id: generateId(name()),
      name: name(),
      description: description(),
      model_id: selectedModel?.id || '',
      prompt_id: selectedPrompt?.id || '',
      system_prompt: store.systemPrompt(),
      memory_id: selectedMemory?.id || '',
      namespace_pattern: '',
      qualifier: store.selectedQualifier(),
      extra_params: {},
      owner: store.userId() || 'unknown',
      visibility: visibility(),
      personality_rating: null,
      accuracy_rating: null,
      forked_from: null,
      created_at: now,
      updated_at: now,
    };

    props.onSave(newRecipe);
  };

  const canSubmit = () => name().trim().length > 0;

  return (
    <Show when={props.isOpen}>
      <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
        <div class="bg-surface rounded-standard border border-border w-full max-w-md">
          <div class="flex items-center justify-between p-4 border-b border-border">
            <h2 class="text-lg font-semibold">Save As Recipe</h2>
            <button
              onClick={props.onClose}
              class="p-1 hover:bg-surface-2 rounded transition-colors"
            >
              <span class="material-icons">close</span>
            </button>
          </div>

          <form onSubmit={handleSubmit} class="p-4">
            <p class="text-sm text-text-dim mb-4">
              Save the current chat configuration as a new recipe.
            </p>

            {/* Preview current settings */}
            <div class="bg-surface-2 rounded-standard p-3 mb-4 text-xs">
              <div class="font-medium text-text mb-2">Current Settings</div>
              <div class="grid grid-cols-2 gap-1 text-text-dim">
                <span>Agent:</span>
                <span class="text-text truncate">
                  {store.agents()[store.selectedAgentIdx()]?.name || 'None'}
                </span>
                <span>Model:</span>
                <span class="text-text truncate">
                  {store.models()[store.selectedModelIdx()]?.label || 'Default'}
                </span>
                <span>Memory:</span>
                <span class="text-text truncate">
                  {store.selectedMemoryIdx() > 0
                    ? store.memories()[store.selectedMemoryIdx() - 1]?.name
                    : 'None'}
                </span>
                <span>Qualifier:</span>
                <span class="text-text">
                  {store.selectedQualifier()}
                </span>
              </div>
            </div>

            <div class="space-y-4">
              <div>
                <label class="block text-sm font-medium mb-1">
                  Name <span class="text-error">*</span>
                </label>
                <input
                  type="text"
                  value={name()}
                  onInput={(e) => setName(e.currentTarget.value)}
                  placeholder="my-snapshot"
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
                  placeholder="Describe this snapshot..."
                  rows={2}
                  class="w-full bg-surface-2 border border-border rounded-standard px-3 py-2 text-sm
                         resize-none focus:outline-none focus:ring-2 focus:ring-accent"
                />
              </div>

              <div>
                <label class="block text-sm font-medium mb-1">Visibility</label>
                <div class="flex gap-4">
                  <label class="flex items-center gap-2 cursor-pointer">
                    <input
                      type="radio"
                      name="visibility"
                      checked={visibility() === 'private'}
                      onChange={() => setVisibility('private')}
                      class="accent-accent"
                    />
                    <span class="text-sm">Private</span>
                  </label>
                  <label class="flex items-center gap-2 cursor-pointer">
                    <input
                      type="radio"
                      name="visibility"
                      checked={visibility() === 'public'}
                      onChange={() => setVisibility('public')}
                      class="accent-accent"
                    />
                    <span class="text-sm">Public</span>
                  </label>
                </div>
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
              <span class="material-icons text-sm mr-1">save</span>
              Save
            </button>
          </div>
        </div>
      </div>
    </Show>
  );
};

export default SaveAsDialog;
