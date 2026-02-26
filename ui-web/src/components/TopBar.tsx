import { createSignal } from 'solid-js';
import type { Component } from 'solid-js';
import ProfileSelector from './ProfileSelector';
import RegionSelector from './RegionSelector';
import AuthStatusIcon from './AuthStatusIcon';
import RefreshButton from './RefreshButton';
import { SettingsDialog } from './settings';

const TopBar: Component = () => {
  const [showSettings, setShowSettings] = createSignal(false);

  return (
    <>
      <header class="bg-surface border-b border-border px-4 py-3 flex items-center justify-between">
        <div class="flex items-center gap-4">
          <h1 class="text-lg font-semibold text-accent">AWS AgentCore</h1>
          <div class="flex items-center gap-2">
            <ProfileSelector />
            <RegionSelector />
          </div>
        </div>
        <div class="flex items-center gap-1">
          <AuthStatusIcon />
          <RefreshButton />
          <button
            onClick={() => setShowSettings(true)}
            class="p-2 rounded-standard hover:bg-surface-2 transition-colors"
            title="Settings"
          >
            <span class="material-icons text-text-dim hover:text-text">settings</span>
          </button>
        </div>
      </header>

      <SettingsDialog
        isOpen={showSettings()}
        onClose={() => setShowSettings(false)}
      />
    </>
  );
};

export default TopBar;
