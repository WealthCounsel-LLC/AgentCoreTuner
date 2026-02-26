import { For, Show, createMemo, createSignal, createEffect } from 'solid-js';
import type { Component } from 'solid-js';
import { store } from '../../stores/app';
import * as tauri from '../../lib/tauri';
import type { RecipeEntry } from '../../types';

const RecipeSelector: Component = () => {
  const [showSaveAs, setShowSaveAs] = createSignal(false);
  const [newName, setNewName] = createSignal('');
  const [saving, setSaving] = createSignal(false);
  const [initialLoadDone, setInitialLoadDone] = createSignal(false);

  const selectedRecipeIdx = createMemo(() => {
    const activeId = store.activeRecipeId();
    if (!activeId) return 0;
    const idx = store.recipes().findIndex(r => r.id === activeId);
    return idx >= 0 ? idx + 1 : 0;
  });

  const activeRecipe = createMemo(() => {
    const activeId = store.activeRecipeId();
    if (!activeId) return null;
    return store.recipes().find(r => r.id === activeId) || null;
  });

  // Apply recipe settings on initial load only (not on subsequent updates)
  createEffect(() => {
    // Skip if we've already done initial load
    if (initialLoadDone()) return;

    const activeId = store.activeRecipeId();
    const recipes = store.recipes();
    const models = store.models();

    // Wait until all required data is loaded
    if (activeId && recipes.length > 0 && models.length > 0) {
      const recipe = recipes.find(r => r.id === activeId);
      if (recipe) {
        loadRecipe(recipe);
      }
      setInitialLoadDone(true);
    }
  });

  const handleChange = (e: Event) => {
    const target = e.target as HTMLSelectElement;
    const idx = parseInt(target.value, 10);

    if (idx === 0) {
      store.setActiveRecipeId('');
    } else {
      const recipe = store.recipes()[idx - 1];
      if (recipe) {
        store.setActiveRecipeId(recipe.id);
        loadRecipe(recipe);
      }
    }
  };

  const loadRecipe = (recipe: RecipeEntry) => {
    // Find and set model
    const modelIdx = store.models().findIndex(m => m.id === recipe.model_id);
    if (modelIdx >= 0) store.setSelectedModelIdx(modelIdx);

    // Find and set memory
    const memoryIdx = store.memories().findIndex(m => m.id === recipe.memory_id);
    store.setSelectedMemoryIdx(memoryIdx >= 0 ? memoryIdx + 1 : 0);

    // Find and set prompt
    const promptIdx = store.prompts().findIndex(p => p.id === recipe.prompt_id);
    store.setSelectedPromptIdx(promptIdx >= 0 ? promptIdx + 1 : 0);

    // Set qualifier
    store.setSelectedQualifier(recipe.qualifier || '');

    // Set system prompt
    if (recipe.system_prompt) {
      store.setSystemPrompt(recipe.system_prompt);
    } else if (promptIdx >= 0) {
      store.setSystemPrompt(store.prompts()[promptIdx].content);
    }
  };

  const buildRecipeFromCurrentSettings = (id: string, name: string): RecipeEntry => {
    const selectedModel = store.models()[store.selectedModelIdx()];
    const selectedPrompt = store.selectedPromptIdx() > 0
      ? store.prompts()[store.selectedPromptIdx() - 1]
      : null;
    const selectedMemory = store.selectedMemoryIdx() > 0
      ? store.memories()[store.selectedMemoryIdx() - 1]
      : null;
    const now = new Date().toISOString();

    return {
      id,
      name,
      description: '',
      model_id: selectedModel?.id || '',
      prompt_id: selectedPrompt?.id || '',
      system_prompt: store.systemPrompt(),
      memory_id: selectedMemory?.id || '',
      namespace_pattern: '',
      qualifier: store.selectedQualifier(),
      extra_params: {},
      owner: store.userId() || 'unknown',
      visibility: 'private',
      personality_rating: null,
      accuracy_rating: null,
      forked_from: null,
      created_at: now,
      updated_at: now,
    };
  };

  const handleSave = async () => {
    const recipe = activeRecipe();
    if (!recipe) return;

    setSaving(true);
    try {
      const updatedRecipe = buildRecipeFromCurrentSettings(recipe.id, recipe.name);
      updatedRecipe.description = recipe.description;
      updatedRecipe.owner = recipe.owner;
      updatedRecipe.visibility = recipe.visibility;
      updatedRecipe.created_at = recipe.created_at;
      updatedRecipe.forked_from = recipe.forked_from;

      await tauri.saveRecipe(updatedRecipe, 'Updated from chat config');
      await store.fetchRecipes();
      store.showStatus(`Snapshot saved: ${recipe.name}`);
    } catch (err) {
      store.showStatus(`Failed to save snapshot: ${err}`, true);
    } finally {
      setSaving(false);
    }
  };

  const handleSaveAs = async () => {
    const name = newName().trim();
    if (!name) return;

    const id = name
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, '-')
      .replace(/^-|-$/g, '')
      .substring(0, 64);

    // Check for duplicate
    if (store.recipes().some(r => r.id === id)) {
      store.showStatus(`Snapshot with ID "${id}" already exists`, true);
      return;
    }

    setSaving(true);
    try {
      const newRecipe = buildRecipeFromCurrentSettings(id, name);
      await tauri.saveRecipe(newRecipe, 'Created from chat config');
      await store.fetchRecipes();
      store.setActiveRecipeId(id);
      setShowSaveAs(false);
      setNewName('');
      store.showStatus(`Snapshot created: ${name}`);
    } catch (err) {
      store.showStatus(`Failed to create snapshot: ${err}`, true);
    } finally {
      setSaving(false);
    }
  };

  return (
    <div class="flex flex-col gap-1">
      <label class="text-xs text-text-dim uppercase tracking-wide">Snapshot</label>
      <div class="flex items-center gap-2">
        <select
          value={selectedRecipeIdx()}
          onChange={handleChange}
          class="flex-1 bg-surface-2 border border-border rounded-small px-3 py-1.5 text-sm
                 focus:outline-none focus:ring-2 focus:ring-accent"
        >
          <option value={0}>(none)</option>
          <For each={store.recipes()}>
            {(recipe, idx) => (
              <option value={idx() + 1}>{recipe.name}</option>
            )}
          </For>
        </select>

        {/* Save button - only enabled when a recipe is selected */}
        <button
          onClick={handleSave}
          disabled={!activeRecipe() || saving()}
          title="Save current settings to selected snapshot"
          class="p-1.5 bg-surface-2 border border-border rounded-small
                 hover:bg-surface-3 transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
        >
          <span class="material-icons text-sm">save</span>
        </button>

        {/* Save As button */}
        <button
          onClick={() => setShowSaveAs(true)}
          disabled={saving()}
          title="Save as new snapshot"
          class="p-1.5 bg-accent text-crust rounded-small
                 hover:bg-accent-hover transition-colors disabled:opacity-50"
        >
          <span class="material-icons text-sm">add</span>
        </button>
      </div>

      {/* Save As mini dialog */}
      <Show when={showSaveAs()}>
        <div class="mt-2 p-3 bg-surface-2 rounded-standard border border-border">
          <div class="flex items-center gap-2">
            <input
              type="text"
              value={newName()}
              onInput={(e) => setNewName(e.currentTarget.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleSaveAs()}
              placeholder="Snapshot name..."
              class="flex-1 bg-surface border border-border rounded-small px-3 py-1.5 text-sm
                     focus:outline-none focus:ring-2 focus:ring-accent"
            />
            <button
              onClick={handleSaveAs}
              disabled={!newName().trim() || saving()}
              class="px-3 py-1.5 text-sm bg-accent text-crust rounded-small
                     hover:bg-accent-hover transition-colors disabled:opacity-50"
            >
              Save
            </button>
            <button
              onClick={() => { setShowSaveAs(false); setNewName(''); }}
              class="px-3 py-1.5 text-sm bg-surface border border-border rounded-small
                     hover:bg-surface-3 transition-colors"
            >
              Cancel
            </button>
          </div>
        </div>
      </Show>
    </div>
  );
};

export default RecipeSelector;
