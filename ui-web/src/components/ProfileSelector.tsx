import { For } from 'solid-js';
import type { Component } from 'solid-js';
import { store } from '../stores/app';

const ProfileSelector: Component = () => {
  const handleChange = (e: Event) => {
    const target = e.target as HTMLSelectElement;
    store.setSelectedProfile(target.value);
    // Verify auth when profile changes
    store.verifyAuth();
  };

  return (
    <select
      value={store.selectedProfile()}
      onChange={handleChange}
      class="bg-surface-2 border border-border rounded-small px-3 py-1.5 text-sm text-text
             focus:outline-none focus:ring-2 focus:ring-accent min-w-[140px]"
    >
      <For each={store.profiles()}>
        {(profile) => <option value={profile}>{profile}</option>}
      </For>
    </select>
  );
};

export default ProfileSelector;
