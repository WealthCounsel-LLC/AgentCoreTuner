import { Show, onMount, onCleanup, createSignal } from 'solid-js';
import type { Component } from 'solid-js';

interface RecipeContextMenuProps {
  isOpen: boolean;
  x: number;
  y: number;
  onClose: () => void;
  onLoad: () => void;
  onEdit: () => void;
  onFork: () => void;
  onSaveAs: () => void;
  onDelete: () => void;
  onCreateAgent: () => void;
  onQuickSave: () => void;
  onHistory: () => void;
}

const RecipeContextMenu: Component<RecipeContextMenuProps> = (props) => {
  let menuRef: HTMLDivElement | undefined;
  const [adjustedPosition, setAdjustedPosition] = createSignal({ x: props.x, y: props.y });

  onMount(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (menuRef && !menuRef.contains(e.target as Node)) {
        props.onClose();
      }
    };

    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        props.onClose();
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    document.addEventListener('keydown', handleEscape);

    // Adjust position if menu would go off screen
    if (menuRef) {
      const rect = menuRef.getBoundingClientRect();
      const viewportWidth = window.innerWidth;
      const viewportHeight = window.innerHeight;

      let newX = props.x;
      let newY = props.y;

      if (props.x + rect.width > viewportWidth) {
        newX = viewportWidth - rect.width - 8;
      }
      if (props.y + rect.height > viewportHeight) {
        newY = viewportHeight - rect.height - 8;
      }

      setAdjustedPosition({ x: newX, y: newY });
    }

    onCleanup(() => {
      document.removeEventListener('mousedown', handleClickOutside);
      document.removeEventListener('keydown', handleEscape);
    });
  });

  const MenuItem: Component<{
    icon: string;
    label: string;
    onClick: () => void;
    danger?: boolean;
  }> = (menuProps) => (
    <button
      onClick={() => {
        menuProps.onClick();
        props.onClose();
      }}
      class={`w-full flex items-center gap-3 px-3 py-2 text-sm text-left
              hover:bg-surface-2 transition-colors ${
                menuProps.danger ? 'text-error hover:bg-error/10' : ''
              }`}
    >
      <span class="material-icons text-sm">{menuProps.icon}</span>
      {menuProps.label}
    </button>
  );

  return (
    <Show when={props.isOpen}>
      <div
        ref={menuRef}
        class="fixed bg-surface border border-border rounded-standard shadow-lg py-1 z-50 min-w-48"
        style={{
          left: `${adjustedPosition().x}px`,
          top: `${adjustedPosition().y}px`,
        }}
      >
        <MenuItem icon="play_arrow" label="Load Snapshot" onClick={props.onLoad} />
        <MenuItem icon="edit" label="Edit" onClick={props.onEdit} />
        <MenuItem icon="save" label="Quick Save" onClick={props.onQuickSave} />

        <div class="border-t border-border my-1" />

        <MenuItem icon="call_split" label="Fork" onClick={props.onFork} />
        <MenuItem icon="content_copy" label="Save As..." onClick={props.onSaveAs} />
        <MenuItem icon="history" label="Version History" onClick={props.onHistory} />

        <div class="border-t border-border my-1" />

        <MenuItem icon="add_circle" label="Create Agent" onClick={props.onCreateAgent} />

        <div class="border-t border-border my-1" />

        <MenuItem icon="delete" label="Delete" onClick={props.onDelete} danger />
      </div>
    </Show>
  );
};

export default RecipeContextMenu;
