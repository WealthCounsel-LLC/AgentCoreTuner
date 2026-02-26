import type { Component } from 'solid-js';
import { store } from '../../stores/app';

const UserIdInput: Component = () => {
  const handleInput = (e: Event) => {
    const target = e.target as HTMLInputElement;
    store.setRuntimeUserId(target.value);
  };

  return (
    <div class="flex flex-col gap-1">
      <label class="text-xs text-text-dim uppercase tracking-wide">User ID</label>
      <input
        type="text"
        value={store.runtimeUserId()}
        onInput={handleInput}
        placeholder="runtime-user-id"
        class="bg-surface-2 border border-border rounded-small px-3 py-1.5 text-sm
               focus:outline-none focus:ring-2 focus:ring-accent
               placeholder:text-text-dim/50"
      />
    </div>
  );
};

export default UserIdInput;
