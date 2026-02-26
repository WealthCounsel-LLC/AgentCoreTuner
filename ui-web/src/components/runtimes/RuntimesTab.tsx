import { createSignal, Show, For } from 'solid-js';
import type { Component } from 'solid-js';
import { store } from '../../stores/app';
import * as tauri from '../../lib/tauri';
import type { Agent, AgentRuntimeDetail, CreateAgentRuntimeParams, RecipeEntry } from '../../types';
import RuntimeList from './RuntimeList';
import RuntimeDialog from './RuntimeDialog';
import RuntimeDetailView from './RuntimeDetailView';
import DeleteRuntimeDialog from './DeleteRuntimeDialog';

const RuntimesTab: Component = () => {
  const [selectedAgent, setSelectedAgent] = createSignal<Agent | null>(null);
  const [dialogAgent, setDialogAgent] = createSignal<Agent | null>(null);
  const [runtimeDetail, setRuntimeDetail] = createSignal<AgentRuntimeDetail | null>(null);
  const [initialRecipe, setInitialRecipe] = createSignal<RecipeEntry | null>(null);

  // Dialog states
  const [showRuntimeDialog, setShowRuntimeDialog] = createSignal(false);
  const [runtimeDialogMode, setRuntimeDialogMode] = createSignal<'create' | 'edit'>('create');
  const [showDeleteDialog, setShowDeleteDialog] = createSignal(false);
  const [showDetailView, setShowDetailView] = createSignal(false);
  const [loadingDetail, setLoadingDetail] = createSignal(false);
  const [showSnapshotPicker, setShowSnapshotPicker] = createSignal(false);

  const handleCreateRuntime = async (params: CreateAgentRuntimeParams) => {
    try {
      await tauri.createAgentRuntime(params);
      await store.fetchAgents();
      setShowRuntimeDialog(false);
      store.showStatus(`Runtime created: ${params.name}`, false);
    } catch (err) {
      store.showStatus(`Failed to create runtime: ${err}`, true);
    }
  };

  const handleUpdateRuntime = async (params: CreateAgentRuntimeParams) => {
    try {
      await tauri.updateAgentRuntime(params);
      await store.fetchAgents();
      setShowRuntimeDialog(false);
      store.showStatus(`Runtime updated: ${params.name}`, false);
    } catch (err) {
      store.showStatus(`Failed to update runtime: ${err}`, true);
    }
  };

  const handleDeleteRuntime = async (id: string) => {
    try {
      await tauri.deleteAgentRuntime(
        store.selectedProfile(),
        store.selectedRegion(),
        id
      );
      await store.fetchAgents();
      setShowDeleteDialog(false);
      setSelectedAgent(null);
      store.showStatus('Runtime deleted', false);
    } catch (err) {
      store.showStatus(`Failed to delete runtime: ${err}`, true);
    }
  };

  const handleViewDetails = async (agent: Agent) => {
    setLoadingDetail(true);
    setDialogAgent(agent);
    setShowDetailView(true);

    try {
      const detail = await tauri.getAgentRuntime(
        store.selectedProfile(),
        store.selectedRegion(),
        agent.id
      );
      setRuntimeDetail(detail);
    } catch (err) {
      store.showStatus(`Failed to load runtime details: ${err}`, true);
      setShowDetailView(false);
    } finally {
      setLoadingDetail(false);
    }
  };

  const openCreateDialog = () => {
    setRuntimeDialogMode('create');
    setRuntimeDetail(null);
    setInitialRecipe(null);
    setShowRuntimeDialog(true);
  };

  const openCreateFromSnapshot = (recipe: RecipeEntry) => {
    setRuntimeDialogMode('create');
    setRuntimeDetail(null);
    setInitialRecipe(recipe);
    setShowSnapshotPicker(false);
    setShowRuntimeDialog(true);
  };

  const openEditDialog = async (agent: Agent) => {
    setLoadingDetail(true);
    setDialogAgent(agent);

    try {
      const detail = await tauri.getAgentRuntime(
        store.selectedProfile(),
        store.selectedRegion(),
        agent.id
      );
      setRuntimeDetail(detail);
      setRuntimeDialogMode('edit');
      setShowRuntimeDialog(true);
    } catch (err) {
      store.showStatus(`Failed to load runtime details: ${err}`, true);
    } finally {
      setLoadingDetail(false);
    }
  };

  const openDeleteDialog = (agent: Agent) => {
    setDialogAgent(agent);
    setShowDeleteDialog(true);
  };

  const handleRefresh = async () => {
    await store.fetchAgents();
    store.showStatus('Runtimes refreshed', false);
  };

  return (
    <div class="h-full flex flex-col">
      {/* Header */}
      <div class="flex items-center justify-between p-4 border-b border-border bg-surface">
        <div>
          <h2 class="text-lg font-semibold">Runtimes</h2>
          <p class="text-sm text-text-dim">
            Manage AWS Bedrock AgentCore Runtimes
          </p>
        </div>
        <div class="flex items-center gap-2">
          <button
            onClick={handleRefresh}
            disabled={store.loading().agents}
            class="px-3 py-1.5 text-sm bg-surface-2 border border-border rounded-standard
                   hover:bg-surface-3 transition-colors flex items-center gap-1
                   disabled:opacity-50"
          >
            <span class={`material-icons text-sm ${store.loading().agents ? 'animate-spin' : ''}`}>
              refresh
            </span>
            Refresh
          </button>

          {/* Build from Snapshot dropdown */}
          <div class="relative">
            <button
              onClick={() => setShowSnapshotPicker(!showSnapshotPicker())}
              class="px-3 py-1.5 text-sm bg-surface-2 border border-border rounded-standard
                     hover:bg-surface-3 transition-colors flex items-center gap-1"
            >
              <span class="material-icons text-sm">bookmark</span>
              Build from Snapshot
              <span class="material-icons text-sm">
                {showSnapshotPicker() ? 'expand_less' : 'expand_more'}
              </span>
            </button>

            <Show when={showSnapshotPicker()}>
              <div class="absolute right-0 top-full mt-1 w-72 bg-surface border border-border
                          rounded-standard shadow-lg z-50 max-h-80 overflow-y-auto">
                <Show when={store.recipes().length === 0}>
                  <div class="p-4 text-center text-text-dim text-sm">
                    No snapshots saved yet
                  </div>
                </Show>
                <For each={store.recipes()}>
                  {(recipe) => (
                    <button
                      onClick={() => openCreateFromSnapshot(recipe)}
                      class="w-full px-4 py-3 text-left hover:bg-surface-2 transition-colors
                             border-b border-border last:border-b-0"
                    >
                      <div class="font-medium text-sm truncate">{recipe.name}</div>
                      <div class="text-xs text-text-dim truncate mt-0.5">
                        {recipe.description || 'No description'}
                      </div>
                      <Show when={recipe.model_id}>
                        <div class="text-xs text-accent mt-1 truncate">
                          Model: {recipe.model_id}
                        </div>
                      </Show>
                    </button>
                  )}
                </For>
              </div>
            </Show>
          </div>

          <button
            onClick={openCreateDialog}
            class="px-3 py-1.5 text-sm bg-accent text-crust rounded-standard
                   hover:bg-accent-hover transition-colors flex items-center gap-1"
          >
            <span class="material-icons text-sm">add</span>
            New Runtime
          </button>
        </div>
      </div>

      {/* Runtime list */}
      <div class="flex-1 overflow-hidden bg-surface rounded-standard border border-border m-4">
        <RuntimeList
          selectedId={selectedAgent()?.id || null}
          onSelect={setSelectedAgent}
          onEdit={openEditDialog}
          onDelete={openDeleteDialog}
          onViewDetails={handleViewDetails}
        />
      </div>

      {/* Selected runtime preview */}
      <Show when={selectedAgent()}>
        <div class="mx-4 mb-4 p-4 bg-surface rounded-standard border border-border">
          <div class="flex items-center justify-between mb-2">
            <div>
              <h3 class="font-medium">{selectedAgent()!.name}</h3>
              <p class="text-xs text-text-dim font-mono">{selectedAgent()!.arn}</p>
            </div>
            <div class="flex items-center gap-1">
              <button
                onClick={() => handleViewDetails(selectedAgent()!)}
                class="p-1 hover:bg-surface-2 rounded transition-colors"
                title="View Details"
              >
                <span class="material-icons text-sm">visibility</span>
              </button>
              <button
                onClick={() => openEditDialog(selectedAgent()!)}
                class="p-1 hover:bg-surface-2 rounded transition-colors"
                title="Edit"
              >
                <span class="material-icons text-sm">edit</span>
              </button>
            </div>
          </div>
          <div class="grid grid-cols-4 gap-4 text-xs">
            <div>
              <span class="text-text-dim">Status:</span>{' '}
              <span class="text-text">{selectedAgent()!.status}</span>
            </div>
            <div>
              <span class="text-text-dim">Version:</span>{' '}
              <span class="text-text">v{selectedAgent()!.version}</span>
            </div>
            <div>
              <span class="text-text-dim">ID:</span>{' '}
              <span class="text-text font-mono truncate">{selectedAgent()!.id}</span>
            </div>
            <div>
              <span class="text-text-dim">Updated:</span>{' '}
              <span class="text-text">{new Date(selectedAgent()!.last_updated).toLocaleDateString()}</span>
            </div>
          </div>
        </div>
      </Show>

      {/* Loading overlay */}
      <Show when={loadingDetail()}>
        <div class="fixed inset-0 bg-black/30 flex items-center justify-center z-40">
          <div class="bg-surface rounded-standard p-4 flex items-center gap-3">
            <span class="material-icons animate-spin text-accent">sync</span>
            <span>Loading runtime details...</span>
          </div>
        </div>
      </Show>

      {/* Dialogs */}
      <RuntimeDialog
        isOpen={showRuntimeDialog()}
        mode={runtimeDialogMode()}
        runtime={runtimeDetail()}
        initialRecipe={initialRecipe()}
        onClose={() => {
          setShowRuntimeDialog(false);
          setInitialRecipe(null);
        }}
        onSave={runtimeDialogMode() === 'create' ? handleCreateRuntime : handleUpdateRuntime}
      />

      <RuntimeDetailView
        isOpen={showDetailView()}
        runtime={runtimeDetail()}
        onClose={() => setShowDetailView(false)}
      />

      <DeleteRuntimeDialog
        isOpen={showDeleteDialog()}
        agent={dialogAgent()}
        onClose={() => setShowDeleteDialog(false)}
        onDelete={handleDeleteRuntime}
      />
    </div>
  );
};

export default RuntimesTab;
