import { Show, createSignal } from 'solid-js';
import type { Component } from 'solid-js';
import type { RecipeEntry } from '../../types';
import { store } from '../../stores/app';
import * as tauri from '../../lib/tauri';

interface RecipeCardProps {
  recipe: RecipeEntry;
  isSelected: boolean;
  onClick: () => void;
  onContextMenu: (e: MouseEvent) => void;
}

const RecipeCard: Component<RecipeCardProps> = (props) => {
  const [publishing, setPublishing] = createSignal(false);

  const formatDate = (dateStr: string) => {
    try {
      const date = new Date(dateStr);
      return date.toLocaleDateString();
    } catch {
      return dateStr;
    }
  };

  const renderStars = (rating: number | null) => {
    if (rating === null) return <span class="text-text-dim">-</span>;
    return (
      <span class="text-yellow">
        {'★'.repeat(rating)}{'☆'.repeat(5 - rating)}
      </span>
    );
  };

  const handlePublish = async (e: MouseEvent) => {
    e.stopPropagation();
    setPublishing(true);
    try {
      const username = store.userId() || 'unknown';
      await tauri.publishRecipe(props.recipe.id, username);
      store.showStatus(`Published: ${props.recipe.name}`);
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
      onContextMenu={props.onContextMenu}
    >
      <div class="flex items-start justify-between">
        <div class="flex-1 min-w-0">
          <div class="flex items-center gap-2">
            <h3 class="font-medium text-text truncate">{props.recipe.name}</h3>
            <Show when={props.recipe.forked_from}>
              <span class="material-icons text-xs text-text-dim" title={`Forked from ${props.recipe.forked_from}`}>
                call_split
              </span>
            </Show>
          </div>
          <p class="text-sm text-text-dim mt-1 line-clamp-2">
            {props.recipe.description || 'No description'}
          </p>
        </div>
        <div class="flex items-center gap-2 ml-4">
          <button
            onClick={handlePublish}
            disabled={publishing()}
            class="p-1.5 rounded-small hover:bg-surface-3 transition-colors disabled:opacity-50"
            title="Share with team"
          >
            <span class={`material-icons text-sm text-accent ${publishing() ? 'animate-spin' : ''}`}>
              {publishing() ? 'sync' : 'share'}
            </span>
          </button>
          <span class={`px-2 py-0.5 rounded-full text-xs ${
            props.recipe.visibility === 'public'
              ? 'bg-success/20 text-success'
              : 'bg-surface-2 text-text-dim'
          }`}>
            {props.recipe.visibility}
          </span>
        </div>
      </div>

      <div class="mt-3 flex items-center gap-4 text-xs">
        <div class="flex items-center gap-1">
          <span class="text-text-dim">Model:</span>
          <span class="text-text truncate max-w-24">
            {props.recipe.model_id || 'Default'}
          </span>
        </div>
        <div class="flex items-center gap-1">
          <span class="text-text-dim">Personality:</span>
          {renderStars(props.recipe.personality_rating)}
        </div>
        <div class="flex items-center gap-1">
          <span class="text-text-dim">Accuracy:</span>
          {renderStars(props.recipe.accuracy_rating)}
        </div>
      </div>

      <div class="mt-2 flex items-center justify-between text-xs text-text-dim">
        <span>by {props.recipe.owner}</span>
        <span>Updated {formatDate(props.recipe.updated_at)}</span>
      </div>
    </div>
  );
};

export default RecipeCard;
