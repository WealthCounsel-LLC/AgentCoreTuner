import { createSignal, For, Show, createMemo } from 'solid-js';
import type { Component } from 'solid-js';
import { store } from '../../stores/app';
import RecipeCard from './RecipeCard';
import type { RecipeEntry } from '../../types';

interface RecipeListProps {
  onSelect: (recipe: RecipeEntry) => void;
  onContextMenu: (recipe: RecipeEntry, e: MouseEvent) => void;
  selectedId: string | null;
}

const RecipeList: Component<RecipeListProps> = (props) => {
  const [searchQuery, setSearchQuery] = createSignal('');
  const [visibilityFilter, setVisibilityFilter] = createSignal<'all' | 'public' | 'private'>('all');
  const [sortBy, setSortBy] = createSignal<'name' | 'updated' | 'rating'>('updated');

  const filteredRecipes = createMemo(() => {
    let recipes = [...store.recipes()];

    // Apply search filter
    const query = searchQuery().toLowerCase().trim();
    if (query) {
      recipes = recipes.filter(r =>
        r.name.toLowerCase().includes(query) ||
        r.description.toLowerCase().includes(query) ||
        r.owner.toLowerCase().includes(query)
      );
    }

    // Apply visibility filter
    if (visibilityFilter() !== 'all') {
      recipes = recipes.filter(r => r.visibility === visibilityFilter());
    }

    // Apply sorting
    switch (sortBy()) {
      case 'name':
        recipes.sort((a, b) => a.name.localeCompare(b.name));
        break;
      case 'updated':
        recipes.sort((a, b) => new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime());
        break;
      case 'rating':
        recipes.sort((a, b) => {
          const aRating = (a.personality_rating || 0) + (a.accuracy_rating || 0);
          const bRating = (b.personality_rating || 0) + (b.accuracy_rating || 0);
          return bRating - aRating;
        });
        break;
    }

    return recipes;
  });

  return (
    <div class="flex flex-col h-full">
      {/* Search and filters */}
      <div class="flex flex-col gap-3 p-4 border-b border-border">
        <div class="relative">
          <span class="material-icons absolute left-3 top-1/2 -translate-y-1/2 text-text-dim text-sm">
            search
          </span>
          <input
            type="text"
            placeholder="Search snapshots..."
            value={searchQuery()}
            onInput={(e) => setSearchQuery(e.currentTarget.value)}
            class="w-full pl-10 pr-4 py-2 bg-surface-2 border border-border rounded-standard
                   text-sm focus:outline-none focus:ring-2 focus:ring-accent
                   placeholder:text-text-dim/50"
          />
        </div>

        <div class="flex items-center gap-4">
          <div class="flex items-center gap-2">
            <label class="text-xs text-text-dim">Visibility:</label>
            <select
              value={visibilityFilter()}
              onChange={(e) => setVisibilityFilter(e.currentTarget.value as 'all' | 'public' | 'private')}
              class="bg-surface-2 border border-border rounded px-2 py-1 text-xs
                     focus:outline-none focus:ring-2 focus:ring-accent"
            >
              <option value="all">All</option>
              <option value="public">Public</option>
              <option value="private">Private</option>
            </select>
          </div>

          <div class="flex items-center gap-2">
            <label class="text-xs text-text-dim">Sort:</label>
            <select
              value={sortBy()}
              onChange={(e) => setSortBy(e.currentTarget.value as 'name' | 'updated' | 'rating')}
              class="bg-surface-2 border border-border rounded px-2 py-1 text-xs
                     focus:outline-none focus:ring-2 focus:ring-accent"
            >
              <option value="updated">Last Updated</option>
              <option value="name">Name</option>
              <option value="rating">Rating</option>
            </select>
          </div>

          <div class="ml-auto text-xs text-text-dim">
            {filteredRecipes().length} of {store.recipes().length} snapshots
          </div>
        </div>
      </div>

      {/* Recipe list */}
      <div class="flex-1 overflow-auto p-4">
        <Show
          when={filteredRecipes().length > 0}
          fallback={
            <div class="flex flex-col items-center justify-center h-full text-text-dim">
              <span class="material-icons text-4xl mb-2">folder_open</span>
              <p>No snapshots found</p>
              <Show when={searchQuery() || visibilityFilter() !== 'all'}>
                <p class="text-sm mt-1">Try adjusting your filters</p>
              </Show>
            </div>
          }
        >
          <div class="flex flex-col gap-3">
            <For each={filteredRecipes()}>
              {(recipe) => (
                <RecipeCard
                  recipe={recipe}
                  isSelected={props.selectedId === recipe.id}
                  onClick={() => props.onSelect(recipe)}
                  onContextMenu={(e) => props.onContextMenu(recipe, e)}
                />
              )}
            </For>
          </div>
        </Show>
      </div>
    </div>
  );
};

export default RecipeList;
