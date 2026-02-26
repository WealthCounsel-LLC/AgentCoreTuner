import { createSignal } from 'solid-js';
import type { Component } from 'solid-js';
import type { PromptEntry } from '../../types';
import { store } from '../../stores/app';
import * as tauri from '../../lib/tauri';

interface PromptCardProps {
  prompt: PromptEntry;
  isSelected: boolean;
  onClick: () => void;
  onEdit: () => void;
  onDelete: () => void;
  onHistory: () => void;
}

const PromptCard: Component<PromptCardProps> = (props) => {
  const [publishing, setPublishing] = createSignal(false);

  const formatDate = (dateStr: string) => {
    try {
      const date = new Date(dateStr);
      return date.toLocaleDateString();
    } catch {
      return dateStr;
    }
  };

  const truncateContent = (content: string, maxLength = 100) => {
    if (content.length <= maxLength) return content;
    return content.substring(0, maxLength) + '...';
  };

  const handlePublish = async (e: MouseEvent) => {
    e.stopPropagation();
    setPublishing(true);
    try {
      const username = store.userId() || 'unknown';
      await tauri.publishPrompt(props.prompt.id, username);
      store.showStatus(`Published: ${props.prompt.name}`);
    } catch (err) {
      store.showStatus(`Publish failed: ${err}`, true);
    } finally {
      setPublishing(false);
    }
  };

  return (
    <div
      class={`p-4 rounded-standard border cursor-pointer transition-colors ${
        props.isSelected
          ? 'bg-accent/10 border-accent'
          : 'bg-surface border-border hover:bg-surface-2'
      }`}
      onClick={props.onClick}
    >
      <div class="flex items-start justify-between">
        <div class="flex-1 min-w-0">
          <h3 class="font-medium text-text truncate">{props.prompt.name}</h3>
          <p class="text-sm text-text-dim mt-1 font-mono whitespace-pre-wrap">
            {truncateContent(props.prompt.content)}
          </p>
        </div>
        <div class="flex items-center gap-1 ml-4">
          <button
            onClick={handlePublish}
            disabled={publishing()}
            class="p-1 hover:bg-surface-2 rounded transition-colors disabled:opacity-50"
            title="Share with team"
          >
            <span class={`material-icons text-sm text-accent ${publishing() ? 'animate-spin' : ''}`}>
              {publishing() ? 'sync' : 'share'}
            </span>
          </button>
          <button
            onClick={(e) => {
              e.stopPropagation();
              props.onHistory();
            }}
            class="p-1 hover:bg-surface-2 rounded transition-colors"
            title="Version History"
          >
            <span class="material-icons text-sm text-text-dim">history</span>
          </button>
          <button
            onClick={(e) => {
              e.stopPropagation();
              props.onEdit();
            }}
            class="p-1 hover:bg-surface-2 rounded transition-colors"
            title="Edit"
          >
            <span class="material-icons text-sm text-text-dim">edit</span>
          </button>
          <button
            onClick={(e) => {
              e.stopPropagation();
              props.onDelete();
            }}
            class="p-1 hover:bg-error/20 rounded transition-colors"
            title="Delete"
          >
            <span class="material-icons text-sm text-error">delete</span>
          </button>
        </div>
      </div>

      <div class="mt-2 flex items-center justify-between text-xs text-text-dim">
        <span>by {props.prompt.owner}</span>
        <span>Updated {formatDate(props.prompt.updated_at)}</span>
      </div>
    </div>
  );
};

export default PromptCard;
