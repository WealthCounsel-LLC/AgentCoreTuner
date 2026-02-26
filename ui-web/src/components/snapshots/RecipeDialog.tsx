import { createSignal, createEffect, For, Show } from 'solid-js';
import type { Component } from 'solid-js';
import type { RecipeEntry } from '../../types';
import { store } from '../../stores/app';

interface RecipeDialogProps {
  isOpen: boolean;
  mode: 'create' | 'edit';
  recipe: RecipeEntry | null;
  onClose: () => void;
  onSave: (recipe: RecipeEntry, message?: string) => void;
}

const generateId = (name: string) => {
  return name
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-|-$/g, '')
    .substring(0, 64);
};

const RecipeDialog: Component<RecipeDialogProps> = (props) => {
  const [name, setName] = createSignal('');
  const [description, setDescription] = createSignal('');
  const [modelId, setModelId] = createSignal('');
  const [promptId, setPromptId] = createSignal('');
  const [systemPrompt, setSystemPrompt] = createSignal('');
  const [memoryId, setMemoryId] = createSignal('');
  const [namespacePattern, setNamespacePattern] = createSignal('');
  const [qualifier, setQualifier] = createSignal('');
  const [visibility, setVisibility] = createSignal<'public' | 'private'>('private');
  const [commitMessage, setCommitMessage] = createSignal('');

  // Reset form when dialog opens
  createEffect(() => {
    if (props.isOpen) {
      if (props.mode === 'edit' && props.recipe) {
        setName(props.recipe.name);
        setDescription(props.recipe.description);
        setModelId(props.recipe.model_id);
        setPromptId(props.recipe.prompt_id);
        setSystemPrompt(props.recipe.system_prompt);
        setMemoryId(props.recipe.memory_id);
        setNamespacePattern(props.recipe.namespace_pattern);
        setQualifier(props.recipe.qualifier);
        setVisibility(props.recipe.visibility as 'public' | 'private');
        setCommitMessage('');
      } else {
        // Create mode - use current chat settings
        setName('');
        setDescription('');
        setModelId(store.models()[store.selectedModelIdx()]?.id || '');
        setPromptId(store.prompts()[store.selectedPromptIdx()]?.id || '');
        setSystemPrompt(store.systemPrompt());
        setMemoryId(store.memories()[store.selectedMemoryIdx() - 1]?.id || '');
        setNamespacePattern('');
        setQualifier('LATEST');
        setVisibility('private');
        setCommitMessage('');
      }
    }
  });

  const handleSubmit = (e: Event) => {
    e.preventDefault();

    const now = new Date().toISOString();
    const recipe: RecipeEntry = {
      id: props.mode === 'edit' && props.recipe ? props.recipe.id : generateId(name()),
      name: name(),
      description: description(),
      model_id: modelId(),
      prompt_id: promptId(),
      system_prompt: systemPrompt(),
      memory_id: memoryId(),
      namespace_pattern: namespacePattern(),
      qualifier: qualifier(),
      extra_params: props.recipe?.extra_params || {},
      owner: props.recipe?.owner || store.userId() || 'unknown',
      visibility: visibility(),
      personality_rating: props.recipe?.personality_rating || null,
      accuracy_rating: props.recipe?.accuracy_rating || null,
      forked_from: props.recipe?.forked_from || null,
      created_at: props.recipe?.created_at || now,
      updated_at: now,
    };

    props.onSave(recipe, commitMessage() || undefined);
  };

  const canSubmit = () => name().trim().length > 0;

  return (
    <Show when={props.isOpen}>
      <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
        <div class="bg-surface rounded-standard border border-border w-full max-w-2xl max-h-[90vh] flex flex-col">
          <div class="flex items-center justify-between p-4 border-b border-border">
            <h2 class="text-lg font-semibold">
              {props.mode === 'create' ? 'Create Snapshot' : 'Edit Snapshot'}
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
                  placeholder="My Snapshot"
                  class="w-full bg-surface-2 border border-border rounded-standard px-3 py-2 text-sm
                         focus:outline-none focus:ring-2 focus:ring-accent"
                />
                <Show when={props.mode === 'create' && name()}>
                  <p class="text-xs text-text-dim mt-1">ID: {generateId(name())}</p>
                </Show>
              </div>

              {/* Description */}
              <div>
                <label class="block text-sm font-medium mb-1">Description</label>
                <textarea
                  value={description()}
                  onInput={(e) => setDescription(e.currentTarget.value)}
                  placeholder="Describe what this snapshot does..."
                  rows={2}
                  class="w-full bg-surface-2 border border-border rounded-standard px-3 py-2 text-sm
                         resize-none focus:outline-none focus:ring-2 focus:ring-accent"
                />
              </div>

              {/* Model */}
              <div>
                <label class="block text-sm font-medium mb-1">Model</label>
                <select
                  value={modelId()}
                  onChange={(e) => setModelId(e.currentTarget.value)}
                  class="w-full bg-surface-2 border border-border rounded-standard px-3 py-2 text-sm
                         focus:outline-none focus:ring-2 focus:ring-accent"
                >
                  <option value="">Default (agent's model)</option>
                  <For each={store.models()}>
                    {(model) => <option value={model.id}>{model.label}</option>}
                  </For>
                </select>
              </div>

              {/* Prompt Selection */}
              <div>
                <label class="block text-sm font-medium mb-1">Prompt</label>
                <select
                  value={promptId()}
                  onChange={(e) => setPromptId(e.currentTarget.value)}
                  class="w-full bg-surface-2 border border-border rounded-standard px-3 py-2 text-sm
                         focus:outline-none focus:ring-2 focus:ring-accent"
                >
                  <option value="">Use inline system prompt</option>
                  <For each={store.prompts()}>
                    {(prompt) => <option value={prompt.id}>{prompt.name}</option>}
                  </For>
                </select>
              </div>

              {/* System Prompt (inline) */}
              <Show when={!promptId()}>
                <div>
                  <label class="block text-sm font-medium mb-1">System Prompt</label>
                  <textarea
                    value={systemPrompt()}
                    onInput={(e) => setSystemPrompt(e.currentTarget.value)}
                    placeholder="Enter system prompt..."
                    rows={4}
                    class="w-full bg-surface-2 border border-border rounded-standard px-3 py-2 text-sm
                           font-mono resize-none focus:outline-none focus:ring-2 focus:ring-accent"
                  />
                </div>
              </Show>

              {/* Memory */}
              <div>
                <label class="block text-sm font-medium mb-1">Memory</label>
                <select
                  value={memoryId()}
                  onChange={(e) => setMemoryId(e.currentTarget.value)}
                  class="w-full bg-surface-2 border border-border rounded-standard px-3 py-2 text-sm
                         focus:outline-none focus:ring-2 focus:ring-accent"
                >
                  <option value="">None</option>
                  <For each={store.memories()}>
                    {(memory) => <option value={memory.id}>{memory.name}</option>}
                  </For>
                </select>
              </div>

              {/* Namespace Pattern */}
              <Show when={memoryId()}>
                <div>
                  <label class="block text-sm font-medium mb-1">Namespace Pattern</label>
                  <input
                    type="text"
                    value={namespacePattern()}
                    onInput={(e) => setNamespacePattern(e.currentTarget.value)}
                    placeholder="/strategy/{id}/actor/{actorId}/"
                    class="w-full bg-surface-2 border border-border rounded-standard px-3 py-2 text-sm
                           font-mono focus:outline-none focus:ring-2 focus:ring-accent"
                  />
                </div>
              </Show>

              {/* Qualifier */}
              <div>
                <label class="block text-sm font-medium mb-1">Qualifier</label>
                <input
                  type="text"
                  value={qualifier()}
                  onInput={(e) => setQualifier(e.currentTarget.value)}
                  placeholder="LATEST"
                  class="w-full bg-surface-2 border border-border rounded-standard px-3 py-2 text-sm
                         focus:outline-none focus:ring-2 focus:ring-accent"
                />
              </div>

              {/* Visibility */}
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

export default RecipeDialog;
