import { For, createMemo } from 'solid-js';
import type { Component } from 'solid-js';
import { store } from '../../stores/app';

const PromptSelector: Component = () => {
  const handleChange = (e: Event) => {
    const target = e.target as HTMLSelectElement;
    const idx = parseInt(target.value, 10);
    store.setSelectedPromptIdx(idx);

    // Update system prompt when prompt is selected
    if (idx > 0) {
      const prompt = store.prompts()[idx - 1];
      if (prompt) {
        store.setSystemPrompt(prompt.content);
      }
    } else {
      store.setSystemPrompt('');
    }
  };

  const selectedPromptName = createMemo(() => {
    const idx = store.selectedPromptIdx();
    if (idx === 0) return '(none)';
    const prompt = store.prompts()[idx - 1];
    return prompt?.name || '(none)';
  });

  return (
    <div class="flex flex-col gap-1">
      <label class="text-xs text-text-dim uppercase tracking-wide">Prompt</label>
      <div class="flex items-center gap-2">
        <select
          value={store.selectedPromptIdx()}
          onChange={handleChange}
          class="flex-1 bg-surface-2 border border-border rounded-small px-3 py-1.5 text-sm
                 focus:outline-none focus:ring-2 focus:ring-accent"
        >
          <option value={0}>(none)</option>
          <For each={store.prompts()}>
            {(prompt, idx) => (
              <option value={idx() + 1}>{prompt.name}</option>
            )}
          </For>
        </select>
        <button
          onClick={() => {
            // TODO: Open prompt edit dialog
            console.log('Edit prompt:', selectedPromptName());
          }}
          title="Edit prompt"
          class="p-1.5 rounded-small hover:bg-surface-2 transition-colors"
        >
          <span class="material-icons text-text-dim text-sm">edit</span>
        </button>
      </div>
    </div>
  );
};

export default PromptSelector;
