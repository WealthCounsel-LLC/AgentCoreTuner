import { createSignal } from 'solid-js';
import type { Component } from 'solid-js';
import { store } from '../../stores/app';
import * as tauri from '../../lib/tauri';
import type { RecipeEntry } from '../../types';
import RecipeList from './RecipeList';
import RecipeDialog from './RecipeDialog';
import ForkDialog from './ForkDialog';
import SaveAsDialog from './SaveAsDialog';
import DeleteRecipeDialog from './DeleteRecipeDialog';
import RecipeContextMenu from './RecipeContextMenu';
import VersionHistoryDialog from './VersionHistoryDialog';
import { BrowseShared } from '../shared';

const SnapshotsTab: Component = () => {
  const [selectedRecipe, setSelectedRecipe] = createSignal<RecipeEntry | null>(null);
  const [contextMenuRecipe, setContextMenuRecipe] = createSignal<RecipeEntry | null>(null);
  const [contextMenuPosition, setContextMenuPosition] = createSignal({ x: 0, y: 0 });

  // Dialog states
  const [showContextMenu, setShowContextMenu] = createSignal(false);
  const [showRecipeDialog, setShowRecipeDialog] = createSignal(false);
  const [recipeDialogMode, setRecipeDialogMode] = createSignal<'create' | 'edit'>('create');
  const [showForkDialog, setShowForkDialog] = createSignal(false);
  const [showSaveAsDialog, setShowSaveAsDialog] = createSignal(false);
  const [showDeleteDialog, setShowDeleteDialog] = createSignal(false);
  const [showHistoryDialog, setShowHistoryDialog] = createSignal(false);

  const handleContextMenu = (recipe: RecipeEntry, e: MouseEvent) => {
    e.preventDefault();
    setContextMenuRecipe(recipe);
    setContextMenuPosition({ x: e.clientX, y: e.clientY });
    setShowContextMenu(true);
  };

  const handleLoadRecipe = async () => {
    const recipe = contextMenuRecipe();
    if (!recipe) return;

    // Find model index
    const modelIdx = store.models().findIndex(m => m.id === recipe.model_id);
    if (modelIdx >= 0) store.setSelectedModelIdx(modelIdx);

    // Find memory index (add 1 for "None" option)
    const memoryIdx = store.memories().findIndex(m => m.id === recipe.memory_id);
    store.setSelectedMemoryIdx(memoryIdx >= 0 ? memoryIdx + 1 : 0);

    // Find prompt index
    const promptIdx = store.prompts().findIndex(p => p.id === recipe.prompt_id);
    if (promptIdx >= 0) store.setSelectedPromptIdx(promptIdx);

    // Set active recipe ID
    store.setActiveRecipeId(recipe.id);

    // Set qualifier directly
    store.setSelectedQualifier(recipe.qualifier || 'LATEST');

    // Set system prompt
    store.setSystemPrompt(recipe.system_prompt);

    store.showStatus(`Loaded snapshot: ${recipe.name}`, false);
  };

  const handleSaveRecipe = async (recipe: RecipeEntry, message?: string) => {
    try {
      await tauri.saveRecipe(recipe, message);
      await store.fetchRecipes();
      setShowRecipeDialog(false);
      store.showStatus(`Snapshot saved: ${recipe.name}`, false);
    } catch (err) {
      store.showStatus(`Failed to save snapshot: ${err}`, true);
    }
  };

  const handleForkRecipe = async (recipe: RecipeEntry) => {
    try {
      await tauri.saveRecipe(recipe, `Forked from ${recipe.forked_from}`);
      await store.fetchRecipes();
      setShowForkDialog(false);
      store.showStatus(`Snapshot forked: ${recipe.name}`, false);
    } catch (err) {
      store.showStatus(`Failed to fork snapshot: ${err}`, true);
    }
  };

  const handleSaveAsRecipe = async (recipe: RecipeEntry) => {
    try {
      await tauri.saveRecipe(recipe, 'Created from chat settings');
      await store.fetchRecipes();
      setShowSaveAsDialog(false);
      store.showStatus(`Snapshot created: ${recipe.name}`, false);
    } catch (err) {
      store.showStatus(`Failed to create snapshot: ${err}`, true);
    }
  };

  const handleDeleteRecipe = async (id: string) => {
    try {
      await tauri.deleteRecipe(id);
      await store.fetchRecipes();
      setShowDeleteDialog(false);
      setSelectedRecipe(null);
      store.showStatus('Snapshot deleted', false);
    } catch (err) {
      store.showStatus(`Failed to delete snapshot: ${err}`, true);
    }
  };

  const handleQuickSave = async () => {
    const recipe = contextMenuRecipe();
    if (!recipe) return;

    // Update recipe with current chat settings
    const selectedModel = store.models()[store.selectedModelIdx()];
    const selectedPrompt = store.prompts()[store.selectedPromptIdx()];
    const selectedMemory = store.selectedMemoryIdx() > 0
      ? store.memories()[store.selectedMemoryIdx() - 1]
      : null;

    const updatedRecipe: RecipeEntry = {
      ...recipe,
      model_id: selectedModel?.id || '',
      prompt_id: selectedPrompt?.id || '',
      system_prompt: store.systemPrompt(),
      memory_id: selectedMemory?.id || '',
      qualifier: store.selectedQualifier(),
      updated_at: new Date().toISOString(),
    };

    try {
      await tauri.saveRecipe(updatedRecipe, 'Quick save from chat');
      await store.fetchRecipes();
      store.showStatus(`Snapshot updated: ${recipe.name}`, false);
    } catch (err) {
      store.showStatus(`Failed to save snapshot: ${err}`, true);
    }
  };

  const handleCreateAgent = () => {
    // TODO: This would open the Runtimes tab with pre-filled data
    store.showStatus('Create Agent feature coming in Phase 6', false);
  };

  const openCreateDialog = () => {
    setRecipeDialogMode('create');
    setContextMenuRecipe(null);
    setShowRecipeDialog(true);
  };

  const openEditDialog = () => {
    setRecipeDialogMode('edit');
    setShowRecipeDialog(true);
  };

  return (
    <div class="h-full flex flex-col">
      {/* Header */}
      <div class="flex items-center justify-between p-4 border-b border-border bg-surface">
        <div>
          <h2 class="text-lg font-semibold">Snapshots</h2>
          <p class="text-sm text-text-dim">
            Save and load chat configurations as reusable snapshots
          </p>
        </div>
        <div class="flex items-center gap-2">
          <button
            onClick={() => setShowSaveAsDialog(true)}
            class="px-3 py-1.5 text-sm bg-surface-2 border border-border rounded-standard
                   hover:bg-surface-3 transition-colors flex items-center gap-1"
          >
            <span class="material-icons text-sm">save</span>
            Save Current
          </button>
          <button
            onClick={openCreateDialog}
            class="px-3 py-1.5 text-sm bg-accent text-crust rounded-standard
                   hover:bg-accent-hover transition-colors flex items-center gap-1"
          >
            <span class="material-icons text-sm">add</span>
            New Snapshot
          </button>
        </div>
      </div>

      {/* Recipe list */}
      <div class="flex-1 overflow-hidden bg-surface rounded-standard border border-border m-4">
        <RecipeList
          selectedId={selectedRecipe()?.id || null}
          onSelect={setSelectedRecipe}
          onContextMenu={handleContextMenu}
        />
      </div>

      {/* Browse Team Recipes */}
      <div class="mx-4 mb-4">
        <BrowseShared type="recipes" onImport={() => store.fetchRecipes()} />
      </div>

      {/* Context Menu */}
      <RecipeContextMenu
        isOpen={showContextMenu()}
        x={contextMenuPosition().x}
        y={contextMenuPosition().y}
        onClose={() => setShowContextMenu(false)}
        onLoad={handleLoadRecipe}
        onEdit={openEditDialog}
        onFork={() => setShowForkDialog(true)}
        onSaveAs={() => setShowSaveAsDialog(true)}
        onDelete={() => setShowDeleteDialog(true)}
        onCreateAgent={handleCreateAgent}
        onQuickSave={handleQuickSave}
        onHistory={() => setShowHistoryDialog(true)}
      />

      {/* Dialogs */}
      <RecipeDialog
        isOpen={showRecipeDialog()}
        mode={recipeDialogMode()}
        recipe={recipeDialogMode() === 'edit' ? contextMenuRecipe() : null}
        onClose={() => setShowRecipeDialog(false)}
        onSave={handleSaveRecipe}
      />

      <ForkDialog
        isOpen={showForkDialog()}
        sourceRecipe={contextMenuRecipe()}
        onClose={() => setShowForkDialog(false)}
        onFork={handleForkRecipe}
      />

      <SaveAsDialog
        isOpen={showSaveAsDialog()}
        baseName={contextMenuRecipe()?.name}
        onClose={() => setShowSaveAsDialog(false)}
        onSave={handleSaveAsRecipe}
      />

      <DeleteRecipeDialog
        isOpen={showDeleteDialog()}
        recipe={contextMenuRecipe()}
        onClose={() => setShowDeleteDialog(false)}
        onDelete={handleDeleteRecipe}
      />

      <VersionHistoryDialog
        isOpen={showHistoryDialog()}
        recipeId={contextMenuRecipe()?.id || null}
        recipeName={contextMenuRecipe()?.name || null}
        onClose={() => setShowHistoryDialog(false)}
      />
    </div>
  );
};

export default SnapshotsTab;
