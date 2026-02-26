import { createSignal, Show } from 'solid-js';
import type { Component } from 'solid-js';
import { store } from '../../stores/app';
import * as tauri from '../../lib/tauri';
import type { PromptEntry } from '../../types';
import PromptList from './PromptList';
import PromptDialog from './PromptDialog';
import DeletePromptDialog from './DeletePromptDialog';
import PromptVersionHistoryDialog from './PromptVersionHistoryDialog';
import { BrowseShared } from '../shared';

const PromptsTab: Component = () => {
  const [selectedPrompt, setSelectedPrompt] = createSignal<PromptEntry | null>(null);
  const [dialogPrompt, setDialogPrompt] = createSignal<PromptEntry | null>(null);

  // Dialog states
  const [showPromptDialog, setShowPromptDialog] = createSignal(false);
  const [promptDialogMode, setPromptDialogMode] = createSignal<'create' | 'edit'>('create');
  const [showDeleteDialog, setShowDeleteDialog] = createSignal(false);
  const [showHistoryDialog, setShowHistoryDialog] = createSignal(false);

  const handleSavePrompt = async (prompt: PromptEntry, message?: string) => {
    try {
      await tauri.savePrompt(prompt, message);
      await store.fetchPrompts();
      setShowPromptDialog(false);

      // If this prompt is currently selected in chat, update the system prompt
      const currentPromptId = store.prompts()[store.selectedPromptIdx()]?.id;
      if (prompt.id === currentPromptId) {
        store.setSystemPrompt(prompt.content);
      }

      store.showStatus(`Prompt saved: ${prompt.name}`, false);
    } catch (err) {
      store.showStatus(`Failed to save prompt: ${err}`, true);
    }
  };

  const handleDeletePrompt = async (id: string) => {
    try {
      await tauri.deletePrompt(id);
      await store.fetchPrompts();
      setShowDeleteDialog(false);
      setSelectedPrompt(null);

      // If this prompt was selected in chat, clear selection
      const currentPromptId = store.prompts()[store.selectedPromptIdx()]?.id;
      if (id === currentPromptId) {
        store.setSelectedPromptIdx(0);
        store.setSystemPrompt('');
      }

      store.showStatus('Prompt deleted', false);
    } catch (err) {
      store.showStatus(`Failed to delete prompt: ${err}`, true);
    }
  };

  const openCreateDialog = () => {
    setPromptDialogMode('create');
    setDialogPrompt(null);
    setShowPromptDialog(true);
  };

  const openEditDialog = (prompt: PromptEntry) => {
    setPromptDialogMode('edit');
    setDialogPrompt(prompt);
    setShowPromptDialog(true);
  };

  const openDeleteDialog = (prompt: PromptEntry) => {
    setDialogPrompt(prompt);
    setShowDeleteDialog(true);
  };

  const openHistoryDialog = (prompt: PromptEntry) => {
    setDialogPrompt(prompt);
    setShowHistoryDialog(true);
  };

  const handleSelectPrompt = (prompt: PromptEntry) => {
    setSelectedPrompt(prompt);
  };

  const handleUseInChat = () => {
    const prompt = selectedPrompt();
    if (!prompt) return;

    // Find the prompt index in the store
    const promptIdx = store.prompts().findIndex(p => p.id === prompt.id);
    if (promptIdx >= 0) {
      store.setSelectedPromptIdx(promptIdx + 1); // +1 for "None" option
      store.setSystemPrompt(prompt.content);
      store.setActiveTab('chat');
      store.showStatus(`Using prompt: ${prompt.name}`, false);
    }
  };

  return (
    <div class="h-full flex flex-col">
      {/* Header */}
      <div class="flex items-center justify-between p-4 border-b border-border bg-surface">
        <div>
          <h2 class="text-lg font-semibold">Prompts</h2>
          <p class="text-sm text-text-dim">
            Create and manage reusable system prompts
          </p>
        </div>
        <div class="flex items-center gap-2">
          <Show when={selectedPrompt()}>
            <button
              onClick={handleUseInChat}
              class="px-3 py-1.5 text-sm bg-surface-2 border border-border rounded-standard
                     hover:bg-surface-3 transition-colors flex items-center gap-1"
            >
              <span class="material-icons text-sm">chat</span>
              Use in Chat
            </button>
          </Show>
          <button
            onClick={openCreateDialog}
            class="px-3 py-1.5 text-sm bg-accent text-crust rounded-standard
                   hover:bg-accent-hover transition-colors flex items-center gap-1"
          >
            <span class="material-icons text-sm">add</span>
            New Prompt
          </button>
        </div>
      </div>

      {/* Prompt list */}
      <div class="flex-1 overflow-hidden bg-surface rounded-standard border border-border m-4">
        <PromptList
          selectedId={selectedPrompt()?.id || null}
          onSelect={handleSelectPrompt}
          onEdit={openEditDialog}
          onDelete={openDeleteDialog}
          onHistory={openHistoryDialog}
        />
      </div>

      {/* Browse Team Prompts */}
      <div class="mx-4 mb-4">
        <BrowseShared type="prompts" onImport={() => store.fetchPrompts()} />
      </div>

      {/* Selected prompt preview */}
      <Show when={selectedPrompt()}>
        <div class="mx-4 mb-4 p-4 bg-surface rounded-standard border border-border">
          <div class="flex items-center justify-between mb-2">
            <h3 class="font-medium">{selectedPrompt()!.name}</h3>
            <div class="flex items-center gap-1">
              <button
                onClick={() => openEditDialog(selectedPrompt()!)}
                class="p-1 hover:bg-surface-2 rounded transition-colors"
                title="Edit"
              >
                <span class="material-icons text-sm">edit</span>
              </button>
              <button
                onClick={() => openHistoryDialog(selectedPrompt()!)}
                class="p-1 hover:bg-surface-2 rounded transition-colors"
                title="History"
              >
                <span class="material-icons text-sm">history</span>
              </button>
            </div>
          </div>
          <pre class="text-sm text-text-dim font-mono whitespace-pre-wrap max-h-32 overflow-auto bg-surface-2 p-3 rounded">
            {selectedPrompt()!.content}
          </pre>
        </div>
      </Show>

      {/* Dialogs */}
      <PromptDialog
        isOpen={showPromptDialog()}
        mode={promptDialogMode()}
        prompt={dialogPrompt()}
        onClose={() => setShowPromptDialog(false)}
        onSave={handleSavePrompt}
      />

      <DeletePromptDialog
        isOpen={showDeleteDialog()}
        prompt={dialogPrompt()}
        onClose={() => setShowDeleteDialog(false)}
        onDelete={handleDeletePrompt}
      />

      <PromptVersionHistoryDialog
        isOpen={showHistoryDialog()}
        promptId={dialogPrompt()?.id || null}
        promptName={dialogPrompt()?.name || null}
        onClose={() => setShowHistoryDialog(false)}
      />
    </div>
  );
};

export default PromptsTab;
