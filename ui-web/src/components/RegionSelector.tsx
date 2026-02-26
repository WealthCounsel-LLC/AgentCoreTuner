import { For } from 'solid-js';
import type { Component } from 'solid-js';
import { store } from '../stores/app';
import { AWS_REGIONS } from '../types';

const RegionSelector: Component = () => {
  const handleChange = (e: Event) => {
    const target = e.target as HTMLSelectElement;
    store.setSelectedRegion(target.value);
    // Refresh data when region changes
    if (store.authStatus() === 'valid') {
      store.refreshAll();
    }
  };

  return (
    <select
      value={store.selectedRegion()}
      onChange={handleChange}
      class="bg-surface-2 border border-border rounded-small px-3 py-1.5 text-sm text-text
             focus:outline-none focus:ring-2 focus:ring-accent min-w-[120px]"
    >
      <For each={AWS_REGIONS}>
        {(region) => <option value={region}>{region}</option>}
      </For>
    </select>
  );
};

export default RegionSelector;
